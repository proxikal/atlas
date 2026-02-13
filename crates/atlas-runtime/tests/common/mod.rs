//! Shared test utilities following Rust best practices
//!
//! This module provides common helpers for Atlas tests to reduce boilerplate
//! and make tests more readable and maintainable.

use atlas_runtime::{Atlas, Value};
use atlas_runtime::diagnostics::Diagnostic;

// Re-export testing utilities
pub use pretty_assertions::{assert_eq, assert_ne};

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
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), expected, "Expected {:?}, got {:?}", expected, s.as_ref()),
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
        Ok(Value::Null) => {},
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
        Ok(_) => {},
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
pub fn run_bytecode(bytecode: atlas_runtime::bytecode::Bytecode) -> Result<Option<Value>, atlas_runtime::value::RuntimeError> {
    use atlas_runtime::vm::VM;
    let mut vm = VM::new(bytecode);
    vm.run()
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
        name.strip_suffix("::f").unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or("unknown")
    }};
}
