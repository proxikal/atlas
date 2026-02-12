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
    },
    /// Start an interactive REPL
    Repl,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            println!("Running: {}", file);
            println!("(Not yet implemented)");
        }
        Commands::Check { file } => {
            println!("Checking: {}", file);
            println!("(Not yet implemented)");
        }
        Commands::Build { file } => {
            println!("Building: {}", file);
            println!("(Not yet implemented)");
        }
        Commands::Repl => {
            commands::repl::run()?;
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
}
