//! Modern parser tests using insta snapshots
//!
//! Converted from manual AST assertions to snapshot testing
//! Original: 789 lines, 44 tests with manual matching
//! Modern: ~200 lines with automatic snapshot validation
//!
//! Run with: INSTA_UPDATE=always cargo test parser_tests
//! Review with: cargo insta review

mod common;
use atlas_runtime::ast::*;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use rstest::rstest;

fn parse_source(source: &str) -> (Program, Vec<atlas_runtime::diagnostic::Diagnostic>) {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, _) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Literal Expressions - Snapshot Testing
// ============================================================================

#[rstest]
#[case::number("42;", "number_literal")]
#[case::float("3.14;", "float_literal")]
#[case::string(r#""hello";"#, "string_literal")]
#[case::bool_true("true;", "bool_true")]
#[case::bool_false("false;", "bool_false")]
#[case::null("null;", "null_literal")]
fn test_parse_literals(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Variables and Identifiers
// ============================================================================

#[test]
fn test_parse_variable_reference() {
    let (program, diagnostics) = parse_source("x;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Binary Operators - Snapshot Testing
// ============================================================================

#[rstest]
#[case::add("1 + 2;", "addition")]
#[case::sub("5 - 3;", "subtraction")]
#[case::mul("3 * 4;", "multiplication")]
#[case::div("10 / 2;", "division")]
#[case::lt("1 < 2;", "less_than")]
#[case::le("1 <= 2;", "less_equal")]
#[case::gt("1 > 2;", "greater_than")]
#[case::ge("1 >= 2;", "greater_equal")]
#[case::eq("1 == 2;", "equality")]
#[case::ne("1 != 2;", "not_equal")]
#[case::and("true && false;", "logical_and")]
#[case::or("true || false;", "logical_or")]
fn test_parse_binary_operators(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Unary Operators
// ============================================================================

#[rstest]
#[case::negate("-5;", "negation")]
#[case::not("!true;", "logical_not")]
fn test_parse_unary_operators(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Grouping and Precedence
// ============================================================================

#[test]
fn test_parse_grouping() {
    let (program, diagnostics) = parse_source("(1 + 2) * 3;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_operator_precedence_multiplication_over_addition() {
    let (program, diagnostics) = parse_source("1 + 2 * 3;");
    assert_eq!(diagnostics.len(), 0);
    // Should parse as: 1 + (2 * 3), not (1 + 2) * 3
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_operator_precedence_comparison_over_logical() {
    let (program, diagnostics) = parse_source("1 < 2 && 3 > 4;");
    assert_eq!(diagnostics.len(), 0);
    // Should parse as: (1 < 2) && (3 > 4)
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Array Literals and Indexing
// ============================================================================

#[rstest]
#[case::empty("[];", "empty_array")]
#[case::with_elements("[1, 2, 3];", "array_with_elements")]
#[case::array_index("arr[0];", "array_index")]
fn test_parse_arrays(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Function Calls
// ============================================================================

#[rstest]
#[case::no_args("foo();", "function_call_no_args")]
#[case::with_args("foo(1, 2, 3);", "function_call_with_args")]
fn test_parse_function_calls(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Variable Declarations
// ============================================================================

#[rstest]
#[case::let_decl("let x = 42;", "let_declaration")]
#[case::var_decl("var x = 42;", "var_declaration")]
#[case::with_type("let x: number = 42;", "var_declaration_with_type")]
fn test_parse_var_declarations(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Assignment Statements
// ============================================================================

#[rstest]
#[case::simple("x = 42;", "simple_assignment")]
#[case::array_element("arr[0] = 42;", "array_element_assignment")]
fn test_parse_assignments(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Control Flow Statements
// ============================================================================

#[rstest]
#[case::if_stmt("if (true) { x; }", "if_statement")]
#[case::if_else("if (true) { x; } else { y; }", "if_else_statement")]
#[case::while_loop("while (true) { x; }", "while_loop")]
#[case::for_loop("for (let i = 0; i < 10; i = i + 1) { x; }", "for_loop")]
fn test_parse_control_flow(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Return, Break, Continue
// ============================================================================

#[rstest]
#[case::return_value("return 42;", "return_statement")]
#[case::return_void("return;", "return_no_value")]
#[case::break_stmt("break;", "break_statement")]
#[case::continue_stmt("continue;", "continue_statement")]
fn test_parse_flow_control_statements(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

// ============================================================================
// Block Statements
// ============================================================================

#[test]
fn test_parse_block_in_if() {
    let (program, diagnostics) = parse_source("if (true) { let x = 1; let y = 2; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_blocks() {
    let (program, diagnostics) = parse_source("if (true) { if (false) { let x = 1; } }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Function Declarations
// ============================================================================

#[test]
fn test_parse_function_no_params() {
    let (program, diagnostics) = parse_source("fn foo() { return 42; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_function_with_params() {
    let (program, diagnostics) =
        parse_source("fn add(x: number, y: number) -> number { return x + y; }");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_function_with_complex_body() {
    let source = r#"
fn factorial(n: number) -> number {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}
    "#;

    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Member Expressions (Method Calls)
// ============================================================================

#[rstest]
#[case::simple_method("obj.method();", "simple_method_call")]
#[case::method_with_one_arg("obj.method(x);", "method_with_one_arg")]
#[case::method_with_multiple_args("obj.method(a, b, c);", "method_with_multiple_args")]
#[case::json_as_string(r#"json["user"].as_string();"#, "json_extraction_as_string")]
#[case::json_as_number("data.as_number();", "json_as_number")]
fn test_parse_member_expressions(#[case] source: &str, #[case] snapshot_name: &str) {
    let (program, diagnostics) = parse_source(source);
    assert_eq!(diagnostics.len(), 0, "Expected no errors for: {}", source);
    insta::assert_yaml_snapshot!(snapshot_name, program);
}

#[test]
fn test_parse_chained_member_calls() {
    let (program, diagnostics) = parse_source("a.b().c();");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_after_index() {
    let (program, diagnostics) = parse_source("arr[0].method();");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_complex_member_chain() {
    let (program, diagnostics) = parse_source(r#"json["data"]["items"][0].as_string();"#);
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_in_expression() {
    let (program, diagnostics) = parse_source("let x = obj.method() + 5;");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_member_as_function_arg() {
    let (program, diagnostics) = parse_source("print(data.as_string());");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

#[test]
fn test_parse_nested_member_calls() {
    let (program, diagnostics) = parse_source("outer.method(inner.method());");
    assert_eq!(diagnostics.len(), 0);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Complex Programs
// ============================================================================

#[test]
fn test_parse_multiple_statements() {
    let (program, diagnostics) = parse_source("let x = 1; let y = 2; let z = x + y;");
    assert_eq!(diagnostics.len(), 0);
    assert_eq!(program.items.len(), 3);
    insta::assert_yaml_snapshot!(program);
}

// ============================================================================
// Error Recovery
// ============================================================================

#[test]
fn test_parse_error_recovery() {
    let (program, diagnostics) = parse_source("let x = ; let y = 2;");
    assert!(!diagnostics.is_empty(), "Expected syntax error");
    // Parser should recover and parse the second statement
    assert!(
        !program.items.is_empty(),
        "Expected at least one item after recovery"
    );
}

#[test]
fn test_parse_missing_semicolon_error() {
    let (_program, diagnostics) = parse_source("let x = 1 let y = 2;");
    assert!(
        !diagnostics.is_empty(),
        "Expected syntax error for missing semicolon"
    );
}
