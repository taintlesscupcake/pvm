//! Package metadata types for deduplication

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// Unique identifier for a cached package
///
/// Packages are identified by their name, version, Python version, and platform.
/// This ensures that packages compiled for different Python versions or platforms
/// are stored separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PackageId {
    /// Normalized package name (lowercase, hyphens replaced with underscores)
    pub name: String,
    /// Package version (e.g., "1.26.0")
    pub version: String,
    /// Python version major.minor (e.g., "3.12")
    pub python_version: String,
    /// Platform identifier (e.g., "aarch64-apple-darwin")
    pub platform: String,
}

impl PackageId {
    /// Create a new PackageId
    pub fn new(name: &str, version: &str, python_version: &str, platform: &str) -> Self {
        Self {
            name: Self::normalize_name(name),
            version: version.to_string(),
            python_version: python_version.to_string(),
            platform: platform.to_string(),
        }
    }

    /// Normalize package name according to PEP 503
    /// - Convert to lowercase
    /// - Replace hyphens, underscores, and periods with a single underscore
    pub fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c == '-' || c == '.' { '_' } else { c })
            .collect()
    }

    /// Generate content-addressable hash for cache storage
    pub fn cache_hash(&self) -> String {
        let input = format!(
            "{}|{}|{}|{}",
            self.name, self.version, self.python_version, self.platform
        );
        let hash = Sha256::digest(input.as_bytes());
        format!("{:x}", hash)
    }

    /// Get cache path prefix (first 2 chars of hash for sharding)
    pub fn cache_prefix(&self) -> String {
        self.cache_hash()[..2].to_string()
    }

    /// Get the directory name for this package in the cache
    pub fn cache_dir_name(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}=={} (py{}, {})",
            self.name, self.version, self.python_version, self.platform
        )
    }
}

/// Metadata for a package stored in the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPackage {
    /// Package identifier
    pub id: PackageId,
    /// Path to the package in the cache store
    pub cache_path: PathBuf,
    /// Total size of the package in bytes
    pub size_bytes: u64,
    /// Number of files in the package
    pub file_count: usize,
    /// When the package was cached
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// Last time the package was used (linked to an env)
    pub last_used: chrono::DateTime<chrono::Utc>,
    /// Number of environments using this package
    pub reference_count: usize,
}

impl CachedPackage {
    /// Create a new CachedPackage
    pub fn new(id: PackageId, cache_path: PathBuf, size_bytes: u64, file_count: usize) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            cache_path,
            size_bytes,
            file_count,
            cached_at: now,
            last_used: now,
            reference_count: 1,
        }
    }

    /// Update last_used timestamp
    pub fn touch(&mut self) {
        self.last_used = chrono::Utc::now();
    }

    /// Increment reference count
    pub fn add_reference(&mut self) {
        self.reference_count += 1;
        self.touch();
    }

    /// Decrement reference count
    pub fn remove_reference(&mut self) {
        self.reference_count = self.reference_count.saturating_sub(1);
    }

    /// Check if package is unreferenced
    pub fn is_orphan(&self) -> bool {
        self.reference_count == 0
    }
}

/// Information about a package installed in a virtual environment
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    /// Package name (from METADATA)
    pub name: String,
    /// Package version (from METADATA)
    pub version: String,
    /// Path to the package directory in site-packages
    pub location: PathBuf,
    /// Path to the .dist-info directory
    pub dist_info: PathBuf,
    /// List of all files belonging to this package
    pub files: Vec<PathBuf>,
}

impl InstalledPackage {
    /// Calculate total size of the package
    pub fn total_size(&self) -> u64 {
        self.files
            .iter()
            .filter_map(|f| std::fs::metadata(f).ok())
            .map(|m| m.len())
            .sum()
    }

    /// Get the number of files in the package
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Create a PackageId from this installed package
    pub fn to_package_id(&self, python_version: &str, platform: &str) -> PackageId {
        PackageId::new(&self.name, &self.version, python_version, platform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(PackageId::normalize_name("NumPy"), "numpy");
        assert_eq!(PackageId::normalize_name("my-package"), "my_package");
        assert_eq!(PackageId::normalize_name("some.pkg"), "some_pkg");
        assert_eq!(
            PackageId::normalize_name("Mixed-Case.Package"),
            "mixed_case_package"
        );
    }

    #[test]
    fn test_package_id_hash() {
        let id1 = PackageId::new("numpy", "1.26.0", "3.12", "aarch64-apple-darwin");
        let id2 = PackageId::new("numpy", "1.26.0", "3.12", "aarch64-apple-darwin");
        let id3 = PackageId::new("numpy", "1.26.0", "3.11", "aarch64-apple-darwin");

        // Same inputs produce same hash
        assert_eq!(id1.cache_hash(), id2.cache_hash());
        // Different Python version produces different hash
        assert_ne!(id1.cache_hash(), id3.cache_hash());
    }

    #[test]
    fn test_cache_prefix() {
        let id = PackageId::new("numpy", "1.26.0", "3.12", "aarch64-apple-darwin");
        let prefix = id.cache_prefix();
        assert_eq!(prefix.len(), 2);
        // Prefix is first 2 chars of hash
        assert!(id.cache_hash().starts_with(&prefix));
    }

    #[test]
    fn test_cached_package_reference_counting() {
        let id = PackageId::new("numpy", "1.26.0", "3.12", "aarch64-apple-darwin");
        let mut cached = CachedPackage::new(id, PathBuf::from("/cache/numpy"), 1000, 10);

        assert_eq!(cached.reference_count, 1);
        assert!(!cached.is_orphan());

        cached.add_reference();
        assert_eq!(cached.reference_count, 2);

        cached.remove_reference();
        cached.remove_reference();
        assert_eq!(cached.reference_count, 0);
        assert!(cached.is_orphan());

        // Should not go below 0
        cached.remove_reference();
        assert_eq!(cached.reference_count, 0);
    }

    #[test]
    fn test_package_id_display() {
        let id = PackageId::new("numpy", "1.26.0", "3.12", "aarch64-apple-darwin");
        let display = format!("{}", id);
        assert!(display.contains("numpy"));
        assert!(display.contains("1.26.0"));
        assert!(display.contains("3.12"));
    }
}
