//! Typecheck dump command - output type information as JSON

use anyhow::{Context, Result};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use std::fs;

/// Dump typecheck information to JSON
///
/// Performs full type checking and outputs symbol and type information as JSON.
pub fn run(file_path: &str) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Lex the source code
    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        for diag in &lex_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Lexer errors"));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        for diag in &parse_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Parse errors"));
    }

    // Bind symbols
    let mut binder = Binder::new();
    let (symbol_table, bind_diagnostics) = binder.bind(&ast);

    if !bind_diagnostics.is_empty() {
        for diag in &bind_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Binding errors"));
    }

    // Type check
    let mut typechecker = TypeChecker::new(&symbol_table);
    let typecheck_diagnostics = typechecker.check(&ast);

    if !typecheck_diagnostics.is_empty() {
        for diag in &typecheck_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Type errors"));
    }

    // Create typecheck dump and output as JSON
    let dump = atlas_runtime::TypecheckDump::from_symbol_table(&symbol_table);
    let json = dump.to_json_string()?;
    println!("{}", json);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_typecheck_dump_simple() {
        // Create a temporary file with valid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = 42;").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_typecheck_dump_with_function() {
        // Create a temporary file with function
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "fn add(a: number, b: number) -> number {{ return a + b; }}"
        )
        .unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_typecheck_dump_type_error() {
        // Create a temporary file with type error
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = \"string\";").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_typecheck_dump_missing_file() {
        let result = run("nonexistent.atl");
        assert!(result.is_err());
    }
}
