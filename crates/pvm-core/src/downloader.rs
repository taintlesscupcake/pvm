//! Python downloader using uv's python-metadata.json
//!
//! Downloads and caches metadata from uv project for Python version info.

use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::platform::Platform;
use crate::version::PythonVersion;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Metadata URL from uv project
const METADATA_URL: &str = "https://raw.githubusercontent.com/astral-sh/uv/main/crates/uv-python/download-metadata.json";

/// Auto-update interval (7 days)
const UPDATE_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Python distribution metadata (matching uv's format)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PythonDistribution {
    pub name: String,
    pub arch: ArchInfo,
    pub os: String,
    pub libc: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    #[serde(default)]
    pub prerelease: String,
    pub url: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
    #[serde(default)]
    pub build: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchInfo {
    pub family: String,
    #[serde(default)]
    pub variant: Option<String>,
}

/// Available Python version info
#[derive(Debug, Clone)]
pub struct AvailablePython {
    pub version: PythonVersion,
    pub release_tag: String,
    pub download_url: String,
    pub sha256: String,
}

/// Python downloader
pub struct Downloader {
    client: reqwest::Client,
    config: Config,
    platform: Platform,
    /// Cached metadata
    metadata: HashMap<String, PythonDistribution>,
}

impl Downloader {
    /// Create a new downloader
    pub fn new(config: Config) -> Result<Self> {
        let platform = Platform::detect()?;
        Self::with_platform(config, platform)
    }

    /// Create a downloader with specific platform (for testing)
    pub fn with_platform(config: Config, platform: Platform) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("pvm/0.1.0")
            .build()
            .map_err(|e| PvmError::DownloadError(e.to_string()))?;

        Ok(Self {
            client,
            config,
            platform,
            metadata: HashMap::new(),
        })
    }

    /// Get metadata file path
    fn metadata_path(&self) -> PathBuf {
        self.config.home.join("python-metadata.json")
    }

    /// Check if metadata needs update
    fn needs_update(&self) -> bool {
        let path = self.metadata_path();
        if !path.exists() {
            return true;
        }

        match std::fs::metadata(&path) {
            Ok(meta) => {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                        return elapsed > UPDATE_INTERVAL;
                    }
                }
                false
            }
            Err(_) => true,
        }
    }

    /// Load metadata from cache
    fn load_cached_metadata(&mut self) -> Result<bool> {
        let path = self.metadata_path();
        if !path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(&path)?;
        self.metadata = serde_json::from_str(&content)
            .map_err(|e| PvmError::DownloadError(format!("Invalid metadata JSON: {}", e)))?;

        Ok(true)
    }

    /// Save metadata to cache
    fn save_metadata(&self) -> Result<()> {
        let path = self.metadata_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string(&self.metadata)
            .map_err(|e| PvmError::DownloadError(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Update metadata from remote
    pub async fn update_metadata(&mut self) -> Result<()> {
        let response = self
            .client
            .get(METADATA_URL)
            .send()
            .await
            .map_err(|e| PvmError::DownloadError(format!("Failed to fetch metadata: {}", e)))?;

        if !response.status().is_success() {
            return Err(PvmError::DownloadError(format!(
                "Failed to fetch metadata: {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| PvmError::DownloadError(format!("Failed to read metadata: {}", e)))?;

        self.metadata = serde_json::from_str(&content)
            .map_err(|e| PvmError::DownloadError(format!("Invalid metadata JSON: {}", e)))?;

        self.save_metadata()?;
        Ok(())
    }

    /// Ensure metadata is loaded (from cache or remote)
    pub async fn ensure_metadata(&mut self) -> Result<()> {
        // Try loading from cache first
        if self.load_cached_metadata()? {
            // Check if update is needed
            if self.needs_update() {
                // Try to update, but don't fail if network is unavailable
                if let Err(e) = self.update_metadata().await {
                    eprintln!("Warning: Failed to update metadata: {}", e);
                }
            }
            return Ok(());
        }

        // No cache, must download
        self.update_metadata().await
    }

    /// Get platform key for metadata lookup
    fn platform_key(&self) -> (&'static str, &'static str, &'static str) {
        match self.platform {
            Platform::MacOsAarch64 => ("darwin", "aarch64", "none"),
            Platform::MacOsX86_64 => ("darwin", "x86_64", "none"),
            Platform::LinuxX86_64 => ("linux", "x86_64", "gnu"),
            Platform::LinuxAarch64 => ("linux", "aarch64", "gnu"),
        }
    }

    /// Fetch available Python versions
    pub async fn fetch_available_versions(&mut self) -> Result<Vec<AvailablePython>> {
        self.ensure_metadata().await?;

        let (os, arch, libc) = self.platform_key();
        let mut available = Vec::new();

        for (key, dist) in &self.metadata {
            // Skip non-cpython
            if dist.name != "cpython" {
                continue;
            }

            // Skip variants (debug, freethreaded, etc.)
            if dist.variant.is_some() {
                continue;
            }

            // Skip prereleases
            if !dist.prerelease.is_empty() {
                continue;
            }

            // Match platform
            if dist.os != os || dist.arch.family != arch || dist.libc != libc {
                continue;
            }

            // Skip debug builds (key contains +debug)
            if key.contains("+debug") {
                continue;
            }

            // Skip entries without sha256 checksum
            let sha256 = match &dist.sha256 {
                Some(s) if !s.is_empty() => s.clone(),
                _ => continue,
            };

            let version = PythonVersion::new(
                dist.major as u8,
                dist.minor as u8,
                dist.patch as u8,
            );

            // Avoid duplicates
            if available.iter().any(|p: &AvailablePython| p.version == version) {
                continue;
            }

            available.push(AvailablePython {
                version,
                release_tag: dist.build.clone(),
                download_url: dist.url.clone(),
                sha256,
            });
        }

        // Sort by version descending
        available.sort_by(|a, b| b.version.cmp(&a.version));

        Ok(available)
    }

    /// Find the best matching version for a version spec
    pub async fn find_version(&mut self, spec: &str) -> Result<AvailablePython> {
        let available = self.fetch_available_versions().await?;

        available
            .into_iter()
            .find(|p| p.version.matches(spec))
            .ok_or_else(|| PvmError::VersionNotFound(spec.to_string()))
    }

    /// Download Python to cache directory
    pub async fn download(&self, python: &AvailablePython) -> Result<PathBuf> {
        let cache_dir = self.config.cache_dir();
        std::fs::create_dir_all(&cache_dir)?;

        let filename = python
            .download_url
            .split('/')
            .last()
            .ok_or_else(|| PvmError::DownloadError("Invalid URL".to_string()))?;

        // URL decode the filename (e.g., %2B -> +)
        let filename = urlencoding::decode(filename)
            .map_err(|e| PvmError::DownloadError(format!("Invalid filename encoding: {}", e)))?;

        let dest_path = cache_dir.join(filename.as_ref());

        // Check if already cached with valid checksum
        if dest_path.exists() {
            if self.verify_checksum(&dest_path, &python.sha256)? {
                return Ok(dest_path);
            }
            std::fs::remove_file(&dest_path)?;
        }

        // Download the file
        self.download_file(&python.download_url, &dest_path).await?;

        // Verify checksum
        if !self.verify_checksum(&dest_path, &python.sha256)? {
            std::fs::remove_file(&dest_path)?;
            return Err(PvmError::ChecksumMismatch);
        }

        Ok(dest_path)
    }

    /// Download a file
    async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| PvmError::DownloadError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PvmError::DownloadError(format!(
                "Download failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| PvmError::DownloadError(e.to_string()))?;

        let mut file = std::fs::File::create(dest)?;
        file.write_all(&bytes)?;

        Ok(())
    }

    /// Verify file checksum
    fn verify_checksum(&self, file_path: &Path, expected: &str) -> Result<bool> {
        let file_bytes = std::fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        let actual = format!("{:x}", hasher.finalize());

        Ok(actual == expected)
    }

    /// Get list of installed Python versions
    pub fn list_installed(&self) -> Result<Vec<PythonVersion>> {
        let pythons_dir = self.config.pythons_dir();
        if !pythons_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        for entry in std::fs::read_dir(&pythons_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if let Ok(version) = PythonVersion::parse(name_str) {
                        versions.push(version);
                    }
                }
            }
        }

        versions.sort_by(|a, b| b.cmp(a));
        Ok(versions)
    }

    /// Get metadata last update time
    pub fn metadata_age(&self) -> Option<Duration> {
        let path = self.metadata_path();
        std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| SystemTime::now().duration_since(t).ok())
    }
}

/// Generate download URL for a specific version and platform
pub fn build_download_url(version: &PythonVersion, platform: &Platform, release_tag: &str) -> String {
    format!(
        "https://github.com/astral-sh/python-build-standalone/releases/download/{}/cpython-{}+{}-{}-install_only.tar.gz",
        release_tag,
        version,
        release_tag,
        platform.target_triple()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_download_url_macos_aarch64() {
        let version = PythonVersion::new(3, 12, 4);
        let platform = Platform::MacOsAarch64;
        let url = build_download_url(&version, &platform, "20241101");

        assert_eq!(
            url,
            "https://github.com/astral-sh/python-build-standalone/releases/download/20241101/cpython-3.12.4+20241101-aarch64-apple-darwin-install_only.tar.gz"
        );
    }

    #[test]
    fn test_build_download_url_linux_x86_64() {
        let version = PythonVersion::new(3, 11, 9);
        let platform = Platform::LinuxX86_64;
        let url = build_download_url(&version, &platform, "20241001");

        assert_eq!(
            url,
            "https://github.com/astral-sh/python-build-standalone/releases/download/20241001/cpython-3.11.9+20241001-x86_64-unknown-linux-gnu-install_only.tar.gz"
        );
    }

    #[test]
    fn test_list_installed_empty() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        let versions = downloader.list_installed().unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_list_installed_with_versions() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let pythons_dir = temp_dir.path().join("pythons");
        std::fs::create_dir_all(&pythons_dir).unwrap();
        std::fs::create_dir(pythons_dir.join("3.11.9")).unwrap();
        std::fs::create_dir(pythons_dir.join("3.12.4")).unwrap();

        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        let versions = downloader.list_installed().unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0], PythonVersion::new(3, 12, 4));
        assert_eq!(versions[1], PythonVersion::new(3, 11, 9));
    }

    #[test]
    fn test_verify_checksum() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();

        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        // SHA256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(downloader.verify_checksum(&file_path, expected).unwrap());
        assert!(!downloader.verify_checksum(&file_path, "wronghash").unwrap());
    }

    #[test]
    fn test_metadata_path() {
        let config = Config {
            home: PathBuf::from("/home/user/.pvm"),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        assert_eq!(
            downloader.metadata_path(),
            PathBuf::from("/home/user/.pvm/python-metadata.json")
        );
    }

    #[test]
    fn test_needs_update_no_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        assert!(downloader.needs_update());
    }

    #[test]
    fn test_needs_update_fresh_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("python-metadata.json");
        std::fs::write(&metadata_path, "{}").unwrap();

        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        let downloader = Downloader::with_platform(config, Platform::MacOsAarch64).unwrap();

        assert!(!downloader.needs_update());
    }
}
