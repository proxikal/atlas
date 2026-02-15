//! Build order computation for package dependencies

use crate::resolver::Resolution;
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum BuildOrderError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Missing dependency: {0}")]
    MissingDependency(String),
}

pub type BuildOrderResult<T> = Result<T, BuildOrderError>;

/// Build order computer using topological sort
pub struct BuildOrderComputer {
    /// Dependency graph: package -> dependencies
    graph: HashMap<String, Vec<String>>,
}

impl BuildOrderComputer {
    /// Create a new build order computer from a resolution
    pub fn new(resolution: &Resolution) -> Self {
        let mut graph = HashMap::new();

        for (name, package) in &resolution.packages {
            graph.insert(name.clone(), package.dependencies.clone());
        }

        Self { graph }
    }

    /// Create from a raw dependency graph
    pub fn from_graph(graph: HashMap<String, Vec<String>>) -> Self {
        Self { graph }
    }

    /// Compute topological build order using Kahn's algorithm
    pub fn compute_build_order(&self) -> BuildOrderResult<Vec<String>> {
        if self.graph.is_empty() {
            return Ok(Vec::new());
        }

        let mut in_degree = self.compute_in_degrees();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Start with packages that have no dependencies (in-degree = 0)
        for (package, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(package.clone());
            }
        }

        while let Some(package) = queue.pop_front() {
            result.push(package.clone());

            // For each package that depends on the current package
            for (dependent, deps) in &self.graph {
                if deps.contains(&package) {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.graph.len() {
            return Err(BuildOrderError::CircularDependency(
                "Dependency cycle detected in graph".to_string(),
            ));
        }

        Ok(result)
    }

    /// Compute in-degrees for all packages
    /// In-degree = number of dependencies this package has
    fn compute_in_degrees(&self) -> HashMap<String, usize> {
        let mut in_degree = HashMap::new();

        // Initialize all packages with their dependency count
        for (package, deps) in &self.graph {
            in_degree.insert(package.clone(), deps.len());
        }

        in_degree
    }

    /// Find packages that can be built in parallel
    /// Returns groups where each group can be built in parallel
    pub fn parallel_build_groups(&self) -> BuildOrderResult<Vec<Vec<String>>> {
        if self.graph.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();
        let mut built = HashSet::new();

        loop {
            // Find all packages that can be built now (all dependencies satisfied)
            let mut group = Vec::new();

            for (package, deps) in &self.graph {
                if built.contains(package) {
                    continue;
                }

                // Check if all dependencies are built
                if deps.iter().all(|d| built.contains(d)) {
                    group.push(package.clone());
                }
            }

            if group.is_empty() {
                break;
            }

            // Sort group for deterministic output
            group.sort();

            // Mark these packages as built
            for package in &group {
                built.insert(package.clone());
            }

            groups.push(group);
        }

        // Check if we built all packages (detect cycles)
        if built.len() != self.graph.len() {
            return Err(BuildOrderError::CircularDependency(
                "Dependency cycle detected in graph".to_string(),
            ));
        }

        Ok(groups)
    }

    /// Get the dependency graph
    pub fn graph(&self) -> &HashMap<String, Vec<String>> {
        &self.graph
    }

    /// Get dependencies for a specific package
    pub fn get_dependencies(&self, package: &str) -> Option<&Vec<String>> {
        self.graph.get(package)
    }

    /// Get all packages in the graph
    pub fn packages(&self) -> Vec<String> {
        let mut packages: Vec<String> = self.graph.keys().cloned().collect();
        packages.sort();
        packages
    }

    /// Count of packages in graph
    pub fn package_count(&self) -> usize {
        self.graph.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{Resolution, ResolvedPackage};
    use semver::Version;

    fn make_resolution(packages: Vec<(&str, Vec<&str>)>) -> Resolution {
        let mut resolution = Resolution::new();

        for (name, deps) in packages {
            let package = ResolvedPackage::with_dependencies(
                name.to_string(),
                Version::new(1, 0, 0),
                deps.iter().map(|s| s.to_string()).collect(),
            );
            resolution.add_package(package);
        }

        resolution
    }

    #[test]
    fn test_build_order_empty_graph() {
        let resolution = Resolution::new();
        let computer = BuildOrderComputer::new(&resolution);

        let order = computer.compute_build_order().unwrap();
        assert!(order.is_empty());
        assert!(computer.is_empty());
    }

    #[test]
    fn test_build_order_single_package() {
        let resolution = make_resolution(vec![("pkg1", vec![])]);
        let computer = BuildOrderComputer::new(&resolution);

        let order = computer.compute_build_order().unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], "pkg1");
        assert_eq!(computer.package_count(), 1);
    }

    #[test]
    fn test_build_order_linear() {
        // pkg1 -> pkg2 -> pkg3
        let resolution = make_resolution(vec![
            ("pkg1", vec!["pkg2"]),
            ("pkg2", vec!["pkg3"]),
            ("pkg3", vec![]),
        ]);
        let computer = BuildOrderComputer::new(&resolution);

        let order = computer.compute_build_order().unwrap();
        assert_eq!(order.len(), 3);

        // pkg3 should come before pkg2, pkg2 before pkg1
        let pkg3_idx = order.iter().position(|p| p == "pkg3").unwrap();
        let pkg2_idx = order.iter().position(|p| p == "pkg2").unwrap();
        let pkg1_idx = order.iter().position(|p| p == "pkg1").unwrap();

        assert!(pkg3_idx < pkg2_idx);
        assert!(pkg2_idx < pkg1_idx);
    }

    #[test]
    fn test_build_order_diamond() {
        // root -> left -> bottom
        //      -> right -> bottom
        let resolution = make_resolution(vec![
            ("root", vec!["left", "right"]),
            ("left", vec!["bottom"]),
            ("right", vec!["bottom"]),
            ("bottom", vec![]),
        ]);
        let computer = BuildOrderComputer::new(&resolution);

        let order = computer.compute_build_order().unwrap();
        assert_eq!(order.len(), 4);

        // bottom should come before left and right, which should come before root
        let bottom_idx = order.iter().position(|p| p == "bottom").unwrap();
        let left_idx = order.iter().position(|p| p == "left").unwrap();
        let right_idx = order.iter().position(|p| p == "right").unwrap();
        let root_idx = order.iter().position(|p| p == "root").unwrap();

        assert!(bottom_idx < left_idx);
        assert!(bottom_idx < right_idx);
        assert!(left_idx < root_idx);
        assert!(right_idx < root_idx);
    }

    #[test]
    fn test_parallel_build_groups_empty() {
        let resolution = Resolution::new();
        let computer = BuildOrderComputer::new(&resolution);

        let groups = computer.parallel_build_groups().unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_parallel_build_groups_single() {
        let resolution = make_resolution(vec![("pkg1", vec![])]);
        let computer = BuildOrderComputer::new(&resolution);

        let groups = computer.parallel_build_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], vec!["pkg1"]);
    }

    #[test]
    fn test_parallel_build_groups_linear() {
        // pkg1 -> pkg2 -> pkg3 (must be sequential)
        let resolution = make_resolution(vec![
            ("pkg1", vec!["pkg2"]),
            ("pkg2", vec!["pkg3"]),
            ("pkg3", vec![]),
        ]);
        let computer = BuildOrderComputer::new(&resolution);

        let groups = computer.parallel_build_groups().unwrap();
        // Each package must be in its own group
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], vec!["pkg3"]);
        assert_eq!(groups[1], vec!["pkg2"]);
        assert_eq!(groups[2], vec!["pkg1"]);
    }

    #[test]
    fn test_parallel_build_groups_diamond() {
        // root -> left -> bottom
        //      -> right -> bottom
        // bottom can be first, left+right parallel, then root
        let resolution = make_resolution(vec![
            ("root", vec!["left", "right"]),
            ("left", vec!["bottom"]),
            ("right", vec!["bottom"]),
            ("bottom", vec![]),
        ]);
        let computer = BuildOrderComputer::new(&resolution);

        let groups = computer.parallel_build_groups().unwrap();
        assert_eq!(groups.len(), 3);

        // First group: bottom only
        assert_eq!(groups[0], vec!["bottom"]);

        // Second group: left and right (parallel)
        let mut second_group = groups[1].clone();
        second_group.sort();
        assert_eq!(second_group, vec!["left", "right"]);

        // Third group: root only
        assert_eq!(groups[2], vec!["root"]);
    }

    #[test]
    fn test_parallel_build_groups_independent() {
        // Three independent packages can all build in parallel
        let resolution =
            make_resolution(vec![("pkg1", vec![]), ("pkg2", vec![]), ("pkg3", vec![])]);
        let computer = BuildOrderComputer::new(&resolution);

        let groups = computer.parallel_build_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn test_get_dependencies() {
        let resolution = make_resolution(vec![
            ("pkg1", vec!["pkg2", "pkg3"]),
            ("pkg2", vec![]),
            ("pkg3", vec![]),
        ]);
        let computer = BuildOrderComputer::new(&resolution);

        let deps = computer.get_dependencies("pkg1").unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"pkg2".to_string()));
        assert!(deps.contains(&"pkg3".to_string()));
    }

    #[test]
    fn test_packages() {
        let resolution =
            make_resolution(vec![("pkg1", vec![]), ("pkg2", vec![]), ("pkg3", vec![])]);
        let computer = BuildOrderComputer::new(&resolution);

        let packages = computer.packages();
        assert_eq!(packages.len(), 3);
        assert_eq!(packages, vec!["pkg1", "pkg2", "pkg3"]);
    }

    #[test]
    fn test_from_graph() {
        let mut graph = HashMap::new();
        graph.insert("pkg1".to_string(), vec!["pkg2".to_string()]);
        graph.insert("pkg2".to_string(), vec![]);

        let computer = BuildOrderComputer::from_graph(graph);
        assert_eq!(computer.package_count(), 2);

        let order = computer.compute_build_order().unwrap();
        assert_eq!(order.len(), 2);
    }
}
