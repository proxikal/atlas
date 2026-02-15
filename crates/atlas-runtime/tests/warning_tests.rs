//! Modern Warning Tests
//!
//! Converted from warning_tests.rs (144 lines → ~95 lines = 34% reduction)

mod common;

use atlas_runtime::{Binder, DiagnosticLevel, Lexer, Parser, TypeChecker};

fn get_all_diagnostics(source: &str) -> Vec<atlas_runtime::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);
    all_diags
}

// ============================================================================
// Unused Variable Warnings (AT2001)
// ============================================================================

#[test]
fn test_unused_variable_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2001 warning");
    assert!(warnings[0].message.contains("Unused variable 'x'"));
}

#[test]
fn test_used_variable_no_warning() {
    let source = r#"fn main() -> number { let x: number = 42; return x; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 0, "Expected no AT2001 warnings");
}

#[test]
fn test_underscore_prefix_suppresses_warning() {
    let source = r#"fn main() -> number { let _unused: number = 42; return 5; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Underscore prefix should suppress warnings"
    );
}

#[test]
fn test_multiple_unused_variables() {
    let source = r#"fn main() -> number {
        let x: number = 1;
        let y: number = 2;
        let z: number = 3;
        return 0;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(warnings.len(), 3, "Expected 3 AT2001 warnings");
}

// ============================================================================
// Unused Parameter Warnings
// ============================================================================

#[test]
fn test_unused_parameter_warning() {
    let source = r#"fn add(a: number, b: number) -> number { return a; }"#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        1,
        "Expected 1 AT2001 warning for unused param"
    );
    assert!(warnings[0].message.contains("Unused parameter 'b'"));
}

#[test]
fn test_used_parameter_in_callback_no_warning() {
    // Bug reproduction: parameter is used in function body, but function is passed as callback
    let source = r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let result: number[] = map([1,2,3], double);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'x' is used in function body - should not warn even when function is passed as callback"
    );
}

#[test]
fn test_used_parameters_in_sort_callback_no_warning() {
    // Bug reproduction: both parameters are used, function passed to sort
    let source = r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let sorted: number[] = sort([3,1,2], compare);
    "#;
    let diags = get_all_diagnostics(source);

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameters 'a' and 'b' are used in function body - should not warn when function is passed to sort"
    );
}

#[test]
fn test_minimal_callback_parameter_usage() {
    // Minimal reproduction: parameter used in intrinsic function call
    let source = r#"
        fn numToStr(n: number) -> string {
            return toString(n);
        }
        let x: string = numToStr(5);
    "#;

    let diags = get_all_diagnostics(source);

    // Debug output
    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in toString call - should not warn"
    );
}

#[test]
fn test_parameter_used_in_user_function_call() {
    // Control test: parameter used in regular user function call
    let source = r#"
        fn helper(x: number) -> number {
            return x + 1;
        }
        fn wrapper(n: number) -> number {
            return helper(n);
        }
        let x: number = wrapper(5);
    "#;

    let diags = get_all_diagnostics(source);

    for diag in &diags {
        eprintln!("{:?}: {} (code: {})", diag.level, diag.message, diag.code);
    }

    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2001").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Parameter 'n' is used in helper call - should not warn"
    );
}

// ============================================================================
// Unreachable Code Warnings (AT2002)
// ============================================================================

#[test]
fn test_unreachable_code_after_return() {
    let source = r#"fn main() -> number {
        return 42;
        let x: number = 10;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(warnings.len(), 1, "Expected 1 AT2002 warning");
    assert!(warnings[0].message.contains("Unreachable code"));
}

#[test]
fn test_no_unreachable_warning_without_return() {
    let source = r#"fn main() -> number {
        let x: number = 42;
        let y: number = 10;
        return x;
    }"#;

    let diags = get_all_diagnostics(source);
    let warnings: Vec<_> = diags.iter().filter(|d| d.code == "AT2002").collect();
    assert_eq!(
        warnings.len(),
        0,
        "Should not have unreachable code warning"
    );
}

// ============================================================================
// Warnings Combined with Errors
// ============================================================================

#[test]
fn test_warnings_with_errors() {
    let source = r#"fn main() -> number { let x: number = "bad"; return 5; }"#;
    let diags = get_all_diagnostics(source);

    // Should have both error (type mismatch) and warning (unused variable)
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();

    assert!(!errors.is_empty(), "Expected type error");
    assert!(!warnings.is_empty(), "Expected unused warning");
}
