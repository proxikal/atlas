//! JSON stdlib tests (Interpreter engine)
//!
//! Tests all 5 JSON functions with comprehensive edge case coverage

mod common;
use common::*;

// ============================================================================
// parseJSON Tests
// ============================================================================

#[test]
fn test_parse_json_null() {
    let code = r#"
        let result: json = parseJSON("null");
        typeof(result)
    "#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_true() {
    // Should return JsonValue, test via typeof
    let code = r#"typeof(parseJSON("true"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_boolean_false() {
    let code = r#"typeof(parseJSON("false"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number() {
    let code = r#"typeof(parseJSON("42"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_float() {
    let code = r#"typeof(parseJSON("3.14"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_number_negative() {
    let code = r#"typeof(parseJSON("-123"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_string() {
    let code = r#"typeof(parseJSON("\"hello\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_empty_string() {
    let code = r#"typeof(parseJSON("\"\""))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_empty() {
    let code = r#"typeof(parseJSON("[]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_numbers() {
    let code = r#"typeof(parseJSON("[1,2,3]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_mixed() {
    let code = r#"typeof(parseJSON("[1,\"two\",true,null]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_array_nested() {
    let code = r#"typeof(parseJSON("[[1,2],[3,4]]"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_empty() {
    let code = r#"typeof(parseJSON("{}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_simple() {
    let code = r#"typeof(parseJSON("{\"name\":\"Alice\",\"age\":30}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_nested() {
    let code = r#"typeof(parseJSON("{\"user\":{\"name\":\"Bob\"}}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_object_with_array() {
    let code = r#"typeof(parseJSON("{\"items\":[1,2,3]}"))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_whitespace() {
    let code = r#"typeof(parseJSON("  { \"a\" : 1 }  "))"#;
    assert_eval_string(code, "json");
}

#[test]
fn test_parse_json_unicode() {
    let code = r#"typeof(parseJSON("{\"emoji\":\"🎉\"}"))"#;
    assert_eval_string(code, "json");
}

// ============================================================================
// parseJSON Error Tests
// ============================================================================

#[test]
fn test_parse_json_invalid_syntax() {
    let code = r#"parseJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_trailing_comma() {
    let code = r#"parseJSON("[1,2,]")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_single_quote() {
    let code = r#"parseJSON("{'key':'value'}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_unquoted_keys() {
    let code = r#"parseJSON("{key:\"value\"}")"#;
    assert_has_error(code);
}

#[test]
fn test_parse_json_wrong_type() {
    let code = r#"parseJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// toJSON Tests
// ============================================================================

#[test]
fn test_to_json_null() {
    let code = r#"toJSON(null)"#;
    assert_eval_string(code, "null");
}

#[test]
fn test_to_json_bool_true() {
    let code = r#"toJSON(true)"#;
    assert_eval_string(code, "true");
}

#[test]
fn test_to_json_bool_false() {
    let code = r#"toJSON(false)"#;
    assert_eval_string(code, "false");
}

#[test]
fn test_to_json_number_int() {
    let code = r#"toJSON(42)"#;
    assert_eval_string(code, "42");
}

#[test]
fn test_to_json_number_float() {
    let code = r#"toJSON(3.14)"#;
    assert_eval_string(code, "3.14");
}

#[test]
fn test_to_json_number_negative() {
    let code = r#"toJSON(-10)"#;
    assert_eval_string(code, "-10");
}

#[test]
fn test_to_json_number_zero() {
    let code = r#"toJSON(0)"#;
    assert_eval_string(code, "0");
}

#[test]
fn test_to_json_string_simple() {
    let code = r#"toJSON("hello")"#;
    assert_eval_string(code, r#""hello""#);
}

#[test]
fn test_to_json_string_empty() {
    let code = r#"toJSON("")"#;
    assert_eval_string(code, r#""""#);
}

#[test]
fn test_to_json_string_with_quotes() {
    let code = r#"toJSON("say \"hi\"")"#;
    assert_eval_string(code, r#""say \"hi\"""#);
}

#[test]
fn test_to_json_array_empty() {
    let code = r#"toJSON([])"#;
    assert_eval_string(code, "[]");
}

#[test]
fn test_to_json_array_numbers() {
    let code = r#"toJSON([1,2,3])"#;
    assert_eval_string(code, "[1,2,3]");
}

// Note: Mixed-type array test removed - Atlas enforces homogeneous arrays.
// For heterogeneous JSON arrays, use parseJSON to create json values.

#[test]
fn test_to_json_array_nested() {
    let code = r#"toJSON([[1,2],[3,4]])"#;
    assert_eval_string(code, "[[1,2],[3,4]]");
}

// ============================================================================
// toJSON Error Tests
// ============================================================================

#[test]
fn test_to_json_nan_error() {
    let code = r#"toJSON(0.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_infinity_error() {
    let code = r#"toJSON(1.0 / 0.0)"#;
    assert_has_error(code);
}

#[test]
fn test_to_json_function_error() {
    let code = r#"
        fn test(): number { return 42; }
        toJSON(test)
    "#;
    assert_has_error(code);
}

// ============================================================================
// isValidJSON Tests
// ============================================================================

#[test]
fn test_is_valid_json_true_null() {
    let code = r#"isValidJSON("null")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_bool() {
    let code = r#"isValidJSON("true")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_number() {
    let code = r#"isValidJSON("42")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_string() {
    let code = r#"isValidJSON("\"hello\"")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_array() {
    let code = r#"isValidJSON("[1,2,3]")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_true_object() {
    let code = r#"isValidJSON("{\"key\":\"value\"}")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_is_valid_json_false_invalid() {
    let code = r#"isValidJSON("{invalid}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_trailing_comma() {
    let code = r#"isValidJSON("[1,2,]")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_empty() {
    let code = r#"isValidJSON("")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_false_single_quote() {
    let code = r#"isValidJSON("{'a':1}")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_is_valid_json_wrong_type() {
    let code = r#"isValidJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// prettifyJSON Tests
// ============================================================================

#[test]
fn test_prettify_json_object() {
    let code = r#"
        let compact: string = "{\"name\":\"Alice\",\"age\":30}";
        let pretty: string = prettifyJSON(compact, 2);
        includes(pretty, "  ")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_array() {
    let code = r#"
        let compact: string = "[1,2,3]";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_indent_zero() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 0);
        typeof(pretty)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_json_indent_four() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let pretty: string = prettifyJSON(compact, 4);
        includes(pretty, "    ")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_nested() {
    let code = r#"
        let compact: string = "{\"user\":{\"name\":\"Bob\"}}";
        let pretty: string = prettifyJSON(compact, 2);
        len(pretty) > len(compact)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_prettify_json_invalid() {
    let code = r#"prettifyJSON("{invalid}", 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_negative_indent() {
    let code = r#"prettifyJSON("{}", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_float_indent() {
    let code = r#"prettifyJSON("{}", 2.5)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_first_arg() {
    let code = r#"prettifyJSON(123, 2)"#;
    assert_has_error(code);
}

#[test]
fn test_prettify_json_wrong_type_second_arg() {
    let code = r#"prettifyJSON("{}", "2")"#;
    assert_has_error(code);
}

// ============================================================================
// minifyJSON Tests
// ============================================================================

#[test]
fn test_minify_json_object() {
    let code = r#"
        let pretty: string = "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_array() {
    let code = r#"
        let pretty: string = "[\n  1,\n  2,\n  3\n]";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_no_whitespace() {
    let code = r#"
        let compact: string = "{\"a\":1}";
        let minified: string = minifyJSON(compact);
        typeof(minified)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_minify_json_nested() {
    let code = r#"
        let pretty: string = "{\n  \"user\": {\n    \"name\": \"Bob\"\n  }\n}";
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_minify_json_invalid() {
    let code = r#"minifyJSON("{invalid}")"#;
    assert_has_error(code);
}

#[test]
fn test_minify_json_wrong_type() {
    let code = r#"minifyJSON(123)"#;
    assert_has_error(code);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_parse_then_serialize() {
    let code = r#"
        let original: string = "{\"name\":\"Alice\",\"age\":30}";
        let parsed: json = parseJSON(original);
        let serialized: string = toJSON(parsed);
        typeof(serialized)
    "#;
    assert_eval_string(code, "string");
}

#[test]
fn test_prettify_then_minify() {
    let code = r#"
        let compact: string = "{\"a\":1,\"b\":2}";
        let pretty: string = prettifyJSON(compact, 2);
        let minified: string = minifyJSON(pretty);
        len(minified) < len(pretty)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_validate_before_parse() {
    let code = r#"
        let json_str: string = "{\"valid\":true}";
        let valid: bool = isValidJSON(json_str);
        let parsed: json = parseJSON(json_str);
        valid && typeof(parsed) == "json"
    "#;
    assert_eval_bool(code, true);
}
