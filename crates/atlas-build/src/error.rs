/// Build system error types
use std::path::PathBuf;
use thiserror::Error;

pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Failed to read manifest at {path}: {error}")]
    ManifestReadError { path: PathBuf, error: String },

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Compilation failed for module '{module}': {error}")]
    CompilationError { module: String, error: String },

    #[error("Missing entry point for binary target '{target}': expected main() function")]
    MissingEntryPoint { target: String },

    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionError(String),

    #[error("Module not found: {module}")]
    ModuleNotFound { module: String },

    #[error("Target not found: {target}")]
    TargetNotFound { target: String },

    #[error("Invalid target configuration: {0}")]
    InvalidTarget(String),

    #[error("Build cache error: {0}")]
    CacheError(String),

    #[error("I/O error at {path}: {error}")]
    IoError {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Package error: {0}")]
    PackageError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),
}

impl BuildError {
    /// Create a manifest read error
    pub fn manifest_read(path: impl Into<PathBuf>, error: impl ToString) -> Self {
        Self::ManifestReadError {
            path: path.into(),
            error: error.to_string(),
        }
    }

    /// Create an I/O error with path context
    pub fn io(path: impl Into<PathBuf>, error: std::io::Error) -> Self {
        Self::IoError {
            path: path.into(),
            error,
        }
    }

    /// Create a compilation error
    pub fn compilation(module: impl Into<String>, error: impl ToString) -> Self {
        Self::CompilationError {
            module: module.into(),
            error: error.to_string(),
        }
    }

    /// Create a missing entry point error
    pub fn missing_entry_point(target: impl Into<String>) -> Self {
        Self::MissingEntryPoint {
            target: target.into(),
        }
    }

    /// Create a module not found error
    pub fn module_not_found(module: impl Into<String>) -> Self {
        Self::ModuleNotFound {
            module: module.into(),
        }
    }
}
