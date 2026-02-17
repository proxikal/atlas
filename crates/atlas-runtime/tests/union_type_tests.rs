//! Tests for union types (Phase typing-04)

mod common;

use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::{Binder, Lexer, Parser, TypeChecker};
use rstest::rstest;

fn typecheck(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diags) = lexer.tokenize_with_comments();
    if !lex_diags.is_empty() {
        return lex_diags;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();
    if !parse_diags.is_empty() {
        return parse_diags;
    }

    let mut binder = Binder::new();
    let (mut table, mut bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let mut type_diags = checker.check(&program);

    bind_diags.append(&mut type_diags);
    bind_diags
}

fn errors(source: &str) -> Vec<Diagnostic> {
    typecheck(source)
        .into_iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect()
}

// ============================================================================
// Union construction tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = 1;")]
#[case("let _x: number | string = \"ok\";")]
#[case("let _x: number | string | bool = true;")]
#[case("let _x: (number | string)[] = [1, 2, 3];")]
#[case("let _x: (number | string)[] = [\"a\", \"b\"]; ")]
#[case("type Id = number | string; let _x: Id = 7;")]
#[case("type Id = number | string; let _x: Id = \"v\";")]
#[case("type Pair = (number, string) -> number | string; fn f(x: number, y: string) -> number { return x; } let _x: Pair = f;")]
#[case("fn f(x: bool) -> number | string { if (x) { return 1; } return \"a\"; }")]
#[case("let _x: number | number = 1;")]
fn test_union_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Union type checking tests
// ============================================================================

#[rstest]
#[case("let _x: number | string = true;")]
#[case("let _x: number | string = null;")]
#[case("fn f() -> number | string { return true; }")]
#[case("let _x: (number | string)[] = [1, \"bad\"]; ")]
#[case("let _x: number | string = 1; let _y: number = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: string = _x; let _z: number = _x;")]
fn test_union_type_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

#[rstest]
#[case("let _x: number | string = 1; let _y: number | string = _x;")]
#[case("let _x: number | string = \"ok\"; let _y: number | string = _x;")]
#[case("let _x: number | string | bool = true; let _y: number | string | bool = _x;")]
#[case("let _x: number | string = 1; let _y: number | string | bool = _x;")]
fn test_union_assignments(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Type narrowing tests
// ============================================================================

#[rstest]
#[case(
    "let x: number | string = 1; if (isString(x)) { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (isNumber(x)) { let _y: number = x; } else { let _z: string = x; }"
)]
#[case(
    "let x: number | null = null; if (x == null) { let _y: null = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: number | string = \"hi\"; if (typeof(x) == \"string\") { let _y: string = x; } else { let _z: number = x; }"
)]
#[case(
    "let x: bool | string = true; if (x == true) { let _y: bool = x; } else { let _z: string = x; }"
)]
#[case("let x: number | null = 1; if (x != null) { let _y: number = x; }")]
fn test_type_narrowing(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Match + union integration
// ============================================================================

#[rstest]
#[case(
    "let v: bool | Option<number> = Some(1); match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
#[case(
    "let v: Option<number> | Result<number, string> = Ok(1); match v { Some(x) => x, None => 0, Ok(y) => y, Err(_e) => 0 };"
)]
#[case(
    "let v: bool | Option<number> = true; match v { true => 1, false => 2, Some(x) => x, None => 0 };"
)]
fn test_union_match_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let v: bool | Option<number> = true; match v { true => 1, Some(x) => x };")]
#[case("let v: Option<number> | Result<number, string> = Ok(1); match v { Ok(y) => y };")]
fn test_union_match_non_exhaustive(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union operations tests
// ============================================================================

#[rstest]
#[case("let x: number | number = 1; let _y: number = x + 1;")]
#[case("let x: string | string = \"a\"; let _y: string = x + \"b\";")]
#[case("let x: number | string = 1; let _y = x + 1;")]
#[case("let x: number | string = \"a\"; let _y = x + \"b\";")]
#[case("let x: number | string = 1; if (x == 1) { let _y: number | string = x; }")]
#[case("let x: number[] | number[] = [1, 2]; let _y = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[0];")]
#[case("let x: number[] | number[] = [1, 2]; let _y: number = x[1];")]
fn test_union_operations(#[case] source: &str) {
    let diags = errors(source);
    if source.contains("number | string") && source.contains("x +") {
        assert!(!diags.is_empty(), "Expected errors, got none");
    } else {
        assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
    }
}
