//! Module Execution Engine
//!
//! Coordinates module loading and execution for both interpreter and VM.
//! Ensures single evaluation per module with proper dependency order.

use crate::ast::{ImportDecl, ImportSpecifier, Item};
use crate::diagnostic::Diagnostic;
use crate::interpreter::Interpreter;
use crate::module_loader::{LoadedModule, ModuleLoader};
use crate::resolver::ModuleResolver;
use crate::security::SecurityContext;
use crate::value::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Module execution result
pub type ModuleResult<T> = Result<T, Vec<Diagnostic>>;

/// Cache of executed modules and their exports
#[derive(Debug, Clone)]
struct ModuleCache {
    /// Map of module path -> exported symbols (name -> value)
    exports: HashMap<PathBuf, HashMap<String, Value>>,
}

impl ModuleCache {
    fn new() -> Self {
        Self {
            exports: HashMap::new(),
        }
    }

    fn is_cached(&self, path: &Path) -> bool {
        self.exports.contains_key(path)
    }

    fn store_exports(&mut self, path: PathBuf, exports: HashMap<String, Value>) {
        self.exports.insert(path, exports);
    }

    fn get_exports(&self, path: &Path) -> Option<&HashMap<String, Value>> {
        self.exports.get(path)
    }
}

/// Module executor for interpreter-based execution
pub struct ModuleExecutor {
    /// Module loader for resolving and loading dependencies
    loader: ModuleLoader,
    /// Module resolver for path resolution
    resolver: ModuleResolver,
    /// Cache of executed modules
    cache: ModuleCache,
    /// Shared interpreter instance
    interpreter: Interpreter,
    /// Security context for permission checks
    security: SecurityContext,
}

impl ModuleExecutor {
    /// Create a new module executor
    ///
    /// # Arguments
    /// * `root` - Project root directory for module resolution
    /// * `security` - Security context for permission checks
    pub fn new(root: PathBuf, security: SecurityContext) -> Self {
        Self {
            loader: ModuleLoader::new(root.clone()),
            resolver: ModuleResolver::new(root),
            cache: ModuleCache::new(),
            interpreter: Interpreter::new(),
            security,
        }
    }

    /// Execute a module file and all its dependencies
    ///
    /// Loads and executes modules in topological order (dependencies first).
    /// Each module executes exactly once. Returns the entry module's result.
    ///
    /// # Arguments
    /// * `entry_path` - Absolute path to the entry module file
    ///
    /// # Returns
    /// The result value from executing the entry module
    pub fn execute_module(&mut self, entry_path: &Path) -> ModuleResult<Value> {
        // Load all modules in dependency order
        let modules = self.loader.load_module(entry_path)?;

        // Execute each module in order
        let mut last_value = Value::Null;
        for module in modules {
            let result = self.execute_single_module(&module)?;
            // The entry module's result is what we return
            if module.path == entry_path {
                last_value = result;
            }
        }

        Ok(last_value)
    }

    /// Execute a single module
    ///
    /// If the module is already cached, skip execution.
    /// Otherwise, process imports, execute the module, and cache exports.
    fn execute_single_module(&mut self, module: &LoadedModule) -> ModuleResult<Value> {
        // Skip if already executed
        if self.cache.is_cached(&module.path) {
            return Ok(Value::Null);
        }

        // Process imports - inject imported symbols into interpreter globals
        for import in &module.imports {
            self.process_import(import, &module.path)?;
        }

        // Execute the module
        let result = self
            .interpreter
            .eval(&module.ast, &self.security)
            .map_err(|e| {
                vec![Diagnostic::error(
                    format!("Runtime error in module {}: {}", module.path.display(), e),
                    e.span(),
                )]
            })?;

        // Extract and cache exports
        let exports = self.extract_exports(module);
        self.cache.store_exports(module.path.clone(), exports);

        Ok(result)
    }

    /// Process an import declaration
    ///
    /// Resolves the module path, retrieves cached exports, and injects
    /// imported symbols into the interpreter's globals.
    fn process_import(&mut self, import: &ImportDecl, current_path: &Path) -> ModuleResult<()> {
        // Resolve the import path relative to current module
        let import_path = self
            .resolver
            .resolve_path(&import.source, current_path, import.span)
            .map_err(|e| vec![e])?;

        // Get cached exports (module should already be executed due to topological order)
        let exports = self.cache.get_exports(&import_path).ok_or_else(|| {
            vec![Diagnostic::error(
                format!(
                    "Module not yet executed: {}. This indicates a bug in topological sorting.",
                    import_path.display()
                ),
                import.span,
            )]
        })?;

        // Process import specifiers
        for specifier in &import.specifiers {
            match specifier {
                ImportSpecifier::Named { name, span } => {
                    // Import specific named export
                    let value = exports.get(&name.name).ok_or_else(|| {
                        vec![Diagnostic::error_with_code(
                            "AT5004",
                            format!("'{}' is not exported from module", name.name),
                            *span,
                        )
                        .with_help("check the module's exports or import a different symbol")]
                    })?;
                    self.interpreter
                        .define_global(name.name.clone(), value.clone());
                }
                ImportSpecifier::Namespace { alias: _, span } => {
                    // Namespace imports not yet supported in v0.2
                    return Err(vec![Diagnostic::error(
                        "Namespace imports (import * as) not yet implemented".to_string(),
                        *span,
                    )]);
                }
            }
        }

        Ok(())
    }

    /// Extract exports from an executed module
    ///
    /// Examines the module's AST to find exported items and retrieves
    /// their values from the interpreter's globals.
    fn extract_exports(&self, module: &LoadedModule) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        for item in &module.ast.items {
            if let Item::Export(export_decl) = item {
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => {
                        // Get function value from interpreter globals
                        if let Some(value) = self.interpreter.globals.get(&func.name.name) {
                            exports.insert(func.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::Variable(var) => {
                        // Get variable value from interpreter globals
                        if let Some(value) = self.interpreter.globals.get(&var.name.name) {
                            exports.insert(var.name.name.clone(), value.clone());
                        }
                    }
                    crate::ast::ExportItem::TypeAlias(_) => {
                        // Type aliases are compile-time only
                    }
                }
            }
        }

        exports
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test module file
    fn create_test_module(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.atl", name));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_module_executor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _executor =
            ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
        // Executor can be created successfully
    }

    #[test]
    fn test_single_module_no_imports() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = create_test_module(temp_dir.path(), "main", "let x: number = 42;\nx;");

        let mut executor =
            ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
        let result = executor.execute_module(&module_path);

        match result {
            Ok(Value::Number(n)) => assert_eq!(n, 42.0),
            Ok(v) => panic!("Expected Number, got {:?}", v),
            Err(e) => panic!("Execution failed: {:?}", e),
        }
    }

    #[test]
    fn test_module_with_export() {
        let temp_dir = TempDir::new().unwrap();
        let module_path = create_test_module(
            temp_dir.path(),
            "math",
            "export fn add(a: number, b: number) -> number { return a + b; }",
        );

        let mut executor =
            ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
        let result = executor.execute_module(&module_path);

        assert!(result.is_ok());
        // Verify export was cached
        assert!(executor.cache.is_cached(&module_path));
        let exports = executor.cache.get_exports(&module_path).unwrap();
        assert!(exports.contains_key("add"));
    }
}
