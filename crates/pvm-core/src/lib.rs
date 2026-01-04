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

pub use error::{PvmError, Result};
pub use version::PythonVersion;
pub use platform::Platform;
pub use config::Config;
pub use downloader::Downloader;
pub use installer::Installer;
pub use venv::VenvManager;
