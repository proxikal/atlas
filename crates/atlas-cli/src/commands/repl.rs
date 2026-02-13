//! REPL command implementation

use anyhow::Result;
use atlas_runtime::ReplCore;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the interactive REPL
///
/// If `use_tui` is true, uses ratatui TUI mode.
/// Otherwise, uses rustyline line-editor mode (default).
pub fn run(use_tui: bool) -> Result<()> {
    if use_tui {
        // Use TUI mode (ratatui)
        return super::repl_tui::run();
    }

    // Use line-editor mode (rustyline) - default
    let mut rl = DefaultEditor::new()?;
    let mut repl = ReplCore::new();

    // Display welcome message
    println!("Atlas v{} REPL", atlas_runtime::VERSION);
    println!("Type expressions or statements, or :quit to exit");
    println!("Commands: :quit (or :q), :reset, :help");
    println!();

    loop {
        // Read a line
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                // Handle REPL commands
                if line.trim() == ":quit" || line.trim() == ":q" {
                    println!("Goodbye!");
                    break;
                }

                if line.trim() == ":reset" {
                    repl.reset();
                    println!("REPL state reset");
                    continue;
                }

                if line.trim() == ":help" || line.trim() == ":h" {
                    print_help();
                    continue;
                }

                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(&line);

                // Evaluate the input
                let result = repl.eval_line(&line);

                // Display diagnostics
                if !result.diagnostics.is_empty() {
                    for diag in &result.diagnostics {
                        println!("{}", format_diagnostic(diag, &line));
                    }
                }

                // Display value (if expression with non-null result)
                if result.diagnostics.is_empty() {
                    if let Some(value) = result.value {
                        // Don't print null values
                        if !matches!(value, atlas_runtime::Value::Null) {
                            println!("{}", value.to_string());
                        }
                    }
                }

                // Display stdout (if any was captured)
                if !result.stdout.is_empty() {
                    print!("{}", result.stdout);
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C
                println!("^C");
                println!("Use :quit or :q to exit");
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D
                println!("^D");
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

/// Print help information
fn print_help() {
    println!("Atlas REPL Commands:");
    println!("  :quit, :q     Exit the REPL");
    println!("  :reset        Clear all variables and functions");
    println!("  :help, :h     Show this help message");
    println!();
    println!("Type any Atlas expression or statement to evaluate it.");
    println!("Examples:");
    println!("  >> 1 + 2;");
    println!("  >> let x = 42;");
    println!("  >> fn double(n: number) -> number {{ return n * 2; }}");
    println!("  >> double(x);");
}

/// Format a diagnostic for display
fn format_diagnostic(diag: &atlas_runtime::Diagnostic, _source: &str) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    // Basic formatting - just show the message
    // In a more complete implementation, we would:
    // - Show the span location in the input
    // - Underline the problematic code
    // - Show related locations
    format!("{}: {}", level_str, diag.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_diagnostic() {
        use atlas_runtime::{Diagnostic, Span};

        let diag = Diagnostic::error("Test error".to_string(), Span::dummy());
        let formatted = format_diagnostic(&diag, "test code");
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }
}
