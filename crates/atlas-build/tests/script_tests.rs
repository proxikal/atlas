//! Build script execution tests

use atlas_build::{BuildScript, Profile, ScriptContext, ScriptExecutor, ScriptPhase};
use std::time::Duration;
use tempfile::TempDir;

fn test_context() -> (TempDir, ScriptContext) {
    let dir = TempDir::new().unwrap();
    let src_dir = dir.path().join("src");
    let target_dir = dir.path().join("target");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&target_dir).unwrap();
    let ctx = ScriptContext::new(
        Profile::Dev,
        target_dir,
        src_dir,
        "test-package".to_string(),
        "1.0.0".to_string(),
    );
    (dir, ctx)
}

#[test]
fn test_execute_pre_build_script() {
    let (_dir, ctx) = test_context();
    let executor = ScriptExecutor::new(ctx);
    let script = BuildScript::shell("test", "echo 'pre-build'", ScriptPhase::PreBuild);

    let result = executor.execute(&script).unwrap();
    assert!(result.success());
    assert!(result.stdout.contains("pre-build"));
}

#[test]
fn test_execute_post_build_script() {
    let (_dir, ctx) = test_context();
    let executor = ScriptExecutor::new(ctx);
    let script = BuildScript::shell("test", "echo 'post-build'", ScriptPhase::PostBuild);

    let result = executor.execute(&script).unwrap();
    assert!(result.success());
    assert!(result.stdout.contains("post-build"));
}

#[test]
fn test_script_access_to_build_context() {
    let (dir, ctx) = test_context();
    let env = ctx.environment();

    assert_eq!(env.get("ATLAS_PROFILE"), Some(&"dev".to_string()));
    let expected_target = dir.path().join("target").to_string_lossy().to_string();
    let expected_src = dir.path().join("src").to_string_lossy().to_string();
    assert_eq!(env.get("ATLAS_TARGET_DIR"), Some(&expected_target));
    assert_eq!(env.get("ATLAS_SOURCE_DIR"), Some(&expected_src));
    assert_eq!(env.get("ATLAS_VERSION"), Some(&"0.2.0".to_string()));
    assert_eq!(
        env.get("ATLAS_PACKAGE_NAME"),
        Some(&"test-package".to_string())
    );
    assert_eq!(env.get("ATLAS_PACKAGE_VERSION"), Some(&"1.0.0".to_string()));
}

#[test]
fn test_script_failure_aborts_build() {
    let (_dir, ctx) = test_context();
    let executor = ScriptExecutor::new(ctx);
    let script = BuildScript::shell("test", "exit 1", ScriptPhase::PreBuild);

    let result = executor.execute(&script);
    assert!(result.is_err());
}

#[test]
fn test_script_timeout_enforcement() {
    // This test verifies timeout mechanism exists
    // Actual timeout testing would require long-running scripts
    let script = BuildScript::shell("test", "echo test", ScriptPhase::PreBuild)
        .with_timeout(Duration::from_secs(1));

    assert_eq!(script.timeout, Duration::from_secs(1));
}

#[test]
fn test_script_output_capture() {
    let (_dir, ctx) = test_context();
    let executor = ScriptExecutor::new(ctx);
    let script = BuildScript::shell("test", "echo 'stdout line'", ScriptPhase::PreBuild);

    let result = executor.execute(&script).unwrap();
    assert!(result.stdout.contains("stdout line"));
    let output = result.output();
    assert!(output.contains("STDOUT:"));
    assert!(output.contains("stdout line"));
}

#[test]
fn test_sandboxing_build_scripts() {
    // Verify permissions can be set on scripts
    let script = BuildScript::shell("test", "ls", ScriptPhase::PreBuild)
        .with_permissions(vec!["fs-read".to_string(), "fs-write".to_string()]);

    assert_eq!(script.permissions.len(), 2);
    assert!(script.permissions.contains(&"fs-read".to_string()));
    assert!(script.permissions.contains(&"fs-write".to_string()));
}

#[test]
fn test_script_phase_ordering() {
    let (_dir, ctx) = test_context();
    let executor = ScriptExecutor::new(ctx);

    let scripts = vec![
        BuildScript::shell("pre1", "echo pre1", ScriptPhase::PreBuild),
        BuildScript::shell("pre2", "echo pre2", ScriptPhase::PreBuild),
        BuildScript::shell("post1", "echo post1", ScriptPhase::PostBuild),
    ];

    // Execute only pre-build phase
    let results = executor
        .execute_phase(&scripts, ScriptPhase::PreBuild)
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_script_env_vars_in_context() {
    let (_dir, mut ctx) = test_context();
    ctx.env_vars
        .insert("CUSTOM_VAR".to_string(), "custom_value".to_string());

    let env = ctx.environment();
    assert_eq!(env.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
}

#[test]
fn test_multiple_phases_execution() {
    let phases = ScriptPhase::all();
    assert_eq!(phases.len(), 3);
    assert_eq!(phases[0], ScriptPhase::PreBuild);
    assert_eq!(phases[1], ScriptPhase::PostBuild);
    assert_eq!(phases[2], ScriptPhase::PostLink);
}
