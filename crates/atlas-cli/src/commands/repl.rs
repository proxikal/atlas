//! REPL command implementation

use anyhow::Result;
use atlas_runtime::{ReplBinding, ReplCore, Type, Value};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Run the interactive REPL
///
/// If `use_tui` is true, uses ratatui TUI mode.
/// Otherwise, uses rustyline line-editor mode (default).
/// If `no_history` is true, disables history persistence.
pub fn run(use_tui: bool, no_history: bool, config: &crate::config::Config) -> Result<()> {
    if use_tui {
        // Use TUI mode (ratatui)
        return super::repl_tui::run();
    }

    // Use line-editor mode (rustyline) - default
    let mut rl = DefaultEditor::new()?;
    let mut repl = ReplCore::new();

    // Load history from file (unless disabled)
    let history_path = config.get_history_path();
    if !no_history {
        if let Some(ref path) = history_path {
            let _ = rl.load_history(path); // Ignore errors if file doesn't exist
        }
    }

    // Display welcome message
    println!("Atlas v{} REPL", atlas_runtime::VERSION);
    println!("Type expressions or statements, or :quit to exit");
    println!("Commands: :quit (or :q), :reset, :help, :type <expr>, :vars [page]");
    println!();

    loop {
        // Read a line
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle REPL commands
                if trimmed == ":quit" || trimmed == ":q" {
                    println!("Goodbye!");
                    break;
                }

                if trimmed == ":reset" {
                    repl.reset();
                    println!("REPL state reset");
                    continue;
                }

                if trimmed == ":help" || trimmed == ":h" {
                    print_help();
                    continue;
                }

                if let Some(expr) = trimmed.strip_prefix(":type").map(str::trim) {
                    if expr.is_empty() {
                        println!("Usage: :type <expression>");
                        continue;
                    }

                    let type_result = repl.type_of_expression(expr);
                    if !type_result.diagnostics.is_empty() {
                        for diag in &type_result.diagnostics {
                            println!("{}", format_diagnostic(diag, expr));
                        }
                    } else if let Some(ty) = type_result.ty {
                        println!("type: {}", format_type(&ty, config.no_color));
                    } else {
                        println!("type: unknown");
                    }
                    continue;
                }

                if trimmed.starts_with(":vars") || trimmed.starts_with(":v") {
                    let page = trimmed
                        .split_whitespace()
                        .nth(1)
                        .and_then(|p| p.parse::<usize>().ok())
                        .filter(|p| *p > 0)
                        .unwrap_or(1);
                    print_vars(&repl.variables(), page, config.no_color);
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
                            println!("{}", value);
                        }
                    }

                    // Automatic type display for bindings and expressions
                    if config.show_types {
                        for binding in &result.bindings {
                            println!(
                                "{}: {} = {}{}",
                                binding.name,
                                format_type(&binding.ty, config.no_color),
                                if binding.mutable { "(mut) " } else { "" },
                                binding.value
                            );
                        }

                        if result.bindings.is_empty() {
                            if let Some(ty) = &result.expr_type {
                                println!("type: {}", format_type(ty, config.no_color));
                            }
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

    // Save history to file (unless disabled)
    if !no_history {
        if let Some(path) = history_path {
            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = rl.save_history(&path); // Ignore errors
        }
    }

    Ok(())
}

/// Print help information
fn print_help() {
    println!("Atlas REPL Commands:");
    println!("  :quit, :q         Exit the REPL");
    println!("  :reset            Clear all variables and functions");
    println!("  :help, :h         Show this help message");
    println!("  :type <expr>      Show inferred type of an expression");
    println!("  :vars [page]      List variables with types and values");
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

fn format_type(ty: &Type, no_color: bool) -> String {
    let text = ty.display_name();
    if no_color {
        text
    } else {
        format!("\x1b[36m{}\x1b[0m", text)
    }
}

fn print_vars(bindings: &[ReplBinding], page: usize, no_color: bool) {
    if bindings.is_empty() {
        println!("No variables defined.");
        return;
    }

    let page_size = 20usize;
    let total_pages = ((bindings.len() + page_size - 1) / page_size).max(1);
    let current_page = page.min(total_pages);
    let start = (current_page - 1) * page_size;
    let end = (start + page_size).min(bindings.len());

    println!(
        "Variables (page {}/{}; showing {}-{} of {}):",
        current_page,
        total_pages,
        start + 1,
        end,
        bindings.len()
    );
    println!("{:<16} {:<18} {:<8} {}", "name", "type", "scope", "value");
    println!("{}", "-".repeat(60));

    for binding in &bindings[start..end] {
        println!(
            "{:<16} {:<18} {:<8} {}{}",
            binding.name,
            format_type(&binding.ty, no_color),
            "global",
            if binding.mutable { "(mut) " } else { "" },
            binding.value
        );
    }
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
