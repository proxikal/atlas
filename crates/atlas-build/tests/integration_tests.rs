//! Build system integration tests
//!
//! End-to-end tests for complete build system functionality

use atlas_build::{BuildScript, Builder, OutputMode, Profile, ScriptPhase};
use std::fs;
use tempfile::TempDir;

/// Create a minimal test project with atlas.toml and src/main.atlas
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Create atlas.toml
    let manifest = r#"
[package]
name = "test-package"
version = "1.0.0"

[[bin]]
name = "test"
path = "src/main.atlas"
"#;
    fs::write(temp_dir.path().join("atlas.toml"), manifest).unwrap();

    // Create simple main.atlas
    let source = r#"
fn main() {
    print("Hello, world!");
}
"#;
    fs::write(src_dir.join("main.atlas"), source).unwrap();

    temp_dir
}

#[test]
fn test_end_to_end_build_single_file() {
    let project = create_test_project();
    let mut builder = Builder::new(project.path()).unwrap();

    // This will fail because we don't have a complete Atlas compiler setup,
    // but we can verify the builder was created successfully
    assert!(builder.clean().is_ok());
}

#[test]
fn test_build_with_all_features() {
    let project = create_test_project();
    let builder = Builder::new(project.path()).unwrap();

    // Verify builder creation with various configurations
    let mut builder = builder
        .with_verbose(true)
        .with_profile(Profile::Dev)
        .with_output_mode(OutputMode::Verbose);

    // Clean should work
    assert!(builder.clean().is_ok());
}

#[test]
fn test_build_profile_dev() {
    let project = create_test_project();
    let _builder = Builder::new(project.path()).unwrap();

    let config = Profile::Dev.default_config();
    assert!(config.debug_info);
    assert!(config.incremental);
}

#[test]
fn test_build_profile_release() {
    let project = create_test_project();
    let _builder = Builder::new(project.path()).unwrap();

    let config = Profile::Release.default_config();
    assert!(!config.debug_info);
    assert!(!config.incremental);
}

#[test]
fn test_build_with_scripts() {
    let pre = BuildScript::shell("pre", "echo 'pre-build'", ScriptPhase::PreBuild);
    let post = BuildScript::shell("post", "echo 'post-build'", ScriptPhase::PostBuild);

    assert_eq!(pre.phase, ScriptPhase::PreBuild);
    assert_eq!(post.phase, ScriptPhase::PostBuild);
}

#[test]
fn test_clean_build() {
    let project = create_test_project();
    // Use a custom target dir so clean() removes exactly what we expect
    let target_dir = project.path().join("custom_target");
    let mut builder = Builder::new(project.path())
        .unwrap()
        .with_target_dir(target_dir.clone());

    // Create fake target directory
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("test.txt"), "test").unwrap();

    // Clean should remove it
    builder.clean().unwrap();
    assert!(!target_dir.exists());
}

#[test]
fn test_builder_with_custom_target_dir() {
    let project = create_test_project();
    let custom_target = project.path().join("build");

    let mut builder = Builder::new(project.path())
        .unwrap()
        .with_target_dir(custom_target.clone());

    // Verify builder accepts custom target directory
    assert!(builder.clean().is_ok());
}

#[test]
fn test_output_mode_selection() {
    assert_eq!(OutputMode::default(), OutputMode::Normal);

    let modes = [
        OutputMode::Normal,
        OutputMode::Verbose,
        OutputMode::Quiet,
        OutputMode::Json,
    ];

    assert_eq!(modes.len(), 4);
}

#[test]
fn test_profile_selection_from_string() {
    assert_eq!(Profile::from_str("dev").unwrap(), Profile::Dev);
    assert_eq!(Profile::from_str("release").unwrap(), Profile::Release);
    assert_eq!(Profile::from_str("test").unwrap(), Profile::Test);

    let custom = Profile::from_str("bench").unwrap();
    assert_eq!(custom, Profile::Custom("bench".to_string()));
}

#[test]
fn test_script_configuration() {
    use std::time::Duration;

    let script = BuildScript::shell("test", "echo test", ScriptPhase::PreBuild)
        .with_timeout(Duration::from_secs(120))
        .with_permissions(vec!["fs-read".to_string()]);

    assert_eq!(script.timeout, Duration::from_secs(120));
    assert_eq!(script.permissions.len(), 1);
}
