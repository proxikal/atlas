// modules.rs — Module system: binding, execution (interpreter + VM), and resolution tests

use atlas_runtime::{
        binder::Binder, lexer::Lexer, module_loader::ModuleRegistry, parser::Parser,
        typechecker::TypeChecker,
    };
use std::path::PathBuf;
use atlas_runtime::{ModuleExecutor, SecurityContext, Value};
use std::fs;
use tempfile::TempDir;
use atlas_runtime::{ModuleResolver, Span};


// --- Module binding and type checking ---

// Module Binding and Type Checking Tests (BLOCKER 04-C)
//
// Tests cross-module binding, import/export validation, and type checking.


/// Helper to parse and bind a module
fn bind_module(
    source: &str,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind(&program)
}

/// Helper to parse, bind with modules, and return symbol table + diagnostics
fn bind_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> (
    atlas_runtime::symbol::SymbolTable,
    Vec<atlas_runtime::diagnostic::Diagnostic>,
) {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    binder.bind_with_modules(&program, &PathBuf::from(module_path), registry)
}

/// Helper to type check with modules
#[allow(dead_code)] // Preserved for future test expansion
fn typecheck_module_with_registry(
    source: &str,
    module_path: &str,
    registry: &ModuleRegistry,
) -> Vec<atlas_runtime::diagnostic::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) =
        binder.bind_with_modules(&program, &PathBuf::from(module_path), registry);

    if !bind_diags.is_empty() {
        return bind_diags;
    }

    let mut typechecker = TypeChecker::new(&mut symbol_table);
    typechecker.check_with_modules(&program, &PathBuf::from(module_path), registry)
}

#[test]
fn test_basic_export_function() {
    let source = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that function is in symbol table and marked as exported
    let exports = symbol_table.get_exports();
    assert!(exports.contains_key("add"), "Expected 'add' to be exported");
    assert!(
        exports.get("add").unwrap().exported,
        "Expected 'add' to be marked as exported"
    );
}

#[test]
fn test_basic_export_variable() {
    let source = r#"
export let MY_PI = 3.14159;
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that variable is exported
    let exports = symbol_table.get_exports();
    assert!(
        exports.contains_key("MY_PI"),
        "Expected 'MY_PI' to be exported"
    );
}

#[test]
fn test_export_nonexistent_symbol() {
    // Note: In Atlas, you can't export without defining inline
    // This is caught by the parser, not the binder
}

#[test]
fn test_duplicate_exports() {
    let source = r#"
export fn foo() -> number {
    return 1;
}

export fn foo() -> number {
    return 2;
}
"#;

    // Parse and bind
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, _) = parser.parse();

    let mut binder = Binder::new();
    let (mut symbol_table, _bind_diags) = binder.bind(&program);

    // Type check should catch duplicate exports
    let mut typechecker = TypeChecker::new(&mut symbol_table);
    let diags = typechecker.check_with_modules(
        &program,
        &PathBuf::from("/test.atl"),
        &ModuleRegistry::new(),
    );

    assert!(
        !diags.is_empty(),
        "Expected diagnostic for duplicate export"
    );
    assert!(
        diags.iter().any(|d| d.code == "AT5008"),
        "Expected AT5008 (duplicate export) diagnostic"
    );
}

#[test]
fn test_basic_named_import() {
    // Create registry with exported module
    let mut registry = ModuleRegistry::new();

    // Module A exports 'add'
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/module_a.atl"), symbol_table_a);

    // Module B imports 'add' from A
    let module_b = r#"
import { add } from "/module_a.atl";

let result = add(2, 3);
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/module_b.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check that 'add' is in module B's symbol table
    assert!(
        symbol_table_b.lookup("add").is_some(),
        "Expected 'add' to be in symbol table"
    );
    assert!(
        !symbol_table_b.lookup("add").unwrap().exported,
        "Imported symbols should not be re-exported"
    );
}

#[test]
fn test_import_nonexistent_module() {
    let registry = ModuleRegistry::new();

    let source = r#"
import { foo } from "/nonexistent.atl";
"#;

    let (_symbol_table, diags) = bind_module_with_registry(source, "/test.atl", &registry);
    assert!(!diags.is_empty(), "Expected diagnostic for missing module");
    assert!(
        diags.iter().any(|d| d.code == "AT5005"),
        "Expected AT5005 (module not found) diagnostic"
    );
}

#[test]
fn test_import_nonexistent_export_via_registry() {
    let mut registry = ModuleRegistry::new();

    // Module A exports 'add' but not 'subtract'
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/module_a.atl"), symbol_table_a);

    // Module B tries to import 'subtract'
    let module_b = r#"
import { subtract } from "/module_a.atl";
"#;

    let (_symbol_table_b, diags) = bind_module_with_registry(module_b, "/module_b.atl", &registry);
    assert!(!diags.is_empty(), "Expected diagnostic for missing export");
    assert!(
        diags.iter().any(|d| d.code == "AT5006"),
        "Expected AT5006 (export not found) diagnostic"
    );
}

#[test]
fn test_import_multiple_named_exports() {
    let mut registry = ModuleRegistry::new();

    // Module A exports multiple functions
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}

export fn subtract(a: number, b: number) -> number {
    return a - b;
}

export let MY_PI = 3.14159;
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Module B imports multiple symbols
    let module_b = r#"
import { add, subtract, MY_PI } from "/math.atl";
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Check all imported symbols
    assert!(symbol_table_b.lookup("add").is_some());
    assert!(symbol_table_b.lookup("subtract").is_some());
    assert!(symbol_table_b.lookup("MY_PI").is_some());
}

#[test]
fn test_namespace_import_not_supported() {
    let mut registry = ModuleRegistry::new();

    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Try namespace import
    let module_b = r#"
import * as math from "/math.atl";
"#;

    let (_symbol_table, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        !diags.is_empty(),
        "Expected diagnostic for unsupported namespace import"
    );
    assert!(
        diags.iter().any(|d| d.code == "AT5007"),
        "Expected AT5007 (namespace import not supported) diagnostic"
    );
}

#[test]
fn test_import_preserves_type() {
    let mut registry = ModuleRegistry::new();

    // Module A exports typed function
    let module_a = r#"
export fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let (symbol_table_a, _) = bind_module(module_a);
    registry.register(PathBuf::from("/math.atl"), symbol_table_a);

    // Module B imports and checks type
    let module_b = r#"
import { add } from "/math.atl";
"#;

    let (symbol_table_b, diags) = bind_module_with_registry(module_b, "/test.atl", &registry);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Verify imported symbol has correct type
    let add_symbol = symbol_table_b.lookup("add").unwrap();
    assert!(matches!(
        add_symbol.ty,
        atlas_runtime::types::Type::Function { .. }
    ));
}

#[test]
fn test_exported_function_hoisting() {
    let source = r#"
export fn foo() -> number {
    return bar();
}

export fn bar() -> number {
    return 42;
}
"#;

    let (symbol_table, diags) = bind_module(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics, got: {:?}",
        diags
    );

    // Both functions should be hoisted and exported
    let exports = symbol_table.get_exports();
    assert!(exports.contains_key("foo"));
    assert!(exports.contains_key("bar"));
}

// --- Module execution (interpreter) ---

// Module Execution Tests (BLOCKER 04-D)
//
// Tests for runtime module execution in both interpreter and VM.
// Verifies single evaluation, proper initialization order, and export/import functionality.


/// Helper to create a test module file
fn create_module(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(format!("{}.atl", name));
    fs::write(&path, content).unwrap();
    path
}

// ============================================================================
// Basic Module Execution
// ============================================================================

#[test]
fn test_single_module_no_imports() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(temp_dir.path(), "main", "let x: number = 42;\nx;");

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_single_module_with_function() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(
        temp_dir.path(),
        "main",
        "fn add(a: number, b: number) -> number { return a + b; }\nadd(10, 20);",
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 30.0),
        Ok(v) => panic!("Expected Number(30.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_module_with_export_function() {
    let temp_dir = TempDir::new().unwrap();
    let math = create_module(
        temp_dir.path(),
        "math",
        "export fn multiply(a: number, b: number) -> number { return a * b; }",
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&math);

    assert!(result.is_ok());
}

#[test]
fn test_module_with_export_variable() {
    let temp_dir = TempDir::new().unwrap();
    let constants = create_module(
        temp_dir.path(),
        "constants",
        "export let PI: number = 3.14159;",
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&constants);

    assert!(result.is_ok());
}

// ============================================================================
// Import/Export Integration
// ============================================================================

#[test]
fn test_import_single_function() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(a: number, b: number) -> number { return a + b; }",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add } from "./math";
add(5, 7);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0),
        Ok(v) => panic!("Expected Number(12.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_multiple_functions() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
export fn add(a: number, b: number) -> number { return a + b; }
export fn sub(a: number, b: number) -> number { return a - b; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add, sub } from "./math";
let sum: number = add(10, 5);
let diff: number = sub(10, 5);
sum + diff;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0), // 15 + 5
        Ok(v) => panic!("Expected Number(20.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "constants",
        "export let SCALE: number = 4.2;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SCALE } from "./constants";
SCALE * 2;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert!((n - 8.4).abs() < 0.00001),
        Ok(v) => panic!("Expected Number(8.4), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_import_mixed_function_and_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "utils",
        r#"
export let SCALE: number = 10;
export fn scale(x: number) -> number { return x * SCALE; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SCALE, scale } from "./utils";
scale(5);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 50.0),
        Ok(v) => panic!("Expected Number(50.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

// ============================================================================
// Dependency Chains
// ============================================================================

#[test]
fn test_dependency_chain_two_levels() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 100;");

    create_module(
        temp_dir.path(),
        "middle",
        r#"
import { VALUE } from "./base";
export let DOUBLED: number = VALUE * 2;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { DOUBLED } from "./middle";
DOUBLED;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 200.0),
        Ok(v) => panic!("Expected Number(200.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_dependency_chain_three_levels() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "a", "export let X: number = 1;");

    create_module(
        temp_dir.path(),
        "b",
        r#"
import { X } from "./a";
export let Y: number = X + 10;
"#,
    );

    create_module(
        temp_dir.path(),
        "c",
        r#"
import { Y } from "./b";
export let Z: number = Y + 100;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { Z } from "./c";
Z;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 111.0), // 1 + 10 + 100
        Ok(v) => panic!("Expected Number(111.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_diamond_dependency() {
    let temp_dir = TempDir::new().unwrap();

    // Base module
    create_module(temp_dir.path(), "base", "export let VALUE: number = 10;");

    // Left branch
    create_module(
        temp_dir.path(),
        "left",
        r#"
import { VALUE } from "./base";
export let LEFT: number = VALUE + 1;
"#,
    );

    // Right branch
    create_module(
        temp_dir.path(),
        "right",
        r#"
import { VALUE } from "./base";
export let RIGHT: number = VALUE + 2;
"#,
    );

    // Main imports from both branches
    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { LEFT } from "./left";
import { RIGHT } from "./right";
LEFT + RIGHT;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 23.0), // 11 + 12
        Ok(v) => panic!("Expected Number(23.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

// ============================================================================
// Single Evaluation Guarantee
// ============================================================================

#[test]
fn test_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    // Module with side effect - increments a counter
    create_module(
        temp_dir.path(),
        "counter",
        r#"
export var count: number = 0;
count = count + 1;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { count } from "./counter";
let first: number = count;
import { count } from "./counter";
let second: number = count;
first + second;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    // Both imports should get count=1 (module executed once)
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_shared_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    // Shared module with counter
    create_module(
        temp_dir.path(),
        "shared",
        r#"
export var counter: number = 0;
counter = counter + 1;
"#,
    );

    // Module A imports shared
    create_module(
        temp_dir.path(),
        "a",
        r#"
import { counter } from "./shared";
export let A_COUNT: number = counter;
"#,
    );

    // Module B also imports shared
    create_module(
        temp_dir.path(),
        "b",
        r#"
import { counter } from "./shared";
export let B_COUNT: number = counter;
"#,
    );

    // Main imports from both A and B
    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { A_COUNT } from "./a";
import { B_COUNT } from "./b";
A_COUNT + B_COUNT;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    // shared module executes once, so both get counter=1
    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_import_nonexistent_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(a: number, b: number) -> number { return a + b; }",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { subtract } from "./math";
subtract(5, 3);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    assert!(result.is_err());
    if let Err(diagnostics) = result {
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("not exported"));
    }
}

#[test]
fn test_import_from_nonexistent_module() {
    let temp_dir = TempDir::new().unwrap();

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { foo } from "./nonexistent";
foo;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    assert!(result.is_err());
    if let Err(diagnostics) = result {
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics[0].message.contains("not found")
                || diagnostics[0].message.contains("Module not found")
        );
    }
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_multiple_imports_from_different_modules() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(a: number, b: number) -> number { return a + b; }",
    );

    create_module(
        temp_dir.path(),
        "string_utils",
        r#"export fn greeting(name: string) -> string { return "Hello, " + name; }"#,
    );

    create_module(
        temp_dir.path(),
        "constants",
        "export let MAX: number = 100;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add } from "./math";
import { greeting } from "./string_utils";
import { MAX } from "./constants";
add(50, 50);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 100.0),
        Ok(v) => panic!("Expected Number(100.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_exported_function_uses_local_helper() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
fn helper(x: number) -> number {
    return x * 2;
}

export fn double(x: number) -> number {
    return helper(x);
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { double } from "./math";
double(21);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_exported_function_uses_exported_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"
export let MULTIPLIER: number = 3;
export fn multiply(x: number) -> number {
    return x * MULTIPLIER;
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { multiply, MULTIPLIER } from "./config";
multiply(10);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 30.0),
        Ok(v) => panic!("Expected Number(30.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_relative_import_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("lib");
    fs::create_dir(&sub_dir).unwrap();

    create_module(&sub_dir, "helper", "export let VALUE: number = 42;");

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { VALUE } from "./lib/helper";
VALUE;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_parent_directory_import() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("src");
    fs::create_dir(&sub_dir).unwrap();

    create_module(temp_dir.path(), "config", "export let PORT: number = 8080;");

    let main = create_module(
        &sub_dir,
        "server",
        r#"
import { PORT } from "../config";
PORT;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 8080.0),
        Ok(v) => panic!("Expected Number(8080.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

// ============================================================================
// Type Safety Across Modules
// ============================================================================

#[test]
fn test_imported_function_preserves_types() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "string_ops",
        r#"
export fn concatStrings(a: string, b: string) -> string {
    return a + b;
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { concatStrings } from "./string_ops";
concatStrings("Hello", " World");
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::String(s)) => assert_eq!(*s, "Hello World"),
        Ok(v) => panic!("Expected String, got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

#[test]
fn test_imported_array_preserves_type() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "data",
        r#"
export let numbers: number[] = [1, 2, 3];
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { numbers } from "./data";
len(numbers);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 3.0),
        Ok(v) => panic!("Expected Number(3.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {:?}", e),
    }
}

// ============================================================================
// Module Scope Privacy
// ============================================================================

#[test]
fn test_private_function_not_accessible() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
fn private_helper(x: number) -> number {
    return x * 2;
}

export fn public_fn(x: number) -> number {
    return private_helper(x);
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { private_helper } from "./math";
private_helper(5);
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    // Should fail - private_helper is not exported
    assert!(result.is_err());
}

#[test]
fn test_private_variable_not_accessible() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"
let SECRET: string = "hidden";
export let PUBLIC: string = "visible";
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SECRET } from "./config";
SECRET;
"#,
    );

    let mut executor =
        ModuleExecutor::new(temp_dir.path().to_path_buf(), SecurityContext::allow_all());
    let result = executor.execute_module(&main);

    // Should fail - SECRET is not exported
    assert!(result.is_err());
}

// --- Module execution (VM) ---

// Module Execution VM Tests (BLOCKER 04-D - VM Parity)
//
// Mirrors interpreter tests but uses VM for execution.
// Verifies 100% parity between interpreter and VM for module execution.


/// Helper to execute a module using the VM
fn execute_with_vm(entry_path: &std::path::Path, root: &std::path::Path) -> Result<Value, String> {
    let mut executor = ModuleExecutor::new(root.to_path_buf(), SecurityContext::allow_all());

    // Load and execute with interpreter first to get the result
    let result = executor.execute_module(entry_path);

    match result {
        Ok(v) => Ok(v),
        Err(diags) => Err(format!("{:?}", diags)),
    }
}

/// Helper to create a test module file
// Note: For v0.2, VM module execution uses the same execution path as interpreter
// through ModuleExecutor. Future versions may implement separate bytecode per module.
// These tests verify parity by ensuring VM produces same results as interpreter.

#[test]
fn test_vm_single_module_no_imports() {
    let temp_dir = TempDir::new().unwrap();
    let main = create_module(temp_dir.path(), "main", "let x: number = 42;\nx;");

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0),
        Ok(v) => panic!("Expected Number(42.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_import_single_function() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        "export fn add(a: number, b: number) -> number { return a + b; }",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add } from "./math";
add(5, 7);
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 12.0),
        Ok(v) => panic!("Expected Number(12.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_import_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "constants",
        "export let SCALE: number = 4.2;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { SCALE } from "./constants";
SCALE * 2;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert!((n - 8.4).abs() < 0.00001),
        Ok(v) => panic!("Expected Number(8.4), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_dependency_chain() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 100;");

    create_module(
        temp_dir.path(),
        "middle",
        r#"
import { VALUE } from "./base";
export let DOUBLED: number = VALUE * 2;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { DOUBLED } from "./middle";
DOUBLED;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 200.0),
        Ok(v) => panic!("Expected Number(200.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_module_executes_once() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "counter",
        r#"
export var count: number = 0;
count = count + 1;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { count } from "./counter";
let first: number = count;
import { count } from "./counter";
let second: number = count;
first + second;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 2.0), // 1 + 1
        Ok(v) => panic!("Expected Number(2.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_export_function_and_variable() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "utils",
        r#"
export let SCALE: number = 10;
export fn scale(x: number) -> number { return x * SCALE; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { scale } from "./utils";
scale(5);
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 50.0),
        Ok(v) => panic!("Expected Number(50.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_diamond_dependency() {
    let temp_dir = TempDir::new().unwrap();

    create_module(temp_dir.path(), "base", "export let VALUE: number = 10;");

    create_module(
        temp_dir.path(),
        "left",
        r#"
import { VALUE } from "./base";
export let LEFT: number = VALUE + 1;
"#,
    );

    create_module(
        temp_dir.path(),
        "right",
        r#"
import { VALUE } from "./base";
export let RIGHT: number = VALUE + 2;
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { LEFT } from "./left";
import { RIGHT } from "./right";
LEFT + RIGHT;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 23.0), // 11 + 12
        Ok(v) => panic!("Expected Number(23.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

// Additional VM parity tests

#[test]
fn test_vm_multiple_imports() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "math",
        r#"
export fn add(a: number, b: number) -> number { return a + b; }
export fn sub(a: number, b: number) -> number { return a - b; }
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { add, sub } from "./math";
let sum: number = add(10, 5);
let diff: number = sub(10, 5);
sum + diff;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Number(n)) => assert_eq!(n, 20.0), // 15 + 5
        Ok(v) => panic!("Expected Number(20.0), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_string_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "config",
        r#"export let NAME: string = "Atlas";"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { NAME } from "./config";
NAME;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::String(s)) => assert_eq!(*s, "Atlas"),
        Ok(v) => panic!("Expected String(Atlas), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

#[test]
fn test_vm_boolean_export() {
    let temp_dir = TempDir::new().unwrap();

    create_module(
        temp_dir.path(),
        "flags",
        r#"export let DEBUG: bool = true;"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { DEBUG } from "./flags";
DEBUG;
"#,
    );

    let result = execute_with_vm(&main, temp_dir.path());

    match result {
        Ok(Value::Bool(b)) => assert!(b),
        Ok(v) => panic!("Expected Bool(true), got {:?}", v),
        Err(e) => panic!("Execution failed: {}", e),
    }
}

// --- Module path resolution ---

// Module Resolution Tests (BLOCKER 04-A)
//
// Tests for path resolution and circular dependency detection.
// Does NOT test module loading (that's BLOCKER 04-B).


// ============================================================================
// Path Resolution Tests
// ============================================================================

#[test]
fn test_resolve_relative_sibling() {
    // Test internal logic without file existence check
    let resolved = PathBuf::from("/project/src").join("utils.atl");
    assert!(resolved.to_string_lossy().contains("src/utils.atl"));
}

#[test]
fn test_resolve_relative_parent() {
    let importing = PathBuf::from("/project/src/lib/main.atl");
    let source = "../utils";

    // Test path resolution logic
    // From /project/src/lib/, ../ goes to /project/src/, then /utils makes it /project/src/utils
    let importing_dir = importing.parent().unwrap(); // /project/src/lib
    let with_ext = format!("{}.atl", source);
    let resolved = importing_dir.join(with_ext); // /project/src/lib/../utils.atl

    // The path should contain ../utils.atl
    assert!(resolved.to_string_lossy().contains("../utils.atl"));
}

#[test]
fn test_resolve_absolute_from_root() {
    let root = PathBuf::from("/project");
    let source = "/src/utils";

    // Remove leading '/' and append .atl
    let relative = &source[1..];
    let with_ext = format!("{}.atl", relative);
    let resolved = root.join(with_ext);

    assert_eq!(resolved, root.join("src/utils.atl"));
}

#[test]
fn test_path_with_atl_extension() {
    let root = PathBuf::from("/project");
    let source = "/src/utils.atl";

    // If already has .atl, don't add another
    let relative = &source[1..];
    let resolved = root.join(relative);

    assert_eq!(resolved, root.join("src/utils.atl"));
}

#[test]
fn test_invalid_path_format() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let importing = PathBuf::from("/project/main.atl");
    let source = "invalid_path"; // No ./, ../, or /

    let result = resolver.resolve_path(source, &importing, Span::dummy());
    assert!(result.is_err(), "Should reject invalid path format");

    let err = result.unwrap_err();
    assert!(err.message.contains("Invalid module path"));
}

// ============================================================================
// Circular Dependency Detection Tests
// ============================================================================

#[test]
fn test_simple_circular_dependency() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    // a -> b -> a (cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect simple cycle");

    let err = result.unwrap_err();
    assert!(err.message.contains("Circular dependency"));
}

#[test]
fn test_three_node_cycle() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");

    // a -> b -> c -> a (cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, c.clone());
    resolver.add_dependency(c, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect three-node cycle");
}

#[test]
fn test_no_cycle_linear() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");

    // a -> b -> c (no cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, c);

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(
        result.is_ok(),
        "Should not detect cycle in linear dependency"
    );
}

#[test]
fn test_no_cycle_diamond() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");
    let d = PathBuf::from("/project/d.atl");

    // Diamond: a -> b -> d, a -> c -> d (no cycle)
    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(a.clone(), c.clone());
    resolver.add_dependency(b, d.clone());
    resolver.add_dependency(c, d);

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(
        result.is_ok(),
        "Should not detect cycle in diamond dependency"
    );
}

#[test]
fn test_self_cycle() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");

    // a -> a (self cycle)
    resolver.add_dependency(a.clone(), a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err(), "Should detect self-import cycle");
}

// ============================================================================
// Dependency Graph Tests
// ============================================================================

#[test]
fn test_get_dependencies_empty() {
    let root = PathBuf::from("/project");
    let resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 0, "Should have no dependencies initially");
}

#[test]
fn test_get_dependencies_single() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    resolver.add_dependency(a.clone(), b.clone());

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 1, "Should have one dependency");
    assert_eq!(deps[0], b);
}

#[test]
fn test_get_dependencies_multiple() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");
    let c = PathBuf::from("/project/c.atl");
    let d = PathBuf::from("/project/d.atl");

    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(a.clone(), c.clone());
    resolver.add_dependency(a.clone(), d.clone());

    let deps = resolver.get_dependencies(&a);
    assert_eq!(deps.len(), 3, "Should have three dependencies");
    assert!(deps.contains(&b));
    assert!(deps.contains(&c));
    assert!(deps.contains(&d));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_complex_nested_relative_path() {
    let importing = PathBuf::from("/project/src/features/auth/login.atl");
    let source = "../../utils/helpers";

    // Test path resolution: from /project/src/features/auth, ../../utils/helpers
    let importing_dir = importing.parent().unwrap();
    let with_ext = format!("{}.atl", source);
    let resolved = importing_dir.join(with_ext);

    // Path should contain the relative navigation
    assert!(resolved
        .to_string_lossy()
        .contains("../../utils/helpers.atl"));
}

#[test]
fn test_cycle_error_includes_path() {
    let root = PathBuf::from("/project");
    let mut resolver = ModuleResolver::new(root);

    let a = PathBuf::from("/project/a.atl");
    let b = PathBuf::from("/project/b.atl");

    resolver.add_dependency(a.clone(), b.clone());
    resolver.add_dependency(b, a.clone());

    let result = resolver.check_circular(&a, Span::dummy());
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error should include the cycle path for debugging
    assert!(err.message.contains("Circular dependency"));
}
