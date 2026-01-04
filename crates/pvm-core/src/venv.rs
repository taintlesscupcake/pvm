//! Virtual environment management

use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::version::PythonVersion;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Information about a virtual environment
#[derive(Debug, Clone)]
pub struct VenvInfo {
    /// Name of the environment
    pub name: String,
    /// Path to the environment
    pub path: PathBuf,
    /// Python version used (if detectable)
    pub python_version: Option<PythonVersion>,
}

/// Virtual environment manager
pub struct VenvManager {
    config: Config,
}

impl VenvManager {
    /// Create a new VenvManager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Create a new virtual environment
    pub fn create(&self, name: &str, python_path: &Path) -> Result<PathBuf> {
        let envs_dir = self.config.envs_dir();
        std::fs::create_dir_all(&envs_dir)?;

        let env_path = envs_dir.join(name);

        // Check if already exists
        if env_path.exists() {
            return Err(PvmError::EnvAlreadyExists(name.to_string()));
        }

        // Create venv using python -m venv
        let status = Command::new(python_path)
            .args(["-m", "venv", env_path.to_str().unwrap()])
            .status()?;

        if !status.success() {
            return Err(PvmError::ExtractError(format!(
                "Failed to create virtual environment: exit code {:?}",
                status.code()
            )));
        }

        // Verify the environment was created
        let activate_script = env_path.join("bin").join("activate");
        if !activate_script.exists() {
            std::fs::remove_dir_all(&env_path).ok();
            return Err(PvmError::ExtractError(
                "Virtual environment creation failed: activate script not found".to_string(),
            ));
        }

        Ok(env_path)
    }

    /// Remove a virtual environment
    pub fn remove(&self, name: &str) -> Result<()> {
        let env_path = self.config.envs_dir().join(name);

        if !env_path.exists() {
            return Err(PvmError::EnvNotFound(name.to_string()));
        }

        std::fs::remove_dir_all(&env_path)?;
        Ok(())
    }

    /// List all virtual environments
    pub fn list(&self) -> Result<Vec<VenvInfo>> {
        let envs_dir = self.config.envs_dir();

        if !envs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut envs = Vec::new();

        for entry in std::fs::read_dir(&envs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                let name = entry
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .to_string();

                // Check if it's a valid venv (has bin/activate)
                if path.join("bin").join("activate").exists() {
                    let python_version = self.detect_python_version(&path);
                    envs.push(VenvInfo {
                        name,
                        path,
                        python_version,
                    });
                }
            }
        }

        // Sort by name
        envs.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(envs)
    }

    /// Get info about a specific environment
    pub fn get(&self, name: &str) -> Result<VenvInfo> {
        let env_path = self.config.envs_dir().join(name);

        if !env_path.exists() {
            return Err(PvmError::EnvNotFound(name.to_string()));
        }

        // Check if it's a valid venv
        if !env_path.join("bin").join("activate").exists() {
            return Err(PvmError::EnvNotFound(name.to_string()));
        }

        let python_version = self.detect_python_version(&env_path);

        Ok(VenvInfo {
            name: name.to_string(),
            path: env_path,
            python_version,
        })
    }

    /// Check if an environment exists
    pub fn exists(&self, name: &str) -> bool {
        let env_path = self.config.envs_dir().join(name);
        env_path.exists() && env_path.join("bin").join("activate").exists()
    }

    /// Get the activation script path for an environment
    pub fn activation_script_path(&self, name: &str) -> Result<PathBuf> {
        let env_path = self.config.envs_dir().join(name);

        if !env_path.exists() {
            return Err(PvmError::EnvNotFound(name.to_string()));
        }

        let activate = env_path.join("bin").join("activate");
        if !activate.exists() {
            return Err(PvmError::EnvNotFound(name.to_string()));
        }

        Ok(activate)
    }

    /// Detect Python version from pyvenv.cfg
    fn detect_python_version(&self, env_path: &Path) -> Option<PythonVersion> {
        let cfg_path = env_path.join("pyvenv.cfg");
        let content = std::fs::read_to_string(&cfg_path).ok()?;

        // Look for "version = X.Y.Z" line
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version") {
                if let Some(version_str) = line.split('=').nth(1) {
                    return PythonVersion::parse(version_str.trim()).ok();
                }
            }
        }

        None
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
        };
        (config, temp_dir)
    }

    fn create_fake_venv(temp: &Path, name: &str) -> PathBuf {
        let env_path = temp.join("envs").join(name);
        let bin_dir = env_path.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("activate"), "# fake activate").unwrap();
        std::fs::write(bin_dir.join("python"), "fake").unwrap();
        env_path
    }

    fn create_fake_venv_with_cfg(temp: &Path, name: &str, version: &str) -> PathBuf {
        let env_path = create_fake_venv(temp, name);
        let cfg_content = format!(
            "home = /usr/bin\nversion = {}\ninclude-system-site-packages = false\n",
            version
        );
        std::fs::write(env_path.join("pyvenv.cfg"), cfg_content).unwrap();
        env_path
    }

    // ========== Exists Tests ==========

    #[test]
    fn test_exists_false() {
        let (config, _temp) = create_test_config();
        let manager = VenvManager::new(config);

        assert!(!manager.exists("nonexistent"));
    }

    #[test]
    fn test_exists_true() {
        let (config, temp) = create_test_config();
        create_fake_venv(temp.path(), "myenv");

        let manager = VenvManager::new(config);
        assert!(manager.exists("myenv"));
    }

    // ========== List Tests ==========

    #[test]
    fn test_list_empty() {
        let (config, _temp) = create_test_config();
        let manager = VenvManager::new(config);

        let envs = manager.list().unwrap();
        assert!(envs.is_empty());
    }

    #[test]
    fn test_list_with_envs() {
        let (config, temp) = create_test_config();
        create_fake_venv(temp.path(), "env-a");
        create_fake_venv(temp.path(), "env-b");

        let manager = VenvManager::new(config);
        let envs = manager.list().unwrap();

        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0].name, "env-a"); // Sorted
        assert_eq!(envs[1].name, "env-b");
    }

    // ========== Get Tests ==========

    #[test]
    fn test_get_nonexistent() {
        let (config, _temp) = create_test_config();
        let manager = VenvManager::new(config);

        let result = manager.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_existing() {
        let (config, temp) = create_test_config();
        create_fake_venv_with_cfg(temp.path(), "myenv", "3.12.4");

        let manager = VenvManager::new(config);
        let info = manager.get("myenv").unwrap();

        assert_eq!(info.name, "myenv");
        assert!(info.python_version.is_some());
        assert_eq!(info.python_version.unwrap(), PythonVersion::new(3, 12, 4));
    }

    // ========== Remove Tests ==========

    #[test]
    fn test_remove_nonexistent() {
        let (config, _temp) = create_test_config();
        let manager = VenvManager::new(config);

        let result = manager.remove("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_existing() {
        let (config, temp) = create_test_config();
        let env_path = create_fake_venv(temp.path(), "myenv");

        let manager = VenvManager::new(config);
        manager.remove("myenv").unwrap();

        assert!(!env_path.exists());
    }

    // ========== Activation Script Tests ==========

    #[test]
    fn test_activation_script_path() {
        let (config, temp) = create_test_config();
        create_fake_venv(temp.path(), "myenv");

        let manager = VenvManager::new(config);
        let path = manager.activation_script_path("myenv").unwrap();

        assert!(path.ends_with("bin/activate"));
    }

    #[test]
    fn test_activation_script_path_nonexistent() {
        let (config, _temp) = create_test_config();
        let manager = VenvManager::new(config);

        let result = manager.activation_script_path("nonexistent");
        assert!(result.is_err());
    }

    // ========== Version Detection Tests ==========

    #[test]
    fn test_detect_python_version() {
        let (config, temp) = create_test_config();
        create_fake_venv_with_cfg(temp.path(), "myenv", "3.11.9");

        let manager = VenvManager::new(config);
        let env_path = temp.path().join("envs").join("myenv");
        let version = manager.detect_python_version(&env_path);

        assert!(version.is_some());
        assert_eq!(version.unwrap(), PythonVersion::new(3, 11, 9));
    }

    #[test]
    fn test_detect_python_version_no_cfg() {
        let (config, temp) = create_test_config();
        create_fake_venv(temp.path(), "myenv"); // No pyvenv.cfg

        let manager = VenvManager::new(config);
        let env_path = temp.path().join("envs").join("myenv");
        let version = manager.detect_python_version(&env_path);

        assert!(version.is_none());
    }
}
