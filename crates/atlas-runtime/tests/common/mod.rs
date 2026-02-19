//! Shared test utilities following Rust best practices
//!
//! This module provides common helpers for Atlas tests to reduce boilerplate
//! and make tests more readable and maintainable.

#![allow(dead_code)]

use atlas_runtime::diagnostic::Diagnostic;
use atlas_runtime::security::SecurityContext;
use atlas_runtime::{Atlas, Value};
use std::fs;
use std::path::{Path, PathBuf};

// Re-export testing utilities
pub use pretty_assertions::assert_eq;

/// Assert that source code evaluates to a number
///
/// # Example
/// ```
/// assert_eval_number("1 + 2", 3.0);
/// ```
pub fn assert_eval_number(source: &str, expected: f64) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::Number(n)) => assert_eq!(n, expected, "Expected {}, got {}", expected, n),
        other => panic!("Expected Number({}), got {:?}", expected, other),
    }
}

/// Assert that source code evaluates to a string
///
/// # Example
/// ```
/// assert_eval_string(r#""hello""#, "hello");
/// ```
pub fn assert_eval_string(source: &str, expected: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::String(s)) => assert_eq!(
            s.as_ref(),
            expected,
            "Expected {:?}, got {:?}",
            expected,
            s.as_ref()
        ),
        other => panic!("Expected String({:?}), got {:?}", expected, other),
    }
}

/// Assert that source code evaluates to a boolean
///
/// # Example
/// ```
/// assert_eval_bool("true && false", false);
/// ```
pub fn assert_eval_bool(source: &str, expected: bool) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::Bool(b)) => assert_eq!(b, expected, "Expected {}, got {}", expected, b),
        other => panic!("Expected Bool({}), got {:?}", expected, other),
    }
}

/// Assert that source code evaluates to null
///
/// # Example
/// ```
/// assert_eval_null("null");
/// ```
pub fn assert_eval_null(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(Value::Null) => {}
        other => panic!("Expected Null, got {:?}", other),
    }
}

/// Assert that source code produces an error with a specific code
///
/// # Example
/// ```
/// assert_error_code("let x: number = \"hello\";", "AT0001");
/// ```
pub fn assert_error_code(source: &str, expected_code: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            assert!(!diags.is_empty(), "Expected error, got success");
            assert_eq!(
                diags[0].code, expected_code,
                "Expected error code {}, got {}",
                expected_code, diags[0].code
            );
        }
        Ok(val) => panic!("Expected error {}, got success: {:?}", expected_code, val),
    }
}

/// Assert that source code produces at least one error (any code)
///
/// # Example
/// ```
/// assert_has_error("undefined_variable");
/// ```
pub fn assert_has_error(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Err(diags) => {
            assert!(!diags.is_empty(), "Expected error, got empty diagnostics");
        }
        Ok(val) => panic!("Expected error, got success: {:?}", val),
    }
}

/// Assert that source code evaluates successfully (no errors)
///
/// # Example
/// ```
/// assert_no_error("let x = 1;");
/// ```
pub fn assert_no_error(source: &str) {
    let runtime = Atlas::new();
    match runtime.eval(source) {
        Ok(_) => {}
        Err(diags) => panic!("Expected success, got errors: {:?}", diags),
    }
}

/// Parse source and return diagnostics (for testing parser)
///
/// # Example
/// ```
/// let diags = parse_and_get_diagnostics("let x =");
/// assert!(!diags.is_empty());
/// ```
pub fn parse_and_get_diagnostics(source: &str) -> Vec<Diagnostic> {
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;

    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();

    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (_program, parse_diags) = parser.parse();
    parse_diags
}

/// Compile source and return bytecode (for testing compiler)
pub fn compile_source(source: &str) -> Result<atlas_runtime::bytecode::Bytecode, Vec<Diagnostic>> {
    use atlas_runtime::compiler::Compiler;
    use atlas_runtime::lexer::Lexer;
    use atlas_runtime::parser::Parser;

    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    if !lex_diags.is_empty() {
        return Err(lex_diags);
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return Err(parse_diags);
    }

    let mut compiler = Compiler::new();
    compiler.compile(&program)
}

/// Run bytecode in VM and return result
pub fn run_bytecode(
    bytecode: atlas_runtime::bytecode::Bytecode,
) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    use atlas_runtime::vm::VM;
    use atlas_runtime::SecurityContext;
    let mut vm = VM::new(bytecode);
    vm.run(&SecurityContext::allow_all())
}

// ============================================================
// Shared helpers extracted from test files (phases 03d-03m)
// ============================================================

/// Evaluate Atlas code, return Value. Panics on error.
pub fn eval_ok(code: &str) -> Value {
    let runtime = Atlas::new_with_security(SecurityContext::allow_all());
    runtime.eval(code).unwrap()
}

/// Extract f64 from a Value::Number. Panics if wrong type.
pub fn extract_number(value: &Value) -> f64 {
    match value {
        Value::Number(n) => *n,
        _ => panic!("Expected number value, got {:?}", value),
    }
}

/// Extract bool from a Value::Bool. Panics if wrong type.
pub fn extract_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => panic!("Expected bool value, got {:?}", value),
    }
}

/// Wrap a &str into a Value::String.
pub fn str_value(s: &str) -> Value {
    Value::string(s.to_string())
}

/// Wrap a slice of &str into a Value::Array of Value::String.
pub fn str_array_value(paths: &[&str]) -> Value {
    let values: Vec<Value> = paths.iter().map(|p| str_value(p)).collect();
    Value::array(values)
}

/// Return a SecurityContext with all permissions allowed.
pub fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

/// Create a file in `dir` with `name` and `content`.
pub fn create_test_file(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(path, content).unwrap();
}

/// Create a subdirectory `name` inside `dir`. Returns the new path.
pub fn create_test_dir(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::create_dir(&path).unwrap();
    path
}

/// Normalize a path for embedding in Atlas source code.
///
/// Converts backslashes to forward slashes to avoid escape sequence issues
/// on Windows. Atlas's file I/O functions accept forward slashes on all platforms.
///
/// # Example
/// ```
/// let path = PathBuf::from(r"C:\Users\test\file.txt");
/// let atlas_path = path_for_atlas(&path);
/// assert_eq!(atlas_path, "C:/Users/test/file.txt");
/// ```
pub fn path_for_atlas(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Create a temporary file path for use in tests.
///
/// Returns a tuple of (TempDir, path_string) where path_string is normalized
/// for Atlas code (forward slashes). The TempDir must be kept alive for the
/// duration of the test.
///
/// # Example
/// ```
/// let (temp, path) = temp_file_path("test.json");
/// let code = format!(r#"writeFile("{}", "content")"#, path);
/// ```
pub fn temp_file_path(filename: &str) -> (tempfile::TempDir, String) {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let path = temp_dir.path().join(filename);
    let path_str = path_for_atlas(&path);
    (temp_dir, path_str)
}

/// Helper to create a snapshot name from test function name
///
/// Use with insta: `insta::assert_yaml_snapshot!(snapshot_name!(), value);`
#[macro_export]
macro_rules! snapshot_name {
    () => {{
        // Get the function name from the call stack
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f")
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or("unknown")
    }};
}
