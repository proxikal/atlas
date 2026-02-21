//! Integration tests for project scaffolding and templates.
//!
//! Tests the `atlas new` command and template system.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Helper functions
// ============================================================================

fn atlas_cmd() -> Command {
    Command::from(assert_cmd::cargo::cargo_bin_cmd!("atlas"))
}

fn read_file(path: &std::path::Path) -> String {
    fs::read_to_string(path).expect("Failed to read file")
}

fn file_exists(dir: &std::path::Path, path: &str) -> bool {
    dir.join(path).exists()
}

fn dir_exists(dir: &std::path::Path, path: &str) -> bool {
    dir.join(path).is_dir()
}

// ============================================================================
// atlas new command tests
// ============================================================================

#[test]
fn test_new_binary_project_default() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my-app", "--author", "Test Author"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created binary project 'my-app'"));

    let project = temp.path().join("my-app");
    assert!(project.exists());
    assert!(file_exists(&project, "atlas.toml"));
    assert!(file_exists(&project, "src/main.atl"));
    assert!(file_exists(&project, "README.md"));
    assert!(file_exists(&project, ".gitignore"));
}

#[test]
fn test_new_library_project() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my-lib", "--lib", "--author", "Test Author"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created library project 'my-lib'"));

    let project = temp.path().join("my-lib");
    assert!(file_exists(&project, "atlas.toml"));
    assert!(file_exists(&project, "src/lib.atl"));
    assert!(file_exists(&project, "tests/lib_test.atl"));
    assert!(file_exists(&project, "examples/basic.atl"));
    assert!(file_exists(&project, "LICENSE"));
    assert!(file_exists(&project, "CONTRIBUTING.md"));
}

#[test]
fn test_new_web_project() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my-api", "--web", "--author", "Test Author"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created web project 'my-api'"));

    let project = temp.path().join("my-api");
    assert!(file_exists(&project, "atlas.toml"));
    assert!(file_exists(&project, "src/main.atl"));
    assert!(file_exists(&project, "src/server.atl"));
    assert!(file_exists(&project, "src/router.atl"));
    assert!(file_exists(&project, "src/routes/api.atl"));
    assert!(file_exists(&project, "static/css/style.css"));
    assert!(file_exists(&project, "templates/index.html"));
    assert!(file_exists(&project, "Dockerfile"));
}

#[test]
fn test_new_with_template_flag_binary() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args([
            "new",
            "test-proj",
            "--template",
            "binary",
            "--author",
            "Test",
        ])
        .assert()
        .success();

    let project = temp.path().join("test-proj");
    assert!(file_exists(&project, "src/main.atl"));
    assert!(file_exists(&project, "src/cli.atl"));
}

#[test]
fn test_new_with_template_flag_library() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args([
            "new",
            "test-lib",
            "--template",
            "library",
            "--author",
            "Test",
        ])
        .assert()
        .success();

    let project = temp.path().join("test-lib");
    assert!(file_exists(&project, "src/lib.atl"));
}

#[test]
fn test_new_with_template_flag_web() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "test-web", "--template", "web", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("test-web");
    assert!(file_exists(&project, "src/server.atl"));
}

#[test]
fn test_new_list_templates() {
    atlas_cmd()
        .args(["new", "dummy", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("binary"))
        .stdout(predicate::str::contains("library"))
        .stdout(predicate::str::contains("web"));
}

// ============================================================================
// Directory structure tests
// ============================================================================

#[test]
fn test_binary_creates_correct_directories() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "dir-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("dir-test");
    assert!(dir_exists(&project, "src"));
    assert!(dir_exists(&project, "tests"));
    assert!(dir_exists(&project, "config"));
}

#[test]
fn test_library_creates_correct_directories() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "lib-test", "--lib", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("lib-test");
    assert!(dir_exists(&project, "src"));
    assert!(dir_exists(&project, "tests"));
    assert!(dir_exists(&project, "examples"));
    assert!(dir_exists(&project, "docs"));
}

#[test]
fn test_web_creates_correct_directories() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "web-test", "--web", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("web-test");
    assert!(dir_exists(&project, "src"));
    assert!(dir_exists(&project, "src/routes"));
    assert!(dir_exists(&project, "src/middleware"));
    assert!(dir_exists(&project, "static"));
    assert!(dir_exists(&project, "static/css"));
    assert!(dir_exists(&project, "static/js"));
    assert!(dir_exists(&project, "templates"));
    assert!(dir_exists(&project, "config"));
}

// ============================================================================
// Variable substitution tests
// ============================================================================

#[test]
fn test_variable_substitution_name() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my-awesome-project", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("my-awesome-project");
    let manifest = read_file(&project.join("atlas.toml"));
    assert!(manifest.contains("name = \"my-awesome-project\""));
}

#[test]
fn test_variable_substitution_author() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "author-test", "--author", "Jane Doe"])
        .assert()
        .success();

    let project = temp.path().join("author-test");
    let manifest = read_file(&project.join("atlas.toml"));
    assert!(manifest.contains("Jane Doe"));
}

#[test]
fn test_variable_substitution_description() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args([
            "new",
            "desc-test",
            "--author",
            "Test",
            "--description",
            "A super cool project",
        ])
        .assert()
        .success();

    let project = temp.path().join("desc-test");
    let manifest = read_file(&project.join("atlas.toml"));
    assert!(manifest.contains("A super cool project"));
}

#[test]
fn test_variable_substitution_readme() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args([
            "new",
            "readme-test",
            "--author",
            "Test Author",
            "--description",
            "My description",
        ])
        .assert()
        .success();

    let project = temp.path().join("readme-test");
    let readme = read_file(&project.join("README.md"));
    assert!(readme.contains("readme-test"));
    assert!(readme.contains("My description"));
}

#[test]
fn test_variable_substitution_license_year() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "license-test", "--lib", "--author", "Test Author"])
        .assert()
        .success();

    let project = temp.path().join("license-test");
    let license = read_file(&project.join("LICENSE"));
    // Should contain current year
    let current_year = chrono::Local::now().format("%Y").to_string();
    assert!(license.contains(&current_year));
    assert!(license.contains("Test Author"));
}

// ============================================================================
// Git initialization tests
// ============================================================================

#[test]
fn test_git_initialized_by_default() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "git-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("git-test");
    assert!(dir_exists(&project, ".git"));
}

#[test]
fn test_no_git_flag() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "no-git-test", "--no-git", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("no-git-test");
    assert!(!dir_exists(&project, ".git"));
}

#[test]
fn test_gitignore_created() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "gitignore-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("gitignore-test");
    let gitignore = read_file(&project.join(".gitignore"));
    assert!(gitignore.contains("/target/"));
    assert!(gitignore.contains(".DS_Store"));
}

// ============================================================================
// Error handling tests
// ============================================================================

#[test]
fn test_invalid_name_rejected_empty() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", ""])
        .assert()
        .failure();
}

#[test]
fn test_invalid_name_rejected_hyphen_start() {
    let temp = TempDir::new().unwrap();

    // Use "--" to pass value starting with hyphen as positional argument
    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "--author", "Test", "--", "-invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must start with a letter"));
}

#[test]
fn test_invalid_name_rejected_number_start() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "123project", "--author", "Test"])
        .assert()
        .failure();
}

#[test]
fn test_reserved_name_rejected() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "atlas", "--author", "Test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("reserved name"));
}

#[test]
fn test_reserved_name_std_rejected() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "std", "--author", "Test"])
        .assert()
        .failure();
}

#[test]
fn test_reserved_name_core_rejected() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "core", "--author", "Test"])
        .assert()
        .failure();
}

#[test]
fn test_existing_directory_error() {
    let temp = TempDir::new().unwrap();

    // Create directory with content
    let existing = temp.path().join("existing");
    fs::create_dir_all(&existing).unwrap();
    fs::write(existing.join("file.txt"), "content").unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "existing", "--author", "Test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_force_overwrites_existing() {
    let temp = TempDir::new().unwrap();

    // Create directory with content
    let existing = temp.path().join("existing");
    fs::create_dir_all(&existing).unwrap();
    fs::write(existing.join("old-file.txt"), "old content").unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "existing", "--force", "--author", "Test"])
        .assert()
        .success();

    // Old file gone, new files present
    assert!(!file_exists(&existing, "old-file.txt"));
    assert!(file_exists(&existing, "atlas.toml"));
}

#[test]
fn test_invalid_template_type() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args([
            "new",
            "test-proj",
            "--template",
            "invalid",
            "--author",
            "Test",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown template type"));
}

// ============================================================================
// Template content tests
// ============================================================================

#[test]
fn test_binary_main_atl_content() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "content-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("content-test");
    let main = read_file(&project.join("src/main.atl"));
    assert!(main.contains("fn main()"));
}

#[test]
fn test_library_lib_atl_content() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "lib-content", "--lib", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("lib-content");
    let lib = read_file(&project.join("src/lib.atl"));
    assert!(lib.contains("export"));
    assert!(lib.contains("fn greet"));
}

#[test]
fn test_web_server_atl_content() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "web-content", "--web", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("web-content");
    let server = read_file(&project.join("src/server.atl"));
    assert!(server.contains("create_server"));
    assert!(server.contains("handle_request"));
}

#[test]
fn test_binary_cli_atl_exists() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "cli-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("cli-test");
    let cli = read_file(&project.join("src/cli.atl"));
    assert!(cli.contains("parse_args"));
    assert!(cli.contains("print_help"));
}

#[test]
fn test_web_dockerfile_content() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "docker-test", "--web", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("docker-test");
    let dockerfile = read_file(&project.join("Dockerfile"));
    assert!(dockerfile.contains("EXPOSE 8080"));
    assert!(dockerfile.contains("docker-test"));
}

// ============================================================================
// Alias tests
// ============================================================================

#[test]
fn test_new_alias_n() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["n", "alias-test", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("alias-test");
    assert!(project.exists());
}

// ============================================================================
// Verbose output tests
// ============================================================================

#[test]
fn test_verbose_output() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "verbose-test", "--verbose", "--author", "Test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Creating"))
        .stdout(predicate::str::contains("Directory"));
}

// ============================================================================
// Template completeness tests
// ============================================================================

#[test]
fn test_library_has_all_required_files() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "complete-lib", "--lib", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("complete-lib");

    // All required files
    let required_files = [
        "atlas.toml",
        "src/lib.atl",
        "src/utils.atl",
        "tests/lib_test.atl",
        "examples/basic.atl",
        "docs/api.md",
        "README.md",
        "LICENSE",
        "CONTRIBUTING.md",
        ".gitignore",
    ];

    for file in &required_files {
        assert!(
            file_exists(&project, file),
            "Missing required file: {}",
            file
        );
    }
}

#[test]
fn test_web_has_all_required_files() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "complete-web", "--web", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("complete-web");

    let required_files = [
        "atlas.toml",
        "src/main.atl",
        "src/server.atl",
        "src/router.atl",
        "src/routes/mod.atl",
        "src/routes/api.atl",
        "src/routes/pages.atl",
        "src/middleware/mod.atl",
        "src/middleware/logger.atl",
        "static/css/style.css",
        "static/js/app.js",
        "templates/index.html",
        "templates/error.html",
        "config/default.toml",
        ".env.example",
        "tests/server_test.atl",
        "README.md",
        "LICENSE",
        ".gitignore",
        "Dockerfile",
    ];

    for file in &required_files {
        assert!(
            file_exists(&project, file),
            "Missing required file: {}",
            file
        );
    }
}

#[test]
fn test_binary_has_all_required_files() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "complete-bin", "--author", "Test"])
        .assert()
        .success();

    let project = temp.path().join("complete-bin");

    let required_files = [
        "atlas.toml",
        "src/main.atl",
        "src/cli.atl",
        "src/config.atl",
        "config/default.toml",
        "tests/main_test.atl",
        "README.md",
        "LICENSE",
        ".gitignore",
    ];

    for file in &required_files {
        assert!(
            file_exists(&project, file),
            "Missing required file: {}",
            file
        );
    }
}

// ============================================================================
// Valid name tests
// ============================================================================

#[test]
fn test_valid_name_with_hyphen() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my-project", "--author", "Test"])
        .assert()
        .success();

    assert!(temp.path().join("my-project").exists());
}

#[test]
fn test_valid_name_with_underscore() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "my_project", "--author", "Test"])
        .assert()
        .success();

    assert!(temp.path().join("my_project").exists());
}

#[test]
fn test_valid_name_with_numbers() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "project123", "--author", "Test"])
        .assert()
        .success();

    assert!(temp.path().join("project123").exists());
}

#[test]
fn test_valid_name_single_char() {
    let temp = TempDir::new().unwrap();

    atlas_cmd()
        .current_dir(temp.path())
        .args(["new", "a", "--author", "Test"])
        .assert()
        .success();

    assert!(temp.path().join("a").exists());
}
