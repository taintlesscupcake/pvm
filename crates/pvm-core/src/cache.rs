//! Global package cache management for deduplication

use crate::config::Config;
use crate::error::{PvmError, Result};
use crate::link::{LinkStats, LinkStrategy};
use crate::package::{CachedPackage, InstalledPackage, PackageId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Current metadata schema version
const METADATA_VERSION: u32 = 1;

/// Cache metadata stored in packages/metadata.json
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Schema version for migrations
    pub version: u32,
    /// Map of cache hash -> cached package info
    pub packages: HashMap<String, CachedPackage>,
    /// Total size of all cached packages
    pub total_size_bytes: u64,
    /// Total bytes saved through deduplication
    pub total_saved_bytes: u64,
}

impl CacheMetadata {
    /// Create a new empty metadata
    pub fn new() -> Self {
        Self {
            version: METADATA_VERSION,
            packages: HashMap::new(),
            total_size_bytes: 0,
            total_saved_bytes: 0,
        }
    }
}

/// Global package cache manager
pub struct PackageCache {
    config: Config,
    metadata: CacheMetadata,
    #[allow(dead_code)]
    link_strategy: LinkStrategy,
}

impl PackageCache {
    /// Create a new PackageCache
    pub fn new(config: Config) -> Result<Self> {
        let mut cache = Self {
            config,
            metadata: CacheMetadata::new(),
            link_strategy: LinkStrategy::default(),
        };
        cache.load_metadata()?;
        Ok(cache)
    }

    /// Create a PackageCache with a specific link strategy
    pub fn with_strategy(config: Config, strategy: LinkStrategy) -> Result<Self> {
        let mut cache = Self {
            config,
            metadata: CacheMetadata::new(),
            link_strategy: strategy,
        };
        cache.load_metadata()?;
        Ok(cache)
    }

    /// Get the packages directory path
    pub fn packages_dir(&self) -> PathBuf {
        self.config.packages_dir()
    }

    /// Get the store directory path
    pub fn store_dir(&self) -> PathBuf {
        self.packages_dir().join("store")
    }

    /// Get metadata file path
    pub fn metadata_path(&self) -> PathBuf {
        self.packages_dir().join("metadata.json")
    }

    /// Ensure cache directories exist
    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.packages_dir())?;
        fs::create_dir_all(self.store_dir())?;
        Ok(())
    }

    /// Load metadata from disk, or initialize if not exists
    pub fn load_metadata(&mut self) -> Result<()> {
        let path = self.metadata_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            self.metadata = serde_json::from_str(&content)?;
        } else {
            self.metadata = CacheMetadata::new();
        }
        Ok(())
    }

    /// Save metadata to disk atomically (write to temp file, then rename)
    pub fn save_metadata(&self) -> Result<()> {
        self.ensure_dirs()?;
        let path = self.metadata_path();
        let temp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(&self.metadata)?;

        // Write to temp file first
        fs::write(&temp_path, &content)?;

        // Atomically rename to final path
        fs::rename(&temp_path, &path)?;

        Ok(())
    }

    /// Check if a package is cached
    pub fn is_cached(&self, id: &PackageId) -> bool {
        let hash = id.cache_hash();
        self.metadata.packages.contains_key(&hash)
    }

    /// Get cache path for a package
    pub fn get_cache_path(&self, id: &PackageId) -> PathBuf {
        let hash = id.cache_hash();
        let prefix = id.cache_prefix();
        self.store_dir()
            .join(&prefix)
            .join(&hash)
            .join(id.cache_dir_name())
    }

    /// Get a cached package by ID
    pub fn get(&self, id: &PackageId) -> Option<&CachedPackage> {
        let hash = id.cache_hash();
        self.metadata.packages.get(&hash)
    }

    /// Add a package to the cache by moving files from site-packages
    pub fn add_package(&mut self, package: &InstalledPackage, id: &PackageId) -> Result<PathBuf> {
        self.ensure_dirs()?;

        let hash = id.cache_hash();
        let prefix = id.cache_prefix();
        // Use hash directory as base (all items go inside this directory)
        let cache_base = self.store_dir().join(&prefix).join(&hash);

        // If already cached, just increment reference
        if self.metadata.packages.contains_key(&hash) {
            let existing = self.metadata.packages.get_mut(&hash).unwrap();
            existing.add_reference();
            let cache_path = existing.cache_path.clone();
            self.save_metadata()?;
            return Ok(cache_path);
        }

        // Create cache directory structure
        fs::create_dir_all(&cache_base)?;

        // Collect names of items we're caching
        let mut pkg_items: Vec<String> = Vec::new();
        let mut total_size: u64 = 0;

        // Move all top-level items to cache (excluding __pycache__ which is shared)
        for item_path in &package.top_level_items {
            // Skip __pycache__ as it's a shared directory, not package-specific
            if item_path.file_name().map(|n| n == "__pycache__").unwrap_or(false) {
                continue;
            }
            if let Some(item_name) = item_path.file_name() {
                let item_name_str = item_name.to_string_lossy().to_string();
                let cache_item_path = cache_base.join(&item_name_str);

                if item_path.exists() {
                    // Use rename if same filesystem, otherwise copy and remove
                    if fs::rename(item_path, &cache_item_path).is_err() {
                        // Cross-filesystem: copy then remove
                        if item_path.is_dir() {
                            copy_dir_all(item_path, &cache_item_path)?;
                        } else {
                            fs::copy(item_path, &cache_item_path)?;
                        }
                        // Ignore remove errors (data is safely in cache)
                        if item_path.is_dir() {
                            let _ = fs::remove_dir_all(item_path);
                        } else {
                            let _ = fs::remove_file(item_path);
                        }
                    }

                    pkg_items.push(item_name_str);
                    total_size += dir_size(&cache_item_path)?;
                }
            }
        }

        // Also move the dist-info directory
        let dist_info_name = package.dist_info.file_name().unwrap_or_default();
        let cache_dist_info = cache_base.join(dist_info_name);
        if package.dist_info.exists() && fs::rename(&package.dist_info, &cache_dist_info).is_err() {
            copy_dir_all(&package.dist_info, &cache_dist_info)?;
            // Ignore remove errors (data is safely in cache)
            let _ = fs::remove_dir_all(&package.dist_info);
        }
        if cache_dist_info.exists() {
            total_size += dir_size(&cache_dist_info)?;
        }

        // Create cached package entry
        let cached = CachedPackage::new(
            id.clone(),
            cache_base.clone(),
            pkg_items,
            total_size,
            package.file_count(),
        );

        self.metadata.total_size_bytes += total_size;
        self.metadata.packages.insert(hash, cached);
        self.save_metadata()?;

        Ok(cache_base)
    }

    /// Link a cached package to a site-packages directory
    pub fn link_to_site_packages(
        &mut self,
        id: &PackageId,
        site_packages: &Path,
    ) -> Result<LinkStats> {
        let hash = id.cache_hash();

        let cached = self
            .metadata
            .packages
            .get(&hash)
            .ok_or_else(|| PvmError::PackageNotCached(id.to_string()))?
            .clone();

        // Determine best link strategy
        let strategy = LinkStrategy::detect(&cached.cache_path, site_packages);

        let mut stats = LinkStats::default();

        // Determine items to link (use pkg_items if available, fallback to pkg_dir_name for legacy)
        let items_to_link: Vec<String> = if !cached.pkg_items.is_empty() {
            cached.pkg_items.clone()
        } else if !cached.pkg_dir_name.is_empty() {
            vec![cached.pkg_dir_name.clone()]
        } else {
            // Fallback: read items from cache directory
            fs::read_dir(&cached.cache_path)?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|name| !name.ends_with(".dist-info"))
                .collect()
        };

        // Link each top-level item
        for item_name in &items_to_link {
            let cache_item_path = cached.cache_path.join(item_name);
            let target_path = site_packages.join(item_name);

            if cache_item_path.exists() {
                if cache_item_path.is_dir() {
                    let item_stats = strategy.link_directory(&cache_item_path, &target_path)?;
                    stats.merge(&item_stats);
                } else {
                    // Single file (like _black_version.py or .so files)
                    strategy.link_file(&cache_item_path, &target_path)?;
                    let file_size = fs::metadata(&cache_item_path)?.len();
                    stats.linked_files += 1;
                    stats.linked_bytes += file_size;
                }
            }
        }

        // Link the dist-info directory
        let dist_info_name = format!("{}-{}.dist-info", id.name, id.version);
        let cache_dist_info = cached.cache_path.join(&dist_info_name);
        if cache_dist_info.exists() {
            let target_dist_info = site_packages.join(&dist_info_name);
            let dist_stats = strategy.link_directory(&cache_dist_info, &target_dist_info)?;
            stats.merge(&dist_stats);
        }

        // Update reference count and saved bytes
        if let Some(pkg) = self.metadata.packages.get_mut(&hash) {
            pkg.add_reference();
        }
        self.metadata.total_saved_bytes += stats.linked_bytes;
        self.save_metadata()?;

        Ok(stats)
    }

    /// Remove reference from a package (when env is deleted)
    pub fn remove_reference(&mut self, id: &PackageId) -> Result<()> {
        let hash = id.cache_hash();
        if let Some(pkg) = self.metadata.packages.get_mut(&hash) {
            pkg.remove_reference();
            self.save_metadata()?;
        }
        Ok(())
    }

    /// Clean up orphaned packages (reference_count == 0)
    pub fn garbage_collect(&mut self) -> Result<GCStats> {
        let mut stats = GCStats::default();

        // Find orphaned packages
        let orphans: Vec<String> = self
            .metadata
            .packages
            .iter()
            .filter(|(_, pkg)| pkg.is_orphan())
            .map(|(hash, _)| hash.clone())
            .collect();

        for hash in orphans {
            if let Some(pkg) = self.metadata.packages.remove(&hash) {
                // Remove the cached files
                if pkg.cache_path.exists() {
                    if let Some(parent) = pkg.cache_path.parent() {
                        // Remove the hash directory (contains package and dist-info)
                        fs::remove_dir_all(parent)?;
                    }
                }

                stats.removed_packages += 1;
                stats.freed_bytes += pkg.size_bytes;
                self.metadata.total_size_bytes =
                    self.metadata.total_size_bytes.saturating_sub(pkg.size_bytes);
            }
        }

        if stats.removed_packages > 0 {
            self.save_metadata()?;
        }

        Ok(stats)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let unique_packages: std::collections::HashSet<_> = self
            .metadata
            .packages
            .values()
            .map(|p| (&p.id.name, &p.id.version))
            .collect();

        CacheStats {
            total_packages: self.metadata.packages.len(),
            unique_packages: unique_packages.len(),
            total_size_bytes: self.metadata.total_size_bytes,
            saved_bytes: self.metadata.total_saved_bytes,
        }
    }

    /// List all cached packages
    pub fn list(&self) -> Vec<&CachedPackage> {
        self.metadata.packages.values().collect()
    }

    /// Clear all cached packages
    pub fn clear(&mut self) -> Result<CacheStats> {
        let stats = self.stats();

        // Remove store directory
        let store = self.store_dir();
        if store.exists() {
            fs::remove_dir_all(&store)?;
        }

        // Reset metadata
        self.metadata = CacheMetadata::new();
        self.save_metadata()?;

        Ok(stats)
    }
}

/// Statistics about the cache
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    /// Total number of cached package entries
    pub total_packages: usize,
    /// Number of unique package name+version combinations
    pub unique_packages: usize,
    /// Total size of cached packages in bytes
    pub total_size_bytes: u64,
    /// Total bytes saved through deduplication
    pub saved_bytes: u64,
}

/// Statistics from garbage collection
#[derive(Debug, Default, Clone)]
pub struct GCStats {
    /// Number of packages removed
    pub removed_packages: usize,
    /// Total bytes freed
    pub freed_bytes: u64,
}

/// Helper: Copy a directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

/// Helper: Calculate directory size
fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0;
    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_file() {
                size += meta.len();
            } else if meta.is_dir() {
                size += dir_size(&entry.path())?;
            }
        }
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp: &TempDir) -> Config {
        Config {
            home: temp.path().to_path_buf(),
            pythons_dir: None,
            envs_dir: None,
            cache_dir: None,
            packages_dir: None,
            dedup: Default::default(),
        }
    }

    fn create_test_package(temp: &TempDir, name: &str, version: &str) -> InstalledPackage {
        let site_packages = temp.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        let pkg_dir = site_packages.join(name);
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("__init__.py"), "# test package").unwrap();
        fs::write(pkg_dir.join("module.py"), "def hello(): pass").unwrap();

        let dist_info = site_packages.join(format!("{}-{}.dist-info", name, version));
        fs::create_dir_all(&dist_info).unwrap();
        fs::write(
            dist_info.join("METADATA"),
            format!("Name: {}\nVersion: {}\n", name, version),
        )
        .unwrap();

        InstalledPackage {
            name: name.to_string(),
            version: version.to_string(),
            location: pkg_dir.clone(),
            dist_info,
            files: vec![
                pkg_dir.join("__init__.py"),
                pkg_dir.join("module.py"),
            ],
            top_level_items: vec![pkg_dir.clone()],
        }
    }

    #[test]
    fn test_cache_new() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let cache = PackageCache::new(config).unwrap();

        assert_eq!(cache.stats().total_packages, 0);
    }

    #[test]
    fn test_add_and_check_package() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut cache = PackageCache::new(config).unwrap();

        let pkg = create_test_package(&temp, "numpy", "1.26.0");
        let id = pkg.to_package_id("3.12", "aarch64-apple-darwin");

        assert!(!cache.is_cached(&id));

        cache.add_package(&pkg, &id).unwrap();

        assert!(cache.is_cached(&id));
        assert_eq!(cache.stats().total_packages, 1);
    }

    #[test]
    fn test_link_to_site_packages() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut cache = PackageCache::new(config).unwrap();

        // Create and cache a package
        let pkg = create_test_package(&temp, "requests", "2.31.0");
        let id = pkg.to_package_id("3.12", "aarch64-apple-darwin");
        cache.add_package(&pkg, &id).unwrap();

        // Create a new site-packages directory
        let new_site_packages = temp.path().join("new-env").join("site-packages");
        fs::create_dir_all(&new_site_packages).unwrap();

        // Link the cached package
        let stats = cache
            .link_to_site_packages(&id, &new_site_packages)
            .unwrap();

        assert!(stats.total_files() > 0);
        // Package directory is linked with original name
        assert!(new_site_packages.join("requests").exists());
        // dist-info is also linked
        assert!(new_site_packages.join("requests-2.31.0.dist-info").exists());
    }

    #[test]
    fn test_garbage_collection() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut cache = PackageCache::new(config).unwrap();

        let pkg = create_test_package(&temp, "flask", "3.0.0");
        let id = pkg.to_package_id("3.12", "aarch64-apple-darwin");
        cache.add_package(&pkg, &id).unwrap();

        // Remove reference to make it orphan
        cache.remove_reference(&id).unwrap();
        assert!(cache.get(&id).unwrap().is_orphan());

        // Run GC
        let gc_stats = cache.garbage_collect().unwrap();

        assert_eq!(gc_stats.removed_packages, 1);
        assert!(!cache.is_cached(&id));
    }

    #[test]
    fn test_reference_counting() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);
        let mut cache = PackageCache::new(config).unwrap();

        let pkg = create_test_package(&temp, "django", "4.2.0");
        let id = pkg.to_package_id("3.12", "aarch64-apple-darwin");
        cache.add_package(&pkg, &id).unwrap();

        // Initial reference count is 1
        assert_eq!(cache.get(&id).unwrap().reference_count, 1);

        // Create site-packages for linking (simulates using package in another env)
        let site_packages = temp.path().join("env2").join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();
        cache.link_to_site_packages(&id, &site_packages).unwrap();

        // Reference count should be 2
        assert_eq!(cache.get(&id).unwrap().reference_count, 2);

        // Remove one reference
        cache.remove_reference(&id).unwrap();
        assert_eq!(cache.get(&id).unwrap().reference_count, 1);

        // Package still not orphan
        assert!(!cache.get(&id).unwrap().is_orphan());
    }

    #[test]
    fn test_metadata_persistence() {
        let temp = TempDir::new().unwrap();
        let config = create_test_config(&temp);

        // Add a package
        {
            let mut cache = PackageCache::new(config.clone()).unwrap();
            let pkg = create_test_package(&temp, "pytest", "7.4.0");
            let id = pkg.to_package_id("3.12", "aarch64-apple-darwin");
            cache.add_package(&pkg, &id).unwrap();
        }

        // Create new cache instance and verify persistence
        {
            let cache = PackageCache::new(config).unwrap();
            assert_eq!(cache.stats().total_packages, 1);

            let id = PackageId::new("pytest", "7.4.0", "3.12", "aarch64-apple-darwin");
            assert!(cache.is_cached(&id));
        }
    }
}
