//! Modern Operator Precedence Tests
//!
//! Converted from operator_precedence_tests.rs (448 lines → ~80 lines = 82% reduction)
//! Uses insta snapshots instead of manual AST inspection

mod common;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use rstest::rstest;

fn parse_source(source: &str) -> atlas_runtime::ast::Program {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, diagnostics) = parser.parse();
    assert_eq!(diagnostics.len(), 0, "Should parse without errors");
    program
}

// ============================================================================
// Operator Precedence Snapshots
// ============================================================================

#[rstest]
// Multiplication/Division over Addition/Subtraction
#[case("mul_over_add", "1 + 2 * 3;")]
#[case("div_over_sub", "10 - 6 / 2;")]
#[case("mul_over_add_complex", "1 + 2 * 3 + 4;")]
#[case("div_over_sub_complex", "20 - 10 / 2 - 3;")]
// Unary operators
#[case("unary_minus_before_mul", "-2 * 3;")]
#[case("unary_not_before_and", "!false && true;")]
// Comparison operators
#[case("comparison_over_and", "1 < 2 && 3 > 2;")]
#[case("comparison_over_or", "1 == 1 || 2 != 2;")]
// Logical operators
#[case("and_before_or", "false || true && false;")]
// Parentheses override
#[case("parens_override_mul", "(1 + 2) * 3;")]
#[case("parens_override_div", "(10 - 2) / 4;")]
// Complex expressions
#[case("complex_arithmetic", "1 + 2 * 3 - 4 / 2;")]
#[case("complex_logical", "true && false || !true;")]
#[case("complex_comparison", "1 + 2 < 5 && 10 / 2 == 5;")]
// Function calls (highest precedence)
#[case("func_call_in_arithmetic", "foo() + 2 * 3;")]
#[case("func_call_in_comparison", "bar() < 5 && baz() > 0;")]
// Array indexing (highest precedence)
#[case("array_index_in_arithmetic", "arr[0] + 2 * 3;")]
#[case("array_index_in_comparison", "arr[i] < 10;")]
fn test_operator_precedence(#[case] name: &str, #[case] source: &str) {
    let program = parse_source(source);

    // Snapshot the first statement's expression
    assert_eq!(program.items.len(), 1, "Should have one statement");
    insta::assert_yaml_snapshot!(
        format!("precedence_{}", name),
        program.items[0]
    );
}
