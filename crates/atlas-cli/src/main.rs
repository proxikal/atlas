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
    /// Compile an Atlas source file to bytecode
    Build {
        /// Path to the Atlas source file
        file: String,
        /// Disassemble bytecode and print to stdout
        #[arg(long)]
        disasm: bool,
        /// Output diagnostics in JSON format
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::from_env();

    match cli.command {
        Commands::Run { file, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || config.default_json;
            commands::run::run(&file, use_json)?;
        }
        Commands::Check { file, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || config.default_json;
            commands::check::run(&file, use_json)?;
        }
        Commands::Build { file, disasm, json } => {
            // Command-line flag overrides environment variable
            let use_json = json || config.default_json;
            commands::build::run(&file, disasm, use_json)?;
        }
        Commands::Repl { tui, no_history } => {
            // Command-line flag overrides environment variable
            let disable_history = no_history || config.no_history;
            commands::repl::run(tui, disable_history, &config)?;
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
