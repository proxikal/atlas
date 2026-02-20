//! Package Manager CLI Integration Tests
//!
//! Tests for atlas init, add, remove, install, update, and publish commands.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn atlas() -> Command {
    Command::cargo_bin("atlas").unwrap()
}

fn create_test_project(dir: &std::path::Path) {
    let manifest = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
foo = "1.0"
bar = "2.0"

[dev-dependencies]
test-utils = "0.1"
"#;
    fs::write(dir.join("atlas.toml"), manifest).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.atl"), "fn main() { print(\"hello\") }").unwrap();
}

fn create_empty_project(dir: &std::path::Path) {
    let manifest = r#"[package]
name = "empty-project"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#;
    fs::write(dir.join("atlas.toml"), manifest).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.atl"), "fn main() { }").unwrap();
}

// ============================================================================
// Init Command Tests
// ============================================================================

#[test]
fn test_init_creates_project() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-project");
    fs::create_dir(&project_dir).unwrap();

    atlas()
        .args(["init", "my-project", "--no-git"])
        .current_dir(&project_dir)
        .assert()
        .success();

    assert!(project_dir.join("atlas.toml").exists());
    assert!(project_dir.join("src/main.atl").exists());
    assert!(project_dir.join(".gitignore").exists());
}

#[test]
fn test_init_creates_library() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my-lib");
    fs::create_dir(&project_dir).unwrap();

    atlas()
        .args(["init", "my-lib", "--lib", "--no-git"])
        .current_dir(&project_dir)
        .assert()
        .success();

    assert!(project_dir.join("atlas.toml").exists());
    assert!(project_dir.join("src/lib.atl").exists());
}

#[test]
fn test_init_fails_if_manifest_exists() {
    let temp = TempDir::new().unwrap();

    // Create existing manifest
    fs::write(
        temp.path().join("atlas.toml"),
        "[package]\nname = \"existing\"\nversion = \"1.0.0\"",
    )
    .unwrap();

    atlas()
        .args(["init", "new-project"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_init_alias() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("alias-test");
    fs::create_dir(&project_dir).unwrap();

    atlas()
        .args(["i", "alias-test", "--no-git"])
        .current_dir(&project_dir)
        .assert()
        .success();
}

#[test]
fn test_init_verbose() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("verbose-test");
    fs::create_dir(&project_dir).unwrap();

    atlas()
        .args(["init", "verbose-test", "--verbose", "--no-git"])
        .current_dir(&project_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating"));
}

// ============================================================================
// Add Command Tests
// ============================================================================

#[test]
fn test_add_dependency() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["add", "new-dep", "--ver", "1.2.3"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(content.contains("new-dep"));
}

#[test]
fn test_add_dev_dependency() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["add", "test-dep", "--ver", "0.1.0", "--dev"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(content.contains("test-dep"));
}

#[test]
fn test_add_with_at_version_syntax() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["add", "versioned@2.0.0"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(content.contains("versioned"));
}

#[test]
fn test_add_dry_run() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());
    let original = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();

    atlas()
        .args(["add", "dry-run-dep", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert_eq!(content, original);
}

#[test]
fn test_add_no_manifest_fails() {
    let temp = TempDir::new().unwrap();

    atlas()
        .args(["add", "some-dep"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("atlas.toml"));
}

#[test]
fn test_add_with_features() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["add", "featured-dep", "-F", "async", "-F", "json"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(content.contains("featured-dep"));
}

#[test]
fn test_add_optional_dependency() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["add", "optional-dep", "--optional"])
        .current_dir(temp.path())
        .assert()
        .success();
}

// ============================================================================
// Remove Command Tests
// ============================================================================

#[test]
fn test_remove_dependency() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["remove", "foo"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(!content.contains("foo = "));
    assert!(content.contains("bar"));
}

#[test]
fn test_remove_multiple_dependencies() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["remove", "foo", "bar"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(!content.contains("foo = "));
    assert!(!content.contains("bar = "));
}

#[test]
fn test_remove_dev_dependency() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["remove", "test-utils", "--dev"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(!content.contains("test-utils"));
}

#[test]
fn test_remove_nonexistent_fails() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["remove", "nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn test_remove_alias() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["rm", "foo"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_remove_dry_run() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());
    let original = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();

    atlas()
        .args(["remove", "foo", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert_eq!(content, original);
}

// ============================================================================
// Install Command Tests
// ============================================================================

#[test]
fn test_install_creates_lockfile() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["install", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("atlas.lock").exists());
}

#[test]
fn test_install_creates_modules_dir() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["install", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("atlas_modules").exists());
}

#[test]
fn test_install_empty_project() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["install"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No dependencies"));
}

#[test]
fn test_install_dry_run() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["install", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(!temp.path().join("atlas.lock").exists());
    assert!(!temp.path().join("atlas_modules").exists());
}

#[test]
fn test_install_production_flag() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["install", "--production", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_install_force_reinstall() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    // First install
    atlas()
        .args(["install", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Force reinstall
    atlas()
        .args(["install", "--force", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_install_no_manifest_fails() {
    let temp = TempDir::new().unwrap();

    atlas()
        .args(["install"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("atlas.toml"));
}

#[test]
fn test_install_verbose() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["install", "--verbose"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Reading manifest"));
}

// ============================================================================
// Update Command Tests
// ============================================================================

#[test]
fn test_update_all_packages() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["update"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("atlas.lock").exists());
}

#[test]
fn test_update_specific_package() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["update", "foo"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_update_nonexistent_package_fails() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["update", "nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_update_dry_run() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["update", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(!temp.path().join("atlas.lock").exists());
}

#[test]
fn test_update_alias() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["up"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_update_empty_project() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["update"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No dependencies"));
}

// ============================================================================
// Publish Command Tests
// ============================================================================

#[test]
fn test_publish_dry_run() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["publish", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));
}

#[test]
fn test_publish_no_verify() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    atlas()
        .args(["publish", "--no-verify", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn test_publish_no_manifest_fails() {
    let temp = TempDir::new().unwrap();

    atlas()
        .args(["publish"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("atlas.toml"));
}

#[test]
fn test_publish_validates_manifest() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["publish", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Manifest validation"));
}

#[test]
fn test_publish_verbose() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["publish", "--dry-run", "--verbose"])
        .current_dir(temp.path())
        .assert()
        .success();
}

// ============================================================================
// Combined Workflow Tests
// ============================================================================

#[test]
fn test_init_add_install_workflow() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("workflow-test");
    fs::create_dir(&project_dir).unwrap();

    // Init project
    atlas()
        .args(["init", "workflow-test", "--no-git"])
        .current_dir(&project_dir)
        .assert()
        .success();

    // Add dependency
    atlas()
        .args(["add", "http", "--ver", "1.0"])
        .current_dir(&project_dir)
        .assert()
        .success();

    // Install
    atlas()
        .args(["install", "--quiet"])
        .current_dir(&project_dir)
        .assert()
        .success();

    assert!(project_dir.join("atlas.lock").exists());
}

#[test]
fn test_add_remove_workflow() {
    let temp = TempDir::new().unwrap();
    create_empty_project(temp.path());

    // Add
    atlas()
        .args(["add", "temp-dep"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(content.contains("temp-dep"));

    // Remove
    atlas()
        .args(["remove", "temp-dep"])
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
    assert!(!content.contains("temp-dep"));
}

#[test]
fn test_install_update_workflow() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    // Install
    atlas()
        .args(["install", "--quiet"])
        .current_dir(temp.path())
        .assert()
        .success();

    assert!(temp.path().join("atlas.lock").exists());

    // Update
    atlas()
        .args(["update"])
        .current_dir(temp.path())
        .assert()
        .success();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_commands_require_project_context() {
    let temp = TempDir::new().unwrap();

    // These commands should fail without atlas.toml
    for cmd in &["add new-dep", "remove dep", "install", "update", "publish"] {
        let args: Vec<&str> = cmd.split_whitespace().collect();
        atlas()
            .args(&args)
            .current_dir(temp.path())
            .assert()
            .failure();
    }
}

#[test]
fn test_remove_requires_packages() {
    let temp = TempDir::new().unwrap();
    create_test_project(temp.path());

    atlas()
        .args(["remove"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

// ============================================================================
// Help Text Tests
// ============================================================================

#[test]
fn test_init_help() {
    atlas()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn test_add_help() {
    atlas()
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dependency"));
}

#[test]
fn test_remove_help() {
    atlas()
        .args(["remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove"));
}

#[test]
fn test_install_help() {
    atlas()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dependencies"));
}

#[test]
fn test_update_help() {
    atlas()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Update"));
}

#[test]
fn test_publish_help() {
    atlas()
        .args(["publish", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("registry"));
}
