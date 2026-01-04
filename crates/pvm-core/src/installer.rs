//! Python installer - extracts and installs Python from archives

use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::version::PythonVersion;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Python installer
pub struct Installer {
    config: Config,
}

impl Installer {
    /// Create a new installer
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Install Python from a downloaded archive
    pub fn install(&self, archive_path: &Path, version: &PythonVersion) -> Result<PathBuf> {
        let pythons_dir = self.config.pythons_dir();
        std::fs::create_dir_all(&pythons_dir)?;

        let install_dir = pythons_dir.join(version.to_string());

        // Check if already installed
        if install_dir.exists() {
            let python_bin = self.python_bin_path(&install_dir);
            if python_bin.exists() {
                return Ok(install_dir);
            }
            // Incomplete installation, remove and reinstall
            std::fs::remove_dir_all(&install_dir)?;
        }

        // Create temp directory for extraction
        let temp_dir = tempfile::TempDir::new()?;

        // Extract archive
        self.extract_archive(archive_path, temp_dir.path())?;

        // Find the python directory inside extracted content
        // python-build-standalone extracts to "python/" subdirectory
        let extracted_python = temp_dir.path().join("python");
        let source_dir = if extracted_python.exists() {
            extracted_python
        } else {
            // Try to find any directory that looks like python install
            self.find_python_dir(temp_dir.path())?
        };

        // Move to final location
        std::fs::create_dir_all(&install_dir)?;
        self.copy_dir_contents(&source_dir, &install_dir)?;

        // Verify installation
        let python_bin = self.python_bin_path(&install_dir);
        if !python_bin.exists() {
            std::fs::remove_dir_all(&install_dir)?;
            return Err(PvmError::ExtractError(
                "Python binary not found after installation".to_string(),
            ));
        }

        Ok(install_dir)
    }

    /// Extract tar.gz archive
    fn extract_archive(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let file = File::open(archive_path)?;

        // Check if it's a zstd file
        let filename = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if filename.ends_with(".tar.zst") || filename.ends_with(".zst") {
            // Decompress with zstd
            let decoder = zstd::Decoder::new(file)?;
            let mut archive = Archive::new(decoder);
            archive.unpack(dest)?;
        } else {
            // Decompress with gzip
            let decoder = GzDecoder::new(file);
            let mut archive = Archive::new(decoder);
            archive.unpack(dest)?;
        }

        Ok(())
    }

    /// Find the Python directory in extracted content
    fn find_python_dir(&self, base: &Path) -> Result<PathBuf> {
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Check if this directory contains bin/python
                let python_bin = path.join("bin").join("python3");
                if python_bin.exists() {
                    return Ok(path);
                }
                // Also check for python
                let python_bin = path.join("bin").join("python");
                if python_bin.exists() {
                    return Ok(path);
                }
            }
        }
        Err(PvmError::ExtractError(
            "Could not find Python installation in archive".to_string(),
        ))
    }

    /// Copy directory contents recursively
    fn copy_dir_contents(&self, src: &Path, dst: &Path) -> Result<()> {
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                std::fs::create_dir_all(&dst_path)?;
                self.copy_dir_contents(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// Get the path to Python binary
    pub fn python_bin_path(&self, install_dir: &Path) -> PathBuf {
        install_dir.join("bin").join("python3")
    }

    /// Get installed Python path for a version
    pub fn get_python_path(&self, version: &PythonVersion) -> Option<PathBuf> {
        let install_dir = self.config.pythons_dir().join(version.to_string());
        let python_bin = self.python_bin_path(&install_dir);
        if python_bin.exists() {
            Some(python_bin)
        } else {
            None
        }
    }

    /// Remove installed Python version
    pub fn uninstall(&self, version: &PythonVersion) -> Result<()> {
        let install_dir = self.config.pythons_dir().join(version.to_string());
        if install_dir.exists() {
            std::fs::remove_dir_all(&install_dir)?;
        }
        Ok(())
    }

    /// Check if a version is installed
    pub fn is_installed(&self, version: &PythonVersion) -> bool {
        self.get_python_path(version).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
            packages_dir: None,
            dedup: Default::default(),
            shell: Default::default(),
            general: Default::default(),
        };
        (config, temp_dir)
    }

    // ========== Path Tests ==========

    #[test]
    fn test_python_bin_path() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let install_dir = PathBuf::from("/home/user/.pvm/pythons/3.12.4");
        let bin_path = installer.python_bin_path(&install_dir);

        assert_eq!(bin_path, PathBuf::from("/home/user/.pvm/pythons/3.12.4/bin/python3"));
    }

    #[test]
    fn test_get_python_path_not_installed() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let version = PythonVersion::new(3, 12, 4);
        assert!(installer.get_python_path(&version).is_none());
    }

    #[test]
    fn test_is_installed_false() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let version = PythonVersion::new(3, 12, 4);
        assert!(!installer.is_installed(&version));
    }

    #[test]
    fn test_is_installed_true() {
        let (config, temp) = create_test_config();

        // Create fake python installation
        let version = PythonVersion::new(3, 12, 4);
        let install_dir = temp.path().join("pythons").join("3.12.4").join("bin");
        std::fs::create_dir_all(&install_dir).unwrap();
        std::fs::write(install_dir.join("python3"), "fake").unwrap();

        let installer = Installer::new(config);
        assert!(installer.is_installed(&version));
    }

    // ========== Uninstall Tests ==========

    #[test]
    fn test_uninstall_existing() {
        let (config, temp) = create_test_config();

        // Create fake python installation
        let version = PythonVersion::new(3, 12, 4);
        let install_dir = temp.path().join("pythons").join("3.12.4");
        std::fs::create_dir_all(&install_dir).unwrap();

        let installer = Installer::new(config);
        installer.uninstall(&version).unwrap();

        assert!(!install_dir.exists());
    }

    #[test]
    fn test_uninstall_nonexistent() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let version = PythonVersion::new(3, 12, 4);
        // Should not error on non-existent
        installer.uninstall(&version).unwrap();
    }

    // ========== Archive Extraction Tests ==========

    #[test]
    fn test_find_python_dir_with_bin() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let temp = TempDir::new().unwrap();
        let python_dir = temp.path().join("python");
        let bin_dir = python_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("python3"), "fake").unwrap();

        let result = installer.find_python_dir(temp.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), python_dir);
    }

    #[test]
    fn test_find_python_dir_not_found() {
        let (config, _temp) = create_test_config();
        let installer = Installer::new(config);

        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join("empty")).unwrap();

        let result = installer.find_python_dir(temp.path());
        assert!(result.is_err());
    }
}
