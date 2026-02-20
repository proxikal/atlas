//! Remove dependency command (atlas remove)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use std::fs;
use std::path::{Path, PathBuf};

/// Arguments for the remove command
#[derive(Debug, Clone)]
pub struct RemoveArgs {
    /// Package names to remove
    pub packages: Vec<String>,
    /// Also remove from dev dependencies
    pub dev: bool,
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Dry run (don't modify files)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for RemoveArgs {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            dev: false,
            project_dir: PathBuf::from("."),
            dry_run: false,
            verbose: false,
        }
    }
}

/// Run the remove command
pub fn run(args: RemoveArgs) -> Result<()> {
    if args.packages.is_empty() {
        bail!("No packages specified to remove");
    }

    let manifest_path = find_manifest(&args.project_dir)?;

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load existing manifest
    let mut manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    let mut removed_count = 0;
    let mut not_found = Vec::new();

    for package in &args.packages {
        let mut found = false;

        // Try to remove from dependencies
        if manifest.dependencies.remove(package).is_some() {
            println!("  {} Removed {} from dependencies", green_check(), package);
            found = true;
            removed_count += 1;
        }

        // Try to remove from dev-dependencies
        if (args.dev || !found) && manifest.dev_dependencies.remove(package).is_some() {
            println!(
                "  {} Removed {} from dev-dependencies",
                green_check(),
                package
            );
            found = true;
            removed_count += 1;
        }

        if !found {
            not_found.push(package.clone());
        }
    }

    // Report packages not found
    for pkg in &not_found {
        println!(
            "  {} Package '{}' not found in dependencies",
            yellow_warning(),
            pkg
        );
    }

    if removed_count == 0 {
        bail!("No packages were removed");
    }

    if args.dry_run {
        println!("\n[Dry run] Would update {}:", manifest_path.display());
        let content = manifest
            .to_string()
            .context("Failed to serialize manifest")?;
        println!("{}", content);
        return Ok(());
    }

    // Write updated manifest
    let content = manifest
        .to_string()
        .context("Failed to serialize manifest")?;
    fs::write(&manifest_path, &content).context("Failed to write atlas.toml")?;

    println!(
        "\nRemoved {} package{}",
        removed_count,
        if removed_count == 1 { "" } else { "s" }
    );

    // Hint about cleanup
    println!("\nRun 'atlas install' to clean up unused packages.");

    Ok(())
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

/// Yellow warning symbol
fn yellow_warning() -> &'static str {
    "\u{26A0}"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path) -> PathBuf {
        let manifest = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
foo = "1.0"
bar = "2.0"

[dev-dependencies]
test-utils = "0.1"
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).unwrap();
        path
    }

    #[test]
    fn test_remove_single_package() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: vec!["foo".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(!manifest.dependencies.contains_key("foo"));
        assert!(manifest.dependencies.contains_key("bar"));
    }

    #[test]
    fn test_remove_multiple_packages() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: vec!["foo".to_string(), "bar".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(!manifest.dependencies.contains_key("foo"));
        assert!(!manifest.dependencies.contains_key("bar"));
    }

    #[test]
    fn test_remove_dev_dependency() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: vec!["test-utils".to_string()],
            dev: true,
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(!manifest.dev_dependencies.contains_key("test-utils"));
    }

    #[test]
    fn test_remove_nonexistent_package() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: vec!["nonexistent".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        // Should fail because nothing was removed
        assert!(run(args).is_err());
    }

    #[test]
    fn test_remove_with_nonexistent_mixed() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: vec!["foo".to_string(), "nonexistent".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        // Should succeed because foo was removed
        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(!manifest.dependencies.contains_key("foo"));
    }

    #[test]
    fn test_dry_run_does_not_modify() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path());
        let original_content = fs::read_to_string(&manifest_path).unwrap();

        let args = RemoveArgs {
            packages: vec!["foo".to_string()],
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };

        run(args).unwrap();

        let content = fs::read_to_string(&manifest_path).unwrap();
        assert_eq!(content, original_content);
    }

    #[test]
    fn test_empty_packages_fails() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = RemoveArgs {
            packages: Vec::new(),
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_remove_auto_finds_dev_dep() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        // Don't specify --dev, but package is only in dev-dependencies
        let args = RemoveArgs {
            packages: vec!["test-utils".to_string()],
            dev: false, // Will still check dev-deps if not found in regular deps
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(!manifest.dev_dependencies.contains_key("test-utils"));
    }
}
