//! Comprehensive configuration loading and precedence tests

use atlas_config::{ConfigLoader, ProjectConfig};
use std::env;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_config_file(dir: &Path, content: &str) -> std::path::PathBuf {
    let config_path = dir.join("atlas.toml");
    fs::write(&config_path, content).unwrap();
    config_path
}

// ============================================================================
// Config Loading Tests
// ============================================================================

#[test]
fn test_load_project_config_basic() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test-project"
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(config.package_name(), Some("test-project"));
    assert!(config.is_project());
}

#[test]
fn test_load_when_no_config_exists() {
    let temp_dir = TempDir::new().unwrap();

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert!(!config.is_project());
    assert_eq!(config.package_name(), None);
}

#[test]
fn test_load_from_subdirectory_finds_parent() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "parent-project"
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    // Create subdirectories
    let sub1 = temp_dir.path().join("sub1");
    let sub2 = sub1.join("sub2");
    fs::create_dir_all(&sub2).unwrap();

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(&sub2).unwrap();

    assert_eq!(config.package_name(), Some("parent-project"));
    assert_eq!(config.project_root(), Some(temp_dir.path()));
}

#[test]
fn test_load_with_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#""#; // Empty config
    create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    // Empty config is valid (all fields optional)
    assert!(config.is_project());
}

#[test]
fn test_load_with_partial_config() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[build]
output = "dist"
"#;
    create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert!(config.is_project());
    assert!(config.project.build.is_some());
}

#[test]
fn test_load_from_specific_file() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "specific"
version = "2.0.0"
"#;
    let config_path = create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_file(&config_path).unwrap();

    assert_eq!(config.package_name(), Some("specific"));
}

// ============================================================================
// Invalid Config Tests
// ============================================================================

#[test]
fn test_invalid_toml_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package
name = "broken
"#;
    create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let result = loader.load_from_directory(temp_dir.path());

    assert!(result.is_err());
}

#[test]
fn test_unknown_field_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
unknown_field = "value"
"#;
    create_config_file(temp_dir.path(), content);

    let mut loader = ConfigLoader::new();
    let result = loader.load_from_directory(temp_dir.path());

    assert!(result.is_err());
}

#[test]
fn test_invalid_version_format() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "not-a-version"
"#;
    create_config_file(temp_dir.path(), content);

    let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
    assert!(result.is_err());
}

#[test]
fn test_empty_package_name() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = ""
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
    assert!(result.is_err());
}

// ============================================================================
// Precedence Tests
// ============================================================================

#[test]
fn test_env_override_edition() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
edition = "2026"
"#;
    create_config_file(temp_dir.path(), content);

    env::set_var("ATLAS_EDITION", "2027");

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(config.edition(), "2027");

    env::remove_var("ATLAS_EDITION");
}

#[test]
fn test_env_override_optimize_true() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    env::set_var("ATLAS_OPTIMIZE", "true");

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(
        config.project.compiler.as_ref().unwrap().optimize,
        Some(true)
    );

    env::remove_var("ATLAS_OPTIMIZE");
}

#[test]
fn test_env_override_optimize_false() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[compiler]
optimize = true
"#;
    create_config_file(temp_dir.path(), content);

    env::set_var("ATLAS_OPTIMIZE", "false");

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(
        config.project.compiler.as_ref().unwrap().optimize,
        Some(false)
    );

    env::remove_var("ATLAS_OPTIMIZE");
}

#[test]
fn test_env_override_debug() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    env::set_var("ATLAS_DEBUG", "1");

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    assert_eq!(config.project.compiler.as_ref().unwrap().debug, Some(true));

    env::remove_var("ATLAS_DEBUG");
}

#[test]
fn test_default_edition_when_none_specified() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"
"#;
    create_config_file(temp_dir.path(), content);

    // Clear any env vars from other tests
    env::remove_var("ATLAS_EDITION");

    let mut loader = ConfigLoader::new();
    let config = loader.load_from_directory(temp_dir.path()).unwrap();

    // Should use default edition (2026)
    assert_eq!(config.edition(), "2026");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_valid_semver_versions() {
    let versions = vec!["1.0.0", "0.1.0", "2.1.3", "1.0.0-alpha", "1.0.0+build"];

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
        create_config_file(temp_dir.path(), &content);

        let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
        assert!(result.is_ok(), "Version {} should be valid", version);
    }
}

#[test]
fn test_invalid_semver_versions() {
    let versions = vec!["", "1", "1.x", "abc", "1.0.0.0"];

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
        create_config_file(temp_dir.path(), &content);

        let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
        assert!(result.is_err(), "Version {} should be invalid", version);
    }
}

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
        create_config_file(temp_dir.path(), &content);

        let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
        assert!(result.is_ok(), "Edition {} should be valid", edition);
    }
}

#[test]
fn test_invalid_editions() {
    let editions = vec!["2025", "2020", "invalid"];

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
        create_config_file(temp_dir.path(), &content);

        let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
        assert!(result.is_err(), "Edition {} should be invalid", edition);
    }
}

// ============================================================================
// Dependency Tests
// ============================================================================

#[test]
fn test_simple_version_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
http = "1.0"
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    assert!(config.dependencies.contains_key("http"));
}

#[test]
fn test_detailed_dependency_with_version() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
json = { version = "0.5" }
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    assert!(config.dependencies.contains_key("json"));
}

#[test]
fn test_path_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
local = { path = "../local-lib" }
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    assert!(config.dependencies.contains_key("local"));
}

#[test]
fn test_git_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
remote = { git = "https://github.com/example/lib" }
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    assert!(config.dependencies.contains_key("remote"));
}

#[test]
fn test_dev_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dev-dependencies]
test-utils = "1.0"
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    assert!(config.dev_dependencies.contains_key("test-utils"));
}

#[test]
fn test_empty_dependency_name_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
"" = "1.0"
"#;
    create_config_file(temp_dir.path(), &content);

    let result = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml"));
    // TOML parser should reject empty keys
    assert!(result.is_err());
}

// ============================================================================
// Build Configuration Tests
// ============================================================================

#[test]
fn test_build_config_paths() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[build]
output = "dist"
source = "src"
entry = "src/main.atl"
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    let build = config.build.as_ref().unwrap();

    assert_eq!(build.output.as_ref().unwrap().to_str().unwrap(), "dist");
    assert_eq!(build.source.as_ref().unwrap().to_str().unwrap(), "src");
    assert_eq!(
        build.entry.as_ref().unwrap().to_str().unwrap(),
        "src/main.atl"
    );
}

// ============================================================================
// Compiler Configuration Tests
// ============================================================================

#[test]
fn test_compiler_config() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[compiler]
optimize = true
debug = false
target = "bytecode"
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    let compiler = config.compiler.as_ref().unwrap();

    assert_eq!(compiler.optimize, Some(true));
    assert_eq!(compiler.debug, Some(false));
    assert_eq!(compiler.target.as_ref().unwrap(), "bytecode");
}

// ============================================================================
// Formatting Configuration Tests
// ============================================================================

#[test]
fn test_formatting_config() {
    let temp_dir = TempDir::new().unwrap();
    let content = r#"
[package]
name = "test"
version = "1.0.0"

[formatting]
indent = 2
max_line_length = 80
use_tabs = true
"#;
    create_config_file(temp_dir.path(), &content);

    let config = ProjectConfig::load_from_file(&temp_dir.path().join("atlas.toml")).unwrap();
    let formatting = config.formatting.as_ref().unwrap();

    assert_eq!(formatting.indent, Some(2));
    assert_eq!(formatting.max_line_length, Some(80));
    assert_eq!(formatting.use_tabs, Some(true));
}
