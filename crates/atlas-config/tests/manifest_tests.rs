//! Package manifest tests

use atlas_config::Manifest;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_manifest_file(dir: &Path, content: &str) -> std::path::PathBuf {
    let manifest_path = dir.join("atlas.toml");
    fs::write(&manifest_path, content).unwrap();
    manifest_path
}

// ============================================================================
// Basic Manifest Tests
// ============================================================================

#[test]
fn test_minimal_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "my-package"
version = "1.0.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.name(), "my-package");
    assert_eq!(manifest.version(), "1.0.0");
    assert!(manifest.dependencies.is_empty());
    assert!(manifest.dev_dependencies.is_empty());
}

#[test]
fn test_manifest_with_all_fields() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "full-package"
version = "2.1.0"
edition = "2026"
description = "A complete package"
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
license = "MIT"
repository = "https://github.com/example/full-package"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.name(), "full-package");
    assert_eq!(manifest.version(), "2.1.0");
    assert_eq!(manifest.edition(), Some("2026"));
    assert_eq!(
        manifest.package.description.as_deref(),
        Some("A complete package")
    );
    assert_eq!(manifest.package.authors.len(), 2);
    assert_eq!(manifest.package.license.as_deref(), Some("MIT"));
    assert_eq!(
        manifest.package.repository.as_deref(),
        Some("https://github.com/example/full-package")
    );
}

// ============================================================================
// Dependency Parsing Tests
// ============================================================================

#[test]
fn test_simple_version_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
http = "1.0"
json = "0.5"
utils = "2.3.4"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.dependencies.len(), 3);
    assert!(manifest.dependencies.contains_key("http"));
    assert!(manifest.dependencies.contains_key("json"));
    assert!(manifest.dependencies.contains_key("utils"));
}

#[test]
fn test_detailed_version_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
lib = { version = "1.2.3" }
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert!(manifest.dependencies.contains_key("lib"));
}

#[test]
fn test_path_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
local-lib = { path = "../local-lib" }
another = { path = "/absolute/path" }
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.dependencies.len(), 2);
    assert!(manifest.dependencies.contains_key("local-lib"));
    assert!(manifest.dependencies.contains_key("another"));
}

#[test]
fn test_git_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
remote = { git = "https://github.com/example/lib" }
gitlab = { git = "https://gitlab.com/example/lib" }
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.dependencies.len(), 2);
    assert!(manifest.dependencies.contains_key("remote"));
    assert!(manifest.dependencies.contains_key("gitlab"));
}

#[test]
fn test_registry_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
custom = { version = "1.0", registry = "my-registry" }
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert!(manifest.dependencies.contains_key("custom"));
}

#[test]
fn test_dev_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dev-dependencies]
test-framework = "1.0"
mock-lib = { version = "0.5" }
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.dev_dependencies.len(), 2);
    assert!(manifest.dev_dependencies.contains_key("test-framework"));
    assert!(manifest.dev_dependencies.contains_key("mock-lib"));
}

#[test]
fn test_mixed_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
http = "1.0"
local = { path = "../local" }
remote = { git = "https://github.com/example/lib" }

[dev-dependencies]
test-utils = "1.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();

    assert_eq!(manifest.dependencies.len(), 3);
    assert_eq!(manifest.dev_dependencies.len(), 1);
    assert_eq!(manifest.all_dependencies().len(), 4);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_empty_name_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = ""
version = "1.0.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let result = Manifest::load_from_file(&manifest_path);

    assert!(result.is_err());
}

#[test]
fn test_invalid_version_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "not-a-version"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let result = Manifest::load_from_file(&manifest_path);

    assert!(result.is_err());
}

#[test]
fn test_missing_package_section() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[dependencies]
http = "1.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let result = Manifest::load_from_file(&manifest_path);

    // Missing package section should fail TOML parsing
    assert!(result.is_err());
}

#[test]
fn test_missing_name() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
version = "1.0.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let result = Manifest::load_from_file(&manifest_path);

    // Missing required field
    assert!(result.is_err());
}

#[test]
fn test_missing_version() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let result = Manifest::load_from_file(&manifest_path);

    // Missing required field
    assert!(result.is_err());
}

// ============================================================================
// Version Format Tests
// ============================================================================

#[test]
fn test_valid_version_formats() {
    let versions = vec![
        "1.0.0",
        "0.1.0",
        "2.1.3",
        "10.20.30",
        "1.0.0-alpha",
        "1.0.0-beta.1",
        "1.0.0+build",
        "1.0.0-rc.1+build.123",
    ];

    for version in versions {
        let temp_dir = TempDir::new().unwrap();
        let content = format!(
            r#"
[package]
name = "test"
version = "{}"
"#,
            version
        );
        let manifest_path = create_manifest_file(temp_dir.path(), &content);

        let result = Manifest::load_from_file(&manifest_path);
        assert!(result.is_ok(), "Version {} should be valid", version);
    }
}

#[test]
fn test_invalid_version_formats() {
    let versions = vec!["", "1", "x.y.z", "1.0.0.0"];

    for version in versions {
        let temp_dir = TempDir::new().unwrap();
        let content = format!(
            r#"
[package]
name = "test"
version = "{}"
"#,
            version
        );
        let manifest_path = create_manifest_file(temp_dir.path(), &content);

        let result = Manifest::load_from_file(&manifest_path);
        assert!(result.is_err(), "Version {} should be invalid", version);
    }
}

// ============================================================================
// Edition Tests
// ============================================================================

#[test]
fn test_valid_editions() {
    let editions = vec!["2026", "2027", "2028"];

    for edition in editions {
        let temp_dir = TempDir::new().unwrap();
        let content = format!(
            r#"
[package]
name = "test"
version = "1.0.0"
edition = "{}"
"#,
            edition
        );
        let manifest_path = create_manifest_file(temp_dir.path(), &content);

        let result = Manifest::load_from_file(&manifest_path);
        assert!(result.is_ok(), "Edition {} should be valid", edition);
    }
}

#[test]
fn test_optional_edition() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    assert_eq!(manifest.edition(), None);
}

// ============================================================================
// Metadata Tests
// ============================================================================

#[test]
fn test_optional_description() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
description = "A test package"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    assert_eq!(
        manifest.package.description.as_deref(),
        Some("A test package")
    );
}

#[test]
fn test_multiple_authors() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
authors = ["Alice", "Bob", "Charlie"]
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    assert_eq!(manifest.package.authors.len(), 3);
}

#[test]
fn test_license_field() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
license = "Apache-2.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    assert_eq!(manifest.package.license.as_deref(), Some("Apache-2.0"));
}

#[test]
fn test_repository_field() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
repository = "https://github.com/example/repo"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    assert_eq!(
        manifest.package.repository.as_deref(),
        Some("https://github.com/example/repo")
    );
}

// ============================================================================
// All Dependencies Tests
// ============================================================================

#[test]
fn test_all_dependencies_combined() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
http = "1.0"
json = "0.5"

[dev-dependencies]
test-utils = "1.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    let all = manifest.all_dependencies();

    assert_eq!(all.len(), 3);
    assert!(all.contains_key("http"));
    assert!(all.contains_key("json"));
    assert!(all.contains_key("test-utils"));
}

#[test]
fn test_all_dependencies_empty() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
"#;
    let manifest_path = create_manifest_file(temp_dir.path(), content);

    let manifest = Manifest::load_from_file(&manifest_path).unwrap();
    let all = manifest.all_dependencies();

    assert!(all.is_empty());
}
