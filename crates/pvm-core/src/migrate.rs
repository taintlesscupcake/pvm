//! Virtual environment migration support
//!
//! Migrates virtual environments from external sources (virtualenvwrapper, mise, etc.)
//! to pvm-managed environments.

use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::version::PythonVersion;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a source environment to migrate
#[derive(Debug, Clone)]
pub struct SourceEnvInfo {
    /// Name of the environment
    pub name: String,
    /// Path to the environment
    pub path: PathBuf,
    /// Python version used
    pub python_version: PythonVersion,
}

/// Result of a migration operation
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Name of the migrated environment
    pub env_name: String,
    /// Python version used
    pub python_version: PythonVersion,
    /// Whether Python was installed as part of migration
    pub python_installed: bool,
}

/// Migration manager for importing external virtualenvs
pub struct Migrator {
    config: Config,
}

impl Migrator {
    /// Create a new Migrator
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// List environments available for migration from a source directory
    pub fn list_source_envs(&self, source_dir: &Path) -> Result<Vec<SourceEnvInfo>> {
        if !source_dir.exists() {
            return Ok(Vec::new());
        }

        let mut envs = Vec::new();

        for entry in fs::read_dir(source_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let path = entry.path();

                // Check if it's a valid venv (has bin/activate)
                if path.join("bin").join("activate").exists() {
                    match self.parse_source_env(&path) {
                        Ok(info) => envs.push(info),
                        Err(_) => continue, // Skip invalid environments
                    }
                }
            }
        }

        // Sort by name
        envs.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(envs)
    }

    /// Parse a source environment's pyvenv.cfg to extract info
    pub fn parse_source_env(&self, env_path: &Path) -> Result<SourceEnvInfo> {
        if !env_path.exists() {
            return Err(PvmError::SourceEnvNotFound(
                env_path.display().to_string(),
            ));
        }

        // Check if it's a valid venv
        if !env_path.join("bin").join("activate").exists() {
            return Err(PvmError::MigrationError(format!(
                "Not a valid virtual environment: {}",
                env_path.display()
            )));
        }

        let cfg_path = env_path.join("pyvenv.cfg");
        if !cfg_path.exists() {
            return Err(PvmError::MigrationError(format!(
                "Missing pyvenv.cfg: {}",
                env_path.display()
            )));
        }

        let content = fs::read_to_string(&cfg_path)?;
        let python_version = Self::parse_pyvenv_cfg_version(&content)?;

        let name = env_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(SourceEnvInfo {
            name,
            path: env_path.to_path_buf(),
            python_version,
        })
    }

    /// Parse Python version from pyvenv.cfg content
    /// Handles both formats:
    /// - "version_info = 3.11.11.final.0" (virtualenv style)
    /// - "version = 3.11.11" (venv style)
    /// Note: version_info takes priority over version
    fn parse_pyvenv_cfg_version(content: &str) -> Result<PythonVersion> {
        // First pass: look for version_info (virtualenv format, takes priority)
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version_info") {
                if let Some(value) = line.split('=').nth(1) {
                    // "3.11.11.final.0" -> extract first 3 parts
                    let parts: Vec<&str> = value.trim().split('.').collect();
                    if parts.len() >= 3 {
                        let version_str = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
                        if let Ok(version) = PythonVersion::parse(&version_str) {
                            return Ok(version);
                        }
                    }
                }
            }
        }

        // Second pass: look for version (standard venv format, fallback)
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version") && !line.starts_with("version_info") {
                if let Some(value) = line.split('=').nth(1) {
                    if let Ok(version) = PythonVersion::parse(value.trim()) {
                        return Ok(version);
                    }
                }
            }
        }

        Err(PvmError::MigrationError(
            "Cannot detect Python version from pyvenv.cfg".to_string(),
        ))
    }

    /// Copy environment directory to pvm
    pub fn copy_env_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        if dst.exists() {
            return Err(PvmError::EnvAlreadyExists(
                dst.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ));
        }

        // Create parent directory if needed
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }

        // Recursive copy
        copy_dir_all(src, dst)?;

        Ok(())
    }

    /// Fix Python symlinks in bin directory to point to pvm Python
    pub fn fix_python_symlinks(&self, env_path: &Path, python_version: &PythonVersion) -> Result<()> {
        let bin_dir = env_path.join("bin");
        let target_python = self
            .config
            .pythons_dir()
            .join(python_version.to_string())
            .join("bin")
            .join("python3");

        // Links to fix: python, python3, python3.X
        let links = [
            "python".to_string(),
            "python3".to_string(),
            format!("python{}.{}", python_version.major, python_version.minor),
        ];

        for name in &links {
            let link_path = bin_dir.join(name);

            // Remove existing file/symlink
            if link_path.exists() || link_path.is_symlink() {
                fs::remove_file(&link_path)?;
            }

            // Create appropriate symlink
            if name == "python" {
                // python -> python3 (relative)
                #[cfg(unix)]
                std::os::unix::fs::symlink("python3", &link_path)?;
            } else if name == "python3" {
                // python3 -> absolute path to pvm python
                #[cfg(unix)]
                std::os::unix::fs::symlink(&target_python, &link_path)?;
            } else {
                // python3.X -> python3 (relative)
                #[cfg(unix)]
                std::os::unix::fs::symlink("python3", &link_path)?;
            }
        }

        Ok(())
    }

    /// Fix activate scripts to point to the new pvm-managed path
    /// Updates VIRTUAL_ENV variable in all activate scripts (bash, fish, csh, etc.)
    pub fn fix_activate_scripts(&self, env_path: &Path, source_path: &Path) -> Result<()> {
        let bin_dir = env_path.join("bin");
        let source_bin = source_path.to_string_lossy();
        let target_bin = env_path.to_string_lossy();

        // List of activate scripts to fix
        let scripts = ["activate", "activate.fish", "activate.csh"];

        for script_name in &scripts {
            let script_path = bin_dir.join(script_name);
            if script_path.exists() {
                let content = fs::read_to_string(&script_path)?;
                // Replace all occurrences of source path with target path
                let updated = content.replace(source_bin.as_ref(), target_bin.as_ref());
                if content != updated {
                    fs::write(&script_path, updated)?;
                }
            }
        }

        Ok(())
    }

    /// Update pyvenv.cfg to point to pvm Python
    pub fn update_pyvenv_cfg(&self, env_path: &Path, python_version: &PythonVersion) -> Result<()> {
        let pvm_python_dir = self.config.pythons_dir().join(python_version.to_string());
        let pvm_python_bin = pvm_python_dir.join("bin").join("python3");

        let content = format!(
            "home = {}/bin\n\
             include-system-site-packages = false\n\
             version = {}\n\
             executable = {}\n\
             command = {} -m venv {}\n",
            pvm_python_dir.display(),
            python_version,
            pvm_python_bin.display(),
            pvm_python_bin.display(),
            env_path.display(),
        );

        fs::write(env_path.join("pyvenv.cfg"), content)?;
        Ok(())
    }

    /// Check if a Python version is installed in pvm
    pub fn is_python_installed(&self, version: &PythonVersion) -> bool {
        let python_path = self
            .config
            .pythons_dir()
            .join(version.to_string())
            .join("bin")
            .join("python3");
        python_path.exists()
    }

    /// Get the target environment path
    pub fn target_env_path(&self, name: &str) -> PathBuf {
        self.config.envs_dir().join(name)
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if file_type.is_symlink() {
            // Copy symlinks as-is (they'll be fixed later if needed)
            #[cfg(unix)]
            {
                let target = fs::read_link(&src_path)?;
                if dst_path.exists() || dst_path.is_symlink() {
                    fs::remove_file(&dst_path)?;
                }
                std::os::unix::fs::symlink(&target, &dst_path)?;
            }
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp: &Path) -> Config {
        Config {
            home: temp.to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
            packages_dir: None,
            dedup: Default::default(),
            shell: Default::default(),
            general: Default::default(),
        }
    }

    fn create_fake_source_env(temp: &Path, name: &str, version_info: &str) -> PathBuf {
        let env_path = temp.join(name);
        let bin_dir = env_path.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("activate"), "# fake activate").unwrap();
        fs::write(bin_dir.join("python"), "fake").unwrap();

        let cfg_content = format!(
            "home = /some/path\n\
             version_info = {}\n\
             include-system-site-packages = false\n",
            version_info
        );
        fs::write(env_path.join("pyvenv.cfg"), cfg_content).unwrap();

        env_path
    }

    // ========== Version Parsing Tests ==========

    #[test]
    fn test_parse_version_info_format() {
        let content = "home = /some/path\nversion_info = 3.11.11.final.0\n";
        let version = Migrator::parse_pyvenv_cfg_version(content).unwrap();
        assert_eq!(version, PythonVersion::new(3, 11, 11));
    }

    #[test]
    fn test_parse_version_format() {
        let content = "home = /some/path\nversion = 3.12.4\n";
        let version = Migrator::parse_pyvenv_cfg_version(content).unwrap();
        assert_eq!(version, PythonVersion::new(3, 12, 4));
    }

    #[test]
    fn test_parse_version_info_priority() {
        // version_info should take precedence over version
        let content = "version = 3.10.0\nversion_info = 3.11.11.final.0\n";
        let version = Migrator::parse_pyvenv_cfg_version(content).unwrap();
        assert_eq!(version, PythonVersion::new(3, 11, 11));
    }

    #[test]
    fn test_parse_version_missing() {
        let content = "home = /some/path\n";
        let result = Migrator::parse_pyvenv_cfg_version(content);
        assert!(result.is_err());
    }

    // ========== Source Environment Tests ==========

    #[test]
    fn test_list_source_envs() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("envs");
        fs::create_dir_all(&source_dir).unwrap();

        create_fake_source_env(&source_dir, "env-a", "3.11.0.final.0");
        create_fake_source_env(&source_dir, "env-b", "3.12.0.final.0");

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        let envs = migrator.list_source_envs(&source_dir).unwrap();
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0].name, "env-a");
        assert_eq!(envs[1].name, "env-b");
    }

    #[test]
    fn test_list_source_envs_empty() {
        let temp = TempDir::new().unwrap();
        let source_dir = temp.path().join("envs");
        fs::create_dir_all(&source_dir).unwrap();

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        let envs = migrator.list_source_envs(&source_dir).unwrap();
        assert!(envs.is_empty());
    }

    #[test]
    fn test_parse_source_env() {
        let temp = TempDir::new().unwrap();
        let env_path = create_fake_source_env(temp.path(), "myenv", "3.11.11.final.0");

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        let info = migrator.parse_source_env(&env_path).unwrap();
        assert_eq!(info.name, "myenv");
        assert_eq!(info.python_version, PythonVersion::new(3, 11, 11));
    }

    #[test]
    fn test_parse_source_env_not_found() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        let result = migrator.parse_source_env(&temp.path().join("nonexistent"));
        assert!(result.is_err());
    }

    // ========== Copy Directory Tests ==========

    #[test]
    fn test_copy_env_directory() {
        let temp = TempDir::new().unwrap();
        let src = create_fake_source_env(temp.path(), "source", "3.11.0.final.0");
        let dst = temp.path().join("target");

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        migrator.copy_env_directory(&src, &dst).unwrap();

        assert!(dst.exists());
        assert!(dst.join("bin").join("activate").exists());
        assert!(dst.join("pyvenv.cfg").exists());
    }

    #[test]
    fn test_copy_env_directory_already_exists() {
        let temp = TempDir::new().unwrap();
        let src = create_fake_source_env(temp.path(), "source", "3.11.0.final.0");
        let dst = temp.path().join("target");
        fs::create_dir_all(&dst).unwrap();

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        let result = migrator.copy_env_directory(&src, &dst);
        assert!(result.is_err());
    }

    // ========== Update pyvenv.cfg Tests ==========

    #[test]
    fn test_update_pyvenv_cfg() {
        let temp = TempDir::new().unwrap();
        let env_path = create_fake_source_env(temp.path(), "myenv", "3.11.0.final.0");

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);
        let version = PythonVersion::new(3, 11, 11);

        migrator.update_pyvenv_cfg(&env_path, &version).unwrap();

        let content = fs::read_to_string(env_path.join("pyvenv.cfg")).unwrap();
        assert!(content.contains("version = 3.11.11"));
        assert!(content.contains(&temp.path().display().to_string()));
    }

    // ========== Fix Activate Scripts Tests ==========

    #[test]
    fn test_fix_activate_scripts() {
        let temp = TempDir::new().unwrap();
        let source_path = temp.path().join("old-location").join("myenv");
        let target_path = temp.path().join("new-location").join("myenv");

        // Create target env with activate script containing old path
        fs::create_dir_all(target_path.join("bin")).unwrap();
        let activate_content = format!(
            "VIRTUAL_ENV={}\nexport PATH=\"$VIRTUAL_ENV/bin:$PATH\"",
            source_path.display()
        );
        fs::write(target_path.join("bin").join("activate"), &activate_content).unwrap();

        let config = create_test_config(temp.path());
        let migrator = Migrator::new(config);

        migrator.fix_activate_scripts(&target_path, &source_path).unwrap();

        let updated = fs::read_to_string(target_path.join("bin").join("activate")).unwrap();
        assert!(updated.contains(&target_path.display().to_string()));
        assert!(!updated.contains(&source_path.display().to_string()));
    }
}
