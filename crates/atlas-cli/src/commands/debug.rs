//! Debug command - interactive debugger for Atlas programs
//!
//! Provides a command-line debugger with breakpoints, stepping,
//! variable inspection, and expression evaluation.

use anyhow::Result;
use atlas_runtime::debugger::DebuggerSession;
use atlas_runtime::{Compiler, DiagnosticLevel, Lexer, Parser};
use std::fs;
use std::path::Path;

use crate::debugger::repl::DebugRepl;

/// Arguments for the debug command
#[derive(Debug, Clone)]
pub struct DebugArgs {
    /// Path to the Atlas source file
    pub file: String,
    /// Initial breakpoints (line numbers)
    pub breakpoints: Vec<u32>,
    /// Stop at entry point (reserved for future use)
    #[allow(dead_code)]
    pub stop_at_entry: bool,
}

impl Default for DebugArgs {
    fn default() -> Self {
        Self {
            file: String::new(),
            breakpoints: Vec::new(),
            stop_at_entry: true,
        }
    }
}

/// Run the debugger
pub fn run(args: DebugArgs) -> Result<()> {
    let path = Path::new(&args.file);

    // Read the source file
    let source = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read source file '{}': {}", args.file, e))?;

    // Compile to bytecode
    let (tokens, lexer_diags) = Lexer::new(&source).tokenize();

    // Check for lexer errors
    let has_lexer_errors = lexer_diags
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error);
    if has_lexer_errors {
        eprintln!("\x1b[31mLexer errors:\x1b[0m");
        for diag in lexer_diags
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
        {
            eprintln!("  {}:{}: {}", diag.line, diag.column, diag.message);
        }
        anyhow::bail!("Failed to lex source file");
    }

    let (ast, parser_diags) = Parser::new(tokens).parse();

    // Check for parser errors
    let has_parser_errors = parser_diags
        .iter()
        .any(|d| d.level == DiagnosticLevel::Error);
    if has_parser_errors {
        eprintln!("\x1b[31mParser errors:\x1b[0m");
        for diag in parser_diags
            .iter()
            .filter(|d| d.level == DiagnosticLevel::Error)
        {
            eprintln!("  {}:{}: {}", diag.line, diag.column, diag.message);
        }
        anyhow::bail!("Failed to parse source file");
    }

    // Compile to bytecode
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|diags| {
        for diag in &diags {
            eprintln!("  {}:{}: {}", diag.line, diag.column, diag.message);
        }
        anyhow::anyhow!("Failed to compile source file")
    })?;

    // Create debugger session
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&args.file);

    let session = DebuggerSession::new(bytecode, &source, file_name);

    // Create and run the debugger REPL
    let mut repl = DebugRepl::new(session, source, file_name.to_string());

    // Set initial breakpoints if specified
    for line in &args.breakpoints {
        repl.execute_command(&format!("break {}", line));
    }

    // Run the REPL
    repl.run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_debug_args_default() {
        let args = DebugArgs::default();
        assert!(args.file.is_empty());
        assert!(args.breakpoints.is_empty());
        assert!(args.stop_at_entry);
    }

    #[test]
    fn test_run_missing_file() {
        let args = DebugArgs {
            file: "nonexistent_file.atlas".to_string(),
            ..Default::default()
        };

        let result = run(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_syntax_error() {
        let file = create_test_file("let x = ;");
        let args = DebugArgs {
            file: file.path().to_str().unwrap().to_string(),
            ..Default::default()
        };

        let result = run(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_args_with_breakpoints() {
        let args = DebugArgs {
            file: "test.atlas".to_string(),
            breakpoints: vec![1, 5, 10],
            stop_at_entry: false,
        };

        assert_eq!(args.breakpoints.len(), 3);
        assert!(!args.stop_at_entry);
    }
}
