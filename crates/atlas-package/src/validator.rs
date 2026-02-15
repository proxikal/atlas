//! Package manifest validation

use crate::manifest::{Dependency, DetailedDependency, PackageManifest};
use std::collections::{HashMap, HashSet};

/// Validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Invalid package name format
    InvalidPackageName(String),
    /// Invalid version
    InvalidVersion(String),
    /// Circular dependency detected
    CircularDependency(String),
    /// Missing required field
    MissingField(String),
    /// Invalid dependency specification
    InvalidDependency { name: String, reason: String },
    /// Invalid feature specification
    InvalidFeature { name: String, reason: String },
    /// Workspace validation error
    WorkspaceError(String),
    /// Conflicting dependency sources
    ConflictingSource { name: String, reason: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidPackageName(name) => {
                write!(f, "Invalid package name: {}", name)
            }
            ValidationError::InvalidVersion(version) => {
                write!(f, "Invalid version: {}", version)
            }
            ValidationError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {}", cycle)
            }
            ValidationError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ValidationError::InvalidDependency { name, reason } => {
                write!(f, "Invalid dependency '{}': {}", name, reason)
            }
            ValidationError::InvalidFeature { name, reason } => {
                write!(f, "Invalid feature '{}': {}", name, reason)
            }
            ValidationError::WorkspaceError(msg) => {
                write!(f, "Workspace error: {}", msg)
            }
            ValidationError::ConflictingSource { name, reason } => {
                write!(f, "Conflicting source for '{}': {}", name, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Package manifest validator
pub struct Validator;

impl Validator {
    /// Validate package manifest
    pub fn validate(manifest: &PackageManifest) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate package metadata
        if let Err(e) = Self::validate_package_name(&manifest.package.name) {
            errors.push(e);
        }

        // Validate dependencies
        errors.extend(Self::validate_dependencies(&manifest.dependencies));
        errors.extend(Self::validate_dependencies(&manifest.dev_dependencies));

        // Validate features
        errors.extend(Self::validate_features(
            &manifest.features,
            &manifest.dependencies,
        ));

        // Check for circular dependencies
        if let Err(e) = Self::check_circular_dependencies(manifest) {
            errors.push(e);
        }

        // Validate workspace if present
        if let Some(ref workspace) = manifest.workspace {
            errors.extend(Self::validate_workspace(workspace));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate package name format
    pub fn validate_package_name(name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::InvalidPackageName(
                "Package name cannot be empty".to_string(),
            ));
        }

        // Package names must start with lowercase letter or digit
        if !name.chars().next().unwrap().is_ascii_lowercase()
            && !name.chars().next().unwrap().is_ascii_digit()
        {
            return Err(ValidationError::InvalidPackageName(format!(
                "'{}' must start with lowercase letter or digit",
                name
            )));
        }

        // Package names can only contain lowercase letters, digits, hyphens, and underscores
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ValidationError::InvalidPackageName(format!(
                "'{}' contains invalid characters (only lowercase, digits, -, _ allowed)",
                name
            )));
        }

        // Cannot start or end with hyphen or underscore
        if name.starts_with('-')
            || name.starts_with('_')
            || name.ends_with('-')
            || name.ends_with('_')
        {
            return Err(ValidationError::InvalidPackageName(format!(
                "'{}' cannot start or end with - or _",
                name
            )));
        }

        // Cannot have consecutive hyphens or underscores
        if name.contains("--") || name.contains("__") {
            return Err(ValidationError::InvalidPackageName(format!(
                "'{}' cannot contain consecutive - or _",
                name
            )));
        }

        // Length limits
        if name.len() > 64 {
            return Err(ValidationError::InvalidPackageName(format!(
                "'{}' exceeds maximum length of 64 characters",
                name
            )));
        }

        Ok(())
    }

    /// Validate dependencies
    fn validate_dependencies(deps: &HashMap<String, Dependency>) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (name, dep) in deps {
            // Validate dependency name
            if let Err(e) = Self::validate_package_name(name) {
                errors.push(ValidationError::InvalidDependency {
                    name: name.clone(),
                    reason: e.to_string(),
                });
            }

            // Validate dependency specification
            match dep {
                Dependency::Simple(version) => {
                    if let Err(e) = crate::manifest::VersionConstraint::parse(version) {
                        errors.push(ValidationError::InvalidDependency {
                            name: name.clone(),
                            reason: format!("Invalid version constraint: {}", e),
                        });
                    }
                }
                Dependency::Detailed(detailed) => {
                    errors.extend(Self::validate_detailed_dependency(name, detailed));
                }
            }
        }

        errors
    }

    /// Validate detailed dependency
    fn validate_detailed_dependency(name: &str, dep: &DetailedDependency) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Count sources
        let mut sources = 0;
        if dep.version.is_some() {
            sources += 1;
        }
        if dep.git.is_some() {
            sources += 1;
        }
        if dep.path.is_some() {
            sources += 1;
        }

        // Must have exactly one source
        if sources == 0 {
            errors.push(ValidationError::InvalidDependency {
                name: name.to_string(),
                reason: "Must specify version, git, or path".to_string(),
            });
        } else if sources > 1 {
            errors.push(ValidationError::ConflictingSource {
                name: name.to_string(),
                reason: "Cannot specify multiple sources (version/git/path)".to_string(),
            });
        }

        // Validate version if present
        if let Some(ref version) = dep.version {
            if let Err(e) = crate::manifest::VersionConstraint::parse(version) {
                errors.push(ValidationError::InvalidDependency {
                    name: name.to_string(),
                    reason: format!("Invalid version constraint: {}", e),
                });
            }
        }

        // Git requires at least one reference type
        if dep.git.is_some() {
            let has_ref = dep.branch.is_some() || dep.tag.is_some() || dep.rev.is_some();
            if !has_ref {
                errors.push(ValidationError::InvalidDependency {
                    name: name.to_string(),
                    reason: "Git dependency must specify branch, tag, or rev".to_string(),
                });
            }

            // Cannot have multiple git references
            let ref_count = [&dep.branch, &dep.tag, &dep.rev]
                .iter()
                .filter(|r| r.is_some())
                .count();
            if ref_count > 1 {
                errors.push(ValidationError::InvalidDependency {
                    name: name.to_string(),
                    reason: "Git dependency cannot specify multiple references".to_string(),
                });
            }
        }

        // Git-specific fields require git source
        if dep.git.is_none() && (dep.branch.is_some() || dep.tag.is_some() || dep.rev.is_some()) {
            errors.push(ValidationError::InvalidDependency {
                name: name.to_string(),
                reason: "branch/tag/rev requires git source".to_string(),
            });
        }

        errors
    }

    /// Validate features
    fn validate_features(
        features: &HashMap<String, crate::manifest::Feature>,
        dependencies: &HashMap<String, Dependency>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for (name, feature) in features {
            // Feature names must be valid identifiers
            if !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                errors.push(ValidationError::InvalidFeature {
                    name: name.clone(),
                    reason: "Feature name contains invalid characters".to_string(),
                });
            }

            // Validate feature dependencies
            for dep in &feature.dependencies {
                // Check if it's a package/feature reference
                if dep.contains('/') {
                    let parts: Vec<_> = dep.split('/').collect();
                    if parts.len() != 2 {
                        errors.push(ValidationError::InvalidFeature {
                            name: name.clone(),
                            reason: format!("Invalid feature dependency format: {}", dep),
                        });
                        continue;
                    }

                    let pkg = parts[0];
                    // Verify package exists in dependencies
                    if !dependencies.contains_key(pkg) {
                        errors.push(ValidationError::InvalidFeature {
                            name: name.clone(),
                            reason: format!("Feature references unknown dependency: {}", pkg),
                        });
                    }
                } else {
                    // Must be another feature
                    if !features.contains_key(dep) {
                        errors.push(ValidationError::InvalidFeature {
                            name: name.clone(),
                            reason: format!("Feature references unknown feature: {}", dep),
                        });
                    }
                }
            }
        }

        errors
    }

    /// Check for circular dependencies
    fn check_circular_dependencies(manifest: &PackageManifest) -> Result<(), ValidationError> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        // Build dependency graph
        for name in manifest.dependencies.keys() {
            graph
                .entry(manifest.package.name.clone())
                .or_default()
                .push(name.clone());
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn visit(
            node: &str,
            graph: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Result<(), String> {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visit(neighbor, graph, visited, rec_stack, path)?;
                    } else if rec_stack.contains(neighbor) {
                        // Found cycle
                        let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                        let cycle: Vec<_> = path[cycle_start..].to_vec();
                        return Err(cycle.join(" -> "));
                    }
                }
            }

            rec_stack.remove(node);
            path.pop();
            Ok(())
        }

        let mut path = Vec::new();
        if let Err(cycle) = visit(
            &manifest.package.name,
            &graph,
            &mut visited,
            &mut rec_stack,
            &mut path,
        ) {
            return Err(ValidationError::CircularDependency(cycle));
        }

        Ok(())
    }

    /// Validate workspace configuration
    fn validate_workspace(workspace: &crate::manifest::Workspace) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if workspace.members.is_empty() {
            errors.push(ValidationError::WorkspaceError(
                "Workspace must have at least one member".to_string(),
            ));
        }

        // Validate member patterns
        for member in &workspace.members {
            if member.is_empty() {
                errors.push(ValidationError::WorkspaceError(
                    "Workspace member path cannot be empty".to_string(),
                ));
            }
        }

        // Validate exclude patterns
        for exclude in &workspace.exclude {
            if exclude.is_empty() {
                errors.push(ValidationError::WorkspaceError(
                    "Workspace exclude path cannot be empty".to_string(),
                ));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_package_names() {
        assert!(Validator::validate_package_name("my-package").is_ok());
        assert!(Validator::validate_package_name("my_package").is_ok());
        assert!(Validator::validate_package_name("mypackage").is_ok());
        assert!(Validator::validate_package_name("my-pkg_123").is_ok());
        assert!(Validator::validate_package_name("123pkg").is_ok());
    }

    #[test]
    fn test_invalid_package_names() {
        // Empty
        assert!(Validator::validate_package_name("").is_err());

        // Invalid start
        assert!(Validator::validate_package_name("_package").is_err());
        assert!(Validator::validate_package_name("-package").is_err());
        assert!(Validator::validate_package_name("Package").is_err());

        // Invalid characters
        assert!(Validator::validate_package_name("my package").is_err());
        assert!(Validator::validate_package_name("my.package").is_err());
        assert!(Validator::validate_package_name("my@package").is_err());

        // Invalid endings
        assert!(Validator::validate_package_name("package-").is_err());
        assert!(Validator::validate_package_name("package_").is_err());

        // Consecutive separators
        assert!(Validator::validate_package_name("my--package").is_err());
        assert!(Validator::validate_package_name("my__package").is_err());

        // Too long
        assert!(Validator::validate_package_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_simple_dependency() {
        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), Dependency::Simple("1.0".to_string()));

        let errors = Validator::validate_dependencies(&deps);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_version() {
        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), Dependency::Simple("invalid".to_string()));

        let errors = Validator::validate_dependencies(&deps);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_conflicting_dependency_sources() {
        use crate::manifest::DetailedDependency;

        let mut deps = HashMap::new();
        deps.insert(
            "foo".to_string(),
            Dependency::Detailed(DetailedDependency {
                version: Some("1.0".to_string()),
                git: Some("https://github.com/example/foo".to_string()),
                branch: None,
                tag: None,
                rev: None,
                path: None,
                registry: None,
                optional: None,
                features: None,
                default_features: None,
                rename: None,
            }),
        );

        let errors = Validator::validate_dependencies(&deps);
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::ConflictingSource { .. })));
    }

    #[test]
    fn test_git_dependency_requires_reference() {
        use crate::manifest::DetailedDependency;

        let mut deps = HashMap::new();
        deps.insert(
            "foo".to_string(),
            Dependency::Detailed(DetailedDependency {
                version: None,
                git: Some("https://github.com/example/foo".to_string()),
                branch: None,
                tag: None,
                rev: None,
                path: None,
                registry: None,
                optional: None,
                features: None,
                default_features: None,
                rename: None,
            }),
        );

        let errors = Validator::validate_dependencies(&deps);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_feature_unknown_dependency() {
        use crate::manifest::Feature;

        let mut features = HashMap::new();
        features.insert(
            "test".to_string(),
            Feature {
                dependencies: vec!["unknown/feature".to_string()],
                default: false,
            },
        );

        let dependencies = HashMap::new();
        let errors = Validator::validate_features(&features, &dependencies);
        assert!(!errors.is_empty());
    }
}
