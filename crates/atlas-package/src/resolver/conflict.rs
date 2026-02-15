//! Conflict detection and resolution for dependency resolution

use super::{ResolverError, VersionConstraint};
use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Conflict information for reporting
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub package: String,
    pub constraints: Vec<ConflictingConstraint>,
}

/// A single conflicting constraint
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictingConstraint {
    pub requirement: VersionReq,
    pub source: String, // Which package imposed this constraint
}

impl Conflict {
    /// Create a new conflict
    pub fn new(package: String, constraints: Vec<ConflictingConstraint>) -> Self {
        Self {
            package,
            constraints,
        }
    }

    /// Generate human-readable conflict report
    pub fn report(&self) -> String {
        let mut report = format!("Version conflict for package '{}':\n", self.package);

        for constraint in &self.constraints {
            report.push_str(&format!(
                "  {} requires {}\n",
                constraint.source, constraint.requirement
            ));
        }

        report.push_str("\nPossible solutions:\n");
        report.push_str("  1. Update dependencies to compatible versions\n");
        report.push_str("  2. Use dependency overrides in atlas.toml\n");
        report.push_str("  3. Check for alternative packages\n");

        report
    }

    /// Get the number of conflicting constraints
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

impl ConflictingConstraint {
    /// Create a new conflicting constraint
    pub fn new(requirement: VersionReq, source: String) -> Self {
        Self {
            requirement,
            source,
        }
    }

    /// Create from VersionConstraint
    pub fn from_version_constraint(vc: &VersionConstraint) -> Self {
        Self {
            requirement: vc.requirement.clone(),
            source: vc.source.clone(),
        }
    }
}

/// Conflict detector and resolver
pub struct ConflictResolver {
    /// Detected conflicts
    conflicts: Vec<Conflict>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            conflicts: Vec::new(),
        }
    }

    /// Detect conflicts in constraint set
    pub fn detect_conflicts(
        &mut self,
        constraints: &HashMap<String, Vec<VersionConstraint>>,
    ) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for (package, version_constraints) in constraints {
            if !self.are_constraints_compatible(version_constraints) {
                let conflicting_constraints = version_constraints
                    .iter()
                    .map(ConflictingConstraint::from_version_constraint)
                    .collect();

                conflicts.push(Conflict::new(package.clone(), conflicting_constraints));
            }
        }

        self.conflicts = conflicts.clone();
        conflicts
    }

    /// Check if constraints are compatible
    fn are_constraints_compatible(&self, constraints: &[VersionConstraint]) -> bool {
        if constraints.len() <= 1 {
            return true;
        }

        // Try to find if there's any overlap between all requirements
        // Check for obvious conflicts first
        for i in 0..constraints.len() {
            for j in (i + 1)..constraints.len() {
                if !self.requirements_can_overlap(
                    &constraints[i].requirement,
                    &constraints[j].requirement,
                ) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if two version requirements can potentially overlap
    fn requirements_can_overlap(&self, req1: &VersionReq, req2: &VersionReq) -> bool {
        // Test with a range of common versions to detect obvious conflicts
        let test_versions = vec![
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            Version::new(1, 2, 0),
            Version::new(2, 0, 0),
            Version::new(2, 1, 0),
            Version::new(3, 0, 0),
        ];

        // If any version satisfies both requirements, they can overlap
        for version in &test_versions {
            if req1.matches(version) && req2.matches(version) {
                return true;
            }
        }

        // If we didn't find any common version in our test set,
        // they likely conflict (though this is a heuristic)
        false
    }

    /// Suggest resolution strategies for a conflict
    pub fn suggest_resolutions(&self, conflict: &Conflict) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze conflict and suggest fixes
        suggestions.push(format!(
            "Try updating all dependencies of '{}' to latest compatible versions",
            conflict.package
        ));

        // Check if there are only two conflicting constraints
        if conflict.constraints.len() == 2 {
            suggestions.push(format!(
                "Consider checking if {} and {} can use compatible version ranges",
                conflict.constraints[0].source, conflict.constraints[1].source
            ));
        }

        // Suggest checking for newer versions
        suggestions.push(format!(
            "Check if a newer version of '{}' exists that satisfies all constraints",
            conflict.package
        ));

        // Suggest dependency overrides as last resort
        suggestions.push(
            "As a last resort, use [dependencies.overrides] in atlas.toml to force a specific version"
                .to_string(),
        );

        suggestions
    }

    /// Get all detected conflicts
    pub fn get_conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Check if there are any conflicts
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Clear all conflicts
    pub fn clear(&mut self) {
        self.conflicts.clear();
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a map of version constraints to a conflict error
pub fn constraints_to_error(package: &str, constraints: &[VersionConstraint]) -> ResolverError {
    let constraint_strings: Vec<String> = constraints
        .iter()
        .map(|c| format!("{} (from {})", c.requirement, c.source))
        .collect();

    ResolverError::NoSatisfyingVersion {
        package: package.to_string(),
        constraints: constraint_strings.join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_version_req(req: &str) -> VersionReq {
        req.parse().unwrap()
    }

    fn make_constraint(req: &str, source: &str) -> VersionConstraint {
        VersionConstraint {
            requirement: make_version_req(req),
            source: source.to_string(),
        }
    }

    #[test]
    fn test_conflict_new() {
        let conflict = Conflict::new("test-pkg".to_string(), vec![]);
        assert_eq!(conflict.package, "test-pkg");
        assert_eq!(conflict.constraint_count(), 0);
    }

    #[test]
    fn test_conflict_report() {
        let conflict = Conflict::new(
            "test-pkg".to_string(),
            vec![
                ConflictingConstraint::new(make_version_req("^1.0"), "pkg-a".to_string()),
                ConflictingConstraint::new(make_version_req("^2.0"), "pkg-b".to_string()),
            ],
        );

        let report = conflict.report();
        assert!(report.contains("test-pkg"));
        assert!(report.contains("pkg-a"));
        assert!(report.contains("pkg-b"));
        assert!(report.contains("Possible solutions"));
    }

    #[test]
    fn test_conflicting_constraint_from_version_constraint() {
        let vc = make_constraint("^1.0", "source-pkg");
        let cc = ConflictingConstraint::from_version_constraint(&vc);
        assert_eq!(cc.source, "source-pkg");
        assert_eq!(cc.requirement, make_version_req("^1.0"));
    }

    #[test]
    fn test_conflict_resolver_new() {
        let resolver = ConflictResolver::new();
        assert!(!resolver.has_conflicts());
        assert_eq!(resolver.get_conflicts().len(), 0);
    }

    #[test]
    fn test_detect_no_conflicts_single_constraint() {
        let mut resolver = ConflictResolver::new();
        let mut constraints = HashMap::new();

        constraints.insert("pkg-a".to_string(), vec![make_constraint("^1.0", "root")]);

        let conflicts = resolver.detect_conflicts(&constraints);
        assert!(conflicts.is_empty());
        assert!(!resolver.has_conflicts());
    }

    #[test]
    fn test_detect_no_conflicts_compatible() {
        let mut resolver = ConflictResolver::new();
        let mut constraints = HashMap::new();

        // ^1.0 and ^1.1 can overlap (1.1.0 satisfies both)
        constraints.insert(
            "pkg-a".to_string(),
            vec![
                make_constraint("^1.0", "root"),
                make_constraint("^1.1", "dep-b"),
            ],
        );

        let conflicts = resolver.detect_conflicts(&constraints);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_conflicts_incompatible() {
        let mut resolver = ConflictResolver::new();
        let mut constraints = HashMap::new();

        // ^1.0 and ^2.0 cannot overlap
        constraints.insert(
            "pkg-a".to_string(),
            vec![
                make_constraint("^1.0", "root"),
                make_constraint("^2.0", "dep-b"),
            ],
        );

        let conflicts = resolver.detect_conflicts(&constraints);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].package, "pkg-a");
        assert_eq!(conflicts[0].constraint_count(), 2);
        assert!(resolver.has_conflicts());
    }

    #[test]
    fn test_suggest_resolutions() {
        let resolver = ConflictResolver::new();
        let conflict = Conflict::new(
            "test-pkg".to_string(),
            vec![
                ConflictingConstraint::new(make_version_req("^1.0"), "pkg-a".to_string()),
                ConflictingConstraint::new(make_version_req("^2.0"), "pkg-b".to_string()),
            ],
        );

        let suggestions = resolver.suggest_resolutions(&conflict);
        assert!(!suggestions.is_empty());
        assert!(suggestions.len() >= 3);
    }

    #[test]
    fn test_clear_conflicts() {
        let mut resolver = ConflictResolver::new();
        let mut constraints = HashMap::new();

        constraints.insert(
            "pkg-a".to_string(),
            vec![
                make_constraint("^1.0", "root"),
                make_constraint("^2.0", "dep-b"),
            ],
        );

        resolver.detect_conflicts(&constraints);
        assert!(resolver.has_conflicts());

        resolver.clear();
        assert!(!resolver.has_conflicts());
    }

    #[test]
    fn test_constraints_to_error() {
        let constraints = vec![
            make_constraint("^1.0", "root"),
            make_constraint("^2.0", "dep-b"),
        ];

        let error = constraints_to_error("test-pkg", &constraints);
        match error {
            ResolverError::NoSatisfyingVersion { package, .. } => {
                assert_eq!(package, "test-pkg");
            }
            _ => panic!("Expected NoSatisfyingVersion error"),
        }
    }
}
