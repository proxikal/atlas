//! Project initialization command (atlas init)

use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Arguments for the init command
#[derive(Debug, Clone)]
pub struct InitArgs {
    /// Project name (defaults to directory name)
    pub name: Option<String>,
    /// Project type (bin or lib)
    pub project_type: ProjectType,
    /// Initialize git repository
    pub git: bool,
    /// Path to create project in
    pub path: PathBuf,
    /// Skip interactive prompts
    pub non_interactive: bool,
    /// Verbose output
    pub verbose: bool,
}

impl Default for InitArgs {
    fn default() -> Self {
        Self {
            name: None,
            project_type: ProjectType::Binary,
            git: true,
            path: PathBuf::from("."),
            non_interactive: false,
            verbose: false,
        }
    }
}

/// Project type for scaffolding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectType {
    #[default]
    Binary,
    Library,
}

impl std::str::FromStr for ProjectType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bin" | "binary" => Ok(ProjectType::Binary),
            "lib" | "library" => Ok(ProjectType::Library),
            _ => Err(format!("Unknown project type: {}", s)),
        }
    }
}

/// Run the init command
pub fn run(args: InitArgs) -> Result<()> {
    let path = args
        .path
        .canonicalize()
        .unwrap_or_else(|_| args.path.clone());

    // Determine project name
    let name = if let Some(ref n) = args.name {
        validate_package_name(n)?;
        n.clone()
    } else if args.non_interactive {
        // Use directory name
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "my-project".to_string());
        validate_package_name(&dir_name)?;
        dir_name
    } else {
        // Interactive prompt
        let default_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project");
        prompt_for_name(default_name)?
    };

    // Check if manifest already exists
    let manifest_path = path.join("atlas.toml");
    if manifest_path.exists() {
        bail!(
            "Project already initialized: atlas.toml exists at {}",
            manifest_path.display()
        );
    }

    // Create project structure
    create_project(&path, &name, args.project_type, args.verbose)?;

    // Initialize git if requested
    if args.git {
        init_git(&path, args.verbose)?;
    }

    println!("\n{} Created Atlas project '{}'", green_check(), name);
    println!("  Path: {}", path.display());
    println!("\nTo get started:");
    if path.as_os_str() != "." {
        println!("  cd {}", path.display());
    }
    match args.project_type {
        ProjectType::Binary => println!("  atlas run src/main.atl"),
        ProjectType::Library => println!("  atlas check src/lib.atl"),
    }

    Ok(())
}

/// Create project structure
fn create_project(path: &Path, name: &str, project_type: ProjectType, verbose: bool) -> Result<()> {
    // Create directories
    fs::create_dir_all(path).context("Failed to create project directory")?;
    fs::create_dir_all(path.join("src")).context("Failed to create src directory")?;

    if verbose {
        println!("Creating project structure...");
    }

    // Create atlas.toml manifest
    let manifest = generate_manifest(name, project_type);
    let manifest_path = path.join("atlas.toml");
    fs::write(&manifest_path, manifest).context("Failed to write atlas.toml")?;
    if verbose {
        println!("  Created {}", manifest_path.display());
    }

    // Create main source file
    match project_type {
        ProjectType::Binary => {
            let main_content = generate_main_file();
            let main_path = path.join("src/main.atl");
            fs::write(&main_path, main_content).context("Failed to write main.atl")?;
            if verbose {
                println!("  Created {}", main_path.display());
            }
        }
        ProjectType::Library => {
            let lib_content = generate_lib_file();
            let lib_path = path.join("src/lib.atl");
            fs::write(&lib_path, lib_content).context("Failed to write lib.atl")?;
            if verbose {
                println!("  Created {}", lib_path.display());
            }
        }
    }

    // Create .gitignore
    let gitignore_content = generate_gitignore();
    let gitignore_path = path.join(".gitignore");
    fs::write(&gitignore_path, gitignore_content).context("Failed to write .gitignore")?;
    if verbose {
        println!("  Created {}", gitignore_path.display());
    }

    Ok(())
}

/// Generate atlas.toml manifest content
fn generate_manifest(name: &str, project_type: ProjectType) -> String {
    let version = "0.1.0";

    let mut manifest = format!(
        r#"[package]
name = "{name}"
version = "{version}"
description = "A new Atlas project"
authors = []
license = "MIT"

"#
    );

    match project_type {
        ProjectType::Binary => {
            manifest.push_str(&format!(
                r#"[[bin]]
name = "{name}"
path = "src/main.atl"

"#
            ));
        }
        ProjectType::Library => {
            manifest.push_str(
                r#"[lib]
path = "src/lib.atl"

"#,
            );
        }
    }

    manifest.push_str(
        r#"[dependencies]
# Add dependencies here
# example = "1.0"

[dev-dependencies]
# Add dev dependencies here
"#,
    );

    manifest
}

/// Generate main.atl content
fn generate_main_file() -> String {
    r#"// Atlas main entry point

fn main() {
    print("Hello, Atlas!")
}
"#
    .to_string()
}

/// Generate lib.atl content
fn generate_lib_file() -> String {
    r#"// Atlas library module

/// A greeting function
/// @param name The name to greet
/// @returns A greeting string
fn greet(name) {
    return "Hello, " + name + "!"
}

/// Exported functions
export { greet }
"#
    .to_string()
}

/// Generate .gitignore content
fn generate_gitignore() -> String {
    r#"# Atlas build artifacts
/target/
/dist/
/.atlas/

# Lock file (uncomment to track)
# atlas.lock

# Editor files
*.swp
*.swo
*~
.idea/
.vscode/

# OS files
.DS_Store
Thumbs.db
"#
    .to_string()
}

/// Initialize git repository
fn init_git(path: &Path, verbose: bool) -> Result<()> {
    // Check if already a git repo
    if path.join(".git").exists() {
        if verbose {
            println!("  Git repository already exists");
        }
        return Ok(());
    }

    // Try to initialize git
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            if verbose {
                println!("  Initialized git repository");
            }
            Ok(())
        }
        Ok(_) => {
            // Git failed but we continue anyway
            if verbose {
                println!("  Warning: Failed to initialize git repository");
            }
            Ok(())
        }
        Err(_) => {
            // Git not found, skip silently
            if verbose {
                println!("  Note: git not found, skipping repository initialization");
            }
            Ok(())
        }
    }
}

/// Prompt user for project name
fn prompt_for_name(default: &str) -> Result<String> {
    print!("Project name [{}]: ", default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let name = if input.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    };

    validate_package_name(&name)?;
    Ok(name)
}

/// Validate package name
fn validate_package_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Package name cannot be empty");
    }

    if name.len() > 64 {
        bail!("Package name must be 64 characters or less");
    }

    // First character must be alphanumeric
    if !name.chars().next().unwrap().is_alphanumeric() {
        bail!("Package name must start with a letter or number");
    }

    // Only alphanumeric, hyphen, and underscore allowed
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            bail!("Package name can only contain letters, numbers, hyphens, and underscores");
        }
    }

    // Reserved names
    let reserved = ["atlas", "std", "core", "test", "debug"];
    if reserved.contains(&name.to_lowercase().as_str()) {
        bail!("'{}' is a reserved package name", name);
    }

    Ok(())
}

/// Green checkmark for success messages
fn green_check() -> &'static str {
    "\u{2713}"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_package_name_valid() {
        assert!(validate_package_name("my-package").is_ok());
        assert!(validate_package_name("my_package").is_ok());
        assert!(validate_package_name("package123").is_ok());
        assert!(validate_package_name("a").is_ok());
    }

    #[test]
    fn test_validate_package_name_invalid() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("-invalid").is_err());
        assert!(validate_package_name("has space").is_err());
        assert!(validate_package_name("has.dot").is_err());
    }

    #[test]
    fn test_validate_package_name_reserved() {
        assert!(validate_package_name("atlas").is_err());
        assert!(validate_package_name("std").is_err());
        assert!(validate_package_name("core").is_err());
    }

    #[test]
    fn test_project_type_from_str() {
        assert_eq!("bin".parse::<ProjectType>().unwrap(), ProjectType::Binary);
        assert_eq!("lib".parse::<ProjectType>().unwrap(), ProjectType::Library);
        assert!("invalid".parse::<ProjectType>().is_err());
    }

    #[test]
    fn test_generate_manifest_binary() {
        let manifest = generate_manifest("test-proj", ProjectType::Binary);
        assert!(manifest.contains("name = \"test-proj\""));
        assert!(manifest.contains("[[bin]]"));
        assert!(manifest.contains("path = \"src/main.atl\""));
    }

    #[test]
    fn test_generate_manifest_library() {
        let manifest = generate_manifest("test-lib", ProjectType::Library);
        assert!(manifest.contains("name = \"test-lib\""));
        assert!(manifest.contains("[lib]"));
        assert!(manifest.contains("path = \"src/lib.atl\""));
    }

    #[test]
    fn test_create_binary_project() {
        let temp = TempDir::new().unwrap();
        let path = temp.path();

        create_project(path, "test-proj", ProjectType::Binary, false).unwrap();

        assert!(path.join("atlas.toml").exists());
        assert!(path.join("src/main.atl").exists());
        assert!(path.join(".gitignore").exists());
    }

    #[test]
    fn test_create_library_project() {
        let temp = TempDir::new().unwrap();
        let path = temp.path();

        create_project(path, "test-lib", ProjectType::Library, false).unwrap();

        assert!(path.join("atlas.toml").exists());
        assert!(path.join("src/lib.atl").exists());
        assert!(path.join(".gitignore").exists());
    }

    #[test]
    fn test_run_creates_project() {
        let temp = TempDir::new().unwrap();

        let args = InitArgs {
            name: Some("my-project".to_string()),
            project_type: ProjectType::Binary,
            git: false, // Skip git to avoid external dependency
            path: temp.path().to_path_buf(),
            non_interactive: true,
            verbose: false,
        };

        run(args).unwrap();

        assert!(temp.path().join("atlas.toml").exists());
        assert!(temp.path().join("src/main.atl").exists());
    }

    #[test]
    fn test_run_fails_if_manifest_exists() {
        let temp = TempDir::new().unwrap();

        // Create existing manifest
        fs::write(
            temp.path().join("atlas.toml"),
            "[package]\nname = \"existing\"",
        )
        .unwrap();

        let args = InitArgs {
            name: Some("new-project".to_string()),
            path: temp.path().to_path_buf(),
            non_interactive: true,
            ..Default::default()
        };

        assert!(run(args).is_err());
    }
}
