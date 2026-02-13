//! Run command - execute Atlas source files

use anyhow::{Context, Result};
use atlas_runtime::Atlas;
use std::fs;

/// Run an Atlas source file
///
/// Compiles and executes the source file, printing the result to stdout.
pub fn run(file_path: &str) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Create runtime and evaluate
    let runtime = Atlas::new();
    match runtime.eval(&source) {
        Ok(value) => {
            // Print the result value if it's not null
            if !matches!(value, atlas_runtime::Value::Null) {
                println!("{}", value.to_string());
            }
            Ok(())
        }
        Err(diagnostics) => {
            // Print all diagnostics
            eprintln!("Errors occurred while running {}:", file_path);
            for diag in &diagnostics {
                eprintln!("{}", format_diagnostic(diag, &source));
            }
            Err(anyhow::anyhow!("Failed to execute program"))
        }
    }
}

/// Format a diagnostic for display
fn format_diagnostic(diag: &atlas_runtime::Diagnostic, _source: &str) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    // Format: line:col: level: message
    format!(
        "{}:{}: {}: {}",
        diag.line, diag.column, level_str, diag.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_runtime::{Diagnostic, Span};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_run_simple_expression() {
        // Create a temporary file with Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1 + 2;").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_missing_file() {
        let result = run("nonexistent.atl");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_diagnostic() {
        let source = "let x = 42;";
        let diag = Diagnostic::error("Test error".to_string(), Span::new(0, 3));
        let formatted = format_diagnostic(&diag, source);
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error"));
    }
}
