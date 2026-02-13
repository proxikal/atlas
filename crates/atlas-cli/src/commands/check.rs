//! Check command - type-check Atlas source files without executing

use anyhow::{Context, Result};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use std::fs;

/// Type-check an Atlas source file without executing it
///
/// Performs lexing, parsing, binding, and type-checking, reporting any errors.
pub fn run(file_path: &str) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Lex the source code
    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        print_diagnostics(&lex_diagnostics, &source, file_path);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        print_diagnostics(&parse_diagnostics, &source, file_path);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Bind symbols
    let mut binder = Binder::new();
    let (symbol_table, bind_diagnostics) = binder.bind(&ast);

    if !bind_diagnostics.is_empty() {
        print_diagnostics(&bind_diagnostics, &source, file_path);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Type check
    let mut typechecker = TypeChecker::new(&symbol_table);
    let typecheck_diagnostics = typechecker.check(&ast);

    if !typecheck_diagnostics.is_empty() {
        print_diagnostics(&typecheck_diagnostics, &source, file_path);
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    // Success!
    println!("{}: No errors found", file_path);
    Ok(())
}

/// Print diagnostics to stderr
fn print_diagnostics(diagnostics: &[atlas_runtime::Diagnostic], source: &str, file_path: &str) {
    for diag in diagnostics {
        eprintln!("{}", format_diagnostic(diag, source, file_path));
    }
}

/// Format a diagnostic for display
fn format_diagnostic(
    diag: &atlas_runtime::Diagnostic,
    _source: &str,
    file_path: &str,
) -> String {
    use atlas_runtime::DiagnosticLevel;

    let level_str = match diag.level {
        DiagnosticLevel::Error => "error",
        DiagnosticLevel::Warning => "warning",
    };

    // Format: filename:line:col: level: message
    format!(
        "{}:{}:{}: {}: {}",
        file_path, diag.line, diag.column, level_str, diag.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_check_valid_file() {
        // Create a temporary file with valid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = 42;").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_invalid_file() {
        // Create a temporary file with invalid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"string\";").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_check_missing_file() {
        let result = run("nonexistent.atl");
        assert!(result.is_err());
    }
}
