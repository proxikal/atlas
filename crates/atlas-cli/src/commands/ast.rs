//! AST dump command - output AST as JSON

use anyhow::{Context, Result};
use atlas_runtime::{Lexer, Parser};
use std::fs;

/// Dump AST to JSON
///
/// Parses the source file and outputs the AST as JSON to stdout.
pub fn run(file_path: &str) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Lex the source code
    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        // Print diagnostics as JSON
        for diag in &lex_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Lexer errors"));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        // Print diagnostics as JSON
        for diag in &parse_diagnostics {
            eprintln!("{}", diag.to_json_string().unwrap());
        }
        return Err(anyhow::anyhow!("Parse errors"));
    }

    // Convert to versioned AST and output as JSON
    let versioned = atlas_runtime::ast::VersionedProgram::new(ast);
    let json = versioned.to_json()?;
    println!("{}", json);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_ast_dump_simple() {
        // Create a temporary file with valid Atlas code
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number = 42;").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_ast_dump_invalid_syntax() {
        // Create a temporary file with invalid syntax
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "let x: number =").unwrap();

        let result = run(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_ast_dump_missing_file() {
        let result = run("nonexistent.atl");
        assert!(result.is_err());
    }
}
