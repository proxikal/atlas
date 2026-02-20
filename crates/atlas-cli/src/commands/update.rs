//! Update dependencies command (atlas update)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use atlas_package::{Lockfile, Resolver};
use std::path::{Path, PathBuf};

/// Arguments for the update command
#[derive(Debug, Clone)]
pub struct UpdateArgs {
    /// Specific packages to update (empty = all)
    pub packages: Vec<String>,
    /// Only update dev dependencies
    pub dev: bool,
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Dry run (don't modify files)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for UpdateArgs {
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

/// Update result for a single package
#[derive(Debug)]
pub struct UpdateResult {
    pub name: String,
    pub old_version: Option<semver::Version>,
    pub new_version: semver::Version,
    pub updated: bool,
}

/// Run the update command
pub fn run(args: UpdateArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;
    let project_dir = manifest_path.parent().unwrap();
    let lockfile_path = project_dir.join("atlas.lock");

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load manifest
    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    // Load existing lockfile if present
    let existing_lockfile = if lockfile_path.exists() {
        Some(Lockfile::from_file(&lockfile_path).context("Failed to read atlas.lock")?)
    } else {
        None
    };

    // Determine which packages to update
    let packages_to_update = if args.packages.is_empty() {
        // Update all packages
        let mut all_deps: Vec<String> = manifest.dependencies.keys().cloned().collect();
        if args.dev {
            all_deps = manifest.dev_dependencies.keys().cloned().collect();
        } else {
            all_deps.extend(manifest.dev_dependencies.keys().cloned());
        }
        all_deps
    } else {
        // Validate specified packages exist
        for pkg in &args.packages {
            if !manifest.dependencies.contains_key(pkg)
                && !manifest.dev_dependencies.contains_key(pkg)
            {
                bail!("Package '{}' not found in dependencies", pkg);
            }
        }
        args.packages.clone()
    };

    if packages_to_update.is_empty() {
        println!("No dependencies to update.");
        return Ok(());
    }

    println!("Checking for updates...");

    // Resolve new versions
    let mut resolver = Resolver::new();
    let resolution = resolver.resolve(&manifest)?;

    // Compare with existing lockfile and collect updates
    let mut updates: Vec<UpdateResult> = Vec::new();

    for pkg_name in &packages_to_update {
        let new_version = resolution.get_package(pkg_name).map(|p| p.version.clone());

        let old_version = existing_lockfile
            .as_ref()
            .and_then(|lf| lf.get_package(pkg_name))
            .map(|p| p.version.clone());

        if let Some(new_ver) = new_version {
            let updated = old_version
                .as_ref()
                .map(|old| &new_ver > old)
                .unwrap_or(true);

            updates.push(UpdateResult {
                name: pkg_name.clone(),
                old_version,
                new_version: new_ver,
                updated,
            });
        }
    }

    // Display results
    let updated_count = updates.iter().filter(|u| u.updated).count();

    if updated_count == 0 {
        println!("\n{} All packages are up to date.", green_check());
        return Ok(());
    }

    println!("\nUpdates available:");
    for update in &updates {
        if update.updated {
            let old_ver = update
                .old_version
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "new".to_string());
            println!(
                "  {} {} {} -> {}",
                arrow_up(),
                update.name,
                old_ver,
                update.new_version
            );
        } else if args.verbose {
            println!(
                "  {} {} {} (current)",
                green_check(),
                update.name,
                update.new_version
            );
        }
    }

    if args.dry_run {
        println!("\n[Dry run] Would update {} package(s)", updated_count);
        return Ok(());
    }

    // Generate new lockfile
    let new_lockfile = resolver.generate_lockfile(&resolution);

    // Write lockfile
    new_lockfile.write_to_file(&lockfile_path)?;

    println!(
        "\n{} Updated {} package{}",
        green_check(),
        updated_count,
        if updated_count == 1 { "" } else { "s" }
    );

    // Hint about install
    println!("\nRun 'atlas install' to download updated packages.");

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

/// Up arrow for updates
fn arrow_up() -> &'static str {
    "\u{2191}"
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_package::{LockedPackage, LockedSource};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path) -> PathBuf {
        let manifest = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
foo = "^1.0"
bar = "^2.0"

[dev-dependencies]
test-utils = "^0.1"
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).unwrap();
        path
    }

    fn create_test_lockfile(dir: &Path) -> PathBuf {
        let mut lockfile = Lockfile::new();

        lockfile.add_package(LockedPackage {
            name: "foo".to_string(),
            version: semver::Version::new(1, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        lockfile.add_package(LockedPackage {
            name: "bar".to_string(),
            version: semver::Version::new(2, 0, 0),
            source: LockedSource::Registry { registry: None },
            checksum: None,
            dependencies: HashMap::new(),
        });

        let path = dir.join("atlas.lock");
        lockfile.write_to_file(&path).unwrap();
        path
    }

    #[test]
    fn test_update_all_packages() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = UpdateArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        // Should succeed even without lockfile
        run(args).unwrap();

        // Lockfile should be created
        assert!(temp.path().join("atlas.lock").exists());
    }

    #[test]
    fn test_update_specific_package() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());
        create_test_lockfile(temp.path());

        let args = UpdateArgs {
            packages: vec!["foo".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();
    }

    #[test]
    fn test_update_nonexistent_package_fails() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = UpdateArgs {
            packages: vec!["nonexistent".to_string()],
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_dry_run_does_not_create_lockfile() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = UpdateArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };

        run(args).unwrap();

        // Lockfile should not be created
        assert!(!temp.path().join("atlas.lock").exists());
    }

    #[test]
    fn test_update_result_comparison() {
        let result = UpdateResult {
            name: "foo".to_string(),
            old_version: Some(semver::Version::new(1, 0, 0)),
            new_version: semver::Version::new(1, 1, 0),
            updated: true,
        };

        assert!(result.updated);
        assert_eq!(result.old_version, Some(semver::Version::new(1, 0, 0)));
    }

    #[test]
    fn test_no_manifest_fails() {
        let temp = TempDir::new().unwrap();

        let args = UpdateArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_empty_dependencies() {
        let temp = TempDir::new().unwrap();

        let manifest = r#"[package]
name = "empty-project"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#;
        fs::write(temp.path().join("atlas.toml"), manifest).unwrap();

        let args = UpdateArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        // Should succeed with no dependencies
        run(args).unwrap();
    }
}
