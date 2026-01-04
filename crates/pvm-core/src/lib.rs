//! pvm-core: Core library for PVM (Python Version Manager)
//!
//! This crate provides the core functionality for managing Python versions
//! and virtual environments.

pub mod error;
pub mod version;
pub mod platform;
pub mod config;
pub mod downloader;
pub mod installer;
pub mod venv;

// Package deduplication modules
pub mod cache;
pub mod link;
pub mod package;
pub mod pip_wrapper;

pub use error::{PvmError, Result};
pub use version::PythonVersion;
pub use platform::Platform;
pub use config::{Config, DedupConfig, GeneralConfig, ShellConfig};
pub use downloader::Downloader;
pub use installer::Installer;
pub use venv::VenvManager;

// Package deduplication exports
pub use cache::{CacheStats, GCStats, PackageCache};
pub use link::{LinkStats, LinkStrategy};
pub use package::{CachedPackage, InstalledPackage, PackageId};
pub use pip_wrapper::{InstallResult, PipWrapper};
