//! Profile command — run an Atlas file with VM profiling enabled

use anyhow::{Context, Result};
use atlas_runtime::binder::Binder;
use atlas_runtime::compiler::Compiler;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::typechecker::TypeChecker;
use atlas_runtime::vm::VM;
use atlas_runtime::Value;
use std::path::PathBuf;

/// Arguments for the profile command
#[derive(Debug, Default)]
pub struct ProfileArgs {
    /// Path to the Atlas source file
    pub file: String,
    /// Hotspot detection threshold percentage (default 1.0)
    pub hotspot_threshold: f64,
    /// Save report to this file instead of printing to stdout
    pub output_file: Option<PathBuf>,
    /// Show full detailed report (true) or summary only (false)
    pub detailed: bool,
}

impl ProfileArgs {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            hotspot_threshold: 1.0,
            output_file: None,
            detailed: true,
        }
    }
}

/// Run an Atlas source file with profiling enabled
///
/// Compiles the source through the full pipeline (lex → parse → bind → check →
/// compile), then executes it in the VM with profiling enabled and prints a
/// performance report.
pub fn run(args: ProfileArgs) -> Result<()> {
    let source = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read file: {}", args.file))?;

    // --- Lex ---
    let mut lexer = Lexer::new(&source);
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(diagnostics_to_error(&args.file, lex_diags));
    }

    // --- Parse ---
    let mut parser = Parser::new(tokens);
    let (ast, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(diagnostics_to_error(&args.file, parse_diags));
    }

    // --- Bind ---
    let mut binder = Binder::new();
    let (mut symbol_table, bind_diags) = binder.bind(&ast);
    if !bind_diags.is_empty() {
        return Err(diagnostics_to_error(&args.file, bind_diags));
    }

    // --- Typecheck ---
    let mut checker = TypeChecker::new(&mut symbol_table);
    let type_diags = checker.check(&ast);
    if !type_diags.is_empty() {
        return Err(diagnostics_to_error(&args.file, type_diags));
    }

    // --- Compile ---
    let mut compiler = Compiler::with_optimization();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|diags| diagnostics_to_error(&args.file, diags))?;

    // --- Run with profiling ---
    let security = SecurityContext::allow_all();
    let mut vm = VM::with_profiling(bytecode);

    let result = vm
        .run(&security)
        .map_err(|e| anyhow::anyhow!("Runtime error: {:?}", e))?;

    // --- Show program result ---
    if let Some(ref val) = result {
        if !matches!(val, Value::Null) {
            println!("Result: {}", val);
        }
    }

    // --- Generate and display report ---
    let profiler = vm
        .profiler()
        .expect("profiler was enabled but is absent after run");

    let report = profiler.generate_report(args.hotspot_threshold);

    let report_text = if args.detailed {
        report.format_detailed()
    } else {
        format!("{}\n", report.format_summary())
    };

    match args.output_file {
        Some(ref path) => {
            std::fs::write(path.as_path(), &report_text)
                .with_context(|| format!("Failed to write report to {}", path.display()))?;
            println!("Profile report saved to {}", path.display());
        }
        None => {
            print!("{}", report_text);
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn diagnostics_to_error(file: &str, diags: Vec<atlas_runtime::Diagnostic>) -> anyhow::Error {
    let messages: Vec<String> = diags
        .iter()
        .map(|d| format!("{}:{}:{}: {}", file, d.line, d.column, d.message))
        .collect();
    anyhow::anyhow!("Compilation failed:\n{}", messages.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(code: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "{}", code).unwrap();
        f
    }

    #[test]
    fn test_profile_simple_expression() {
        let f = write_temp("let x: number = 1 + 2;");
        let args = ProfileArgs::new(f.path().to_str().unwrap());
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_profile_missing_file() {
        let args = ProfileArgs::new("nonexistent_file_that_does_not_exist.at");
        assert!(run(args).is_err());
    }

    #[test]
    fn test_profile_saves_to_file() {
        let src = write_temp("let x: number = 42;");
        let out = NamedTempFile::new().unwrap();
        let args = ProfileArgs {
            file: src.path().to_str().unwrap().to_string(),
            hotspot_threshold: 1.0,
            output_file: Some(out.path().to_path_buf()),
            detailed: true,
        };
        run(args).unwrap();
        let content = std::fs::read_to_string(out.path()).unwrap();
        assert!(
            content.contains("Atlas VM Profile Report"),
            "report content: {}",
            content
        );
    }

    #[test]
    fn test_profile_summary_only() {
        let f = write_temp("let y: number = 5 * 3;");
        let args = ProfileArgs {
            file: f.path().to_str().unwrap().to_string(),
            hotspot_threshold: 1.0,
            output_file: None,
            detailed: false,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_profile_with_loop() {
        let src =
            "let sum: number = 0; let i: number = 0; while i < 10 { sum = sum + i; i = i + 1; }";
        let f = write_temp(src);
        let args = ProfileArgs::new(f.path().to_str().unwrap());
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_profile_custom_threshold() {
        let f = write_temp("let z: number = 1 + 2 + 3;");
        let args = ProfileArgs {
            file: f.path().to_str().unwrap().to_string(),
            hotspot_threshold: 50.0,
            output_file: None,
            detailed: true,
        };
        assert!(run(args).is_ok());
    }

    #[test]
    fn test_profile_syntax_error() {
        let f = write_temp("let x: number = ;");
        let args = ProfileArgs::new(f.path().to_str().unwrap());
        assert!(run(args).is_err());
    }
}
