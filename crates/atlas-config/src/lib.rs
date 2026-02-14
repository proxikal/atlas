//! Atlas Configuration System
//!
//! Provides configuration management for Atlas projects including:
//! - Project configuration (atlas.toml)
//! - Global user configuration (~/.atlas/config.toml)
//! - Package manifests
//! - Configuration precedence and merging
//!
//! # Configuration Hierarchy
//!
//! Configuration is loaded and merged in the following order (later overrides earlier):
//! 1. Global config (~/.atlas/config.toml)
//! 2. Project config (./atlas.toml)
//! 3. Environment variables (ATLAS_*)
//! 4. CLI flags
//!
//! # Example
//!
//! ```no_run
//! use atlas_config::ConfigLoader;
//! use std::path::Path;
//!
//! let mut loader = ConfigLoader::new();
//! let config = loader.load_from_directory(Path::new(".")).unwrap();
//! ```

pub mod global;
pub mod loader;
pub mod manifest;
pub mod project;

use std::path::PathBuf;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read configuration file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid TOML syntax in {file}: {error}")]
    TomlParseError {
        file: PathBuf,
        error: toml::de::Error,
    },

    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    #[error("Unknown field '{field}' in {file}")]
    UnknownField { field: String, file: PathBuf },

    #[error("Missing required field '{field}' in {file}")]
    MissingField { field: String, file: PathBuf },

    #[error("Invalid value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Invalid semver version: {0}")]
    InvalidVersion(String),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Home directory not found")]
    HomeNotFound,
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

// Re-export main types
pub use global::GlobalConfig;
pub use loader::ConfigLoader;
pub use manifest::Manifest;
pub use project::ProjectConfig;
