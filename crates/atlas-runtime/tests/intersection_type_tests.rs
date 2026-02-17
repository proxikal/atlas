//! Tests for intersection types (Phase typing-04)

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
// Intersection construction tests
// ============================================================================

#[rstest]
#[case("let _x: number & number = 1;")]
#[case("let _x: string & string = \"ok\";")]
#[case("let _x: bool & bool = true;")]
#[case("let _x: number[] & number[] = [1, 2];")]
#[case("type Same = number & number; let _x: Same = 1;")]
#[case("fn f(x: number) -> number { return x; } let _x: ((number) -> number) & ((number) -> number) = f;")]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string) & number = 2;")]
#[case("let _x: (number | string) & string = \"hi\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number & number)[] = [1];")]
#[case("type Id<T> = T & T; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (number | string) = \"ok\";")]
#[case("let _x: (number | string) & (number | string) = 2;")]
fn test_intersection_construction(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

// ============================================================================
// Intersection error tests
// ============================================================================

#[rstest]
#[case("let _x: number & string = 1;")]
#[case("let _x: number & string = \"ok\";")]
#[case("let _x: bool & number = true;")]
#[case("let _x: string & null = \"ok\";")]
#[case("let _x: null & string = null;")]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string) & string = 1;")]
#[case("let _x: (bool | string) & number = 1;")]
#[case(
    "fn f(x: number) -> number { return x; } let _x: (number) -> number & (string) -> string = f;"
)]
#[case("let _x: number & string & bool = 1;")]
#[case("type Id<T> = T & string; let _x: Id<number> = 1;")]
#[case("let _x: (number | string) & (bool | string) = 1;")]
#[case("let _x: (number | string) & (bool | string) = true;")]
#[case("let _x: (number | string) & (bool | string) = null;")]
fn test_intersection_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Union/intersection interaction tests
// ============================================================================

#[rstest]
#[case("let _x: (number | string) & number = 1;")]
#[case("let _x: (number | string | bool) & number = 1;")]
#[case("let _x: (number | string) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & bool = true;")]
#[case("let _x: (number | string | bool) & string = \"ok\";")]
#[case("let _x: (number | string | bool) & number = 2;")]
#[case("let _x: (number | string) & (bool | string) = \"ok\";")]
#[case("let _x: (number | string) & (bool | string) = \"yes\";")]
fn test_intersection_distribution(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}

#[rstest]
#[case("let _x: (number | string) & number = \"bad\";")]
#[case("let _x: (number | string | bool) & string = 10;")]
#[case("let _x: (number | string | bool) & bool = \"no\";")]
#[case("let _x: (number | string | bool) & number = false;")]
fn test_intersection_distribution_errors(#[case] source: &str) {
    let diags = errors(source);
    assert!(!diags.is_empty(), "Expected errors, got none");
}

// ============================================================================
// Intersection + method/index operations
// ============================================================================

#[rstest]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[1];")]
#[case("let _x: number[] & number[] = [1, 2]; let _y: number = _x[0] + _x[1];")]
fn test_intersection_operations(#[case] source: &str) {
    let diags = errors(source);
    assert!(diags.is_empty(), "Expected no errors, got: {:?}", diags);
}
