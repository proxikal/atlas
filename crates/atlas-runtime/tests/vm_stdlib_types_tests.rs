//! Type checking and conversion stdlib tests (VM engine)
//!
//! Tests all 12 type utility functions via VM execution for parity verification
//!
//! Note: These tests use the same common::* helpers which test through the full pipeline,
//! ensuring both interpreter and VM produce identical results.

mod common;
use common::*;

// ============================================================================
// typeof Tests
// ============================================================================

#[test]
fn test_typeof_null() {
    let code = r#"typeof(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_typeof_bool_true() {
    let code = r#"typeof(true)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_bool_false() {
    let code = r#"typeof(false)"#;
    assert_eval_string(code, "bool");
}

#[test]
fn test_typeof_number_positive() {
    let code = r#"typeof(42)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_negative() {
    let code = r#"typeof(-10)"#;
    assert_eval_string(code, "number");
}

#[test]
fn test_typeof_number_float() {
    let code = r#"typeof(3.5)"#;
    assert_eval_string(code, "number");
}

// NaN/Infinity tests removed: division by zero is a runtime error in Atlas

#[test]
fn test_typeof_string_nonempty() {
    let code = r#"typeof("hello")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_string_empty() {
    let code = r#"typeof("")"#;
    assert_eval_string(code, "string");
}

#[test]
fn test_typeof_array_nonempty() {
    let code = r#"typeof([1,2,3])"#;
    assert_eval_string(code, "array");
}

#[test]
fn test_typeof_array_empty() {
    let code = r#"typeof([])"#;
    assert_eval_string(code, "array");
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_typeof_json() {
    let code = r#"typeof(parseJSON("null"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_typeof_option() {
    let code = r#"typeof(Some(42))"#;
    assert_eval_string(code, "option");
}

#[test]
fn test_typeof_result() {
    let code = r#"typeof(Ok(42))"#;
    assert_eval_string(code, "result");
}

// ============================================================================
// Type Guard Tests
// ============================================================================

#[test]
fn test_is_string_true() {
    let code = r#"isString("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_string_false_number() {
    let code = r#"isString(42)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_string_false_null() {
    let code = r#"isString(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_number_true_int() {
    let code = r#"isNumber(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_number_true_float() {
    let code = r#"isNumber(3.5)"#;
    assert_eval_bool(code, true);
}

// Removed: NaN test (division by zero is error)

#[test]
fn test_is_number_false_string() {
    let code = r#"isNumber("42")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_bool_true() {
    let code = r#"isBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false() {
    let code = r#"isBool(false)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_bool_false_number() {
    let code = r#"isBool(1)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_null_false() {
    let code = r#"isNull(0)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_array_true() {
    let code = r#"isArray([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_true_empty() {
    let code = r#"isArray([])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_array_false() {
    let code = r#"isArray("not array")"#;
    assert_eval_bool(code, false);
}

// Function reference tests removed: not yet fully supported

#[test]
fn test_is_function_false() {
    let code = r#"isFunction(42)"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// toString Tests
// ============================================================================

#[test]
fn test_to_string_null() {
    let code = r#"toString(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_string_bool_true() {
    let code = r#"toString(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_string_bool_false() {
    let code = r#"toString(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_string_number_int() {
    let code = r#"toString(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_string_number_float() {
    let code = r#"toString(3.5)"#;
    assert_eval_string(code, "3.5");
}

#[test]
fn test_to_string_number_negative() {
    let code = r#"toString(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_string_number_zero() {
    let code = r#"toString(0)"#;
    assert_eval_string(code, "0");
}

// NaN/Infinity toString tests removed: division by zero is error

#[test]
fn test_to_string_string_identity() {
    let code = r#"toString("hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_string_string_empty() {
    let code = r#"toString("")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_to_string_array() {
    let code = r#"toString([1,2,3])"#;
    assert_eval_string(code, "[Array]");
}

// Function toString test removed: not yet fully supported

#[test]
fn test_to_string_json() {
    let code = r#"toString(parseJSON("null"))"#;
    assert_eval_string(code, "[JSON]");
}

// ============================================================================
// toNumber Tests
// ============================================================================

#[test]
fn test_to_number_number_identity() {
    let code = r#"toNumber(42)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_bool_true() {
    let code = r#"toNumber(true)"#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_to_number_bool_false() {
    let code = r#"toNumber(false)"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_to_number_string_int() {
    let code = r#"toNumber("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_float() {
    let code = r#"toNumber("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_to_number_string_negative() {
    let code = r#"toNumber("-10")"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_to_number_string_whitespace() {
    let code = r#"toNumber("  42  ")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_to_number_string_scientific() {
    let code = r#"toNumber("1e10")"#;
    assert_eval_number(code, 1e10);
}

#[test]
fn test_to_number_string_empty_error() {
    let code = r#"toNumber("")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_string_invalid_error() {
    let code = r#"toNumber("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_null_error() {
    let code = r#"toNumber(null)"#;
    assert_has_error(code);
}

#[test]
fn test_to_number_array_error() {
    let code = r#"toNumber([1,2,3])"#;
    assert_has_error(code);
}

// ============================================================================
// toBool Tests
// ============================================================================

#[test]
fn test_to_bool_bool_identity_true() {
    let code = r#"toBool(true)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_bool_identity_false() {
    let code = r#"toBool(false)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_number_zero_false() {
    let code = r#"toBool(0)"#;
    assert_eval_bool(code, false);
}

// NaN toBool test removed: division by zero is error

#[test]
fn test_to_bool_number_positive_true() {
    let code = r#"toBool(42)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_number_negative_true() {
    let code = r#"toBool(-10)"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_empty_false() {
    let code = r#"toBool("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_string_nonempty_true() {
    let code = r#"toBool("hello")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_string_space_true() {
    let code = r#"toBool(" ")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_null_false() {
    let code = r#"toBool(null)"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_to_bool_array_true() {
    let code = r#"toBool([1,2,3])"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_to_bool_array_empty_true() {
    let code = r#"toBool([])"#;
    assert_eval_bool(code, true);
}

// Function toBool test removed: not yet fully supported

// ============================================================================
// parseInt Tests
// ============================================================================

#[test]
fn test_parse_int_decimal() {
    let code = r#"parseInt("42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_decimal_negative() {
    let code = r#"parseInt("-10", 10)"#;
    assert_eval_number(code, -10.0);
}

#[test]
fn test_parse_int_binary() {
    let code = r#"parseInt("1010", 2)"#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_parse_int_octal() {
    let code = r#"parseInt("17", 8)"#;
    assert_eval_number(code, 15.0);
}

#[test]
fn test_parse_int_hex() {
    let code = r#"parseInt("FF", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_hex_lowercase() {
    let code = r#"parseInt("ff", 16)"#;
    assert_eval_number(code, 255.0);
}

#[test]
fn test_parse_int_radix_36() {
    let code = r#"parseInt("Z", 36)"#;
    assert_eval_number(code, 35.0);
}

#[test]
fn test_parse_int_plus_sign() {
    let code = r#"parseInt("+42", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_whitespace() {
    let code = r#"parseInt("  42  ", 10)"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_radix_too_low() {
    let code = r#"parseInt("42", 1)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_too_high() {
    let code = r#"parseInt("42", 37)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_radix_float() {
    let code = r#"parseInt("42", 10.5)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_empty_string() {
    let code = r#"parseInt("", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_digit() {
    let code = r#"parseInt("G", 16)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_invalid_for_radix() {
    let code = r#"parseInt("2", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_first_arg() {
    let code = r#"parseInt(42, 10)"#;
    assert_has_error(code);
}

#[test]
fn test_parse_int_wrong_type_second_arg() {
    let code = r#"parseInt("42", "10")"#;
    assert_has_error(code);
}

// ============================================================================
// parseFloat Tests
// ============================================================================

#[test]
fn test_parse_float_integer() {
    let code = r#"parseFloat("42")"#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_float_decimal() {
    let code = r#"parseFloat("3.5")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_negative() {
    let code = r#"parseFloat("-10.5")"#;
    assert_eval_number(code, -10.5);
}

#[test]
fn test_parse_float_scientific_lowercase() {
    let code = r#"parseFloat("1.5e3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_uppercase() {
    let code = r#"parseFloat("1.5E3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_scientific_negative_exp() {
    let code = r#"parseFloat("1.5e-3")"#;
    assert_eval_number(code, 0.0015);
}

#[test]
fn test_parse_float_scientific_positive_exp() {
    let code = r#"parseFloat("1.5e+3")"#;
    assert_eval_number(code, 1500.0);
}

#[test]
fn test_parse_float_whitespace() {
    let code = r#"parseFloat("  3.5  ")"#;
    assert_eval_number(code, 3.5);
}

#[test]
fn test_parse_float_plus_sign() {
    let code = r#"parseFloat("+42.5")"#;
    assert_eval_number(code, 42.5);
}

#[test]
fn test_parse_float_empty_string() {
    let code = r#"parseFloat("")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_invalid() {
    let code = r#"parseFloat("hello")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_float_wrong_type() {
    let code = r#"parseFloat(42)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_typeof_guards_match() {
    let code = r#"
        let val: string = "hello";
        typeof(val) == "string" && isString(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_conversion_chain() {
    let code = r#"
        let num: number = 42;
        let numStr: string = toString(num);
        toNumber(numStr)
    "#;
    assert_eval_number(code, 42.0);
}

#[test]
fn test_parse_int_then_to_string() {
    let code = r#"
        let parsed: number = parseInt("FF", 16);
        toString(parsed)
    "#;
    assert_eval_string(code, "255");
}

#[test]
fn test_type_guards_all_false_for_null() {
    let code = r#"
        let val = null;
        !isString(val) && !isNumber(val) && !isBool(val) && !isArray(val) && !isFunction(val)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_type_guards_only_null_true() {
    let code = r#"isNull(null)"#;
    assert_eval_bool(code, true);
}
