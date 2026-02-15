//! Atlas Package Management (phase-07)
//!
//! Package manifest system for atlas.toml files, dependency management,
//! lockfile generation, and workspace support.

pub mod lockfile;
pub mod manifest;
pub mod validator;

pub use lockfile::{Lockfile, LockedPackage, LockedSource, Resolver};
pub use manifest::{
    Dependency, DependencySource, Feature, PackageManifest, VersionConstraint, Workspace,
};
pub use validator::{ValidationError, Validator};

/// Package management errors
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Failed to parse manifest: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize manifest: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Semver error: {0}")]
    SemverError(#[from] semver::Error),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field} - {reason}")]
    InvalidField { field: String, reason: String },
}

pub type Result<T> = std::result::Result<T, PackageError>;
