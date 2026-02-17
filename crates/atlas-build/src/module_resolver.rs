//! Cross-module symbol resolution
//!
//! Resolves import sources to module paths and manages the export/import
//! matching pipeline during multi-module builds.

use atlas_runtime::module_loader::ModuleRegistry;
use atlas_runtime::symbol::SymbolTable;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Tracks compiled module exports and provides import resolution.
///
/// As modules compile in topological order, their symbol tables are
/// registered here. When a dependent module is compiled, the resolver
/// provides a `ModuleRegistry` populated with its dependencies' exports.
pub struct ModuleResolver {
    /// Module name → symbol table (post-binding, with exports marked)
    module_symbols: HashMap<String, SymbolTable>,
    /// Module name → source file path
    module_paths: HashMap<String, PathBuf>,
}

impl ModuleResolver {
    /// Create a new empty resolver
    pub fn new() -> Self {
        Self {
            module_symbols: HashMap::new(),
            module_paths: HashMap::new(),
        }
    }

    /// Register a compiled module's symbol table and path.
    ///
    /// Called after each module is successfully compiled and bound.
    pub fn register_module(
        &mut self,
        module_name: String,
        path: PathBuf,
        symbol_table: SymbolTable,
    ) {
        self.module_paths.insert(module_name.clone(), path);
        self.module_symbols.insert(module_name, symbol_table);
    }

    /// Build a `ModuleRegistry` containing only the given dependencies.
    ///
    /// The registry is keyed by the import source string (e.g., "math")
    /// converted to a `PathBuf`, matching how `Binder::bind_import()` looks
    /// them up.
    pub fn build_registry_for(&self, dependencies: &[String]) -> ModuleRegistry {
        let mut registry = ModuleRegistry::new();

        for dep_name in dependencies {
            if let Some(symbol_table) = self.module_symbols.get(dep_name) {
                // The binder does `PathBuf::from(&import_decl.source)` to look up
                // in the registry, so we register with the same key the import uses.
                registry.register(PathBuf::from(dep_name), symbol_table.clone());
            }
        }

        registry
    }

    /// Resolve an import source string to a module name.
    ///
    /// Import sources like "math" or "./utils" are normalized to module names
    /// that match those produced by `path_to_module_name()`.
    pub fn resolve_import_source(source: &str, _importing_module: &str) -> String {
        // Strip leading "./" if present
        let normalized = source.strip_prefix("./").unwrap_or(source);
        // Replace path separators with ::
        normalized.replace('/', "::")
    }

    /// Check if a module has been registered
    pub fn has_module(&self, name: &str) -> bool {
        self.module_symbols.contains_key(name)
    }

    /// Get the exports of a registered module
    pub fn get_exports(
        &self,
        module_name: &str,
    ) -> Option<HashMap<String, atlas_runtime::symbol::Symbol>> {
        self.module_symbols
            .get(module_name)
            .map(|st| st.get_exports())
    }

    /// Get path for a module
    pub fn get_module_path(&self, module_name: &str) -> Option<&Path> {
        self.module_paths.get(module_name).map(|p| p.as_path())
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_import_source_simple() {
        assert_eq!(
            ModuleResolver::resolve_import_source("math", "main"),
            "math"
        );
    }

    #[test]
    fn test_resolve_import_source_relative() {
        assert_eq!(
            ModuleResolver::resolve_import_source("./utils", "main"),
            "utils"
        );
    }

    #[test]
    fn test_resolve_import_source_nested() {
        assert_eq!(
            ModuleResolver::resolve_import_source("utils/helpers", "main"),
            "utils::helpers"
        );
    }

    #[test]
    fn test_empty_resolver() {
        let resolver = ModuleResolver::new();
        assert!(!resolver.has_module("math"));
        assert!(resolver.get_exports("math").is_none());
    }

    #[test]
    fn test_build_registry_for_missing_dep() {
        let resolver = ModuleResolver::new();
        let registry = resolver.build_registry_for(&["nonexistent".to_string()]);
        // Should produce an empty registry (no panic)
        assert!(registry.get(&PathBuf::from("nonexistent")).is_none());
    }
}
