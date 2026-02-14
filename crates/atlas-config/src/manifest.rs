//! Package Manifest
//!
//! Handles package metadata and dependencies for Atlas packages.

use crate::project::{DependencySpec, PackageConfig};
use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Package manifest (subset of ProjectConfig focused on package metadata)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    /// Package metadata
    pub package: PackageConfig,

    /// Dependencies
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, DependencySpec>,

    /// Development dependencies
    #[serde(default, rename = "dev-dependencies")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dev_dependencies: HashMap<String, DependencySpec>,
}

impl Manifest {
    /// Load manifest from a file
    pub fn load_from_file(path: &Path) -> ConfigResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ConfigError::NotFound(path.to_path_buf())
            } else {
                ConfigError::IoError(e)
            }
        })?;

        let manifest: Self = toml::from_str(&content).map_err(|e| ConfigError::TomlParseError {
            file: path.to_path_buf(),
            error: e,
        })?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate package name
        if self.package.name.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "package.name".to_string(),
                reason: "name cannot be empty".to_string(),
            });
        }

        // Validate version
        if !is_valid_version(&self.package.version) {
            return Err(ConfigError::InvalidVersion(self.package.version.clone()));
        }

        // Validate dependencies
        for (name, spec) in &self.dependencies {
            validate_dependency(name, spec)?;
        }

        for (name, spec) in &self.dev_dependencies {
            validate_dependency(name, spec)?;
        }

        Ok(())
    }

    /// Get package name
    pub fn name(&self) -> &str {
        &self.package.name
    }

    /// Get package version
    pub fn version(&self) -> &str {
        &self.package.version
    }

    /// Get edition
    pub fn edition(&self) -> Option<&str> {
        self.package.edition.as_deref()
    }

    /// Get all dependencies (regular + dev)
    pub fn all_dependencies(&self) -> HashMap<String, &DependencySpec> {
        let mut all = HashMap::new();
        for (name, spec) in &self.dependencies {
            all.insert(name.clone(), spec);
        }
        for (name, spec) in &self.dev_dependencies {
            all.insert(name.clone(), spec);
        }
        all
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
    fn test_parse_minimal_manifest() {
        let toml = r#"
[package]
name = "my-package"
version = "1.0.0"
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.name(), "my-package");
        assert_eq!(manifest.version(), "1.0.0");
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn test_parse_manifest_with_dependencies() {
        let toml = r#"
[package]
name = "my-package"
version = "1.0.0"
edition = "2026"

[dependencies]
http = "1.0"
json = { version = "0.5" }
local-lib = { path = "../local-lib" }

[dev-dependencies]
test-utils = "0.1"
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.dependencies.len(), 3);
        assert_eq!(manifest.dev_dependencies.len(), 1);
        assert_eq!(manifest.all_dependencies().len(), 4);
    }

    #[test]
    fn test_invalid_empty_name() {
        let manifest = Manifest {
            package: PackageConfig {
                name: "".to_string(),
                version: "1.0.0".to_string(),
                edition: None,
                description: None,
                authors: vec![],
                license: None,
                repository: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_invalid_version() {
        let manifest = Manifest {
            package: PackageConfig {
                name: "test".to_string(),
                version: "invalid".to_string(),
                edition: None,
                description: None,
                authors: vec![],
                license: None,
                repository: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
        };

        assert!(manifest.validate().is_err());
    }
}
