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
fn test_build_multi_file_project_with_imports() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { add } from "math";

fn main() -> void {
    let result: number = add(1, 2);
    print(result);
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
    print(result);
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
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

// ============================================================
// Cross-module symbol resolution tests (Phase 21a)
// ============================================================

#[test]
fn test_single_import_resolves_symbol() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { greet } from "utils";

fn main() -> void {
    greet("world");
}"#,
        ),
        (
            "src/utils.atlas",
            r#"export fn greet(name: string) -> void {
    print(name);
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(result.is_ok(), "Single import should resolve: {:?}", result);
}

#[test]
fn test_multiple_imports_from_one_module() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { add, subtract } from "math";

fn main() -> void {
    let a: number = add(10, 5);
    let b: number = subtract(10, 5);
    print(a);
    print(b);
}"#,
        ),
        (
            "src/math.atlas",
            r#"export fn add(x: number, y: number) -> number {
    return x + y;
}

export fn subtract(x: number, y: number) -> number {
    return x - y;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Multiple imports from one module should work: {:?}",
        result
    );
}

#[test]
fn test_import_from_missing_module() {
    let (_temp, project_path) = create_test_project(&[(
        "src/main.atlas",
        r#"import { foo } from "nonexistent";

fn main() -> void {
    foo();
}"#,
    )]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(result.is_err(), "Import from missing module should fail");
}

#[test]
fn test_import_missing_symbol_from_valid_module() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { nonexistent } from "utils";

fn main() -> void {
    nonexistent();
}"#,
        ),
        (
            "src/utils.atlas",
            r#"export fn helper() -> void {
    print("hi");
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_err(),
        "Import of missing symbol should fail: {:?}",
        result
    );
}

#[test]
fn test_diamond_dependency() {
    // A imports B and C; B imports C
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { from_b } from "b";
import { from_c } from "c";

fn main() -> void {
    let x: number = from_b();
    let y: number = from_c();
    print(x);
    print(y);
}"#,
        ),
        (
            "src/b.atlas",
            r#"import { from_c } from "c";

export fn from_b() -> number {
    return from_c() + 1;
}"#,
        ),
        (
            "src/c.atlas",
            r#"export fn from_c() -> number {
    return 42;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Diamond dependency should resolve: {:?}",
        result
    );
}

#[test]
fn test_chain_dependency() {
    // A -> B -> C (transitive chain)
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { middle } from "b";

fn main() -> void {
    let x: number = middle();
    print(x);
}"#,
        ),
        (
            "src/b.atlas",
            r#"import { base } from "c";

export fn middle() -> number {
    return base() + 10;
}"#,
        ),
        (
            "src/c.atlas",
            r#"export fn base() -> number {
    return 1;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Chain dependency should resolve: {:?}",
        result
    );
}

#[test]
fn test_export_variable() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { MAGIC } from "constants";

fn main() -> void {
    print(MAGIC);
}"#,
        ),
        ("src/constants.atlas", r#"export let MAGIC: number = 42;"#),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Exported variable should resolve: {:?}",
        result
    );
}

#[test]
fn test_selective_import() {
    // Import only some symbols from a module with multiple exports
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { add } from "math";

fn main() -> void {
    let x: number = add(1, 2);
    print(x);
}"#,
        ),
        (
            "src/math.atlas",
            r#"export fn add(x: number, y: number) -> number {
    return x + y;
}

export fn multiply(x: number, y: number) -> number {
    return x * y;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(result.is_ok(), "Selective import should work: {:?}", result);
}

#[test]
fn test_no_imports_still_works() {
    // Modules without imports should still compile fine
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"fn main() -> void {
    print(42);
}"#,
        ),
        (
            "src/helper.atlas",
            r#"export fn unused() -> number {
    return 0;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Modules without imports should compile: {:?}",
        result
    );
}

#[test]
fn test_import_function_type_propagates() {
    // Verify the imported function's type is known to the type checker
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { get_name } from "data";

fn main() -> void {
    let name: string = get_name();
    print(name);
}"#,
        ),
        (
            "src/data.atlas",
            r#"export fn get_name() -> string {
    return "Atlas";
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Imported function type should propagate: {:?}",
        result
    );
}

#[test]
fn test_multiple_modules_import_same_dependency() {
    // Two modules both import from the same dependency
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { helper } from "shared";

fn main() -> void {
    let x: number = helper();
    print(x);
}"#,
        ),
        (
            "src/lib.atlas",
            r#"import { helper } from "shared";

export fn lib_fn() -> number {
    return helper() + 1;
}"#,
        ),
        (
            "src/shared.atlas",
            r#"export fn helper() -> number {
    return 99;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Multiple modules importing same dep should work: {:?}",
        result
    );
}

#[test]
fn test_incremental_build_with_imports() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { value } from "config";

fn main() -> void {
    print(value());
}"#,
        ),
        (
            "src/config.atlas",
            r#"export fn value() -> number {
    return 42;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);

    // First build
    let result = builder.build();
    assert!(result.is_ok(), "First build should succeed: {:?}", result);

    // Incremental build
    let result = builder.build_incremental();
    assert!(
        result.is_ok(),
        "Incremental build with imports should succeed: {:?}",
        result
    );
}

#[test]
fn test_non_exported_symbol_not_importable() {
    // A non-exported function should not be visible to importers
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { internal } from "mod";

fn main() -> void {
    internal();
}"#,
        ),
        (
            "src/mod.atlas",
            r#"fn internal() -> void {
    print("private");
}

export fn public_fn() -> void {
    internal();
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_err(),
        "Non-exported symbol should not be importable"
    );
}

#[test]
fn test_import_used_in_expression() {
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { base } from "values";

fn main() -> void {
    let result: number = base() * 2 + 1;
    print(result);
}"#,
        ),
        (
            "src/values.atlas",
            r#"export fn base() -> number {
    return 10;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "Imported symbol in expression should work: {:?}",
        result
    );
}

#[test]
fn test_three_level_dependency_chain() {
    // main -> a -> b -> c (4 modules, 3-level chain)
    let (_temp, project_path) = create_test_project(&[
        (
            "src/main.atlas",
            r#"import { level_a } from "a";

fn main() -> void {
    let x: number = level_a();
    print(x);
}"#,
        ),
        (
            "src/a.atlas",
            r#"import { level_b } from "b";

export fn level_a() -> number {
    return level_b() + 1;
}"#,
        ),
        (
            "src/b.atlas",
            r#"import { level_c } from "c";

export fn level_b() -> number {
    return level_c() + 10;
}"#,
        ),
        (
            "src/c.atlas",
            r#"export fn level_c() -> number {
    return 100;
}"#,
        ),
    ]);

    let mut builder = make_builder(&project_path);
    let result = builder.build();
    assert!(
        result.is_ok(),
        "3-level dependency chain should resolve: {:?}",
        result
    );
}
