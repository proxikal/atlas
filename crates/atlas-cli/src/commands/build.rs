//! Build command - compile Atlas projects with profiles, scripts, and caching

use anyhow::{Context, Result};
use atlas_build::{BuildScript, Builder, OutputMode, Profile, ScriptPhase};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Build command arguments
#[derive(Default)]
pub struct BuildArgs {
    /// Build profile (dev, release, test, or custom)
    pub profile: Option<String>,
    /// Build in release mode (shorthand for --profile=release)
    pub release: bool,
    /// Specific target to build
    #[allow(dead_code)]
    pub target: Option<String>,
    /// Clean build (ignore cache)
    pub clean: bool,
    /// Verbose output
    pub verbose: bool,
    /// Quiet output (errors only)
    pub quiet: bool,
    /// JSON output
    pub json: bool,
    /// Number of parallel jobs
    #[allow(dead_code)]
    pub jobs: Option<usize>,
    /// Target directory
    pub target_dir: Option<PathBuf>,
    /// Project directory (defaults to current directory)
    pub project_dir: Option<PathBuf>,
}

/// Run the build command
pub fn run(args: BuildArgs) -> Result<()> {
    // Determine project directory
    let project_dir = args
        .project_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    // Create builder
    let mut builder = Builder::new(&project_dir).context("Failed to create builder")?;

    // Determine build profile
    let profile = determine_profile(&args)?;

    // Determine output mode
    let output_mode = determine_output_mode(&args);

    // Clean if requested
    if args.clean {
        if !args.quiet {
            println!("Cleaning build artifacts...");
        }
        builder.clean().context("Failed to clean build artifacts")?;
    }

    // Set target directory if specified
    if let Some(ref target_dir) = args.target_dir {
        builder = builder.with_target_dir(target_dir.clone());
    }

    // Set verbose mode
    if args.verbose {
        builder = builder.with_verbose(true);
    }

    // Load build scripts from manifest
    let scripts = load_build_scripts(&builder, &project_dir)?;

    // Execute build with profile
    let context = builder
        .build_with_profile(profile.clone(), &scripts, output_mode)
        .context("Build failed")?;

    // Display results
    if args.json {
        // JSON output
        let summary = context.stats;
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "profile": profile.name(),
                "total_time": summary.total_time.as_secs_f64(),
                "compilation_time": summary.compilation_time.as_secs_f64(),
                "linking_time": summary.linking_time.as_secs_f64(),
                "modules": summary.total_modules,
                "compiled_modules": summary.compiled_modules,
                "parallel_groups": summary.parallel_groups,
                "artifacts": context.artifacts.len(),
            })
        );
    } else if !args.quiet {
        // Human-readable output
        println!("\n{}", "=".repeat(60));
        println!(
            "Build succeeded in {:.2}s",
            context.stats.total_time.as_secs_f64()
        );
        println!("{}", "=".repeat(60));
        println!("  Profile: {}", profile.name());
        println!("  Modules: {} compiled", context.stats.compiled_modules);
        println!("  Parallel groups: {}", context.stats.parallel_groups);
        println!("  Artifacts: {}", context.artifacts.len());
        println!("{}", "=".repeat(60));
    }

    Ok(())
}

/// Determine build profile from arguments
fn determine_profile(args: &BuildArgs) -> Result<Profile> {
    if args.release {
        Ok(Profile::Release)
    } else if let Some(ref profile_name) = args.profile {
        Profile::from_str(profile_name).map_err(|e| anyhow::anyhow!("Invalid profile: {}", e))
    } else {
        // Check environment variable
        if let Ok(profile_env) = std::env::var("ATLAS_PROFILE") {
            Profile::from_str(&profile_env)
                .map_err(|e| anyhow::anyhow!("Invalid ATLAS_PROFILE: {}", e))
        } else {
            Ok(Profile::Dev)
        }
    }
}

/// Determine output mode from arguments
fn determine_output_mode(args: &BuildArgs) -> OutputMode {
    if args.json {
        OutputMode::Json
    } else if args.quiet {
        OutputMode::Quiet
    } else if args.verbose {
        OutputMode::Verbose
    } else {
        OutputMode::Normal
    }
}

/// Load build scripts from package manifest
fn load_build_scripts(_builder: &Builder, project_dir: &Path) -> Result<Vec<BuildScript>> {
    // Read manifest
    let manifest_path = project_dir.join("atlas.toml");
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }

    let manifest_content =
        std::fs::read_to_string(&manifest_path).context("Failed to read atlas.toml")?;

    let manifest: atlas_build::PackageManifest =
        toml::from_str(&manifest_content).context("Failed to parse atlas.toml")?;

    // Extract build scripts if present
    let mut scripts = Vec::new();

    if let Some(build_config) = manifest.build {
        for script_config in build_config.scripts {
            let phase = parse_script_phase(&script_config.phase)?;

            let script = if let Some(path) = script_config.path {
                BuildScript::atlas(&script_config.name, path, phase)
            } else if let Some(shell) = script_config.shell {
                BuildScript::shell(&script_config.name, shell, phase)
            } else {
                anyhow::bail!(
                    "Build script '{}' must specify either 'path' or 'shell'",
                    script_config.name
                );
            };

            let script = if let Some(timeout_secs) = script_config.timeout {
                script.with_timeout(Duration::from_secs(timeout_secs))
            } else {
                script
            };

            let script = if !script_config.permissions.is_empty() {
                script.with_permissions(script_config.permissions)
            } else {
                script
            };

            scripts.push(script);
        }
    }

    Ok(scripts)
}

/// Parse script phase from string
fn parse_script_phase(phase_str: &str) -> Result<ScriptPhase> {
    match phase_str {
        "pre-build" | "prebuild" => Ok(ScriptPhase::PreBuild),
        "post-build" | "postbuild" => Ok(ScriptPhase::PostBuild),
        "post-link" | "postlink" => Ok(ScriptPhase::PostLink),
        _ => anyhow::bail!(
            "Invalid script phase: {}. Valid phases are: pre-build, post-build, post-link",
            phase_str
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_profile_default() {
        let args = BuildArgs::default();
        let profile = determine_profile(&args).unwrap();
        assert_eq!(profile, Profile::Dev);
    }

    #[test]
    fn test_determine_profile_release() {
        let args = BuildArgs {
            release: true,
            ..Default::default()
        };
        let profile = determine_profile(&args).unwrap();
        assert_eq!(profile, Profile::Release);
    }

    #[test]
    fn test_determine_profile_custom() {
        let args = BuildArgs {
            profile: Some("test".to_string()),
            ..Default::default()
        };
        let profile = determine_profile(&args).unwrap();
        assert_eq!(profile, Profile::Test);
    }

    #[test]
    fn test_determine_profile_release_priority() {
        let args = BuildArgs {
            release: true,
            profile: Some("dev".to_string()),
            ..Default::default()
        };
        let profile = determine_profile(&args).unwrap();
        assert_eq!(profile, Profile::Release); // --release takes priority
    }

    #[test]
    fn test_determine_output_mode_default() {
        let args = BuildArgs::default();
        let mode = determine_output_mode(&args);
        assert_eq!(mode, OutputMode::Normal);
    }

    #[test]
    fn test_determine_output_mode_verbose() {
        let args = BuildArgs {
            verbose: true,
            ..Default::default()
        };
        let mode = determine_output_mode(&args);
        assert_eq!(mode, OutputMode::Verbose);
    }

    #[test]
    fn test_determine_output_mode_quiet() {
        let args = BuildArgs {
            quiet: true,
            ..Default::default()
        };
        let mode = determine_output_mode(&args);
        assert_eq!(mode, OutputMode::Quiet);
    }

    #[test]
    fn test_determine_output_mode_json() {
        let args = BuildArgs {
            json: true,
            ..Default::default()
        };
        let mode = determine_output_mode(&args);
        assert_eq!(mode, OutputMode::Json);
    }

    #[test]
    fn test_parse_script_phase_prebuild() {
        assert_eq!(
            parse_script_phase("pre-build").unwrap(),
            ScriptPhase::PreBuild
        );
        assert_eq!(
            parse_script_phase("prebuild").unwrap(),
            ScriptPhase::PreBuild
        );
    }

    #[test]
    fn test_parse_script_phase_postbuild() {
        assert_eq!(
            parse_script_phase("post-build").unwrap(),
            ScriptPhase::PostBuild
        );
        assert_eq!(
            parse_script_phase("postbuild").unwrap(),
            ScriptPhase::PostBuild
        );
    }

    #[test]
    fn test_parse_script_phase_postlink() {
        assert_eq!(
            parse_script_phase("post-link").unwrap(),
            ScriptPhase::PostLink
        );
        assert_eq!(
            parse_script_phase("postlink").unwrap(),
            ScriptPhase::PostLink
        );
    }

    #[test]
    fn test_parse_script_phase_invalid() {
        assert!(parse_script_phase("invalid").is_err());
    }
}
