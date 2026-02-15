use semver::{Version, VersionReq};
use std::collections::HashMap;

/// Version solver for finding compatible package versions
#[derive(Debug, Clone)]
pub struct VersionSolver {
    /// Available package versions (mock data for phase-08a)
    /// In phase-08b, this will query the registry
    available_versions: HashMap<String, Vec<Version>>,
}

impl VersionSolver {
    pub fn new() -> Self {
        Self {
            available_versions: HashMap::new(),
        }
    }

    /// Register available versions for a package
    pub fn add_package_versions(&mut self, package: &str, mut versions: Vec<Version>) {
        // Sort versions in ascending order
        versions.sort();
        self.available_versions
            .insert(package.to_string(), versions);
    }

    /// Find maximum version satisfying all constraints
    pub fn max_satisfying_version(
        &self,
        package: &str,
        constraints: &[VersionReq],
    ) -> Option<Version> {
        let versions = self.available_versions.get(package)?;

        // Filter versions that satisfy all constraints
        let mut satisfying: Vec<&Version> = versions
            .iter()
            .filter(|v| constraints.iter().all(|req| req.matches(v)))
            .collect();

        // Return maximum version
        satisfying.sort();
        satisfying.last().map(|v| (*v).clone())
    }

    /// Check if constraints are satisfiable
    pub fn is_satisfiable(&self, package: &str, constraints: &[VersionReq]) -> bool {
        self.max_satisfying_version(package, constraints).is_some()
    }

    /// Get all available versions for a package
    pub fn get_versions(&self, package: &str) -> Option<&Vec<Version>> {
        self.available_versions.get(package)
    }

    /// Find all versions satisfying constraints
    pub fn find_all_satisfying(&self, package: &str, constraints: &[VersionReq]) -> Vec<Version> {
        let Some(versions) = self.available_versions.get(package) else {
            return Vec::new();
        };

        versions
            .iter()
            .filter(|v| constraints.iter().all(|req| req.matches(v)))
            .cloned()
            .collect()
    }

    /// Check if two constraint sets are compatible
    pub fn are_constraints_compatible(
        &self,
        package: &str,
        constraints1: &[VersionReq],
        constraints2: &[VersionReq],
    ) -> bool {
        let mut combined = constraints1.to_vec();
        combined.extend_from_slice(constraints2);

        self.is_satisfiable(package, &combined)
    }

    /// Find minimum version satisfying constraints
    pub fn min_satisfying_version(
        &self,
        package: &str,
        constraints: &[VersionReq],
    ) -> Option<Version> {
        let versions = self.available_versions.get(package)?;

        // Filter versions that satisfy all constraints
        let mut satisfying: Vec<&Version> = versions
            .iter()
            .filter(|v| constraints.iter().all(|req| req.matches(v)))
            .collect();

        // Return minimum version
        satisfying.sort();
        satisfying.first().map(|v| (*v).clone())
    }
}

impl Default for VersionSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_version_req(req: &str) -> VersionReq {
        req.parse().unwrap()
    }

    #[test]
    fn test_new_solver() {
        let solver = VersionSolver::new();
        assert!(solver.available_versions.is_empty());
    }

    #[test]
    fn test_add_package_versions() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions("test", vec![Version::new(1, 0, 0), Version::new(2, 0, 0)]);

        let versions = solver.get_versions("test");
        assert!(versions.is_some());
        assert_eq!(versions.unwrap().len(), 2);
    }

    #[test]
    fn test_max_satisfying_version_exact() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req = create_version_req("=1.0.0");
        let version = solver.max_satisfying_version("test", &[req]);
        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn test_max_satisfying_version_caret() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(1, 2, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req = create_version_req("^1.0.0");
        let version = solver.max_satisfying_version("test", &[req]);
        // ^1.0.0 matches >=1.0.0, <2.0.0, so max is 1.2.0
        assert_eq!(version, Some(Version::new(1, 2, 0)));
    }

    #[test]
    fn test_max_satisfying_version_tilde() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 2, 0),
                Version::new(1, 2, 1),
                Version::new(1, 2, 2),
                Version::new(1, 3, 0),
            ],
        );

        let req = create_version_req("~1.2.0");
        let version = solver.max_satisfying_version("test", &[req]);
        // ~1.2.0 matches >=1.2.0, <1.3.0, so max is 1.2.2
        assert_eq!(version, Some(Version::new(1, 2, 2)));
    }

    #[test]
    fn test_max_satisfying_version_range() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 5, 0),
                Version::new(2, 0, 0),
                Version::new(2, 5, 0),
            ],
        );

        let req = create_version_req(">=1.0.0, <2.0.0");
        let version = solver.max_satisfying_version("test", &[req]);
        assert_eq!(version, Some(Version::new(1, 5, 0)));
    }

    #[test]
    fn test_max_satisfying_version_multiple_constraints() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 5, 0),
                Version::new(1, 8, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req1 = create_version_req("^1.0.0");
        let req2 = create_version_req("<1.8.0");
        let version = solver.max_satisfying_version("test", &[req1, req2]);
        // ^1.0.0 = >=1.0.0, <2.0.0
        // Combined with <1.8.0 = >=1.0.0, <1.8.0
        // Max is 1.5.0
        assert_eq!(version, Some(Version::new(1, 5, 0)));
    }

    #[test]
    fn test_max_satisfying_version_no_match() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions("test", vec![Version::new(1, 0, 0)]);

        let req = create_version_req("^2.0.0");
        let version = solver.max_satisfying_version("test", &[req]);
        assert_eq!(version, None);
    }

    #[test]
    fn test_is_satisfiable_true() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions("test", vec![Version::new(1, 0, 0)]);

        let req = create_version_req("^1.0.0");
        assert!(solver.is_satisfiable("test", &[req]));
    }

    #[test]
    fn test_is_satisfiable_false() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions("test", vec![Version::new(1, 0, 0)]);

        let req = create_version_req("^2.0.0");
        assert!(!solver.is_satisfiable("test", &[req]));
    }

    #[test]
    fn test_find_all_satisfying() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(1, 2, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req = create_version_req("^1.0.0");
        let versions = solver.find_all_satisfying("test", &[req]);
        assert_eq!(versions.len(), 3);
        assert!(versions.contains(&Version::new(1, 0, 0)));
        assert!(versions.contains(&Version::new(1, 1, 0)));
        assert!(versions.contains(&Version::new(1, 2, 0)));
    }

    #[test]
    fn test_are_constraints_compatible_yes() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 5, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req1 = vec![create_version_req("^1.0.0")];
        let req2 = vec![create_version_req(">=1.0.0")];
        assert!(solver.are_constraints_compatible("test", &req1, &req2));
    }

    #[test]
    fn test_are_constraints_compatible_no() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                Version::new(3, 0, 0),
            ],
        );

        let req1 = vec![create_version_req("^1.0.0")];
        let req2 = vec![create_version_req("^2.0.0")];
        assert!(!solver.are_constraints_compatible("test", &req1, &req2));
    }

    #[test]
    fn test_min_satisfying_version() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(1, 1, 0),
                Version::new(1, 2, 0),
                Version::new(2, 0, 0),
            ],
        );

        let req = create_version_req("^1.0.0");
        let version = solver.min_satisfying_version("test", &[req]);
        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn test_wildcard_version() {
        let mut solver = VersionSolver::new();
        solver.add_package_versions(
            "test",
            vec![
                Version::new(1, 0, 0),
                Version::new(2, 0, 0),
                Version::new(3, 0, 0),
            ],
        );

        let req = create_version_req("*");
        let version = solver.max_satisfying_version("test", &[req]);
        assert_eq!(version, Some(Version::new(3, 0, 0)));
    }
}
