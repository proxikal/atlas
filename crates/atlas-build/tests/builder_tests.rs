//! Integration tests for the build system
//!
//! Tests the complete build pipeline with real Atlas projects

use atlas_build::{Builder, OptLevel};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test project with the given structure
fn create_test_project(files: &[(&str, &str)]) -> (TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();

    // Create src directory
    fs::create_dir(path.join("src")).unwrap();

    // Create manifest
    let manifest = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(path.join("atlas.toml"), manifest).unwrap();

    // Create source files
    for (file_path, content) in files {
        let full_path = path.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    let path_str = path.to_string_lossy().to_string();
    (dir, path_str)
}

/// Create a builder with target dir inside the temp project (avoids cross-test interference)
fn make_builder(path: &str) -> Builder {
    let target_dir = PathBuf::from(path).join("target/debug");
    Builder::new(path).unwrap().with_target_dir(target_dir)
}

#[test]
fn test_build_simple_single_file_project() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 42;
    print(x);
}"#,
    )]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_ok(), "Build should succeed: {:?}", result);
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 1);
    assert_eq!(context.stats.compiled_modules, 1);
}

#[test]
#[ignore = "requires cross-module symbol resolution (not yet implemented)"]
fn test_build_multi_file_project_with_imports() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { add } from "math";

fn main() -> void {
    let result: number = add(1, 2);
}"#,
        ),
        (
            "src/math.atlas",
            r#"export fn add(x: number, y: number) -> number {
    return x + y;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_ok(), "Build should succeed: {:?}", result);
    let context = result.unwrap();
    assert_eq!(context.stats.total_modules, 2);
    assert_eq!(context.stats.compiled_modules, 2);
}

#[test]
fn test_build_library_target() {
    let (_temp, project_path) = create_test_project(&[(
        "src/lib.atlas",
        r#"export fn greet(name: string) -> string {
    return "Hello, " + name;
}"#,
    )]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_ok(), "Build should succeed");
    let context = result.unwrap();
    assert_eq!(context.artifacts.len(), 1);
    assert_eq!(
        context.artifacts[0].target.kind,
        atlas_build::TargetKind::Library
    );
}

#[test]
fn test_build_binary_target() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"
fn main() -> void {
    let x: number = 42;
    print(x);
}
"#,
    )]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_ok(), "Build should succeed");
    let context = result.unwrap();
    assert_eq!(context.artifacts.len(), 1);
    assert_eq!(
        context.artifacts[0].target.kind,
        atlas_build::TargetKind::Binary
    );
}

#[test]
fn test_build_with_optimization() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: number = 1 + 1;
    print(x);
}"#,
    )]);

    let mut builder = make_builder(&project_path).with_optimization(OptLevel::O2);
    let result = builder.build();

    assert!(result.is_ok(), "Optimized build should succeed");
}

#[test]
fn test_build_error_missing_src_dir() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    // Create manifest but no src directory
    let manifest = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(path.join("atlas.toml"), manifest).unwrap();

    let mut builder = Builder::new(path).unwrap();
    let result = builder.build();

    assert!(result.is_err(), "Build should fail without src directory");
}

#[test]
fn test_build_error_no_source_files() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path();

    // Create manifest and empty src directory
    let manifest = r#"
[package]
name = "test-project"
version = "0.1.0"
"#;
    fs::write(path.join("atlas.toml"), manifest).unwrap();
    fs::create_dir(path.join("src")).unwrap();

    let mut builder = Builder::new(path).unwrap();
    let result = builder.build();

    assert!(result.is_err(), "Build should fail with no source files");
}

#[test]
fn test_build_stats_tracking() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            "fn main() -> void { let x: number = 42; print(x); }",
        ),
        (
            "src/utils.atlas",
            "export fn helper() -> number { return 1; }",
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build().unwrap();

    assert_eq!(result.stats.total_modules, 2);
    assert_eq!(result.stats.compiled_modules, 2);
    // Use as_nanos instead of as_millis to avoid flaky failures on fast machines
    assert!(result.stats.total_time.as_nanos() > 0);
    assert!(result.stats.compilation_time.as_nanos() > 0);
}

#[test]
fn test_build_output_directory_structure() {
    let (_temp, project_path) =
        create_test_project(&[("src/main.atlas", "fn main() -> number { return 42; }")]);

    let mut builder = make_builder(&project_path);
    let result = builder.build().unwrap();

    // Check that output directory was created (target dir is inside project temp dir)
    let target_dir = PathBuf::from(&project_path).join("target/debug");
    assert!(target_dir.exists(), "Target directory should exist");

    // Check that artifact exists
    assert!(!result.artifacts.is_empty());
    assert!(result.artifacts[0].output_path.exists());
}

#[test]
#[ignore = "requires cross-module symbol resolution (not yet implemented)"]
fn test_multiple_targets_library_and_binary() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/lib.atlas",
            r#"export fn double(x: number) -> number {
    return x * 2;
}"#,
        ),
        (
            "src/main.atlas",
            r#"import { double } from "lib";

fn main() -> void {
    let result: number = double(21);
}"#,
        ),
    ]);

    let mut builder = Builder::new(&project_path).unwrap();
    let result = builder.build().unwrap();

    // Should have both library and binary targets
    assert_eq!(result.artifacts.len(), 2);

    let kinds: Vec<_> = result.artifacts.iter().map(|a| a.target.kind).collect();
    assert!(kinds.contains(&atlas_build::TargetKind::Library));
    assert!(kinds.contains(&atlas_build::TargetKind::Binary));
}

#[test]
fn test_build_with_compile_error() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"fn main() -> void {
    let x: string = 42;
}"#,
    )]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_err(), "Build should fail with type error");
}

#[test]
#[ignore = "requires cross-module symbol resolution (not yet implemented)"]
fn test_build_order_respects_dependencies() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { VALUE } from "constants";

fn main() -> void {
    let x: number = VALUE;
    print(x);
}"#,
        ),
        ("src/constants.atlas", r#"export let VALUE: number = 42;"#),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();

    assert!(result.is_ok(), "Build should respect dependency order");
}
