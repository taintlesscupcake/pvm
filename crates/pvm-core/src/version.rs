//! Python version parsing and comparison

use crate::error::{PvmError, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represents a Python version (major.minor.patch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PythonVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl PythonVersion {
    /// Create a new PythonVersion
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }

    /// Parse a version string (e.g., "3.11.9", "3.11", "3")
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(PvmError::InvalidVersion("empty string".to_string()));
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(PvmError::InvalidVersion(s.to_string()));
        }

        let major = parts[0]
            .parse::<u8>()
            .map_err(|_| PvmError::InvalidVersion(s.to_string()))?;

        let minor = if parts.len() > 1 {
            parts[1]
                .parse::<u8>()
                .map_err(|_| PvmError::InvalidVersion(s.to_string()))?
        } else {
            0
        };

        let patch = if parts.len() > 2 {
            parts[2]
                .parse::<u8>()
                .map_err(|_| PvmError::InvalidVersion(s.to_string()))?
        } else {
            0
        };

        Ok(Self { major, minor, patch })
    }

    /// Check if this version matches a partial version specification
    /// e.g., 3.11.9 matches "3.11" and "3"
    pub fn matches(&self, spec: &str) -> bool {
        if let Ok(spec_version) = Self::parse(spec) {
            let parts: Vec<&str> = spec.split('.').collect();
            match parts.len() {
                1 => self.major == spec_version.major,
                2 => self.major == spec_version.major && self.minor == spec_version.minor,
                3 => *self == spec_version,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Return version as tuple for comparison
    pub fn as_tuple(&self) -> (u8, u8, u8) {
        (self.major, self.minor, self.patch)
    }
}

impl fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for PythonVersion {
    type Err = PvmError;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

impl PartialOrd for PythonVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PythonVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_tuple().cmp(&other.as_tuple())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Parsing Tests ==========

    #[test]
    fn test_parse_full_version() {
        let v = PythonVersion::parse("3.11.9").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 11);
        assert_eq!(v.patch, 9);
    }

    #[test]
    fn test_parse_major_minor_only() {
        let v = PythonVersion::parse("3.12").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 12);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_parse_major_only() {
        let v = PythonVersion::parse("3").unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let v = PythonVersion::parse("  3.11.9  ").unwrap();
        assert_eq!(v, PythonVersion::new(3, 11, 9));
    }

    #[test]
    fn test_parse_invalid_empty() {
        assert!(PythonVersion::parse("").is_err());
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(PythonVersion::parse("not-a-version").is_err());
        assert!(PythonVersion::parse("3.x.1").is_err());
        assert!(PythonVersion::parse("3.11.9.1").is_err());
    }

    #[test]
    fn test_parse_invalid_overflow() {
        assert!(PythonVersion::parse("256.0.0").is_err());
    }

    // ========== Display Tests ==========

    #[test]
    fn test_display() {
        let v = PythonVersion::new(3, 11, 9);
        assert_eq!(v.to_string(), "3.11.9");
    }

    // ========== Comparison Tests ==========

    #[test]
    fn test_comparison_equal() {
        let v1 = PythonVersion::new(3, 11, 9);
        let v2 = PythonVersion::new(3, 11, 9);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_comparison_major_diff() {
        let v1 = PythonVersion::new(3, 11, 9);
        let v2 = PythonVersion::new(2, 11, 9);
        assert!(v1 > v2);
    }

    #[test]
    fn test_comparison_minor_diff() {
        let v1 = PythonVersion::new(3, 12, 0);
        let v2 = PythonVersion::new(3, 11, 9);
        assert!(v1 > v2);
    }

    #[test]
    fn test_comparison_patch_diff() {
        let v1 = PythonVersion::new(3, 11, 9);
        let v2 = PythonVersion::new(3, 11, 8);
        assert!(v1 > v2);
    }

    #[test]
    fn test_sorting() {
        let mut versions = vec![
            PythonVersion::new(3, 10, 0),
            PythonVersion::new(3, 12, 4),
            PythonVersion::new(3, 11, 9),
            PythonVersion::new(2, 7, 18),
        ];
        versions.sort();
        assert_eq!(
            versions,
            vec![
                PythonVersion::new(2, 7, 18),
                PythonVersion::new(3, 10, 0),
                PythonVersion::new(3, 11, 9),
                PythonVersion::new(3, 12, 4),
            ]
        );
    }

    // ========== Matching Tests ==========

    #[test]
    fn test_matches_exact() {
        let v = PythonVersion::new(3, 11, 9);
        assert!(v.matches("3.11.9"));
        assert!(!v.matches("3.11.8"));
    }

    #[test]
    fn test_matches_major_minor() {
        let v = PythonVersion::new(3, 11, 9);
        assert!(v.matches("3.11"));
        assert!(!v.matches("3.12"));
    }

    #[test]
    fn test_matches_major_only() {
        let v = PythonVersion::new(3, 11, 9);
        assert!(v.matches("3"));
        assert!(!v.matches("2"));
    }

    // ========== FromStr Tests ==========

    #[test]
    fn test_from_str() {
        let v: PythonVersion = "3.11.9".parse().unwrap();
        assert_eq!(v, PythonVersion::new(3, 11, 9));
    }
}
