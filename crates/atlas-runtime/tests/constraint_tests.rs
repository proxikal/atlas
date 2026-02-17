use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;
use rstest::rstest;

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, mut parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, mut bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let mut check_diags = checker.check(&program);

    parse_diags.append(&mut bind_diags);
    parse_diags.append(&mut check_diags);
    parse_diags
}

fn has_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diag| diag.level == DiagnosticLevel::Error)
}

fn has_error_code(diagnostics: &[Diagnostic], code: &str) -> bool {
    diagnostics
        .iter()
        .any(|diag| diag.level == DiagnosticLevel::Error && diag.code == code)
}

fn assert_no_errors(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        !has_error(&diagnostics),
        "Expected no errors, got: {:?}",
        diagnostics
    );
}

fn assert_has_error(source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error(&diagnostics),
        "Expected errors, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Constraint syntax tests
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; }")]
#[case("fn f<T extends number & number>(x: T) -> T { return x; }")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; }")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> T { return x; }")]
#[case("type Box<T extends number> = T;")]
#[case("type Box<T extends number | string> = T;")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; }")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); }")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; }")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; }")]
fn test_constraint_syntax_valid(#[case] source: &str) {
    assert_no_errors(source);
}

#[rstest]
#[case("fn f<T extends>(x: T) -> T { return x; }")]
#[case("fn f<T extends number,>(x: T) -> T { return x; }")]
#[case("fn f<T extends {>(x: T) -> T { return x; }")]
#[case("fn f<T extends number>(x: T) -> T { return x }")]
#[case("type Box<T extends> = T;")]
#[case("type Box<T extends number,> = T;")]
#[case("fn f<T extends { as_string () -> string }>(x: T) -> T { return x; }")]
#[case(
    "fn f<T extends { as_string: () -> string, as_number: () -> number }(x: T) -> T { return x; }"
)]
fn test_constraint_syntax_invalid(#[case] source: &str) {
    assert_has_error(source);
}

// -----------------------------------------------------------------------------
// Constraint checking (success)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number & number>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(1);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1, 2]);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("type Box<T extends number> = T; let x: Box<number> = 1;")]
#[case("type Box<T extends number | string> = T; let x: Box<string> = \"hi\";")]
fn test_constraint_checking_success(#[case] source: &str) {
    assert_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint checking (failure)
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(true);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f([1, 2]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y = f(1);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends number & string>(x: T) -> T { return x; } let y = f(1);")]
#[case("type Box<T extends number & string> = T;")]
#[case("fn f<T extends UnknownConstraint>(x: T) -> T { return x; }")]
fn test_constraint_checking_failure(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error(&diagnostics),
        "Expected errors, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Multiple constraints and normalization
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number & Serializable>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Serializable & Equatable>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable & Equatable>(x: T) -> T { return x; } let y = f(false);")]
#[case("fn f<T extends number & Comparable>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Iterable & Serializable>(x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T extends number & number & number>(x: T) -> T { return x; } let y = f(1);")]
#[case(
    "fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(\"a\");"
)]
#[case("fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(1);")]
#[case(
    "fn f<T extends (number | string) & Serializable>(x: T) -> T { return x; } let y = f(true);"
)]
#[case("fn f<T extends (number | string) & Equatable>(x: T) -> T { return x; } let y = f(\"a\");")]
fn test_multiple_constraints(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    if source.contains("true") || source.contains("Iterable & Serializable") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else {
        assert!(
            !has_error(&diagnostics),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }
}

// -----------------------------------------------------------------------------
// Constraint inference success
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(3);")]
#[case("fn f<T extends number | string>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(99);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1, 2, 3]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(false);")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(42);")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends { value: json }>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("type Box<T extends number> = T; fn f<T extends number>(x: T) -> Box<T> { return x; } let y = f(1);")]
#[case("type Box<T extends Serializable> = T; fn f<T extends Serializable>(x: T) -> Box<T> { return x; } let y = f(\"a\");")]
fn test_constraint_inference_success(#[case] source: &str) {
    assert_no_errors(source);
}

// -----------------------------------------------------------------------------
// Constraint inference failure
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends number>(x: T) -> T { return x; } let y = f(true);")]
#[case("fn f<T extends number>(x: T, y: T) -> T { return x; } let z = f(1, \"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> T { return x; } let y = f([1]);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(\"a\");")]
#[case("fn f<T extends Equatable>(x: T) -> T { return x; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends Comparable>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends number>() -> T { return 1; } let y = f();")]
#[case("fn f<T extends { as_string: () -> string }>(x: T) -> string { return x.as_string(); } let y = f(1);")]
fn test_constraint_inference_failure(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert!(
        has_error_code(&diagnostics, "AT3001") || has_error_code(&diagnostics, "AT9999"),
        "Expected AT3001/AT9999, got: {:?}",
        diagnostics
    );
}

// -----------------------------------------------------------------------------
// Practical constraint patterns
// -----------------------------------------------------------------------------

#[rstest]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(1);")]
#[case("fn f<T extends Numeric>(x: T) -> T { return x; } let y = f(1);")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f([1]);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f(true);")]
#[case("fn f<T extends Comparable>(x: T) -> bool { return true; } let y = f(\"a\");")]
#[case("fn f<T extends Numeric>(x: T) -> T { return x; } let y = f(\"a\");")]
#[case("fn f<T extends Iterable>(x: T) -> number { return 0; } let y = f(1);")]
#[case("fn f<T extends Equatable>(x: T) -> bool { return true; } let y: json = parseJSON(\"{}\"); let z = f(y);")]
#[case("fn f<T extends Serializable>(x: T) -> string { return str(x); } let y = f([1]);")]
fn test_practical_constraint_patterns(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    if source.contains("\"a\"") && source.contains("Numeric") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else if source.contains("Iterable") && source.contains("= f(1)") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else if source.contains("Comparable") && source.contains("\"a\"") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else if source.contains("Equatable") && source.contains("json") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else if source.contains("Serializable") && source.contains("[1]") {
        assert!(
            has_error(&diagnostics),
            "Expected errors, got: {:?}",
            diagnostics
        );
    } else {
        assert!(
            !has_error(&diagnostics),
            "Expected no errors, got: {:?}",
            diagnostics
        );
    }
}
