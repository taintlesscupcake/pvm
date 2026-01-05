//! Error types for pvm-core

use thiserror::Error;

/// Result type alias for pvm-core operations
pub type Result<T> = std::result::Result<T, PvmError>;

/// Main error type for pvm-core
#[derive(Debug, Error)]
pub enum PvmError {
    #[error("Invalid Python version format: {0}")]
    InvalidVersion(String),

    #[error("Python version not found: {0}")]
    VersionNotFound(String),

    #[error("Virtual environment not found: {0}")]
    EnvNotFound(String),

    #[error("Virtual environment already exists: {0}")]
    EnvAlreadyExists(String),

    #[error("Failed to download: {0}")]
    DownloadError(String),

    #[error("Failed to extract archive: {0}")]
    ExtractError(String),

    #[error("Checksum verification failed")]
    ChecksumMismatch,

    #[error("Unsupported platform: {os} {arch}")]
    UnsupportedPlatform { os: String, arch: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Package not found in cache: {0}")]
    PackageNotCached(String),

    #[error("Failed to create link: {0}")]
    LinkError(String),

    #[error("pip command failed: {0}")]
    PipError(String),

    #[error("Cache corruption detected: {0}")]
    CacheCorruption(String),

    #[error("Migration failed: {0}")]
    MigrationError(String),

    #[error("Source environment not found: {0}")]
    SourceEnvNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PvmError::InvalidVersion("abc".to_string());
        assert_eq!(err.to_string(), "Invalid Python version format: abc");

        let err = PvmError::EnvNotFound("myenv".to_string());
        assert_eq!(err.to_string(), "Virtual environment not found: myenv");

        let err = PvmError::UnsupportedPlatform {
            os: "unknown".to_string(),
            arch: "unknown".to_string(),
        };
        assert_eq!(err.to_string(), "Unsupported platform: unknown unknown");
    }
}
