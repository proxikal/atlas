//! Modern Assignment Target Tests
//!
//! Converted from assignment_target_tests.rs (457 lines → ~100 lines = 78% reduction)
//! Uses insta snapshots instead of manual AST inspection

mod common;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use rstest::rstest;

fn parse_source(source: &str) -> (atlas_runtime::ast::Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Assignment Target Validation with Snapshots
// ============================================================================

#[rstest]
#[case("simple_name", "x = 42;")]
#[case("longer_name", "myVariable = 100;")]
#[case("expression_value", "x = 1 + 2 * 3;")]
#[case("array_index", "arr[0] = 42;")]
#[case("array_expression_index", "arr[i + 1] = 99;")]
#[case("nested_array", "matrix[i][j] = 5;")]
#[case("string_value", r#"name = "Alice";"#)]
#[case("boolean_value", "flag = true;")]
#[case("null_value", "value = null;")]
#[case("function_call_value", "result = foo();")]
#[case("array_literal_value", "items = [1, 2, 3];")]
fn test_assignment_targets(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0, "Should parse without errors");
    assert_eq!(program.items.len(), 1, "Should have one statement");

    // Snapshot the AST to verify structure
    insta::assert_yaml_snapshot!(
        format!("assignment_{}", name),
        program.items[0]
    );
}

// ============================================================================
// Compound Assignment Operators
// ============================================================================

#[rstest]
#[case("add_assign", "x += 5;")]
#[case("sub_assign", "x -= 3;")]
#[case("mul_assign", "x *= 2;")]
#[case("div_assign", "x /= 4;")]
#[case("mod_assign", "x %= 10;")]
#[case("array_add_assign", "arr[i] += 1;")]
fn test_compound_assignments(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(
        format!("compound_{}", name),
        program.items[0]
    );
}

// ============================================================================
// Increment/Decrement Operators
// ============================================================================

#[rstest]
#[case("increment_name", "x++;")]
#[case("decrement_name", "x--;")]
#[case("increment_array", "arr[0]++;")]
#[case("decrement_array", "arr[i]--;")]
fn test_increment_decrement(#[case] name: &str, #[case] source: &str) {
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(
        format!("incdec_{}", name),
        program.items[0]
    );
}

// ============================================================================
// Multiple Assignments in Sequence
// ============================================================================

#[test]
fn test_multiple_assignments() {
    let source = "x = 1; y = 2; z = 3;";
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);

    insta::assert_yaml_snapshot!("multiple_assignments", program);
}

// ============================================================================
// Complex Assignment Expressions
// ============================================================================

#[test]
fn test_chained_array_access_assignment() {
    let source = "matrix[row][col] = value;";
    let (program, diagnostics) = parse_source(source);

    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!("chained_array_assignment", program.items[0]);
}
