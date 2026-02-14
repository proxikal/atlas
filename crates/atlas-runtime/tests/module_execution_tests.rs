//! Module Execution Tests (BLOCKER 04-D)
//!
//! Tests for runtime module execution in both interpreter and VM.
//! Verifies single evaluation, proper initialization order, and export/import functionality.

use atlas_runtime::{ModuleExecutor, Value};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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
        "export let PI: number = 3.14159;",
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { PI } from "./constants";
PI * 2;
"#,
    );

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    match result {
        Ok(Value::Number(n)) => assert!((n - 6.28318).abs() < 0.00001),
        Ok(v) => panic!("Expected Number(6.28318), got {:?}", v),
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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
export fn concat(a: string, b: string) -> string {
    return a + b;
}
"#,
    );

    let main = create_module(
        temp_dir.path(),
        "main",
        r#"
import { concat } from "./string_ops";
concat("Hello", " World");
"#,
    );

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
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

    let mut executor = ModuleExecutor::new(temp_dir.path().to_path_buf());
    let result = executor.execute_module(&main);

    // Should fail - SECRET is not exported
    assert!(result.is_err());
}
