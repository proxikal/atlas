use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

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
    },
    /// Type-check an Atlas source file without running
    Check {
        /// Path to the Atlas source file
        file: String,
    },
    /// Compile an Atlas source file to bytecode
    Build {
        /// Path to the Atlas source file
        file: String,
        /// Disassemble bytecode and print to stdout
        #[arg(long)]
        disasm: bool,
    },
    /// Start an interactive REPL
    Repl {
        /// Use TUI mode (ratatui) instead of line editor
        #[arg(long)]
        tui: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            commands::run::run(&file)?;
        }
        Commands::Check { file } => {
            commands::check::run(&file)?;
        }
        Commands::Build { file, disasm } => {
            commands::build::run(&file, disasm)?;
        }
        Commands::Repl { tui } => {
            commands::repl::run(tui)?;
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
            Commands::Repl { tui } => assert!(tui),
            _ => panic!("Expected Repl command"),
        }
    }
}
