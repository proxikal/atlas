//! Comprehensive tests for type system rules
//!
//! Tests cover:
//! - Arithmetic operators (+, -, *, /, %)
//! - Equality operators (==, !=)
//! - Comparison operators (<, <=, >, >=)
//! - Logical operators (&&, ||)
//! - Array literal typing and indexing
//! - String concatenation rules
//! - Array element assignment type rules

use atlas_runtime::binder::Binder;
use atlas_runtime::diagnostic::{Diagnostic, DiagnosticLevel};
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;
use rstest::rstest;

fn typecheck_source(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let (tokens, lex_diags) = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (program, parse_diags) = parser.parse();

    let mut binder = Binder::new();
    let (mut table, bind_diags) = binder.bind(&program);

    let mut checker = TypeChecker::new(&mut table);
    let type_diags = checker.check(&program);

    // Combine all diagnostics
    let mut all_diags = Vec::new();
    all_diags.extend(lex_diags);
    all_diags.extend(parse_diags);
    all_diags.extend(bind_diags);
    all_diags.extend(type_diags);
    all_diags
}

fn assert_no_errors(diagnostics: &[Diagnostic]) {
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

fn assert_has_error(diagnostics: &[Diagnostic], code: &str) {
    assert!(
        !diagnostics.is_empty(),
        "Expected at least one diagnostic with code {}",
        code
    );
    let found = diagnostics.iter().any(|d| d.code == code);
    assert!(
        found,
        "Expected diagnostic with code {}, got: {:?}",
        code,
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

// ========== Arithmetic Operators ==========

#[rstest]
#[case::add_numbers("let x = 5 + 3;", true)]
#[case::subtract_numbers("let x = 10 - 3;", true)]
#[case::multiply_numbers("let x = 5 * 3;", true)]
#[case::divide_numbers("let x = 10 / 2;", true)]
#[case::modulo_numbers("let x = 10 % 3;", true)]
#[case::arithmetic_chain("let x = 1 + 2 - 3 * 4 / 5 % 6;", true)]
#[case::complex_arithmetic("let x = (1 + 2) * (3 - 4) / (5 % 6);", true)]
#[case::arithmetic_with_vars("let a: number = 5; let b: number = 3; let c = a + b; let d = a - b; let e = a * b; let f = a / b; let g = a % b;", true)]
fn test_arithmetic_operations(#[case] source: &str, #[case] should_pass: bool) {
    let diagnostics = typecheck_source(source);
    if should_pass {
        assert_no_errors(&diagnostics);
    }
}

#[rstest]
#[case::add_number_string(r#"let x = 5 + "hello";"#)]
#[case::add_string_number(r#"let x = "hello" + 5;"#)]
#[case::add_bool_bool("let x = true + false;")]
#[case::subtract_strings(r#"let x = "hello" - "world";"#)]
#[case::subtract_number_bool("let x = 5 - true;")]
#[case::multiply_string(r#"let x = "hello" * 3;"#)]
#[case::divide_bools("let x = true / false;")]
#[case::modulo_string(r#"let x = 10 % "hello";"#)]
#[case::null_arithmetic("let x = null + null;")]
fn test_arithmetic_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== String Concatenation ==========

#[rstest]
#[case::concat_strings(r#"let x = "hello" + " world";"#)]
#[case::concat_chain(r#"let x = "hello" + " " + "world";"#)]
#[case::concat_variables(r#"let a: string = "hello"; let b: string = "world"; let c = a + b;"#)]
fn test_string_concatenation(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::string_number(r#"let x = "hello" + 123;"#)]
#[case::number_string(r#"let x = 123 + "hello";"#)]
#[case::string_bool(r#"let x = "hello" + true;"#)]
#[case::string_null(r#"let x = "hello" + null;"#)]
fn test_string_concatenation_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Equality Operators ==========

#[rstest]
#[case::numbers_equal("let x = 5 == 3;")]
#[case::strings_equal(r#"let x = "hello" == "world";"#)]
#[case::bools_equal("let x = true == false;")]
#[case::nulls_equal("let x = null == null;")]
#[case::numbers_not_equal("let x = 5 != 3;")]
#[case::strings_not_equal(r#"let x = "hello" != "world";"#)]
#[case::nulls_not_equal("let x = null != null;")]
#[case::arrays_equal("let x = [1, 2] == [1, 2];")]
fn test_equality_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::number_string(r#"let x = 5 == "hello";"#)]
#[case::number_bool("let x = 5 == true;")]
#[case::string_bool(r#"let x = "hello" == false;"#)]
#[case::null_number("let x = null == 5;")]
#[case::not_equal_types(r#"let x = 5 != "hello";"#)]
#[case::not_equal_bool_string(r#"let x = true != "false";"#)]
#[case::mixed_array_types(r#"let x = [1, 2] == ["a", "b"];"#)]
fn test_equality_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Comparison Operators ==========

#[rstest]
#[case::less_than("let x = 5 < 10;")]
#[case::less_than_equal("let x = 5 <= 10;")]
#[case::greater_than("let x = 10 > 5;")]
#[case::greater_than_equal("let x = 10 >= 5;")]
#[case::comparison_chain("let x = 1 < 2; let y = 3 > 2; let z = 5 >= 5; let w = 4 <= 10;")]
fn test_comparison_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::strings_less(r#"let x = "hello" < "world";"#)]
#[case::bools_less("let x = true < false;")]
#[case::mixed_less(r#"let x = 5 < "10";"#)]
#[case::strings_less_equal(r#"let x = "a" <= "b";"#)]
#[case::bools_greater("let x = true > false;")]
#[case::nulls_greater("let x = null > null;")]
#[case::mixed_greater_equal("let x = 5 >= true;")]
#[case::nulls_less("let x = null < null;")]
fn test_comparison_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Logical Operators ==========

#[rstest]
#[case::and_bools("let x = true && false;")]
#[case::or_bools("let x = true || false;")]
#[case::logical_chain("let x = true && false || true;")]
#[case::with_comparisons("let x = (5 < 10) && (3 > 1);")]
#[case::with_equality("let x = (5 == 5) || (3 != 3);")]
#[case::complex_boolean("let x = (5 < 10) && (3 > 1) || (2 == 2);")]
fn test_logical_operators(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::and_numbers("let x = 5 && 10;")]
#[case::and_strings(r#"let x = "hello" && "world";"#)]
#[case::and_bool_number("let x = true && 5;")]
#[case::and_number_bool("let x = 5 && false;")]
#[case::or_numbers("let x = 0 || 1;")]
#[case::or_strings(r#"let x = "" || "hello";"#)]
#[case::or_bool_string(r#"let x = true || "hello";"#)]
#[case::mixed_expression("let x = (5 + 3) && (2 < 4);")]
#[case::null_logical("let x = null && null;")]
fn test_logical_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3002");
}

// ========== Array Literals ==========

#[rstest]
#[case::numbers("let x = [1, 2, 3];")]
#[case::strings(r#"let x = ["a", "b", "c"];"#)]
#[case::bools("let x = [true, false, true];")]
#[case::empty("let x = [];")]
#[case::nested("let x = [[1, 2], [3, 4]];")]
#[case::with_expressions("let x = [1 + 2, 3 * 4, 5 - 6];")]
#[case::type_inference("let x = [1, 2, 3];")]
fn test_array_literals(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::mixed_types(r#"let x = [1, "hello", true];"#)]
#[case::number_string(r#"let x = [1, 2, "three"];"#)]
#[case::string_bool(r#"let x = ["hello", "world", true];"#)]
#[case::type_mismatch(r#"let x = [1, 2, "three"];"#)]
#[case::nested_mismatch(r#"let x = [[1, 2], ["a", "b"]];"#)]
fn test_array_literal_type_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Array Indexing ==========

#[rstest]
#[case::number_index("let x = [1, 2, 3]; let y = x[0];")]
#[case::variable_index("let x = [1, 2, 3]; let i: number = 1; let y = x[i];")]
#[case::expression_index("let x = [1, 2, 3]; let y = x[1 + 1];")]
#[case::nested_index("let x = [[1, 2], [3, 4]]; let y = x[0][1];")]
fn test_array_indexing(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::string_index(r#"let x = [1, 2, 3]; let y = x["hello"];"#)]
#[case::bool_index("let x = [1, 2, 3]; let y = x[true];")]
#[case::non_array("let x: number = 5; let y = x[0];")]
#[case::string_indexing(r#"let x: string = "hello"; let y = x[0];"#)]
fn test_array_indexing_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Array Element Assignment ==========

#[rstest]
#[case::same_type("let x = [1, 2, 3]; x[0] = 10;")]
#[case::string_array(r#"let x = ["a", "b", "c"]; x[1] = "world";"#)]
#[case::nested_array("let x = [[1, 2], [3, 4]]; x[0][1] = 99;")]
#[case::array_chain(
    "let arr = [1, 2, 3]; let idx: number = 1; let val = arr[idx] + arr[0]; arr[2] = val;"
)]
fn test_array_element_assignment(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::variable_type_mismatch(r#"let x: string = 42;"#)]
#[case::bool_type_mismatch(r#"let x: bool = 42;"#)]
#[case::string_index_assign(r#"let x = [1, 2, 3]; x["hello"] = 10;"#)]
#[case::non_array_assign("let x: number = 5; x[0] = 10;")]
fn test_array_assignment_errors(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

// ========== Complex Context Tests ==========

#[rstest]
#[case::function_arithmetic(r#"fn add(a: number, b: number) -> number { return a + b; }"#)]
#[case::conditional_operators(
    r#"let x: number = 5; let y: number = 10; if (x < y && y > 0) { let z = x + y; }"#
)]
#[case::loop_operators(r#"let i: number = 0; while (i < 10) { let x = i + 1; }"#)]
fn test_operators_in_context(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

// ========== Method Call Type Checking ==========

#[rstest]
#[case::json_as_string(r#"let data: json = parseJSON("{\"name\":\"Alice\"}"); let name: string = data["name"].as_string();"#)]
#[case::json_as_number(
    r#"let data: json = parseJSON("{\"age\":30}"); let age: number = data["age"].as_number();"#
)]
#[case::json_as_bool(r#"let data: json = parseJSON("{\"active\":true}"); let active: bool = data["active"].as_bool();"#)]
#[case::json_is_null(r#"let data: json = parseJSON("{\"value\":null}"); let is_null: bool = data["value"].is_null();"#)]
#[case::chained_json_access(r#"let data: json = parseJSON("{\"user\":{\"name\":\"Bob\"}}"); let name: string = data["user"]["name"].as_string();"#)]
#[case::method_in_expression(r#"let data: json = parseJSON("{\"count\":5}"); let x: number = data["count"].as_number() + 10;"#)]
#[case::method_as_arg(
    r#"let data: json = parseJSON("{\"msg\":\"hi\"}"); print(data["msg"].as_string());"#
)]
fn test_valid_method_calls(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[rstest]
#[case::invalid_method_name(
    r#"let data: json = parseJSON("{}"); data.invalid_method();"#,
    "AT3010"
)]
#[case::method_on_wrong_type("let x: number = 42; x.as_string();", "AT3010")]
#[case::method_on_string_type(r#"let s: string = "hello"; s.as_number();"#, "AT3010")]
#[case::method_on_bool_type("let b: bool = true; b.as_string();", "AT3010")]
fn test_invalid_method_calls(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::too_many_args(r#"let data: json = parseJSON("{}"); data.as_string(42);"#, "AT3005")]
#[case::too_many_multiple(r#"let data: json = parseJSON("{}"); data.is_null(1, 2);"#, "AT3005")]
fn test_method_argument_count_errors(#[case] source: &str, #[case] error_code: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, error_code);
}

#[rstest]
#[case::wrong_return_type_string(
    r#"let data: json = parseJSON("{\"x\":1}"); let x: string = data["x"].as_number();"#
)]
#[case::wrong_return_type_number(
    r#"let data: json = parseJSON("{\"x\":\"y\"}"); let x: number = data["x"].as_string();"#
)]
#[case::wrong_return_type_bool(
    r#"let data: json = parseJSON("{\"x\":1}"); let x: bool = data["x"].as_number();"#
)]
fn test_method_return_type_mismatch(#[case] source: &str) {
    let diagnostics = typecheck_source(source);
    assert_has_error(&diagnostics, "AT3001");
}

#[test]
fn test_chained_method_calls_type_correctly() {
    let source = r#"
        let data: json = parseJSON("{\"a\":{\"b\":{\"c\":\"value\"}}}");
        let result: string = data["a"]["b"]["c"].as_string();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_method_call_in_conditional() {
    let source = r#"
        let data: json = parseJSON("{\"enabled\":true}");
        if (data["enabled"].as_bool()) {
            print("Enabled");
        }
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}

#[test]
fn test_multiple_method_calls_in_expression() {
    let source = r#"
        let data: json = parseJSON("{\"a\":5,\"b\":10}");
        let sum: number = data["a"].as_number() + data["b"].as_number();
    "#;
    let diagnostics = typecheck_source(source);
    assert_no_errors(&diagnostics);
}
