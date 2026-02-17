use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;

#[derive(Parser)]
#[command(name = "atlas")]
#[command(about = "Atlas programming language compiler and runtime", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an Atlas source file
    Run {
        /// Path to the Atlas source file
        file: String,
        /// Output diagnostics in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Type-check an Atlas source file without running
    Check {
        /// Path to the Atlas source file
        file: String,
        /// Output diagnostics in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Build an Atlas project
    Build {
        /// Build profile (dev, release, test, or custom)
        #[arg(long, short = 'p')]
        profile: Option<String>,
        /// Build in release mode (shorthand for --profile=release)
        #[arg(long)]
        release: bool,
        /// Clean build (ignore cache)
        #[arg(long)]
        clean: bool,
        /// Verbose output
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Quiet output (errors only)
        #[arg(long, short = 'q')]
        quiet: bool,
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Start an interactive REPL
    Repl {
        /// Use TUI mode (ratatui) instead of line editor
        #[arg(long)]
        tui: bool,
        /// Disable history persistence (for privacy)
        #[arg(long)]
        no_history: bool,
    },
    /// Dump AST to JSON
    Ast {
        /// Path to the Atlas source file
        file: String,
    },
    /// Dump typecheck information to JSON
    Typecheck {
        /// Path to the Atlas source file
        file: String,
    },
    /// Profile an Atlas source file (VM execution analysis)
    Profile {
        /// Path to the Atlas source file
        file: String,
        /// Hotspot detection threshold percentage (default: 1.0)
        #[arg(long, default_value = "1.0")]
        threshold: f64,
        /// Save profile report to this file
        #[arg(long, short = 'o')]
        output: Option<String>,
        /// Print summary only (no detailed report)
        #[arg(long)]
        summary: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cli_config = config::Config::from_env();

    // Load project configuration (atlas.toml) if in a project directory
    // This is available for commands that need project-level settings
    let _project_config = atlas_config::ConfigLoader::new()
        .load_from_directory(&std::env::current_dir()?)
        .ok(); // Optional - not all commands run in a project

    match cli.command {
        Commands::Run { file, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;
            commands::run::run(&file, use_json)?;
        }
        Commands::Check { file, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;
            commands::check::run(&file, use_json)?;
        }
        Commands::Build {
            profile,
            release,
            clean,
            verbose,
            quiet,
            json,
        } => {
            // Command-line flag overrides environment variable
            let use_json = json || cli_config.default_json;
            let args = commands::build::BuildArgs {
                profile,
                release,
                clean,
                verbose,
                quiet,
                json: use_json,
                ..Default::default()
            };
            commands::build::run(args)?;
        }
        Commands::Repl { tui, no_history } => {
            // Command-line flag overrides environment variable
            let disable_history = no_history || cli_config.no_history;
            commands::repl::run(tui, disable_history, &cli_config)?;
        }
        Commands::Ast { file } => {
            commands::ast::run(&file)?;
        }
        Commands::Typecheck { file } => {
            commands::typecheck::run(&file)?;
        }
        Commands::Profile {
            file,
            threshold,
            output,
            summary,
        } => {
            let mut args = commands::profile::ProfileArgs::new(file);
            args.hotspot_threshold = threshold;
            args.output_file = output.map(std::path::PathBuf::from);
            args.detailed = !summary;
            commands::profile::run(args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_smoke() {
        // Verify CLI can be instantiated
        // This test ensures the binary compiles and basic structure works
        let _cli = Cli::parse_from(["atlas", "repl"]);
        // If we get here without panicking, the CLI structure is valid
    }

    #[test]
    fn test_cli_repl_tui_flag() {
        // Verify TUI flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "repl", "--tui"]);
        match cli.command {
            Commands::Repl { tui, .. } => assert!(tui),
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_repl_no_history_flag() {
        // Verify no-history flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "repl", "--no-history"]);
        match cli.command {
            Commands::Repl { no_history, .. } => assert!(no_history),
            _ => panic!("Expected Repl command"),
        }
    }

    #[test]
    fn test_cli_json_flag() {
        // Verify JSON flag is parsed correctly
        let cli = Cli::parse_from(["atlas", "check", "file.atl", "--json"]);
        match cli.command {
            Commands::Check { json, .. } => assert!(json),
            _ => panic!("Expected Check command"),
        }
    }
}
