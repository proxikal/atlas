//! Integration tests for typecheck dump command

use insta::assert_snapshot;
use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary file with content and run typecheck command
fn run_typecheck_dump(source: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.atl");
    fs::write(&file_path, source).unwrap();

    let output = assert_cmd::cargo::cargo_bin_cmd!("atlas")
        .arg("typecheck")
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "Command failed: {:?}", output);
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_typecheck_dump_simple_variable() {
    let source = "let x: number = 42;";
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_multiple_variables() {
    let source = r#"
let x: number = 42;
let y: string = "hello";
let z: bool = true;
"#;
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_function() {
    let source = r#"
fn add(a: number, b: number) -> number {
    return a + b;
}
"#;
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_function_with_local_vars() {
    let source = r#"
fn calculate(x: number) -> number {
    let temp: number = x * 2;
    return temp;
}
"#;
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_array_type() {
    let source = "let arr: number[] = [1, 2, 3];";
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_mutable_variable() {
    let source = "var counter: number = 0;";
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}

#[test]
fn test_typecheck_dump_function_with_loop() {
    let source = r#"
fn sum(n: number) -> number {
    var total: number = 0;
    var i: number = 0;
    while (i < n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
"#;
    let json = run_typecheck_dump(source);
    assert_snapshot!(json);
}
