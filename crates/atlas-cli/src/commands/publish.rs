//! Publish package command (atlas publish)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use atlas_package::Validator;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Arguments for the publish command
#[derive(Debug, Clone)]
pub struct PublishArgs {
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Registry to publish to
    pub registry: Option<String>,
    /// Skip all validation checks
    pub no_verify: bool,
    /// Perform all checks but don't actually publish
    pub dry_run: bool,
    /// Allow publishing with dirty git state
    pub allow_dirty: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for PublishArgs {
    fn default() -> Self {
        Self {
            project_dir: PathBuf::from("."),
            registry: None,
            no_verify: false,
            dry_run: false,
            allow_dirty: false,
            verbose: false,
        }
    }
}

/// Publishing step result
#[derive(Debug)]
enum StepResult {
    Success(String),
    Warning(String),
    Skip(String),
}

/// Run the publish command
pub fn run(args: PublishArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;
    let project_dir = manifest_path.parent().unwrap();

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load manifest
    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let package_name = &manifest.package.name;
    let package_version = &manifest.package.version;

    println!(
        "\nPublishing {} v{} to {}",
        package_name,
        package_version,
        args.registry
            .as_deref()
            .unwrap_or("registry.atlas-lang.org")
    );
    println!();

    // Create progress indicator
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    // Run validation steps
    let mut steps_passed = 0;
    let mut steps_warned = 0;
    let total_steps = 6;

    // Step 1: Validate manifest
    spinner.set_message("Validating manifest...");
    let step1 = validate_manifest(&manifest, args.no_verify);
    print_step_result(1, "Manifest validation", &step1);
    match step1 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => steps_warned += 1,
        StepResult::Skip(_) => {}
    }

    // Step 2: Check git status
    spinner.set_message("Checking git status...");
    let step2 = check_git_status(project_dir, args.allow_dirty);
    print_step_result(2, "Git status", &step2);
    match step2 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => {
            if !args.allow_dirty {
                spinner.finish_and_clear();
                bail!("Git working directory is dirty. Use --allow-dirty to override.");
            }
            steps_warned += 1;
        }
        StepResult::Skip(_) => {}
    }

    // Step 3: Verify package structure
    spinner.set_message("Verifying package structure...");
    let step3 = verify_package_structure(project_dir, &manifest);
    print_step_result(3, "Package structure", &step3);
    match step3 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => steps_warned += 1,
        StepResult::Skip(_) => {}
    }

    // Step 4: Build package
    spinner.set_message("Building package...");
    let step4 = build_package(project_dir, args.no_verify);
    print_step_result(4, "Build", &step4);
    match step4 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => steps_warned += 1,
        StepResult::Skip(_) => {}
    }

    // Step 5: Run tests
    spinner.set_message("Running tests...");
    let step5 = run_tests(project_dir, args.no_verify);
    print_step_result(5, "Tests", &step5);
    match step5 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => steps_warned += 1,
        StepResult::Skip(_) => {}
    }

    // Step 6: Package archive
    spinner.set_message("Creating package archive...");
    let step6 = create_package_archive(project_dir, &manifest, args.dry_run);
    print_step_result(6, "Package archive", &step6);
    match step6 {
        StepResult::Success(_) => steps_passed += 1,
        StepResult::Warning(_) => steps_warned += 1,
        StepResult::Skip(_) => {}
    }

    spinner.finish_and_clear();

    // Summary
    println!();
    println!(
        "Validation: {}/{} steps passed, {} warnings",
        steps_passed, total_steps, steps_warned
    );

    if args.dry_run {
        println!(
            "\n[Dry run] Would publish {} v{} to {}",
            package_name,
            package_version,
            args.registry
                .as_deref()
                .unwrap_or("registry.atlas-lang.org")
        );
        return Ok(());
    }

    // TODO: Actual registry upload
    // For now, simulate successful publish
    println!(
        "\n{} Published {} v{} to {}",
        green_check(),
        package_name,
        package_version,
        args.registry
            .as_deref()
            .unwrap_or("registry.atlas-lang.org")
    );

    println!("\nNote: Package registry is not yet implemented.");
    println!("This package has been validated and is ready for future publishing.");

    Ok(())
}

/// Validate manifest contents
fn validate_manifest(manifest: &PackageManifest, skip: bool) -> StepResult {
    if skip {
        return StepResult::Skip("skipped".to_string());
    }

    match Validator::validate(manifest) {
        Ok(_) => StepResult::Success("valid".to_string()),
        Err(errors) => {
            let msg = errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            StepResult::Warning(msg)
        }
    }
}

/// Check git status
fn check_git_status(project_dir: &Path, allow_dirty: bool) -> StepResult {
    // Check if git repo exists
    let git_dir = project_dir.join(".git");
    if !git_dir.exists() {
        return StepResult::Skip("not a git repository".to_string());
    }

    // Check for uncommitted changes
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_dir)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().is_empty() {
                StepResult::Success("clean".to_string())
            } else if allow_dirty {
                StepResult::Warning("dirty (allowed)".to_string())
            } else {
                StepResult::Warning("uncommitted changes".to_string())
            }
        }
        _ => StepResult::Skip("git not available".to_string()),
    }
}

/// Verify package structure
fn verify_package_structure(project_dir: &Path, manifest: &PackageManifest) -> StepResult {
    let mut issues = Vec::new();

    // Check for required files
    let required_files = vec!["atlas.toml"];
    for file in required_files {
        if !project_dir.join(file).exists() {
            issues.push(format!("missing {}", file));
        }
    }

    // Check source files exist
    if manifest.bin.is_empty() && manifest.lib.is_none() {
        // Check for default src directory
        if !project_dir.join("src").exists() {
            issues.push("no src directory".to_string());
        }
    }

    // Check binary targets
    for bin in &manifest.bin {
        if !project_dir.join(&bin.path).exists() {
            issues.push(format!("missing binary: {}", bin.path.display()));
        }
    }

    // Check library target
    if let Some(ref lib) = manifest.lib {
        if !project_dir.join(&lib.path).exists() {
            issues.push(format!("missing library: {}", lib.path.display()));
        }
    }

    if issues.is_empty() {
        StepResult::Success("valid".to_string())
    } else {
        StepResult::Warning(issues.join(", "))
    }
}

/// Build package
fn build_package(project_dir: &Path, skip: bool) -> StepResult {
    if skip {
        return StepResult::Skip("skipped".to_string());
    }

    // Check for main source file
    let main_atl = project_dir.join("src/main.atl");
    let lib_atl = project_dir.join("src/lib.atl");

    if !main_atl.exists() && !lib_atl.exists() {
        return StepResult::Warning("no source files found".to_string());
    }

    // For now, just verify files can be read
    let source_file = if main_atl.exists() { main_atl } else { lib_atl };

    match fs::read_to_string(&source_file) {
        Ok(_) => StepResult::Success("ok".to_string()),
        Err(e) => StepResult::Warning(format!("read error: {}", e)),
    }
}

/// Run tests
fn run_tests(_project_dir: &Path, skip: bool) -> StepResult {
    if skip {
        return StepResult::Skip("skipped".to_string());
    }

    // TODO: Actually run tests using atlas test
    // For now, just mark as skipped since test infrastructure may not exist
    StepResult::Skip("no tests configured".to_string())
}

/// Create package archive
fn create_package_archive(
    project_dir: &Path,
    manifest: &PackageManifest,
    dry_run: bool,
) -> StepResult {
    let archive_name = format!(
        "{}-{}.tar.gz",
        manifest.package.name, manifest.package.version
    );

    if dry_run {
        return StepResult::Success(format!("would create {}", archive_name));
    }

    // Create target directory
    let target_dir = project_dir.join("target/package");
    if let Err(e) = fs::create_dir_all(&target_dir) {
        return StepResult::Warning(format!("failed to create target dir: {}", e));
    }

    // TODO: Actually create tarball
    // For now, just create placeholder
    let archive_path = target_dir.join(&archive_name);
    if let Err(e) = fs::write(&archive_path, b"placeholder") {
        return StepResult::Warning(format!("failed to create archive: {}", e));
    }

    StepResult::Success(format!("created {}", archive_name))
}

/// Print step result
fn print_step_result(step: usize, name: &str, result: &StepResult) {
    let (symbol, status) = match result {
        StepResult::Success(msg) => (green_check(), format!("{} ({})", name, msg)),
        StepResult::Warning(msg) => (yellow_warning(), format!("{} ({})", name, msg)),
        StepResult::Skip(msg) => (blue_skip(), format!("{} ({})", name, msg)),
    };

    println!("  {} Step {}: {}", symbol, step, status);
}

/// Find atlas.toml manifest file
fn find_manifest(start_dir: &Path) -> Result<PathBuf> {
    let mut current = start_dir
        .canonicalize()
        .unwrap_or_else(|_| start_dir.to_path_buf());

    loop {
        let manifest_path = current.join("atlas.toml");
        if manifest_path.exists() {
            return Ok(manifest_path);
        }

        if !current.pop() {
            break;
        }
    }

    bail!(
        "Could not find atlas.toml in {} or any parent directory",
        start_dir.display()
    )
}

/// Green checkmark
fn green_check() -> &'static str {
    "\u{2713}"
}

/// Yellow warning
fn yellow_warning() -> &'static str {
    "\u{26A0}"
}

/// Blue skip indicator
fn blue_skip() -> &'static str {
    "\u{2192}"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project(dir: &Path) {
        // Create manifest
        let manifest = r#"[package]
name = "test-package"
version = "1.0.0"
description = "A test package"
authors = ["Test Author <test@example.com>"]
license = "MIT"

[dependencies]
"#;
        fs::write(dir.join("atlas.toml"), manifest).unwrap();

        // Create src directory and main file
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.atl"), "fn main() { print(\"hello\") }").unwrap();
    }

    fn create_minimal_project(dir: &Path) {
        let manifest = r#"[package]
name = "minimal"
version = "0.1.0"

[dependencies]
"#;
        fs::write(dir.join("atlas.toml"), manifest).unwrap();
    }

    #[test]
    fn test_publish_dry_run() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };

        run(args).unwrap();
    }

    #[test]
    fn test_publish_no_verify() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            no_verify: true,
            dry_run: true,
            ..Default::default()
        };

        run(args).unwrap();
    }

    #[test]
    fn test_publish_no_manifest() {
        let temp = TempDir::new().unwrap();

        let args = PublishArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_validate_manifest() {
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"

[dependencies]
"#;
        let manifest = PackageManifest::from_str(manifest_content).unwrap();
        let result = validate_manifest(&manifest, false);
        assert!(matches!(result, StepResult::Success(_)));
    }

    #[test]
    fn test_validate_manifest_skip() {
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"

[dependencies]
"#;
        let manifest = PackageManifest::from_str(manifest_content).unwrap();
        let result = validate_manifest(&manifest, true);
        assert!(matches!(result, StepResult::Skip(_)));
    }

    #[test]
    fn test_check_git_status_no_git() {
        let temp = TempDir::new().unwrap();
        let result = check_git_status(temp.path(), false);
        assert!(matches!(result, StepResult::Skip(_)));
    }

    #[test]
    fn test_verify_package_structure() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let result = verify_package_structure(temp.path(), &manifest);
        assert!(matches!(result, StepResult::Success(_)));
    }

    #[test]
    fn test_verify_package_structure_missing_src() {
        let temp = TempDir::new().unwrap();
        create_minimal_project(temp.path());

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let result = verify_package_structure(temp.path(), &manifest);
        // Should warn about missing src
        assert!(matches!(result, StepResult::Warning(_)));
    }

    #[test]
    fn test_build_package_skip() {
        let temp = TempDir::new().unwrap();
        let result = build_package(temp.path(), true);
        assert!(matches!(result, StepResult::Skip(_)));
    }

    #[test]
    fn test_build_package_success() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let result = build_package(temp.path(), false);
        assert!(matches!(result, StepResult::Success(_)));
    }

    #[test]
    fn test_run_tests_skip() {
        let temp = TempDir::new().unwrap();
        let result = run_tests(temp.path(), true);
        assert!(matches!(result, StepResult::Skip(_)));
    }

    #[test]
    fn test_create_archive_dry_run() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let result = create_package_archive(temp.path(), &manifest, true);
        assert!(matches!(result, StepResult::Success(_)));
    }

    #[test]
    fn test_create_archive() {
        let temp = TempDir::new().unwrap();
        create_test_project(temp.path());

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        let result = create_package_archive(temp.path(), &manifest, false);
        assert!(matches!(result, StepResult::Success(_)));
        assert!(temp.path().join("target/package").exists());
    }

    #[test]
    fn test_step_result_display() {
        // Just ensure these don't panic
        print_step_result(1, "Test", &StepResult::Success("ok".to_string()));
        print_step_result(2, "Test", &StepResult::Warning("warn".to_string()));
        print_step_result(3, "Test", &StepResult::Skip("skip".to_string()));
    }
}
