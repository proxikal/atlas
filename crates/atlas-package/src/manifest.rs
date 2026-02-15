//! Package manifest parsing and types (atlas.toml)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package manifest (atlas.toml)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageManifest {
    pub package: PackageMetadata,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub lib: Option<LibConfig>,
    #[serde(default)]
    pub bin: Vec<BinConfig>,
    #[serde(default)]
    pub features: HashMap<String, Feature>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
}

impl PackageManifest {
    /// Parse manifest from TOML string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Load manifest from file
    pub fn from_file(path: &std::path::Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&content)?)
    }

    /// Serialize to TOML string
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: semver::Version,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version constraint
    Simple(String),
    /// Detailed dependency
    Detailed(DetailedDependency),
}

impl Dependency {
    /// Get version constraint if applicable
    pub fn version_constraint(&self) -> Option<&str> {
        match self {
            Dependency::Simple(v) => Some(v),
            Dependency::Detailed(d) => d.version.as_deref(),
        }
    }

    /// Check if dependency is optional
    pub fn is_optional(&self) -> bool {
        match self {
            Dependency::Simple(_) => false,
            Dependency::Detailed(d) => d.optional.unwrap_or(false),
        }
    }
}

/// Detailed dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedDependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "default-features")]
    pub default_features: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "package")]
    pub rename: Option<String>,
}

/// Dependency source type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySource {
    Registry,
    Git {
        url: String,
        reference: GitReference,
    },
    Path(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitReference {
    Branch(String),
    Tag(String),
    Rev(String),
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildConfig {
    #[serde(default)]
    pub optimize: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

/// Library configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LibConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub name: Option<String>,
}

/// Binary configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BinConfig {
    pub name: String,
    pub path: PathBuf,
}

/// Feature flag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feature {
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub default: bool,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub members: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

/// Version constraint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionConstraint {
    Exact(semver::Version),
    Range(semver::VersionReq),
    Caret(semver::Version),
    Tilde(semver::Version),
    Wildcard,
    Any,
}

impl VersionConstraint {
    /// Parse version constraint from string
    pub fn parse(s: &str) -> Result<Self, semver::Error> {
        if s == "*" {
            return Ok(VersionConstraint::Wildcard);
        }

        if let Some(stripped) = s.strip_prefix('^') {
            let version = semver::Version::parse(stripped)?;
            Ok(VersionConstraint::Caret(version))
        } else if let Some(stripped) = s.strip_prefix('~') {
            let version = semver::Version::parse(stripped)?;
            Ok(VersionConstraint::Tilde(version))
        } else if s.contains(['<', '>', '=']) {
            let req = semver::VersionReq::parse(s)?;
            Ok(VersionConstraint::Range(req))
        } else {
            // Try exact version
            match semver::Version::parse(s) {
                Ok(v) => Ok(VersionConstraint::Exact(v)),
                Err(_) => {
                    // Try as requirement
                    let req = semver::VersionReq::parse(s)?;
                    Ok(VersionConstraint::Range(req))
                }
            }
        }
    }

    /// Check if version satisfies constraint
    pub fn matches(&self, version: &semver::Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Range(req) => req.matches(version),
            VersionConstraint::Caret(v) => {
                // ^1.2.3 := >=1.2.3, <2.0.0
                version >= v && version.major == v.major
            }
            VersionConstraint::Tilde(v) => {
                // ~1.2.3 := >=1.2.3, <1.3.0
                version >= v && version.major == v.major && version.minor == v.minor
            }
            VersionConstraint::Wildcard | VersionConstraint::Any => true,
        }
    }
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

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-package");
        assert_eq!(manifest.package.version.to_string(), "1.0.0");
    }

    #[test]
    fn test_parse_complete_manifest() {
        let toml = r#"
            [package]
            name = "my-package"
            version = "1.2.3"
            description = "A test package"
            authors = ["Alice <alice@example.com>"]
            license = "MIT"

            [dependencies]
            foo = "1.0"
            bar = { version = "2.0", optional = true }

            [dev-dependencies]
            test-utils = "0.1"
        "#;

        let manifest = PackageManifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "my-package");
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dev_dependencies.len(), 1);
    }

    #[test]
    fn test_version_constraint_exact() {
        let constraint = VersionConstraint::parse("1.2.3").unwrap();
        let v = semver::Version::new(1, 2, 3);
        assert!(constraint.matches(&v));
        assert!(!constraint.matches(&semver::Version::new(1, 2, 4)));
    }

    #[test]
    fn test_version_constraint_caret() {
        let constraint = VersionConstraint::parse("^1.2.3").unwrap();
        assert!(constraint.matches(&semver::Version::new(1, 2, 3)));
        assert!(constraint.matches(&semver::Version::new(1, 9, 9)));
        assert!(!constraint.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_version_constraint_tilde() {
        let constraint = VersionConstraint::parse("~1.2.3").unwrap();
        assert!(constraint.matches(&semver::Version::new(1, 2, 3)));
        assert!(constraint.matches(&semver::Version::new(1, 2, 9)));
        assert!(!constraint.matches(&semver::Version::new(1, 3, 0)));
    }

    #[test]
    fn test_dependency_optional() {
        let dep = Dependency::Detailed(DetailedDependency {
            version: Some("1.0".to_string()),
            optional: Some(true),
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: None,
            registry: None,
            features: None,
            default_features: None,
            rename: None,
        });

        assert!(dep.is_optional());
    }
}
