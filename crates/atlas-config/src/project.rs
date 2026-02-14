//! Project Configuration (atlas.toml)
//!
//! Handles project-level configuration stored in `atlas.toml` at the project root.

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Project configuration from atlas.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Package metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageConfig>,

    /// Build configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,

    /// Compiler configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<CompilerConfig>,

    /// Formatting configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<FormattingConfig>,

    /// Dependencies
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, DependencySpec>,

    /// Development dependencies
    #[serde(default, rename = "dev-dependencies")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dev_dependencies: HashMap<String, DependencySpec>,
}

/// Package metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackageConfig {
    /// Package name
    pub name: String,

    /// Package version (semver)
    pub version: String,

    /// Atlas edition (e.g., "2026")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,

    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Package authors
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,

    /// License identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BuildConfig {
    /// Output directory (default: "target")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,

    /// Source directory (default: "src")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PathBuf>,

    /// Entry point file (default: "src/main.atl")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<PathBuf>,
}

/// Compiler configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct CompilerConfig {
    /// Enable optimizations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimize: Option<bool>,

    /// Target (interpreter, bytecode, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Enable debug info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
}

/// Formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FormattingConfig {
    /// Indentation size (default: 4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent: Option<usize>,

    /// Maximum line length (default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_line_length: Option<usize>,

    /// Use tabs instead of spaces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version string (e.g., "1.0")
    Version(String),

    /// Detailed dependency spec
    Detailed {
        /// Version requirement
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,

        /// Git repository URL
        #[serde(skip_serializing_if = "Option::is_none")]
        git: Option<String>,

        /// Local path
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,

        /// Registry name
        #[serde(skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
    },
}

impl ProjectConfig {
    /// Load project configuration from a file
    pub fn load_from_file(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ConfigError::NotFound(path.to_path_buf())
            } else {
                ConfigError::IoError(e)
            }
        })?;

        let config: Self = toml::from_str(&content).map_err(|e| ConfigError::TomlParseError {
            file: path.to_path_buf(),
            error: e,
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the project configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate package config if present
        if let Some(pkg) = &self.package {
            if pkg.name.is_empty() {
                return Err(ConfigError::InvalidValue {
                    field: "package.name".to_string(),
                    reason: "name cannot be empty".to_string(),
                });
            }

            // Basic semver validation (just check format, not full parsing)
            if !is_valid_version(&pkg.version) {
                return Err(ConfigError::InvalidVersion(pkg.version.clone()));
            }

            // Validate edition if present
            if let Some(edition) = &pkg.edition {
                if !is_valid_edition(edition) {
                    return Err(ConfigError::InvalidValue {
                        field: "package.edition".to_string(),
                        reason: format!("invalid edition '{}'", edition),
                    });
                }
            }
        }

        // Note: Build paths are validated relative to project root at runtime,
        // not during config parsing (no validation needed here)

        // Validate dependencies
        for (name, spec) in &self.dependencies {
            validate_dependency(name, spec)?;
        }

        for (name, spec) in &self.dev_dependencies {
            validate_dependency(name, spec)?;
        }

        Ok(())
    }

    /// Get the package name, if present
    pub fn package_name(&self) -> Option<&str> {
        self.package.as_ref().map(|p| p.name.as_str())
    }

    /// Get the package version, if present
    pub fn package_version(&self) -> Option<&str> {
        self.package.as_ref().map(|p| p.version.as_str())
    }

    /// Get the edition, if present
    pub fn edition(&self) -> Option<&str> {
        self.package.as_ref().and_then(|p| p.edition.as_deref())
    }

    /// Merge another project config into this one
    /// Other config takes precedence for non-None values
    pub fn merge(&mut self, other: &ProjectConfig) {
        if other.package.is_some() {
            self.package = other.package.clone();
        }
        if other.build.is_some() {
            self.build = other.build.clone();
        }
        if other.compiler.is_some() {
            self.compiler = other.compiler.clone();
        }
        if other.formatting.is_some() {
            self.formatting = other.formatting.clone();
        }
        if !other.dependencies.is_empty() {
            self.dependencies.extend(other.dependencies.clone());
        }
        if !other.dev_dependencies.is_empty() {
            self.dev_dependencies.extend(other.dev_dependencies.clone());
        }
    }
}

/// Basic semver validation (simplified)
fn is_valid_version(version: &str) -> bool {
    if version.is_empty() {
        return false;
    }

    // Split on '-' or '+' to separate version from pre-release/build
    let main_version = version.split(['-', '+']).next().unwrap_or("");

    if main_version.is_empty() {
        return false;
    }

    // Main version should be X.Y or X.Y.Z where X, Y, Z are digits
    let parts: Vec<&str> = main_version.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    // All main version parts must be non-empty digits
    parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Check if edition is valid
fn is_valid_edition(edition: &str) -> bool {
    matches!(edition, "2026" | "2027" | "2028") // Future-proof
}

/// Validate a dependency specification
fn validate_dependency(name: &str, spec: &DependencySpec) -> ConfigResult<()> {
    if name.is_empty() {
        return Err(ConfigError::InvalidValue {
            field: "dependency name".to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }

    match spec {
        DependencySpec::Version(v) => {
            if v.is_empty() {
                return Err(ConfigError::InvalidValue {
                    field: format!("dependency '{}'", name),
                    reason: "version cannot be empty".to_string(),
                });
            }
        }
        DependencySpec::Detailed {
            version,
            git,
            path,
            registry,
        } => {
            // Must have at least one source
            if version.is_none() && git.is_none() && path.is_none() && registry.is_none() {
                return Err(ConfigError::InvalidValue {
                    field: format!("dependency '{}'", name),
                    reason: "must specify version, git, path, or registry".to_string(),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_project_config() {
        let toml = r#"
[package]
name = "my-app"
version = "0.1.0"
"#;

        let config: ProjectConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.package_name(), Some("my-app"));
        assert_eq!(config.package_version(), Some("0.1.0"));
    }

    #[test]
    fn test_parse_full_project_config() {
        let toml = r#"
[package]
name = "my-app"
version = "1.0.0"
edition = "2026"
description = "A test application"
authors = ["Alice <alice@example.com>"]

[build]
output = "dist"
source = "src"

[compiler]
optimize = true
debug = false

[formatting]
indent = 4
max_line_length = 100

[dependencies]
http = "1.0"
json = { version = "0.5" }

[dev-dependencies]
test-utils = { path = "../test-utils" }
"#;

        let config: ProjectConfig = toml::from_str(toml).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.package_name(), Some("my-app"));
        assert_eq!(config.edition(), Some("2026"));
        assert!(config.dependencies.contains_key("http"));
    }

    #[test]
    fn test_version_validation() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("0.1.0"));
        assert!(is_valid_version("1.0"));
        assert!(is_valid_version("1.0.0-alpha"));
        assert!(!is_valid_version(""));
        assert!(!is_valid_version("1"));
        assert!(!is_valid_version("invalid"));
    }

    #[test]
    fn test_edition_validation() {
        assert!(is_valid_edition("2026"));
        assert!(is_valid_edition("2027"));
        assert!(!is_valid_edition("2025"));
        assert!(!is_valid_edition("invalid"));
    }

    #[test]
    fn test_merge_configs() {
        let mut base = ProjectConfig::default();
        let override_config = ProjectConfig {
            package: Some(PackageConfig {
                name: "override".to_string(),
                version: "2.0.0".to_string(),
                edition: None,
                description: None,
                authors: vec![],
                license: None,
                repository: None,
            }),
            ..Default::default()
        };

        base.merge(&override_config);
        assert_eq!(base.package_name(), Some("override"));
    }
}
