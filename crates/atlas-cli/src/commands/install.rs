//! Install dependencies command (atlas install)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::PackageManifest;
use atlas_package::{Lockfile, Resolver};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Arguments for the install command
#[derive(Debug, Clone)]
pub struct InstallArgs {
    /// Specific packages to install (empty = all from manifest)
    pub packages: Vec<String>,
    /// Only install production dependencies (skip dev)
    pub production: bool,
    /// Force reinstall even if packages exist
    pub force: bool,
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Dry run (don't actually install)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
    /// Quiet output (errors only)
    pub quiet: bool,
}

impl Default for InstallArgs {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            production: false,
            force: false,
            project_dir: PathBuf::from("."),
            dry_run: false,
            verbose: false,
            quiet: false,
        }
    }
}

/// Installation statistics
#[derive(Debug, Default)]
struct InstallStats {
    resolved: usize,
    downloaded: usize,
    cached: usize,
    failed: usize,
}

/// Run the install command
pub fn run(args: InstallArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;
    let project_dir = manifest_path.parent().unwrap();
    let lockfile_path = project_dir.join("atlas.lock");
    let deps_dir = project_dir.join("atlas_modules");

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load manifest
    let manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    // Check if there are any dependencies
    let has_deps = !manifest.dependencies.is_empty()
        || (!args.production && !manifest.dev_dependencies.is_empty());

    if !has_deps && args.packages.is_empty() {
        if !args.quiet {
            println!("No dependencies to install.");
        }
        return Ok(());
    }

    // Load or create lockfile
    let existing_lockfile = if lockfile_path.exists() {
        if args.verbose {
            println!("Using existing lockfile");
        }
        Some(Lockfile::from_file(&lockfile_path).context("Failed to read atlas.lock")?)
    } else {
        None
    };

    // Create progress indicator
    let spinner = if !args.quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    // Resolve dependencies
    if let Some(ref pb) = spinner {
        pb.set_message("Resolving dependencies...");
    }

    let mut resolver = Resolver::new();
    let resolution = resolver
        .resolve_with_lockfile(&manifest, existing_lockfile.as_ref())
        .context("Failed to resolve dependencies")?;

    let mut stats = InstallStats {
        resolved: resolution.package_count(),
        ..Default::default()
    };

    if args.verbose {
        println!("Resolved {} packages", stats.resolved);
    }

    if stats.resolved == 0 {
        if let Some(ref pb) = spinner {
            pb.finish_with_message("No packages to install.");
        }
        return Ok(());
    }

    // Create dependencies directory
    if !args.dry_run {
        fs::create_dir_all(&deps_dir).context("Failed to create atlas_modules directory")?;
    }

    // Track installed packages
    let mut installed_packages: HashSet<String> = HashSet::new();

    // Check existing installed packages
    if deps_dir.exists() {
        if let Ok(entries) = fs::read_dir(&deps_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        installed_packages.insert(name.to_string());
                    }
                }
            }
        }
    }

    // Download and install packages
    if let Some(ref pb) = spinner {
        pb.set_message(format!("Installing {} packages...", stats.resolved));
    }

    for (name, package) in &resolution.packages {
        if args.verbose {
            println!("  Installing {}@{}", name, package.version);
        }

        // Check if already installed
        if !args.force && installed_packages.contains(name) {
            stats.cached += 1;
            continue;
        }

        if args.dry_run {
            stats.downloaded += 1;
            continue;
        }

        // TODO: Actually download from registry in future phase
        // For now, we simulate the installation
        let pkg_dir = deps_dir.join(name);
        fs::create_dir_all(&pkg_dir)?;

        // Create placeholder module file
        let module_content = format!(
            "// Auto-installed: {}@{}\n// Package source: registry\n",
            name, package.version
        );
        fs::write(pkg_dir.join("mod.atl"), module_content)?;

        stats.downloaded += 1;
    }

    // Generate/update lockfile
    if !args.dry_run {
        let new_lockfile = resolver.generate_lockfile(&resolution);
        new_lockfile.write_to_file(&lockfile_path)?;

        if args.verbose {
            println!("Updated {}", lockfile_path.display());
        }
    }

    // Finish progress
    if let Some(ref pb) = spinner {
        pb.finish_and_clear();
    }

    // Print summary
    if !args.quiet {
        print_summary(&stats, args.dry_run);
    }

    Ok(())
}

/// Print installation summary
fn print_summary(stats: &InstallStats, dry_run: bool) {
    if dry_run {
        println!("\n[Dry run] Would install:");
    } else {
        println!();
    }

    let mut parts = Vec::new();

    if stats.downloaded > 0 {
        parts.push(format!(
            "{} {}",
            stats.downloaded,
            if stats.downloaded == 1 {
                "package installed"
            } else {
                "packages installed"
            }
        ));
    }

    if stats.cached > 0 {
        parts.push(format!(
            "{} {} from cache",
            stats.cached,
            if stats.cached == 1 {
                "package"
            } else {
                "packages"
            }
        ));
    }

    if stats.failed > 0 {
        parts.push(format!(
            "{} {} failed",
            stats.failed,
            if stats.failed == 1 {
                "package"
            } else {
                "packages"
            }
        ));
    }

    if parts.is_empty() {
        println!("{} Already up to date.", green_check());
    } else {
        println!("{} {}", green_check(), parts.join(", "));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_empty_manifest(dir: &Path) -> PathBuf {
        let manifest = r#"[package]
name = "empty-project"
version = "0.1.0"

[dependencies]

[dev-dependencies]
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).unwrap();
        path
    }

    #[test]
    fn test_install_creates_lockfile() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        run(args).unwrap();

        assert!(temp.path().join("atlas.lock").exists());
    }

    #[test]
    fn test_install_creates_modules_dir() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        run(args).unwrap();

        assert!(temp.path().join("atlas_modules").exists());
    }

    #[test]
    fn test_install_empty_deps() {
        let temp = TempDir::new().unwrap();
        create_empty_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };

        run(args).unwrap();
    }

    #[test]
    fn test_install_dry_run() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            quiet: true,
            ..Default::default()
        };

        run(args).unwrap();

        // Should not create lockfile or modules dir
        assert!(!temp.path().join("atlas.lock").exists());
        assert!(!temp.path().join("atlas_modules").exists());
    }

    #[test]
    fn test_install_production_only() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            production: true,
            quiet: true,
            ..Default::default()
        };

        run(args).unwrap();
    }

    #[test]
    fn test_install_no_manifest() {
        let temp = TempDir::new().unwrap();

        let args = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        assert!(run(args).is_err());
    }

    #[test]
    fn test_install_stats_default() {
        let stats = InstallStats::default();
        assert_eq!(stats.resolved, 0);
        assert_eq!(stats.downloaded, 0);
        assert_eq!(stats.cached, 0);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_print_summary_format() {
        // Just ensure it doesn't panic
        let stats = InstallStats {
            resolved: 5,
            downloaded: 3,
            cached: 2,
            failed: 0,
        };
        print_summary(&stats, false);
    }

    #[test]
    fn test_force_reinstall() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        // First install
        let args1 = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            quiet: true,
            ..Default::default()
        };
        run(args1).unwrap();

        // Force reinstall
        let args2 = InstallArgs {
            project_dir: temp.path().to_path_buf(),
            force: true,
            quiet: true,
            ..Default::default()
        };
        run(args2).unwrap();
    }
}
