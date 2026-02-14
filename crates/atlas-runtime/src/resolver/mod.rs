//! Module resolution
//!
//! Resolves module paths and detects circular dependencies.
//! This is BLOCKER 04-A - syntax and resolution only.
//! Actual loading and execution happens in BLOCKER 04-B, 04-C, 04-D.

// Allow large error variants (Diagnostic) - consistent with rest of codebase
#![allow(clippy::result_large_err)]

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Module resolver - handles path resolution and circular dependency detection
pub struct ModuleResolver {
    /// Root directory for absolute paths
    root: PathBuf,
    /// Cache of resolved module paths (source path -> absolute path)
    path_cache: HashMap<String, PathBuf>,
    /// Module dependency graph for circular detection
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ModuleResolver {
    /// Create a new module resolver with the given root directory
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            path_cache: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Resolve a module path to an absolute file path
    ///
    /// # Arguments
    /// * `source` - The module path from import statement (e.g., "./math", "/src/utils")
    /// * `importing_file` - The file that contains the import statement
    ///
    /// # Returns
    /// The absolute path to the module file, or an error if not found
    pub fn resolve_path(
        &mut self,
        source: &str,
        importing_file: &Path,
        span: Span,
    ) -> Result<PathBuf, Diagnostic> {
        // Check cache first
        let cache_key = format!("{}:{}", importing_file.display(), source);
        if let Some(cached) = self.path_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let resolved = if source.starts_with('/') {
            // Absolute path: resolve from root
            self.resolve_absolute(source)?
        } else if source.starts_with("./") || source.starts_with("../") {
            // Relative path: resolve from importing file's directory
            self.resolve_relative(source, importing_file)?
        } else {
            return Err(Diagnostic::error_with_code(
                "AT5001",
                format!("Invalid module path: '{}'. Paths must start with './', '../', or '/'", source),
                span,
            )
            .with_help("Use './file' for same directory, '../file' for parent, or '/src/file' for absolute paths".to_string()));
        };

        // Verify file exists
        if !resolved.exists() {
            return Err(Diagnostic::error_with_code(
                "AT5002",
                format!("Module not found: '{}'", source),
                span,
            )
            .with_label(format!("resolved to: {}", resolved.display()))
            .with_help("Check that the file exists and the path is correct".to_string()));
        }

        // Cache the resolved path
        self.path_cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    /// Resolve an absolute path (starts with '/')
    fn resolve_absolute(&self, source: &str) -> Result<PathBuf, Diagnostic> {
        // Remove leading '/'
        let relative = &source[1..];

        // Append .atl if no extension
        let with_ext = if relative.ends_with(".atl") {
            relative.to_string()
        } else {
            format!("{}.atl", relative)
        };

        Ok(self.root.join(with_ext))
    }

    /// Resolve a relative path (starts with './' or '../')
    fn resolve_relative(&self, source: &str, importing_file: &Path) -> Result<PathBuf, Diagnostic> {
        // Get directory of importing file
        let importing_dir = importing_file.parent().unwrap_or(Path::new("."));

        // Append .atl if no extension
        let with_ext = if source.ends_with(".atl") {
            source.to_string()
        } else {
            format!("{}.atl", source)
        };

        // Resolve relative to importing directory
        let resolved = importing_dir.join(with_ext);

        // Canonicalize to get absolute path
        Ok(resolved)
    }

    /// Add a module dependency to the graph
    ///
    /// This is used to track which modules import which, for circular detection.
    pub fn add_dependency(&mut self, from: PathBuf, to: PathBuf) {
        self.dependencies.entry(from).or_default().push(to);
    }

    /// Check for circular dependencies starting from a given module
    ///
    /// Returns an error if a cycle is detected, with the cycle path for debugging.
    pub fn check_circular(&self, start: &Path, span: Span) -> Result<(), Diagnostic> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if let Some(cycle) = self.find_cycle(start, &mut visited, &mut path) {
            let cycle_str = cycle
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");

            return Err(Diagnostic::error_with_code(
                "AT5003",
                "Circular dependency detected",
                span,
            )
            .with_label(format!("cycle: {}", cycle_str))
            .with_help("Modules cannot import each other in a cycle. Refactor to remove circular dependencies.".to_string()));
        }

        Ok(())
    }

    /// Depth-first search to find cycles in the dependency graph
    fn find_cycle(
        &self,
        current: &Path,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
    ) -> Option<Vec<PathBuf>> {
        let current_buf = current.to_path_buf();

        // If we've seen this node in the current path, we have a cycle
        if path.contains(&current_buf) {
            let cycle_start = path.iter().position(|p| p == &current_buf).unwrap();
            let mut cycle = path[cycle_start..].to_vec();
            cycle.push(current_buf);
            return Some(cycle);
        }

        // If we've visited this node before (but not in current path), skip it
        if visited.contains(&current_buf) {
            return None;
        }

        // Mark as visited and add to current path
        visited.insert(current_buf.clone());
        path.push(current_buf.clone());

        // Check all dependencies
        if let Some(deps) = self.dependencies.get(&current_buf) {
            for dep in deps {
                if let Some(cycle) = self.find_cycle(dep, visited, path) {
                    return Some(cycle);
                }
            }
        }

        // Remove from current path (backtrack)
        path.pop();

        None
    }

    /// Get all dependencies of a module (for debugging/testing)
    pub fn get_dependencies(&self, module: &Path) -> Vec<PathBuf> {
        self.dependencies
            .get(&module.to_path_buf())
            .cloned()
            .unwrap_or_default()
    }

    /// Clear the resolver state (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.path_cache.clear();
        self.dependencies.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_relative_path() {
        let root = PathBuf::from("/project");
        let resolver = ModuleResolver::new(root);

        let importing = PathBuf::from("/project/src/main.atl");
        let source = "./utils";

        let result = resolver.resolve_relative(source, &importing);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert!(resolved.to_string_lossy().contains("./utils.atl"));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let root = PathBuf::from("/project");
        let resolver = ModuleResolver::new(root.clone());

        let result = resolver.resolve_absolute("/src/utils");
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved, root.join("src/utils.atl"));
    }

    #[test]
    fn test_circular_detection() {
        let root = PathBuf::from("/project");
        let mut resolver = ModuleResolver::new(root);

        let a = PathBuf::from("/project/a.atl");
        let b = PathBuf::from("/project/b.atl");

        // a -> b -> a (cycle)
        resolver.add_dependency(a.clone(), b.clone());
        resolver.add_dependency(b.clone(), a.clone());

        let result = resolver.check_circular(&a, Span::dummy());
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.message.contains("Circular dependency"));
    }
}
