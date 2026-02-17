//! Module Loading and Caching
//!
//! Loads module files, builds dependency graphs, and returns modules in topological order.
//! This is BLOCKER 04-B - loading and caching only.
//! Type checking happens in BLOCKER 04-C.

use crate::ast::{ImportDecl, Item, Program};
use crate::diagnostic::Diagnostic;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::resolver::ModuleResolver;
use crate::span::Span;
use crate::symbol::SymbolTable;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

/// A loaded module with its AST and metadata
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Absolute path to the module file
    pub path: PathBuf,
    /// Parsed AST
    pub ast: Program,
    /// List of exported names (for validation in 04-C)
    pub exports: Vec<String>,
    /// List of import declarations (for dependency tracking)
    pub imports: Vec<ImportDecl>,
}

/// Registry of bound modules with their symbol tables
///
/// Used during binding and type checking to resolve cross-module references.
/// This is BLOCKER 04-C - cross-module type checking.
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    /// Map of module path -> symbol table
    modules: HashMap<PathBuf, SymbolTable>,
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a module's symbol table
    pub fn register(&mut self, path: PathBuf, symbol_table: SymbolTable) {
        self.modules.insert(path, symbol_table);
    }

    /// Get a module's symbol table
    pub fn get(&self, path: &Path) -> Option<&SymbolTable> {
        self.modules.get(path)
    }

    /// Get a mutable reference to a module's symbol table
    pub fn get_mut(&mut self, path: &Path) -> Option<&mut SymbolTable> {
        self.modules.get_mut(path)
    }

    /// Check if a module is registered
    pub fn contains(&self, path: &Path) -> bool {
        self.modules.contains_key(path)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Module loader - loads files, builds dependency graphs, performs topological sort
pub struct ModuleLoader {
    /// Module resolver for path resolution
    resolver: ModuleResolver,
    /// Cache of loaded modules (by absolute path)
    cache: HashMap<PathBuf, LoadedModule>,
    /// Dependency graph (module -> its dependencies)
    dependencies: HashMap<PathBuf, Vec<PathBuf>>,
    /// Modules currently being loaded (for cycle detection during loading)
    loading: HashSet<PathBuf>,
}

impl ModuleLoader {
    /// Create a new module loader with the given project root
    pub fn new(root: PathBuf) -> Self {
        Self {
            resolver: ModuleResolver::new(root),
            cache: HashMap::new(),
            dependencies: HashMap::new(),
            loading: HashSet::new(),
        }
    }

    /// Load a module and all its dependencies
    ///
    /// Returns modules in topological order (dependencies first).
    ///
    /// # Arguments
    /// * `entry_point` - Absolute path to the entry module
    ///
    /// # Returns
    /// List of modules in initialization order, or diagnostics if errors occurred
    pub fn load_module(
        &mut self,
        entry_point: &Path,
    ) -> Result<Vec<LoadedModule>, Vec<Diagnostic>> {
        // Load the entry module and all dependencies recursively
        self.load_recursive(entry_point)?;

        // Check for circular dependencies
        self.resolver
            .check_circular(entry_point, Span::dummy())
            .map_err(|e| vec![e])?;

        // Return modules in topological order
        let ordered = self.topological_sort(entry_point)?;

        // Convert paths to loaded modules
        let modules = ordered
            .into_iter()
            .map(|path| {
                self.cache
                    .get(&path)
                    .expect("module should exist in cache after loading")
                    .clone()
            })
            .collect();

        Ok(modules)
    }

    /// Recursively load a module and its dependencies
    fn load_recursive(&mut self, module_path: &Path) -> Result<(), Vec<Diagnostic>> {
        let abs_path = module_path.to_path_buf();

        // Check cache - if already loaded, skip
        if self.cache.contains_key(&abs_path) {
            return Ok(());
        }

        // Check if currently being loaded (circular dependency)
        if self.loading.contains(&abs_path) {
            return Err(vec![Diagnostic::error_with_code(
                "AT5003",
                "Circular dependency detected",
                Span::dummy(),
            )
            .with_label(format!("module: {}", abs_path.display()))
            .with_help(
                "Refactor to remove circular dependencies between modules".to_string(),
            )]);
        }

        // Mark as currently loading
        self.loading.insert(abs_path.clone());

        // Load and parse the module file
        let loaded = self.load_and_parse(&abs_path)?;

        // Extract dependencies from imports (deduplicate)
        let mut deps = Vec::new();
        let mut seen_deps = HashSet::new();

        for import in &loaded.imports {
            // Resolve import path relative to current module
            let dep_path = self
                .resolver
                .resolve_path(&import.source, &abs_path, import.span)
                .map_err(|e| vec![e])?;

            // Skip if already processed (multiple imports from same module)
            if !seen_deps.insert(dep_path.clone()) {
                continue;
            }

            deps.push(dep_path.clone());

            // Add to resolver's dependency graph
            self.resolver
                .add_dependency(abs_path.clone(), dep_path.clone());

            // Recursively load the dependency
            self.load_recursive(&dep_path)?;
        }

        // Store dependencies in our graph
        self.dependencies.insert(abs_path.clone(), deps);

        // Cache the loaded module
        self.cache.insert(abs_path.clone(), loaded);

        // Remove from loading set (done loading)
        self.loading.remove(&abs_path);

        Ok(())
    }

    /// Load and parse a single module file
    fn load_and_parse(&self, path: &Path) -> Result<LoadedModule, Vec<Diagnostic>> {
        // Read file contents
        let source = fs::read_to_string(path).map_err(|e| {
            vec![Diagnostic::error_with_code(
                "AT5002",
                format!("Failed to read module file: {}", e),
                Span::dummy(),
            )
            .with_label(format!("path: {}", path.display()))
            .with_help("ensure the file exists and you have read permissions")]
        })?;

        // Lex
        let mut lexer = Lexer::new(&source);
        let (tokens, lex_diags) = lexer.tokenize();
        if !lex_diags.is_empty() {
            return Err(lex_diags);
        }

        // Parse
        let mut parser = Parser::new(tokens);
        let (ast, parse_diags) = parser.parse();
        if !parse_diags.is_empty() {
            return Err(parse_diags);
        }

        // Extract exports and imports
        let mut exports = Vec::new();
        let mut imports = Vec::new();

        for item in &ast.items {
            match item {
                Item::Export(export_decl) => {
                    let name = match &export_decl.item {
                        crate::ast::ExportItem::Function(func) => func.name.name.clone(),
                        crate::ast::ExportItem::Variable(var) => var.name.name.clone(),
                        crate::ast::ExportItem::TypeAlias(alias) => alias.name.name.clone(),
                    };
                    exports.push(name);
                }
                Item::Import(import_decl) => {
                    imports.push(import_decl.clone());
                }
                _ => {}
            }
        }

        Ok(LoadedModule {
            path: path.to_path_buf(),
            ast,
            exports,
            imports,
        })
    }

    /// Perform topological sort to get initialization order
    ///
    /// Returns modules in dependency order (dependencies before dependents).
    /// Uses Kahn's algorithm.
    /// Only includes modules reachable from the entry point.
    fn topological_sort(&self, entry: &Path) -> Result<Vec<PathBuf>, Vec<Diagnostic>> {
        // First, find all modules reachable from entry using DFS
        let reachable = self.find_reachable(entry);

        // Build in-degree map (count of incoming edges) for reachable nodes only
        let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();

        // Initialize in-degrees for reachable nodes
        for node in &reachable {
            in_degree.insert(node.clone(), 0);
        }

        // Calculate in-degrees (only for reachable nodes)
        for from in &reachable {
            if let Some(deps) = self.dependencies.get(from) {
                for _dep in deps {
                    if reachable.contains(_dep) {
                        *in_degree
                            .get_mut(from)
                            .expect("in_degree should contain all reachable nodes") += 1;
                    }
                }
            }
        }

        // Queue of nodes with no incoming edges
        let mut queue: VecDeque<PathBuf> = VecDeque::new();
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        // Process nodes in topological order
        let mut sorted = Vec::new();
        while let Some(node) = queue.pop_front() {
            sorted.push(node.clone());

            // For each dependent of this node (in reachable set)
            for from in &reachable {
                if let Some(deps) = self.dependencies.get(from) {
                    if deps.contains(&node) {
                        // Decrease in-degree
                        let degree = in_degree
                            .get_mut(from)
                            .expect("in_degree should contain all reachable nodes");
                        *degree -= 1;

                        // If no more dependencies, add to queue
                        if *degree == 0 {
                            queue.push_back(from.clone());
                        }
                    }
                }
            }
        }

        // Check if all reachable nodes were processed (no cycles)
        if sorted.len() != reachable.len() {
            return Err(vec![Diagnostic::error_with_code(
                "AT5003",
                "Circular dependency detected during topological sort",
                Span::dummy(),
            )
            .with_help("refactor your modules to remove circular imports - modules cannot import each other in a cycle")]);
        }

        Ok(sorted)
    }

    /// Find all modules reachable from a given entry point using DFS
    fn find_reachable(&self, entry: &Path) -> HashSet<PathBuf> {
        let mut reachable = HashSet::new();
        let mut stack = vec![entry.to_path_buf()];

        while let Some(node) = stack.pop() {
            if reachable.insert(node.clone()) {
                // If this is a new node, explore its dependencies
                if let Some(deps) = self.dependencies.get(&node) {
                    for dep in deps {
                        stack.push(dep.clone());
                    }
                }
            }
        }

        reachable
    }

    /// Get a loaded module from cache
    pub fn get_module(&self, path: &Path) -> Option<&LoadedModule> {
        self.cache.get(path)
    }

    /// Clear all caches (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.dependencies.clear();
        self.loading.clear();
        self.resolver.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a test module file
    fn create_module(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.atl", name));
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_load_simple_module() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create a simple module
        create_module(
            &root,
            "main",
            r#"
            export fn greet(name: string) -> string {
                return "Hello, " + name;
            }
            "#,
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].path, entry);
        assert_eq!(modules[0].exports, vec!["greet"]);
    }

    #[test]
    fn test_load_with_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create dependency
        create_module(
            &root,
            "math",
            "export fn add(a: number, b: number) -> number { return a + b; }",
        );

        // Create main module
        create_module(
            &root,
            "main",
            "import { add } from \"./math\";\nexport fn calculate() -> number { return add(1, 2); }",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let result = loader.load_module(&entry);

        if let Err(ref diags) = result {
            eprintln!("Diagnostics: {:#?}", diags);
        }

        let modules = result.unwrap();

        assert_eq!(modules.len(), 2);
        // math should come before main (dependency order)
        assert!(modules[0].path.ends_with("math.atl"));
        assert!(modules[1].path.ends_with("main.atl"));
    }

    #[test]
    fn test_caching_prevents_reload() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "shared", "export let x = 42;");
        create_module(
            &root,
            "a",
            "import { x } from \"./shared\";\nexport let a = x + 1;",
        );
        create_module(
            &root,
            "b",
            "import { x } from \"./shared\";\nexport let b = x + 2;",
        );
        create_module(
            &root,
            "main",
            "import { a } from \"./a\";\nimport { b } from \"./b\";",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        // shared, a, b, main = 4 modules
        assert_eq!(modules.len(), 4);

        // shared should appear only once (cached)
        let shared_count = modules
            .iter()
            .filter(|m| m.path.ends_with("shared.atl"))
            .count();
        assert_eq!(shared_count, 1);
    }

    #[test]
    fn test_circular_dependency_detected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "a", "import { b } from \"./b\";\nexport let a = 1;");
        create_module(&root, "b", "import { a } from \"./a\";\nexport let b = 2;");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].message.contains("Circular dependency"));
    }

    #[test]
    fn test_three_level_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "c", "export fn c() -> number { return 3; }");
        create_module(
            &root,
            "b",
            "import { c } from \"./c\";\nexport fn b() -> number { return c(); }",
        );
        create_module(
            &root,
            "a",
            "import { b } from \"./b\";\nexport fn a() -> number { return b(); }",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 3);
        // Order should be: c, b, a
        assert!(modules[0].path.ends_with("c.atl"));
        assert!(modules[1].path.ends_with("b.atl"));
        assert!(modules[2].path.ends_with("a.atl"));
    }

    #[test]
    fn test_diamond_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "d", "export let x = 1;");
        create_module(&root, "b", "import { x } from \"./d\";\nexport let b = x;");
        create_module(&root, "c", "import { x } from \"./d\";\nexport let c = x;");
        create_module(
            &root,
            "a",
            "import { b } from \"./b\";\nimport { c } from \"./c\";\nexport let a = b + c;",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 4);
        // d should come first
        assert!(modules[0].path.ends_with("d.atl"));
        // a should come last
        assert!(modules[3].path.ends_with("a.atl"));
    }

    #[test]
    fn test_module_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "main", "import { x } from \"./missing\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].message.contains("Module not found"));
    }

    #[test]
    fn test_parse_error_in_module() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "broken", "export fn bad syntax");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("broken.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
    }

    #[test]
    fn test_export_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "lib",
            "export fn foo() -> void {}\nexport let bar = 42;\nexport var baz = true;",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("lib.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].exports.len(), 3);
        assert!(modules[0].exports.contains(&"foo".to_string()));
        assert!(modules[0].exports.contains(&"bar".to_string()));
        assert!(modules[0].exports.contains(&"baz".to_string()));
    }

    #[test]
    fn test_namespace_import() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "lib", "export fn foo() -> void {}");
        create_module(&root, "main", "import * as lib from \"./lib\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_multiple_imports_from_same_module() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "lib",
            "export fn a() -> void {}\nexport fn b() -> void {}",
        );
        create_module(
            &root,
            "main",
            "import { a } from \"./lib\";\nimport { b } from \"./lib\";",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_absolute_path_import() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create nested directory
        let lib_dir = root.join("lib");
        fs::create_dir(&lib_dir).unwrap();
        create_module(&lib_dir, "util", "export fn helper() -> void {}");

        create_module(&root, "main", "import { helper } from \"/lib/util\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_relative_parent_import() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "util", "export fn helper() -> void {}");

        let sub_dir = root.join("sub");
        fs::create_dir(&sub_dir).unwrap();
        create_module(&sub_dir, "main", "import { helper } from \"../util\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = sub_dir.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_complex_dependency_graph() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        /*
         * Dependency graph:
         *     a
         *    / \
         *   b   c
         *  / \ /
         * d   e
         */
        create_module(&root, "d", "export let d = 1;");
        create_module(&root, "e", "export let e = 2;");
        create_module(
            &root,
            "b",
            "import { d } from \"./d\";\nimport { e } from \"./e\";\nexport let b = d + e;",
        );
        create_module(&root, "c", "import { e } from \"./e\";\nexport let c = e;");
        create_module(
            &root,
            "a",
            "import { b } from \"./b\";\nimport { c } from \"./c\";\nexport let a = b + c;",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 5);
        // a should be last
        assert!(modules[4].path.ends_with("a.atl"));
        // d and e should come before b and c
        let d_idx = modules
            .iter()
            .position(|m| m.path.ends_with("d.atl"))
            .unwrap();
        let e_idx = modules
            .iter()
            .position(|m| m.path.ends_with("e.atl"))
            .unwrap();
        let b_idx = modules
            .iter()
            .position(|m| m.path.ends_with("b.atl"))
            .unwrap();
        let c_idx = modules
            .iter()
            .position(|m| m.path.ends_with("c.atl"))
            .unwrap();

        assert!(d_idx < b_idx);
        assert!(e_idx < b_idx);
        assert!(e_idx < c_idx);
    }

    #[test]
    fn test_self_import_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "main",
            "import { x } from \"./main\";\nexport let x = 1;",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].message.contains("Circular dependency"));
    }

    #[test]
    fn test_empty_module() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "empty", "");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("empty.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert!(modules[0].exports.is_empty());
        assert!(modules[0].imports.is_empty());
    }

    #[test]
    fn test_module_with_only_imports() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "lib", "export fn foo() -> void {}");
        create_module(&root, "consumer", "import { foo } from \"./lib\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("consumer.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
        assert!(modules[1].exports.is_empty());
    }

    #[test]
    fn test_module_with_only_exports() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "lib",
            "export fn foo() -> void {}\nexport let bar = 1;",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("lib.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].exports.len(), 2);
        assert!(modules[0].imports.is_empty());
    }

    #[test]
    fn test_deep_nesting() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create a chain of 10 modules: a0 -> a1 -> a2 -> ... -> a9
        for i in 0..10 {
            let content = if i == 9 {
                "export let value = 1;".to_string()
            } else {
                format!(
                    "import {{ value }} from \"./a{}\";\nexport let value = value;",
                    i + 1
                )
            };
            create_module(&root, &format!("a{}", i), &content);
        }

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a0.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 10);
        // Last module (a9) should be first in topological order
        assert!(modules[0].path.ends_with("a9.atl"));
        // First module (a0) should be last
        assert!(modules[9].path.ends_with("a0.atl"));
    }

    #[test]
    fn test_wide_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create 10 independent modules
        for i in 0..10 {
            create_module(&root, &format!("lib{}", i), "export let value = 1;");
        }

        // Create main that imports from all of them
        let imports: Vec<String> = (0..10)
            .map(|i| format!("import {{ value }} from \"./lib{}\";", i))
            .collect();
        create_module(&root, "main", &imports.join("\n"));

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 11);
        // Main should be last
        assert!(modules[10].path.ends_with("main.atl"));
    }

    #[test]
    fn test_long_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create a cycle: a -> b -> c -> d -> e -> a
        create_module(&root, "a", "import { e } from \"./e\";\nexport let a = 1;");
        create_module(&root, "b", "import { a } from \"./a\";\nexport let b = 1;");
        create_module(&root, "c", "import { b } from \"./b\";\nexport let c = 1;");
        create_module(&root, "d", "import { c } from \"./c\";\nexport let d = 1;");
        create_module(&root, "e", "import { d } from \"./d\";\nexport let e = 1;");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("b.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].message.contains("Circular dependency"));
    }

    #[test]
    fn test_indirect_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        // a -> b, b -> c, c -> a (indirect cycle)
        create_module(&root, "a", "import { b } from \"./b\";\nexport let a = 1;");
        create_module(&root, "b", "import { c } from \"./c\";\nexport let b = 1;");
        create_module(&root, "c", "import { a } from \"./a\";\nexport let c = 1;");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("a.atl");
        let result = loader.load_module(&entry);

        assert!(result.is_err());
        let diags = result.unwrap_err();
        assert!(diags[0].message.contains("Circular dependency"));
    }

    #[test]
    fn test_module_with_same_name_different_directories() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        let dir_a = root.join("a");
        let dir_b = root.join("b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        create_module(&dir_a, "utils", "export let valueA = 1;");
        create_module(&dir_b, "utils", "export let valueB = 2;");
        create_module(
            &root,
            "main",
            "import { valueA } from \"/a/utils\";\nimport { valueB } from \"/b/utils\";",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 3);
    }

    #[test]
    fn test_module_path_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "lib", "export let value = 1;");
        create_module(&root, "main", "import { value } from \"./lib.atl\";");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_no_exports_no_imports() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "main", "let x = 1;");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert!(modules[0].exports.is_empty());
        assert!(modules[0].imports.is_empty());
    }

    #[test]
    fn test_mixed_export_types() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "lib",
            "export fn func1() -> void {}\nexport let const1 = 1;\nexport var var1 = 2;\nexport fn func2() -> void {}",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("lib.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].exports.len(), 4);
        assert!(modules[0].exports.contains(&"func1".to_string()));
        assert!(modules[0].exports.contains(&"const1".to_string()));
        assert!(modules[0].exports.contains(&"var1".to_string()));
        assert!(modules[0].exports.contains(&"func2".to_string()));
    }

    #[test]
    fn test_reusing_module_loader() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "shared", "export let shared = 0;");
        create_module(
            &root,
            "a",
            "import { shared } from \"./shared\";\nexport let a = 1;",
        );
        create_module(
            &root,
            "b",
            "import { shared } from \"./shared\";\nexport let b = 2;",
        );

        let mut loader = ModuleLoader::new(root.clone());

        // Load first module tree (shared, a)
        let entry_a = root.join("a.atl");
        let modules_a = loader.load_module(&entry_a).unwrap();
        assert_eq!(modules_a.len(), 2);

        // Load second module tree (shared already cached, so just b)
        // But load_module returns all dependencies, so we get both shared and b
        let entry_b = root.join("b.atl");
        let modules_b = loader.load_module(&entry_b).unwrap();
        assert_eq!(modules_b.len(), 2); // shared + b

        // Verify shared module is only loaded once (same instance)
        let shared_path = root.join("shared.atl");
        assert_eq!(loader.get_module(&shared_path).unwrap().exports.len(), 1);
    }

    #[test]
    fn test_complex_import_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(
            &root,
            "lib",
            "export fn a() -> void {}\nexport fn b() -> void {}",
        );
        create_module(
            &root,
            "main",
            "import { a } from \"./lib\";\nimport { b } from \"./lib\";\nimport * as lib from \"./lib\";",
        );

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("main.atl");
        let modules = loader.load_module(&entry).unwrap();

        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_get_module_from_cache() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().to_path_buf();

        create_module(&root, "lib", "export let value = 1;");

        let mut loader = ModuleLoader::new(root.clone());
        let entry = root.join("lib.atl");
        loader.load_module(&entry).unwrap();

        // Get module from cache
        let cached = loader.get_module(&entry);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().exports.len(), 1);

        // Try to get non-existent module
        let missing = loader.get_module(&root.join("missing.atl"));
        assert!(missing.is_none());
    }
}
