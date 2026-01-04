//! pip wrapper for package deduplication
//!
//! This module provides functionality to intercept pip install commands
//! and deduplicate installed packages using the global cache.

use crate::cache::PackageCache;
use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::package::{InstalledPackage, PackageId};
use crate::platform::Platform;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper around pip that integrates with the package cache
pub struct PipWrapper {
    /// Path to the virtual environment
    venv_path: PathBuf,
    /// Path to the Python executable
    python_path: PathBuf,
    /// Python version (major.minor)
    python_version: String,
    /// Platform identifier
    platform: String,
    /// Package cache
    cache: PackageCache,
}

impl PipWrapper {
    /// Create a new PipWrapper for a virtual environment
    pub fn new(venv_path: PathBuf, config: Config) -> Result<Self> {
        let python_path = venv_path.join("bin").join("python");
        if !python_path.exists() {
            return Err(PvmError::EnvNotFound(
                venv_path.display().to_string(),
            ));
        }

        // Detect Python version from the venv
        let python_version = Self::detect_python_version(&python_path)?;
        let platform = Platform::detect()?.to_string();
        let cache = PackageCache::new(config)?;

        Ok(Self {
            venv_path,
            python_path,
            python_version,
            platform,
            cache,
        })
    }

    /// Detect Python version from the Python executable
    fn detect_python_version(python_path: &Path) -> Result<String> {
        let output = Command::new(python_path)
            .args(["-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
            .output()?;

        if !output.status.success() {
            return Err(PvmError::PipError(
                "Failed to detect Python version".to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get site-packages path for this venv
    pub fn site_packages(&self) -> Result<PathBuf> {
        let lib_dir = self.venv_path.join("lib");

        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if let Some(s) = name.to_str() {
                if s.starts_with("python") {
                    let site_packages = entry.path().join("site-packages");
                    if site_packages.exists() {
                        return Ok(site_packages);
                    }
                }
            }
        }

        Err(PvmError::EnvNotFound(
            "site-packages directory not found".to_string(),
        ))
    }

    /// Install packages with deduplication
    pub fn install(&mut self, packages: &[&str], extra_args: &[&str]) -> Result<InstallResult> {
        // 1. Get current package list
        let before = self.list_installed()?;

        // 2. Run pip install
        let mut cmd = Command::new(&self.python_path);
        cmd.args(["-m", "pip", "install", "--quiet"]);
        cmd.args(extra_args);
        cmd.args(packages);
        cmd.current_dir(&self.venv_path);

        let status = cmd.status()?;

        if !status.success() {
            return Err(PvmError::PipError(format!(
                "pip install failed with exit code {:?}",
                status.code()
            )));
        }

        // 3. Get new package list
        let after = self.list_installed()?;

        // 4. Find newly installed packages
        let new_packages: Vec<_> = after
            .iter()
            .filter(|p| {
                !before
                    .iter()
                    .any(|b| b.name == p.name && b.version == p.version)
            })
            .collect();

        // 5. Process each new package for deduplication
        let mut result = InstallResult::default();
        for package in new_packages {
            self.deduplicate_package(package, &mut result)?;
        }

        Ok(result)
    }

    /// Deduplicate a single package
    fn deduplicate_package(
        &mut self,
        package: &InstalledPackage,
        result: &mut InstallResult,
    ) -> Result<()> {
        let id = PackageId::new(
            &package.name,
            &package.version,
            &self.python_version,
            &self.platform,
        );

        if self.cache.is_cached(&id) {
            // Already in cache - remove installed files and create hardlinks
            self.replace_with_cache_links(package, &id)?;
            result.from_cache += 1;
            result.saved_bytes += package.total_size();
        } else {
            // Not in cache - move to cache, then create hardlinks
            self.move_to_cache_and_link(package, &id)?;
            result.added_to_cache += 1;
        }

        result.packages_installed += 1;
        Ok(())
    }

    /// Replace installed package with hardlinks from cache
    fn replace_with_cache_links(
        &mut self,
        package: &InstalledPackage,
        id: &PackageId,
    ) -> Result<()> {
        let site_packages = self.site_packages()?;

        // Remove the installed files
        if package.location.exists() {
            fs::remove_dir_all(&package.location)?;
        }
        if package.dist_info.exists() {
            fs::remove_dir_all(&package.dist_info)?;
        }

        // Create hardlinks from cache
        self.cache.link_to_site_packages(id, &site_packages)?;

        Ok(())
    }

    /// Move package to cache and create hardlinks back
    fn move_to_cache_and_link(
        &mut self,
        package: &InstalledPackage,
        id: &PackageId,
    ) -> Result<()> {
        let site_packages = self.site_packages()?;

        // Add to cache (moves files)
        self.cache.add_package(package, id)?;

        // Create hardlinks back to site-packages
        self.cache.link_to_site_packages(id, &site_packages)?;

        Ok(())
    }

    /// List installed packages in the venv
    pub fn list_installed(&self) -> Result<Vec<InstalledPackage>> {
        let site_packages = self.site_packages()?;
        let mut packages = Vec::new();

        for entry in fs::read_dir(&site_packages)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Look for .dist-info directories
            if name.ends_with(".dist-info") {
                if let Some(pkg) = self.parse_dist_info(&entry.path(), &site_packages)? {
                    packages.push(pkg);
                }
            }
        }

        Ok(packages)
    }

    /// Parse METADATA from .dist-info directory
    fn parse_dist_info(
        &self,
        dist_info: &Path,
        site_packages: &Path,
    ) -> Result<Option<InstalledPackage>> {
        let metadata_path = dist_info.join("METADATA");
        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&metadata_path)?;

        let mut name = None;
        let mut version = None;

        for line in content.lines() {
            if line.starts_with("Name: ") {
                name = Some(line[6..].trim().to_string());
            } else if line.starts_with("Version: ") {
                version = Some(line[9..].trim().to_string());
            }
            if name.is_some() && version.is_some() {
                break;
            }
        }

        let (name, version) = match (name, version) {
            (Some(n), Some(v)) => (n, v),
            _ => return Ok(None),
        };

        // Find the package directory
        let normalized_name = PackageId::normalize_name(&name);
        let location = site_packages.join(&normalized_name);

        // Get list of files from RECORD
        let files = self.get_package_files(dist_info, site_packages)?;

        Ok(Some(InstalledPackage {
            name,
            version,
            location,
            dist_info: dist_info.to_path_buf(),
            files,
        }))
    }

    /// Get list of files belonging to a package from RECORD
    fn get_package_files(
        &self,
        dist_info: &Path,
        site_packages: &Path,
    ) -> Result<Vec<PathBuf>> {
        let record_path = dist_info.join("RECORD");
        let mut files = Vec::new();

        if record_path.exists() {
            let content = fs::read_to_string(&record_path)?;
            for line in content.lines() {
                // RECORD format: path,hash,size
                if let Some(path) = line.split(',').next() {
                    let full_path = site_packages.join(path);
                    if full_path.exists() {
                        files.push(full_path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Sync all packages in the environment with the cache
    pub fn sync_all(&mut self) -> Result<InstallResult> {
        let packages = self.list_installed()?;
        let mut result = InstallResult::default();

        for package in &packages {
            self.deduplicate_package(package, &mut result)?;
        }

        Ok(result)
    }
}

/// Result of an install operation
#[derive(Debug, Default, Clone)]
pub struct InstallResult {
    /// Number of packages installed
    pub packages_installed: usize,
    /// Number of packages retrieved from cache
    pub from_cache: usize,
    /// Number of packages added to cache
    pub added_to_cache: usize,
    /// Bytes saved through deduplication
    pub saved_bytes: u64,
}

impl InstallResult {
    /// Check if any deduplication occurred
    pub fn had_deduplication(&self) -> bool {
        self.from_cache > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_result_default() {
        let result = InstallResult::default();
        assert_eq!(result.packages_installed, 0);
        assert_eq!(result.from_cache, 0);
        assert_eq!(result.added_to_cache, 0);
        assert!(!result.had_deduplication());
    }

    #[test]
    fn test_package_id_normalize() {
        // This is tested in package.rs, but verify it works here
        assert_eq!(PackageId::normalize_name("My-Package"), "my_package");
    }
}
