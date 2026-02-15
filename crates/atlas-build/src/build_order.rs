//! Build order computation for module compilation using topological sort
use crate::error::{BuildError, BuildResult};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// A module in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleNode {
    /// Module name
    pub name: String,
    /// Source file path
    pub path: PathBuf,
    /// Module dependencies (other module names)
    pub dependencies: Vec<String>,
}

impl ModuleNode {
    /// Create a new module node
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            dependencies: Vec::new(),
        }
    }

    /// Add dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}

/// Build graph for module compilation order
#[derive(Debug, Clone)]
pub struct BuildGraph {
    /// Modules by name
    modules: HashMap<String, ModuleNode>,
}

impl BuildGraph {
    /// Create a new empty build graph
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, module: ModuleNode) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&ModuleNode> {
        self.modules.get(name)
    }

    /// Get all modules
    pub fn modules(&self) -> &HashMap<String, ModuleNode> {
        &self.modules
    }

    /// Get module count
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// Validate the graph
    pub fn validate(&self) -> BuildResult<()> {
        // Check all dependencies exist
        for (name, module) in &self.modules {
            for dep in &module.dependencies {
                if !self.modules.contains_key(dep) {
                    return Err(BuildError::ModuleNotFound {
                        module: format!("{} (required by {})", dep, name),
                    });
                }
            }
        }
        Ok(())
    }

    /// Compute topological build order using Kahn's algorithm
    /// Returns modules in the order they should be compiled
    pub fn compute_build_order(&self) -> BuildResult<Vec<String>> {
        if self.modules.is_empty() {
            return Ok(Vec::new());
        }

        let mut in_degree = self.compute_in_degrees();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Start with modules that have no dependencies (in-degree = 0)
        for (module_name, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(module_name.clone());
            }
        }

        while let Some(module_name) = queue.pop_front() {
            result.push(module_name.clone());

            // For each module that depends on the current module
            for (dependent, module) in &self.modules {
                if module.dependencies.contains(&module_name) {
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
        if result.len() != self.modules.len() {
            let cycle = self.find_cycle();
            return Err(BuildError::CircularDependency(format!(
                "Circular dependency detected: {}",
                cycle
            )));
        }

        Ok(result)
    }

    /// Compute in-degrees for all modules
    /// In-degree = number of dependencies this module has
    fn compute_in_degrees(&self) -> HashMap<String, usize> {
        let mut in_degree = HashMap::new();

        // Initialize all modules with their dependency count
        for (name, module) in &self.modules {
            in_degree.insert(name.clone(), module.dependencies.len());
        }

        in_degree
    }

    /// Find modules that can be compiled in parallel
    /// Returns groups where each group can be compiled concurrently
    pub fn parallel_build_groups(&self) -> BuildResult<Vec<Vec<String>>> {
        if self.modules.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups = Vec::new();
        let mut built = HashSet::new();

        loop {
            // Find all modules that can be built now (all dependencies satisfied)
            let mut group = Vec::new();

            for (module_name, module) in &self.modules {
                if built.contains(module_name) {
                    continue;
                }

                // Check if all dependencies are built
                if module.dependencies.iter().all(|d| built.contains(d)) {
                    group.push(module_name.clone());
                }
            }

            if group.is_empty() {
                break;
            }

            // Sort group for deterministic output
            group.sort();

            // Mark these modules as built
            for module_name in &group {
                built.insert(module_name.clone());
            }

            groups.push(group);
        }

        // Check if we built all modules (detect cycles)
        if built.len() != self.modules.len() {
            let cycle = self.find_cycle();
            return Err(BuildError::CircularDependency(format!(
                "Circular dependency detected: {}",
                cycle
            )));
        }

        Ok(groups)
    }

    /// Find a cycle in the graph (for error reporting)
    fn find_cycle(&self) -> String {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for module_name in self.modules.keys() {
            if let Some(cycle) = self.dfs_find_cycle(
                module_name,
                &mut visited,
                &mut rec_stack,
                &mut path,
            ) {
                return cycle;
            }
        }

        "unknown cycle".to_string()
    }

    /// DFS to find a cycle
    fn dfs_find_cycle(
        &self,
        module_name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<String> {
        if rec_stack.contains(module_name) {
            // Found cycle - extract the cycle from path
            path.push(module_name.to_string());
            if let Some(start) = path.iter().position(|m| m == module_name) {
                let cycle = path[start..].join(" -> ");
                return Some(cycle);
            }
            return Some(path.join(" -> "));
        }

        if visited.contains(module_name) {
            return None;
        }

        visited.insert(module_name.to_string());
        rec_stack.insert(module_name.to_string());
        path.push(module_name.to_string());

        if let Some(module) = self.modules.get(module_name) {
            for dep in &module.dependencies {
                if let Some(cycle) = self.dfs_find_cycle(dep, visited, rec_stack, path) {
                    return Some(cycle);
                }
            }
        }

        rec_stack.remove(module_name);
        path.pop();
        None
    }
}

impl Default for BuildGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph = BuildGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert_eq!(graph.compute_build_order().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn test_single_module_no_deps() {
        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("main", PathBuf::from("main.atlas")));

        let order = graph.compute_build_order().unwrap();
        assert_eq!(order, vec!["main"]);
    }

    #[test]
    fn test_linear_dependency_chain() {
        let mut graph = BuildGraph::new();
        graph.add_module(
            ModuleNode::new("a", PathBuf::from("a.atlas"))
                .with_dependencies(vec!["b".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("b", PathBuf::from("b.atlas"))
                .with_dependencies(vec!["c".to_string()]),
        );
        graph.add_module(ModuleNode::new("c", PathBuf::from("c.atlas")));

        let order = graph.compute_build_order().unwrap();
        // c must come before b, b must come before a
        assert_eq!(order, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = BuildGraph::new();
        graph.add_module(
            ModuleNode::new("a", PathBuf::from("a.atlas"))
                .with_dependencies(vec!["b".to_string(), "c".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("b", PathBuf::from("b.atlas"))
                .with_dependencies(vec!["d".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("c", PathBuf::from("c.atlas"))
                .with_dependencies(vec!["d".to_string()]),
        );
        graph.add_module(ModuleNode::new("d", PathBuf::from("d.atlas")));

        let order = graph.compute_build_order().unwrap();
        // d must be first, a must be last, b and c can be either order
        assert_eq!(order[0], "d");
        assert_eq!(order[3], "a");
        assert!(order[1..3].contains(&"b".to_string()));
        assert!(order[1..3].contains(&"c".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = BuildGraph::new();
        graph.add_module(
            ModuleNode::new("a", PathBuf::from("a.atlas"))
                .with_dependencies(vec!["b".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("b", PathBuf::from("b.atlas"))
                .with_dependencies(vec!["a".to_string()]),
        );

        let result = graph.compute_build_order();
        assert!(result.is_err());
        match result {
            Err(BuildError::CircularDependency(msg)) => {
                assert!(msg.contains("Circular dependency"));
            }
            _ => panic!("Expected CircularDependency error"),
        }
    }

    #[test]
    fn test_missing_dependency() {
        let mut graph = BuildGraph::new();
        graph.add_module(
            ModuleNode::new("a", PathBuf::from("a.atlas"))
                .with_dependencies(vec!["nonexistent".to_string()]),
        );

        let result = graph.validate();
        assert!(result.is_err());
        match result {
            Err(BuildError::ModuleNotFound { module }) => {
                assert!(module.contains("nonexistent"));
            }
            _ => panic!("Expected ModuleNotFound error"),
        }
    }

    #[test]
    fn test_parallel_build_groups_empty() {
        let graph = BuildGraph::new();
        let groups = graph.parallel_build_groups().unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn test_parallel_build_groups_independent() {
        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("a", PathBuf::from("a.atlas")));
        graph.add_module(ModuleNode::new("b", PathBuf::from("b.atlas")));
        graph.add_module(ModuleNode::new("c", PathBuf::from("c.atlas")));

        let groups = graph.parallel_build_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
        assert!(groups[0].contains(&"a".to_string()));
        assert!(groups[0].contains(&"b".to_string()));
        assert!(groups[0].contains(&"c".to_string()));
    }

    #[test]
    fn test_parallel_build_groups_diamond() {
        let mut graph = BuildGraph::new();
        graph.add_module(
            ModuleNode::new("a", PathBuf::from("a.atlas"))
                .with_dependencies(vec!["b".to_string(), "c".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("b", PathBuf::from("b.atlas"))
                .with_dependencies(vec!["d".to_string()]),
        );
        graph.add_module(
            ModuleNode::new("c", PathBuf::from("c.atlas"))
                .with_dependencies(vec!["d".to_string()]),
        );
        graph.add_module(ModuleNode::new("d", PathBuf::from("d.atlas")));

        let groups = graph.parallel_build_groups().unwrap();
        assert_eq!(groups.len(), 3);
        // Group 0: d (no dependencies)
        assert_eq!(groups[0], vec!["d"]);
        // Group 1: b and c (both depend only on d)
        assert_eq!(groups[1].len(), 2);
        assert!(groups[1].contains(&"b".to_string()));
        assert!(groups[1].contains(&"c".to_string()));
        // Group 2: a (depends on b and c)
        assert_eq!(groups[2], vec!["a"]);
    }

    #[test]
    fn test_module_node_creation() {
        let node = ModuleNode::new("test", PathBuf::from("test.atlas"));
        assert_eq!(node.name, "test");
        assert_eq!(node.path, PathBuf::from("test.atlas"));
        assert!(node.dependencies.is_empty());
    }

    #[test]
    fn test_module_node_with_deps() {
        let node = ModuleNode::new("test", PathBuf::from("test.atlas"))
            .with_dependencies(vec!["dep1".to_string(), "dep2".to_string()]);
        assert_eq!(node.dependencies.len(), 2);
    }

    #[test]
    fn test_graph_len_and_is_empty() {
        let mut graph = BuildGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);

        graph.add_module(ModuleNode::new("test", PathBuf::from("test.atlas")));
        assert!(!graph.is_empty());
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_graph_get_module() {
        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("test", PathBuf::from("test.atlas")));

        let module = graph.get_module("test");
        assert!(module.is_some());
        assert_eq!(module.unwrap().name, "test");

        let missing = graph.get_module("missing");
        assert!(missing.is_none());
    }

    #[test]
    fn test_build_order_contains_all_modules() {
        let mut graph = BuildGraph::new();
        graph.add_module(ModuleNode::new("a", PathBuf::from("a.atlas")));
        graph.add_module(ModuleNode::new("b", PathBuf::from("b.atlas")));
        graph.add_module(ModuleNode::new("c", PathBuf::from("c.atlas")));

        let order = graph.compute_build_order().unwrap();

        // All modules should be present
        assert_eq!(order.len(), 3);
        assert!(order.contains(&"a".to_string()));
        assert!(order.contains(&"b".to_string()));
        assert!(order.contains(&"c".to_string()));
    }
}
