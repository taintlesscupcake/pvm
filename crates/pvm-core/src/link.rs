//! File linking strategies for package deduplication

use crate::error::{PvmError, Result};
use std::fs;
use std::path::Path;

/// Strategy for linking files from cache to site-packages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkStrategy {
    /// Use hardlinks (default on Linux/Windows)
    /// Most efficient when cache and target are on the same filesystem
    #[default]
    Hardlink,
    /// Use reflinks/clones (Copy-on-Write)
    /// Efficient on APFS (macOS) and Btrfs (Linux)
    Clone,
    /// Fall back to copying files
    /// Works across filesystems but uses more disk space
    Copy,
}

impl LinkStrategy {
    /// Detect the best strategy for the current platform and paths
    pub fn detect(source: &Path, target_dir: &Path) -> Self {
        // Check if source and target are on the same filesystem
        if !Self::same_filesystem(source, target_dir) {
            return Self::Copy;
        }

        // Prefer hardlinks on all platforms for maximum deduplication
        // Hardlinks share inodes and provide true disk space savings
        Self::Hardlink
    }

    /// Check if two paths are on the same filesystem
    #[cfg(unix)]
    fn same_filesystem(a: &Path, b: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;

        let a_meta = match fs::metadata(a) {
            Ok(m) => m,
            Err(_) => return false,
        };

        // For target, check parent directory (target may not exist yet)
        let b_check = if b.exists() {
            b.to_path_buf()
        } else {
            b.parent().unwrap_or(b).to_path_buf()
        };

        let b_meta = match fs::metadata(&b_check) {
            Ok(m) => m,
            Err(_) => return false,
        };

        a_meta.dev() == b_meta.dev()
    }

    #[cfg(not(unix))]
    fn same_filesystem(_a: &Path, _b: &Path) -> bool {
        // On non-Unix systems, assume same filesystem
        // Windows would need different logic
        true
    }

    /// Link a single file from source to target
    pub fn link_file(&self, source: &Path, target: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        match self {
            Self::Hardlink => self.hardlink_file(source, target),
            Self::Clone => self.clone_file(source, target),
            Self::Copy => self.copy_file(source, target),
        }
    }

    /// Create a hardlink
    fn hardlink_file(&self, source: &Path, target: &Path) -> Result<()> {
        fs::hard_link(source, target).map_err(|e| {
            if e.raw_os_error() == Some(libc::EXDEV) {
                PvmError::LinkError(format!(
                    "Cannot hardlink across filesystems: {} -> {}",
                    source.display(),
                    target.display()
                ))
            } else {
                PvmError::Io(e)
            }
        })
    }

    /// Clone a file (Copy-on-Write)
    #[cfg(target_os = "macos")]
    fn clone_file(&self, source: &Path, target: &Path) -> Result<()> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let src = CString::new(source.as_os_str().as_bytes())
            .map_err(|e| PvmError::LinkError(format!("Invalid source path: {}", e)))?;
        let dst = CString::new(target.as_os_str().as_bytes())
            .map_err(|e| PvmError::LinkError(format!("Invalid target path: {}", e)))?;

        // clonefile(src, dst, 0) - CLONE_NOFOLLOW = 0x0001
        let result = unsafe { libc::clonefile(src.as_ptr(), dst.as_ptr(), 0) };

        if result == 0 {
            Ok(())
        } else {
            // Fall back to copy if clone fails
            self.copy_file(source, target)
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn clone_file(&self, source: &Path, target: &Path) -> Result<()> {
        // On non-macOS, fall back to copy
        self.copy_file(source, target)
    }

    /// Copy a file
    fn copy_file(&self, source: &Path, target: &Path) -> Result<()> {
        fs::copy(source, target)?;
        Ok(())
    }

    /// Link an entire directory tree recursively
    pub fn link_directory(&self, source: &Path, target: &Path) -> Result<LinkStats> {
        let mut stats = LinkStats::default();
        self.link_directory_recursive(source, target, &mut stats)?;
        Ok(stats)
    }

    fn link_directory_recursive(
        &self,
        source: &Path,
        target: &Path,
        stats: &mut LinkStats,
    ) -> Result<()> {
        fs::create_dir_all(target)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = target.join(entry.file_name());

            if src_path.is_dir() {
                self.link_directory_recursive(&src_path, &dst_path, stats)?;
            } else if src_path.is_file() {
                let file_size = entry.metadata()?.len();

                match self.link_file(&src_path, &dst_path) {
                    Ok(_) => {
                        if *self == Self::Copy {
                            stats.copied_files += 1;
                            stats.copied_bytes += file_size;
                        } else {
                            stats.linked_files += 1;
                            stats.linked_bytes += file_size;
                        }
                    }
                    Err(_) => {
                        // Fallback to copy on error
                        fs::copy(&src_path, &dst_path)?;
                        stats.copied_files += 1;
                        stats.copied_bytes += file_size;
                    }
                }
            }
            // Skip symlinks and other special files
        }

        Ok(())
    }

    /// Remove a directory that contains hardlinks (safe removal)
    pub fn remove_linked_directory(path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for LinkStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hardlink => write!(f, "hardlink"),
            Self::Clone => write!(f, "clone"),
            Self::Copy => write!(f, "copy"),
        }
    }
}

impl std::str::FromStr for LinkStrategy {
    type Err = PvmError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hardlink" | "hard" => Ok(Self::Hardlink),
            "clone" | "cow" | "reflink" => Ok(Self::Clone),
            "copy" => Ok(Self::Copy),
            "auto" => Ok(Self::default()),
            _ => Err(PvmError::ConfigError(format!(
                "Invalid link strategy: {}. Valid options: hardlink, clone, copy, auto",
                s
            ))),
        }
    }
}

/// Statistics from a link operation
#[derive(Debug, Default, Clone)]
pub struct LinkStats {
    /// Number of files successfully linked (hardlink or clone)
    pub linked_files: usize,
    /// Total bytes in linked files
    pub linked_bytes: u64,
    /// Number of files that fell back to copy
    pub copied_files: usize,
    /// Total bytes in copied files
    pub copied_bytes: u64,
}

impl LinkStats {
    /// Total number of files processed
    pub fn total_files(&self) -> usize {
        self.linked_files + self.copied_files
    }

    /// Total bytes processed
    pub fn total_bytes(&self) -> u64 {
        self.linked_bytes + self.copied_bytes
    }

    /// Bytes saved by linking (not copying)
    pub fn saved_bytes(&self) -> u64 {
        self.linked_bytes
    }

    /// Merge another stats into this one
    pub fn merge(&mut self, other: &LinkStats) {
        self.linked_files += other.linked_files;
        self.linked_bytes += other.linked_bytes;
        self.copied_files += other.copied_files;
        self.copied_bytes += other.copied_bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_link_strategy_from_str() {
        assert_eq!(
            "hardlink".parse::<LinkStrategy>().unwrap(),
            LinkStrategy::Hardlink
        );
        assert_eq!(
            "clone".parse::<LinkStrategy>().unwrap(),
            LinkStrategy::Clone
        );
        assert_eq!(
            "copy".parse::<LinkStrategy>().unwrap(),
            LinkStrategy::Copy
        );
        assert_eq!(
            "auto".parse::<LinkStrategy>().unwrap(),
            LinkStrategy::Hardlink
        ); // Default

        assert!("invalid".parse::<LinkStrategy>().is_err());
    }

    #[test]
    fn test_link_strategy_display() {
        assert_eq!(format!("{}", LinkStrategy::Hardlink), "hardlink");
        assert_eq!(format!("{}", LinkStrategy::Clone), "clone");
        assert_eq!(format!("{}", LinkStrategy::Copy), "copy");
    }

    #[test]
    fn test_copy_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("dest.txt");

        fs::write(&src, "hello world").unwrap();

        LinkStrategy::Copy.link_file(&src, &dst).unwrap();

        assert!(dst.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello world");
    }

    #[test]
    fn test_hardlink_file() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("source.txt");
        let dst = temp.path().join("dest.txt");

        fs::write(&src, "hello world").unwrap();

        LinkStrategy::Hardlink.link_file(&src, &dst).unwrap();

        assert!(dst.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "hello world");

        // Verify it's a hardlink (same inode on Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let src_meta = fs::metadata(&src).unwrap();
            let dst_meta = fs::metadata(&dst).unwrap();
            assert_eq!(src_meta.ino(), dst_meta.ino());
        }
    }

    #[test]
    fn test_link_directory() {
        let temp = TempDir::new().unwrap();
        let src_dir = temp.path().join("source");
        let dst_dir = temp.path().join("dest");

        // Create source structure
        fs::create_dir_all(src_dir.join("subdir")).unwrap();
        fs::write(src_dir.join("file1.txt"), "content1").unwrap();
        fs::write(src_dir.join("subdir/file2.txt"), "content2").unwrap();

        let stats = LinkStrategy::Hardlink
            .link_directory(&src_dir, &dst_dir)
            .unwrap();

        assert!(dst_dir.join("file1.txt").exists());
        assert!(dst_dir.join("subdir/file2.txt").exists());
        assert_eq!(stats.total_files(), 2);
    }

    #[test]
    fn test_link_stats_merge() {
        let mut stats1 = LinkStats {
            linked_files: 5,
            linked_bytes: 1000,
            copied_files: 1,
            copied_bytes: 100,
        };

        let stats2 = LinkStats {
            linked_files: 3,
            linked_bytes: 500,
            copied_files: 2,
            copied_bytes: 200,
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.linked_files, 8);
        assert_eq!(stats1.linked_bytes, 1500);
        assert_eq!(stats1.copied_files, 3);
        assert_eq!(stats1.copied_bytes, 300);
    }

    #[test]
    fn test_same_filesystem() {
        let temp = TempDir::new().unwrap();
        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        // Same temp directory should be same filesystem
        assert!(LinkStrategy::same_filesystem(&dir1, &dir2));
    }
}
