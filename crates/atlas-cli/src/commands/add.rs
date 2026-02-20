//! Add dependency command (atlas add)

use anyhow::{bail, Context, Result};
use atlas_package::manifest::{Dependency, DetailedDependency, PackageManifest};
use std::fs;
use std::path::{Path, PathBuf};

/// Arguments for the add command
#[derive(Debug, Clone)]
pub struct AddArgs {
    /// Package name to add
    pub package: String,
    /// Version constraint (e.g., "1.0", "^1.2.3")
    pub version: Option<String>,
    /// Add as dev dependency
    pub dev: bool,
    /// Git repository URL
    pub git: Option<String>,
    /// Git branch
    pub branch: Option<String>,
    /// Git tag
    pub tag: Option<String>,
    /// Git revision
    pub rev: Option<String>,
    /// Local path dependency
    pub path: Option<PathBuf>,
    /// Enable specific features
    pub features: Vec<String>,
    /// Disable default features
    pub no_default_features: bool,
    /// Mark as optional dependency
    pub optional: bool,
    /// Rename the dependency
    pub rename: Option<String>,
    /// Project directory (defaults to current)
    pub project_dir: PathBuf,
    /// Dry run (don't modify files)
    pub dry_run: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for AddArgs {
    fn default() -> Self {
        Self {
            package: String::new(),
            version: None,
            dev: false,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            path: None,
            features: Vec::new(),
            no_default_features: false,
            optional: false,
            rename: None,
            project_dir: PathBuf::from("."),
            dry_run: false,
            verbose: false,
        }
    }
}

/// Run the add command
pub fn run(args: AddArgs) -> Result<()> {
    let manifest_path = find_manifest(&args.project_dir)?;

    if args.verbose {
        println!("Reading manifest from {}", manifest_path.display());
    }

    // Load existing manifest
    let mut manifest =
        PackageManifest::from_file(&manifest_path).context("Failed to read atlas.toml")?;

    // Build dependency specification
    let dependency = build_dependency(&args)?;

    // Add to appropriate section
    let dep_name = args.rename.as_ref().unwrap_or(&args.package);
    let section_name = if args.dev {
        "dev-dependencies"
    } else {
        "dependencies"
    };

    if args.dev {
        if manifest.dev_dependencies.contains_key(dep_name) {
            println!("Updating {} in {}", dep_name, section_name);
        } else {
            println!("Adding {} to {}", dep_name, section_name);
        }
        manifest
            .dev_dependencies
            .insert(dep_name.clone(), dependency.clone());
    } else {
        if manifest.dependencies.contains_key(dep_name) {
            println!("Updating {} in {}", dep_name, section_name);
        } else {
            println!("Adding {} to {}", dep_name, section_name);
        }
        manifest
            .dependencies
            .insert(dep_name.clone(), dependency.clone());
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

    if args.verbose {
        println!("Updated {}", manifest_path.display());
    }

    // Print summary
    let version_info = format_dependency_info(&dependency);
    println!("  {} {} {}", green_check(), dep_name, version_info);

    // Hint about install
    println!("\nRun 'atlas install' to download the package.");

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

/// Build dependency from arguments
fn build_dependency(args: &AddArgs) -> Result<Dependency> {
    // Check for conflicting source types
    let source_count = [
        args.git.is_some(),
        args.path.is_some(),
        args.version.is_some(),
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    if source_count > 1 && args.git.is_none() && args.path.is_none() {
        // version with git/path is allowed
    } else if args.git.is_some() && args.path.is_some() {
        bail!("Cannot specify both --git and --path");
    }

    // Simple version-only dependency
    if args.git.is_none()
        && args.path.is_none()
        && args.features.is_empty()
        && !args.no_default_features
        && !args.optional
        && args.rename.is_none()
    {
        let version = args.version.clone().unwrap_or_else(|| "*".to_string());
        return Ok(Dependency::Simple(version));
    }

    // Detailed dependency
    let detailed = DetailedDependency {
        version: if args.git.is_none() && args.path.is_none() {
            Some(args.version.clone().unwrap_or_else(|| "*".to_string()))
        } else {
            args.version.clone()
        },
        git: args.git.clone(),
        branch: args.branch.clone(),
        tag: args.tag.clone(),
        rev: args.rev.clone(),
        path: args.path.clone(),
        registry: None,
        optional: if args.optional { Some(true) } else { None },
        features: if args.features.is_empty() {
            None
        } else {
            Some(args.features.clone())
        },
        default_features: if args.no_default_features {
            Some(false)
        } else {
            None
        },
        rename: args.rename.clone(),
    };

    Ok(Dependency::Detailed(detailed))
}

/// Format dependency info for display
fn format_dependency_info(dep: &Dependency) -> String {
    match dep {
        Dependency::Simple(v) => format!("v{}", v),
        Dependency::Detailed(d) => {
            if let Some(ref git) = d.git {
                let ref_info = d
                    .branch
                    .as_ref()
                    .map(|b| format!(" (branch: {})", b))
                    .or_else(|| d.tag.as_ref().map(|t| format!(" (tag: {})", t)))
                    .or_else(|| d.rev.as_ref().map(|r| format!(" (rev: {})", r)))
                    .unwrap_or_default();
                format!("git: {}{}", git, ref_info)
            } else if let Some(ref path) = d.path {
                format!("path: {}", path.display())
            } else if let Some(ref version) = d.version {
                format!("v{}", version)
            } else {
                "latest".to_string()
            }
        }
    }
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

[dev-dependencies]
"#;
        let path = dir.join("atlas.toml");
        fs::write(&path, manifest).unwrap();
        path
    }

    #[test]
    fn test_find_manifest() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let result = find_manifest(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("atlas.toml"));
    }

    #[test]
    fn test_find_manifest_not_found() {
        let temp = TempDir::new().unwrap();
        let result = find_manifest(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_build_simple_dependency() {
        let args = AddArgs {
            package: "foo".to_string(),
            version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let dep = build_dependency(&args).unwrap();
        assert!(matches!(dep, Dependency::Simple(v) if v == "1.0.0"));
    }

    #[test]
    fn test_build_git_dependency() {
        let args = AddArgs {
            package: "foo".to_string(),
            git: Some("https://github.com/example/foo".to_string()),
            branch: Some("main".to_string()),
            ..Default::default()
        };

        let dep = build_dependency(&args).unwrap();
        match dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.git, Some("https://github.com/example/foo".to_string()));
                assert_eq!(d.branch, Some("main".to_string()));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_build_path_dependency() {
        let args = AddArgs {
            package: "foo".to_string(),
            path: Some(PathBuf::from("../local-pkg")),
            ..Default::default()
        };

        let dep = build_dependency(&args).unwrap();
        match dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.path, Some(PathBuf::from("../local-pkg")));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_build_dependency_with_features() {
        let args = AddArgs {
            package: "foo".to_string(),
            version: Some("1.0".to_string()),
            features: vec!["async".to_string(), "json".to_string()],
            ..Default::default()
        };

        let dep = build_dependency(&args).unwrap();
        match dep {
            Dependency::Detailed(d) => {
                assert_eq!(
                    d.features,
                    Some(vec!["async".to_string(), "json".to_string()])
                );
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_build_optional_dependency() {
        let args = AddArgs {
            package: "foo".to_string(),
            version: Some("1.0".to_string()),
            optional: true,
            ..Default::default()
        };

        let dep = build_dependency(&args).unwrap();
        match dep {
            Dependency::Detailed(d) => {
                assert_eq!(d.optional, Some(true));
            }
            _ => panic!("Expected detailed dependency"),
        }
    }

    #[test]
    fn test_conflicting_git_and_path() {
        let args = AddArgs {
            package: "foo".to_string(),
            git: Some("https://example.com/foo".to_string()),
            path: Some(PathBuf::from("../local")),
            ..Default::default()
        };

        let result = build_dependency(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_adds_dependency() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = AddArgs {
            package: "new-dep".to_string(),
            version: Some("1.2.3".to_string()),
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let content = fs::read_to_string(temp.path().join("atlas.toml")).unwrap();
        assert!(content.contains("new-dep"));
    }

    #[test]
    fn test_run_adds_dev_dependency() {
        let temp = TempDir::new().unwrap();
        create_test_manifest(temp.path());

        let args = AddArgs {
            package: "test-dep".to_string(),
            version: Some("0.1.0".to_string()),
            dev: true,
            project_dir: temp.path().to_path_buf(),
            ..Default::default()
        };

        run(args).unwrap();

        let manifest = PackageManifest::from_file(&temp.path().join("atlas.toml")).unwrap();
        assert!(manifest.dev_dependencies.contains_key("test-dep"));
    }

    #[test]
    fn test_dry_run_does_not_modify() {
        let temp = TempDir::new().unwrap();
        let manifest_path = create_test_manifest(temp.path());
        let original_content = fs::read_to_string(&manifest_path).unwrap();

        let args = AddArgs {
            package: "new-dep".to_string(),
            version: Some("1.0.0".to_string()),
            project_dir: temp.path().to_path_buf(),
            dry_run: true,
            ..Default::default()
        };

        run(args).unwrap();

        let content = fs::read_to_string(&manifest_path).unwrap();
        assert_eq!(content, original_content);
    }

    #[test]
    fn test_format_dependency_info_simple() {
        let dep = Dependency::Simple("1.2.3".to_string());
        assert_eq!(format_dependency_info(&dep), "v1.2.3");
    }

    #[test]
    fn test_format_dependency_info_git() {
        let dep = Dependency::Detailed(DetailedDependency {
            git: Some("https://github.com/example/repo".to_string()),
            branch: Some("main".to_string()),
            version: None,
            tag: None,
            rev: None,
            path: None,
            registry: None,
            optional: None,
            features: None,
            default_features: None,
            rename: None,
        });
        let info = format_dependency_info(&dep);
        assert!(info.contains("git:"));
        assert!(info.contains("branch: main"));
    }
}
