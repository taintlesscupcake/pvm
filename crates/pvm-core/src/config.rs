//! Configuration management for pvm

use crate::error::{PvmError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default directory name for pvm
const PVM_DIR_NAME: &str = ".pvm";

/// Configuration for pvm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base directory for pvm (default: ~/.pvm)
    #[serde(default = "Config::default_home")]
    pub home: PathBuf,

    /// Directory for installed Python versions
    #[serde(default)]
    pub pythons_dir: Option<PathBuf>,

    /// Directory for virtual environments
    #[serde(default)]
    pub envs_dir: Option<PathBuf>,

    /// Directory for download cache
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

impl Config {
    /// Create a new Config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| PvmError::ConfigError(e.to_string()))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::default_home().join("config.toml"))
    }

    /// Get the default home directory
    pub fn default_home() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(PVM_DIR_NAME)
    }

    /// Get the pythons directory
    pub fn pythons_dir(&self) -> PathBuf {
        self.pythons_dir
            .clone()
            .unwrap_or_else(|| self.home.join("pythons"))
    }

    /// Get the envs directory
    pub fn envs_dir(&self) -> PathBuf {
        self.envs_dir
            .clone()
            .unwrap_or_else(|| self.home.join("envs"))
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.cache_dir
            .clone()
            .unwrap_or_else(|| self.home.join("cache"))
    }

    /// Get the bin directory
    pub fn bin_dir(&self) -> PathBuf {
        self.home.join("bin")
    }

    /// Ensure all directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.home)?;
        std::fs::create_dir_all(self.pythons_dir())?;
        std::fs::create_dir_all(self.envs_dir())?;
        std::fs::create_dir_all(self.cache_dir())?;
        std::fs::create_dir_all(self.bin_dir())?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            home: Self::default_home(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== Default Config Tests ==========

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.home.ends_with(PVM_DIR_NAME));
    }

    #[test]
    fn test_default_directories() {
        let config = Config::default();
        assert!(config.pythons_dir().ends_with("pythons"));
        assert!(config.envs_dir().ends_with("envs"));
        assert!(config.cache_dir().ends_with("cache"));
        assert!(config.bin_dir().ends_with("bin"));
    }

    // ========== Custom Config Tests ==========

    #[test]
    fn test_custom_home() {
        let config = Config {
            home: PathBuf::from("/custom/path"),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };
        assert_eq!(config.home, PathBuf::from("/custom/path"));
        assert_eq!(config.pythons_dir(), PathBuf::from("/custom/path/pythons"));
        assert_eq!(config.envs_dir(), PathBuf::from("/custom/path/envs"));
    }

    #[test]
    fn test_custom_subdirs() {
        let config = Config {
            home: PathBuf::from("/base"),
            pythons_dir: Some(PathBuf::from("/custom/pythons")),
            envs_dir: Some(PathBuf::from("/custom/envs")),
            cache_dir: Some(PathBuf::from("/custom/cache")),
        };
        assert_eq!(config.pythons_dir(), PathBuf::from("/custom/pythons"));
        assert_eq!(config.envs_dir(), PathBuf::from("/custom/envs"));
        assert_eq!(config.cache_dir(), PathBuf::from("/custom/cache"));
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_serialize_deserialize() {
        let config = Config {
            home: PathBuf::from("/test/home"),
            pythons_dir: Some(PathBuf::from("/test/pythons")),
            envs_dir: None,
            cache_dir: None,
        };

        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.home, config.home);
        assert_eq!(parsed.pythons_dir, config.pythons_dir);
    }

    // ========== File Operations Tests ==========

    #[test]
    fn test_ensure_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            home: temp_dir.path().join("pvm"),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
        };

        config.ensure_dirs().unwrap();

        assert!(config.home.exists());
        assert!(config.pythons_dir().exists());
        assert!(config.envs_dir().exists());
        assert!(config.cache_dir().exists());
        assert!(config.bin_dir().exists());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create custom config
        let config = Config {
            home: temp_dir.path().to_path_buf(),
            pythons_dir: Some(temp_dir.path().join("custom_pythons")),
            envs_dir: None,
            cache_dir: None,
        };

        // Save directly to temp location
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, &content).unwrap();

        // Read back
        let loaded_content = std::fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.pythons_dir, config.pythons_dir);
    }
}
