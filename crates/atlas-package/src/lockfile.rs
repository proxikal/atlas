//! Package lockfile (atlas.lock) for reproducible builds

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Lockfile structure (atlas.lock)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lockfile {
    /// Lockfile format version
    pub version: u32,
    /// Resolved packages
    pub packages: Vec<LockedPackage>,
    /// Metadata
    #[serde(default)]
    pub metadata: LockfileMetadata,
}

impl Lockfile {
    /// Current lockfile format version
    pub const VERSION: u32 = 1;

    /// Create new empty lockfile
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            packages: Vec::new(),
            metadata: LockfileMetadata::default(),
        }
    }

    /// Parse lockfile from TOML string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Load lockfile from file
    pub fn from_file(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&content)?)
    }

    /// Serialize to TOML string
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Write lockfile to file
    pub fn write_to_file(&self, path: &Path) -> crate::Result<()> {
        let content = self.to_string()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add or update a locked package
    pub fn add_package(&mut self, package: LockedPackage) {
        // Remove existing entry if present
        self.packages.retain(|p| p.name != package.name);
        self.packages.push(package);
        self.packages.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Get locked package by name
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Remove package from lockfile
    pub fn remove_package(&mut self, name: &str) -> bool {
        let len = self.packages.len();
        self.packages.retain(|p| p.name != name);
        len != self.packages.len()
    }

    /// Verify lockfile integrity
    pub fn verify(&self) -> Result<(), String> {
        // Check version compatibility
        if self.version > Self::VERSION {
            return Err(format!(
                "Lockfile version {} is newer than supported version {}",
                self.version,
                Self::VERSION
            ));
        }

        // Check for duplicate packages
        let mut seen = std::collections::HashSet::new();
        for pkg in &self.packages {
            if !seen.insert(&pkg.name) {
                return Err(format!("Duplicate package in lockfile: {}", pkg.name));
            }
        }

        Ok(())
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Locked package entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockedPackage {
    /// Package name
    pub name: String,
    /// Resolved version
    pub version: semver::Version,
    /// Source information
    pub source: LockedSource,
    /// Checksum for integrity verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Direct dependencies (name -> version)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, semver::Version>,
}

/// Locked dependency source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LockedSource {
    /// Registry source
    Registry {
        /// Registry URL
        #[serde(skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
    },
    /// Git source
    Git {
        /// Git repository URL
        url: String,
        /// Resolved commit hash
        rev: String,
    },
    /// Path source
    Path {
        /// Path to package
        path: std::path::PathBuf,
    },
}

/// Lockfile metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LockfileMetadata {
    /// When lockfile was generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    /// Atlas version used to generate lockfile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atlas_version: Option<String>,
}

/// Dependency resolver
pub struct Resolver;

impl Resolver {
    /// Resolve dependencies and generate lockfile
    pub fn resolve(manifest: &crate::manifest::PackageManifest) -> crate::Result<Lockfile> {
        let mut lockfile = Lockfile::new();

        // Add current package metadata
        lockfile.metadata.atlas_version = Some(env!("CARGO_PKG_VERSION").to_string());
        lockfile.metadata.generated_at =
            Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));

        // Resolve all dependencies
        for (name, dep) in &manifest.dependencies {
            if let Some(locked) = Self::resolve_dependency(name, dep)? {
                lockfile.add_package(locked);
            }
        }

        Ok(lockfile)
    }

    /// Resolve single dependency
    fn resolve_dependency(
        name: &str,
        dep: &crate::manifest::Dependency,
    ) -> crate::Result<Option<LockedPackage>> {
        use crate::manifest::Dependency;

        match dep {
            Dependency::Simple(version) => {
                // Parse version constraint
                let constraint = crate::manifest::VersionConstraint::parse(version)?;

                // For now, just use the constraint as-is (real resolver would query registry)
                let resolved_version = match constraint {
                    crate::manifest::VersionConstraint::Exact(v) => v,
                    crate::manifest::VersionConstraint::Caret(v)
                    | crate::manifest::VersionConstraint::Tilde(v) => v,
                    _ => {
                        // Would query registry for latest matching version
                        return Ok(None);
                    }
                };

                Ok(Some(LockedPackage {
                    name: name.to_string(),
                    version: resolved_version,
                    source: LockedSource::Registry { registry: None },
                    checksum: None,
                    dependencies: HashMap::new(),
                }))
            }
            Dependency::Detailed(detailed) => {
                let source = if let Some(ref git_url) = detailed.git {
                    // Git source - would resolve to actual commit hash
                    LockedSource::Git {
                        url: git_url.clone(),
                        rev: detailed.rev.clone().unwrap_or_else(|| "HEAD".to_string()),
                    }
                } else if let Some(ref path) = detailed.path {
                    // Path source
                    LockedSource::Path { path: path.clone() }
                } else {
                    // Registry source
                    LockedSource::Registry {
                        registry: detailed.registry.clone(),
                    }
                };

                // Resolve version
                let version = if let Some(ref v) = detailed.version {
                    let constraint = crate::manifest::VersionConstraint::parse(v)?;
                    match constraint {
                        crate::manifest::VersionConstraint::Exact(ver) => ver,
                        crate::manifest::VersionConstraint::Caret(ver)
                        | crate::manifest::VersionConstraint::Tilde(ver) => ver,
                        _ => return Ok(None),
                    }
                } else {
                    // Git/path dependencies might not have version - skip for now
                    return Ok(None);
                };

                Ok(Some(LockedPackage {
                    name: name.to_string(),
                    version,
                    source,
                    checksum: None,
                    dependencies: HashMap::new(),
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_lockfile() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, Lockfile::VERSION);
        assert!(lockfile.packages.is_empty());
    }

    #[test]
    fn test_add_package() {
        let mut lockfile = Lockfile::new();

        let pkg = LockedPackage {
            name: "test-pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: Some("abc123".to_string()),
            dependencies: HashMap::new(),
        };

        lockfile.add_package(pkg.clone());
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.get_package("test-pkg"), Some(&pkg));
    }

    #[test]
    fn test_remove_package() {
        let mut lockfile = Lockfile::new();

        let pkg = LockedPackage {
            name: "test-pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        };

        lockfile.add_package(pkg);
        assert!(lockfile.remove_package("test-pkg"));
        assert!(lockfile.packages.is_empty());
        assert!(!lockfile.remove_package("test-pkg"));
    }

    #[test]
    fn test_serialize_lockfile() {
        let mut lockfile = Lockfile::new();

        lockfile.add_package(LockedPackage {
            name: "foo".to_string(),
            version: semver::Version::new(1, 2, 3),
            source: LockedSource::Registry { registry: None },
            checksum: Some("abc123".to_string()),
            dependencies: HashMap::new(),
        });

        let toml = lockfile.to_string().unwrap();
        assert!(toml.contains("version = 1"));
        assert!(toml.contains("name = \"foo\""));
        assert!(toml.contains("version = \"1.2.3\""));
    }

    #[test]
    fn test_parse_lockfile() {
        let toml = r#"
            version = 1

            [[packages]]
            name = "foo"
            version = "1.0.0"
            checksum = "abc123"

            [packages.source]
            type = "registry"
        "#;

        let lockfile = Lockfile::from_str(toml).unwrap();
        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages[0].name, "foo");
    }

    #[test]
    fn test_verify_duplicate_packages() {
        let mut lockfile = Lockfile::new();

        lockfile.packages.push(LockedPackage {
            name: "foo".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        lockfile.packages.push(LockedPackage {
            name: "foo".to_string(),
            version: semver::Version::new(2, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        assert!(lockfile.verify().is_err());
    }

    #[test]
    fn test_git_source_serialization() {
        let pkg = LockedPackage {
            name: "git-pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Git {
                url: "https://github.com/example/repo".to_string(),
                rev: "abc123def456".to_string(),
            },
            checksum: None,
            dependencies: HashMap::new(),
        };

        let mut lockfile = Lockfile::new();
        lockfile.add_package(pkg);

        let toml = lockfile.to_string().unwrap();
        assert!(toml.contains("type = \"git\""));
        assert!(toml.contains("url = \"https://github.com/example/repo\""));
        assert!(toml.contains("rev = \"abc123def456\""));
    }

    #[test]
    fn test_path_source_serialization() {
        let pkg = LockedPackage {
            name: "local-pkg".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Path {
                path: std::path::PathBuf::from("../local-package"),
            },
            checksum: None,
            dependencies: HashMap::new(),
        };

        let mut lockfile = Lockfile::new();
        lockfile.add_package(pkg);

        let toml = lockfile.to_string().unwrap();
        assert!(toml.contains("type = \"path\""));
        assert!(toml.contains("path = \"../local-package\""));
    }
}
