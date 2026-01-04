//! Platform detection for python-build-standalone

use crate::error::{PvmError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported platforms for python-build-standalone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// macOS on Apple Silicon (M1/M2/M3)
    MacOsAarch64,
    /// macOS on Intel
    MacOsX86_64,
    /// Linux on x86_64
    LinuxX86_64,
    /// Linux on ARM64
    LinuxAarch64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Result<Self> {
        Self::from_os_arch(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// Create platform from OS and architecture strings
    pub fn from_os_arch(os: &str, arch: &str) -> Result<Self> {
        match (os, arch) {
            ("macos", "aarch64") => Ok(Self::MacOsAarch64),
            ("macos", "x86_64") => Ok(Self::MacOsX86_64),
            ("linux", "x86_64") => Ok(Self::LinuxX86_64),
            ("linux", "aarch64") => Ok(Self::LinuxAarch64),
            _ => Err(PvmError::UnsupportedPlatform {
                os: os.to_string(),
                arch: arch.to_string(),
            }),
        }
    }

    /// Get the python-build-standalone target triple
    pub fn target_triple(&self) -> &'static str {
        match self {
            Self::MacOsAarch64 => "aarch64-apple-darwin",
            Self::MacOsX86_64 => "x86_64-apple-darwin",
            Self::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Self::LinuxAarch64 => "aarch64-unknown-linux-gnu",
        }
    }

    /// Get the OS name
    pub fn os(&self) -> &'static str {
        match self {
            Self::MacOsAarch64 | Self::MacOsX86_64 => "macos",
            Self::LinuxX86_64 | Self::LinuxAarch64 => "linux",
        }
    }

    /// Get the architecture name
    pub fn arch(&self) -> &'static str {
        match self {
            Self::MacOsAarch64 | Self::LinuxAarch64 => "aarch64",
            Self::MacOsX86_64 | Self::LinuxX86_64 => "x86_64",
        }
    }

    /// Check if this is a macOS platform
    pub fn is_macos(&self) -> bool {
        matches!(self, Self::MacOsAarch64 | Self::MacOsX86_64)
    }

    /// Check if this is a Linux platform
    pub fn is_linux(&self) -> bool {
        matches!(self, Self::LinuxX86_64 | Self::LinuxAarch64)
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.target_triple())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Platform Creation Tests ==========

    #[test]
    fn test_from_os_arch_macos_aarch64() {
        let p = Platform::from_os_arch("macos", "aarch64").unwrap();
        assert_eq!(p, Platform::MacOsAarch64);
    }

    #[test]
    fn test_from_os_arch_macos_x86_64() {
        let p = Platform::from_os_arch("macos", "x86_64").unwrap();
        assert_eq!(p, Platform::MacOsX86_64);
    }

    #[test]
    fn test_from_os_arch_linux_x86_64() {
        let p = Platform::from_os_arch("linux", "x86_64").unwrap();
        assert_eq!(p, Platform::LinuxX86_64);
    }

    #[test]
    fn test_from_os_arch_linux_aarch64() {
        let p = Platform::from_os_arch("linux", "aarch64").unwrap();
        assert_eq!(p, Platform::LinuxAarch64);
    }

    #[test]
    fn test_from_os_arch_unsupported() {
        let result = Platform::from_os_arch("windows", "x86_64");
        assert!(result.is_err());

        let result = Platform::from_os_arch("linux", "arm");
        assert!(result.is_err());
    }

    // ========== Target Triple Tests ==========

    #[test]
    fn test_target_triple() {
        assert_eq!(
            Platform::MacOsAarch64.target_triple(),
            "aarch64-apple-darwin"
        );
        assert_eq!(
            Platform::MacOsX86_64.target_triple(),
            "x86_64-apple-darwin"
        );
        assert_eq!(
            Platform::LinuxX86_64.target_triple(),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            Platform::LinuxAarch64.target_triple(),
            "aarch64-unknown-linux-gnu"
        );
    }

    // ========== OS and Arch Tests ==========

    #[test]
    fn test_os() {
        assert_eq!(Platform::MacOsAarch64.os(), "macos");
        assert_eq!(Platform::MacOsX86_64.os(), "macos");
        assert_eq!(Platform::LinuxX86_64.os(), "linux");
        assert_eq!(Platform::LinuxAarch64.os(), "linux");
    }

    #[test]
    fn test_arch() {
        assert_eq!(Platform::MacOsAarch64.arch(), "aarch64");
        assert_eq!(Platform::MacOsX86_64.arch(), "x86_64");
        assert_eq!(Platform::LinuxX86_64.arch(), "x86_64");
        assert_eq!(Platform::LinuxAarch64.arch(), "aarch64");
    }

    // ========== Platform Check Tests ==========

    #[test]
    fn test_is_macos() {
        assert!(Platform::MacOsAarch64.is_macos());
        assert!(Platform::MacOsX86_64.is_macos());
        assert!(!Platform::LinuxX86_64.is_macos());
        assert!(!Platform::LinuxAarch64.is_macos());
    }

    #[test]
    fn test_is_linux() {
        assert!(!Platform::MacOsAarch64.is_linux());
        assert!(!Platform::MacOsX86_64.is_linux());
        assert!(Platform::LinuxX86_64.is_linux());
        assert!(Platform::LinuxAarch64.is_linux());
    }

    // ========== Display Tests ==========

    #[test]
    fn test_display() {
        assert_eq!(Platform::MacOsAarch64.to_string(), "aarch64-apple-darwin");
        assert_eq!(Platform::LinuxX86_64.to_string(), "x86_64-unknown-linux-gnu");
    }

    // ========== Detection Test ==========

    #[test]
    fn test_detect_current_platform() {
        // This test verifies that detect() doesn't panic on the current platform
        // The result depends on the actual platform
        let result = Platform::detect();
        // On supported platforms, this should succeed
        #[cfg(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
        ))]
        assert!(result.is_ok());
    }
}
