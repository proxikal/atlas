//! String stdlib tests (VM engine)
//!
//! Tests all 18 string functions via VM execution for parity verification
//!
//! Note: These tests use the same common::* helpers which test through the full pipeline,
//! ensuring both interpreter and VM produce identical results.

mod common;
use common::*;

// All tests are identical to stdlib_string_tests.rs to verify parity
// The common test helpers automatically test through both interpreter and VM

// ============================================================================
// Core Operations Tests
// ============================================================================

#[test]
fn test_split_basic() {
    let code = r#"
        let result: string[] = split("a,b,c", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_empty_separator() {
    let code = r#"
        let result: string[] = split("abc", "");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_split_no_match() {
    let code = r#"
        let result: string[] = split("hello", ",");
        len(result)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_split_unicode() {
    let code = r#"
        let result: string[] = split("🎉,🔥,✨", ",");
        len(result)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_join_basic() {
    let code = r#"join(["a", "b", "c"], ",")"#;
    assert_eval_string(code, "a,b,c");
}

#[test]
fn test_join_empty_array() {
    let code = r#"join([], ",")"#;
    assert_eval_string(code, "");
}

#[test]
fn test_join_empty_separator() {
    let code = r#"join(["a", "b", "c"], "")"#;
    assert_eval_string(code, "abc");
}

#[test]
fn test_trim_basic() {
    let code = r#"trim("  hello  ")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_unicode_whitespace() {
    let code = "trim(\"\u{00A0}hello\u{00A0}\")";
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_start() {
    let code = r#"trimStart("  hello")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_trim_end() {
    let code = r#"trimEnd("hello  ")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Search Operations Tests
// ============================================================================

#[test]
fn test_index_of_found() {
    let code = r#"indexOf("hello", "ll")"#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_index_of_not_found() {
    let code = r#"indexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_index_of_empty_needle() {
    let code = r#"indexOf("hello", "")"#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_last_index_of_found() {
    let code = r#"lastIndexOf("hello", "l")"#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_last_index_of_not_found() {
    let code = r#"lastIndexOf("hello", "x")"#;
    assert_eval_number(code, -1.0);
}

#[test]
fn test_includes_found() {
    let code = r#"includes("hello", "ll")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_includes_not_found() {
    let code = r#"includes("hello", "x")"#;
    assert_eval_bool(code, false);
}

// ============================================================================
// Transformation Tests
// ============================================================================

#[test]
fn test_to_upper_case() {
    let code = r#"toUpperCase("hello")"#;
    assert_eval_string(code, "HELLO");
}

#[test]
fn test_to_upper_case_unicode() {
    let code = r#"toUpperCase("café")"#;
    assert_eval_string(code, "CAFÉ");
}

#[test]
fn test_to_lower_case() {
    let code = r#"toLowerCase("HELLO")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_to_lower_case_unicode() {
    let code = r#"toLowerCase("CAFÉ")"#;
    assert_eval_string(code, "café");
}

#[test]
fn test_substring_basic() {
    let code = r#"substring("hello", 1, 4)"#;
    assert_eval_string(code, "ell");
}

#[test]
fn test_substring_empty() {
    let code = r#"substring("hello", 2, 2)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_substring_out_of_bounds() {
    let code = r#"substring("hello", 0, 100)"#;
    assert_has_error(code);
}

#[test]
fn test_char_at_basic() {
    let code = r#"charAt("hello", 0)"#;
    assert_eval_string(code, "h");
}

#[test]
fn test_char_at_unicode() {
    let code = r#"charAt("🎉🔥✨", 1)"#;
    assert_eval_string(code, "🔥");
}

#[test]
fn test_char_at_out_of_bounds() {
    let code = r#"charAt("hello", 10)"#;
    assert_has_error(code);
}

#[test]
fn test_repeat_basic() {
    let code = r#"repeat("ha", 3)"#;
    assert_eval_string(code, "hahaha");
}

#[test]
fn test_repeat_zero() {
    let code = r#"repeat("ha", 0)"#;
    assert_eval_string(code, "");
}

#[test]
fn test_repeat_negative() {
    let code = r#"repeat("ha", -1)"#;
    assert_has_error(code);
}

#[test]
fn test_replace_basic() {
    let code = r#"replace("hello", "l", "L")"#;
    assert_eval_string(code, "heLlo");
}

#[test]
fn test_replace_not_found() {
    let code = r#"replace("hello", "x", "y")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_replace_empty_search() {
    let code = r#"replace("hello", "", "x")"#;
    assert_eval_string(code, "hello");
}

// ============================================================================
// Formatting Tests
// ============================================================================

#[test]
fn test_pad_start_basic() {
    let code = r#"padStart("5", 3, "0")"#;
    assert_eval_string(code, "005");
}

#[test]
fn test_pad_start_already_long() {
    let code = r#"padStart("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_pad_start_multichar_fill() {
    let code = r#"padStart("x", 5, "ab")"#;
    assert_eval_string(code, "ababx");
}

#[test]
fn test_pad_end_basic() {
    let code = r#"padEnd("5", 3, "0")"#;
    assert_eval_string(code, "500");
}

#[test]
fn test_pad_end_already_long() {
    let code = r#"padEnd("hello", 3, "0")"#;
    assert_eval_string(code, "hello");
}

#[test]
fn test_starts_with_true() {
    let code = r#"startsWith("hello", "he")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_starts_with_false() {
    let code = r#"startsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_starts_with_empty() {
    let code = r#"startsWith("hello", "")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_true() {
    let code = r#"endsWith("hello", "lo")"#;
    assert_eval_bool(code, true);
}

#[test]
fn test_ends_with_false() {
    let code = r#"endsWith("hello", "x")"#;
    assert_eval_bool(code, false);
}

#[test]
fn test_ends_with_empty() {
    let code = r#"endsWith("hello", "")"#;
    assert_eval_bool(code, true);
}
