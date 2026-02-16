//! Regex operations tests (Phase-08b)
//!
//! Tests regex replacement operations, splitting, and advanced features.
//! All tests use Atlas::eval() API.

use atlas_runtime::Atlas;

// ============================================================================
// Test Helpers
// ============================================================================

fn eval_ok(code: &str) -> String {
    let atlas = Atlas::new();
    let result = atlas.eval(code).expect("Execution should succeed");
    result.to_string()
}

// ============================================================================
// Basic Replacement Tests (10 tests)
// ============================================================================

#[test]
fn test_replace_first_match_only() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplace(pattern, "a1b2c3", "X")
    "#;
    assert_eq!(eval_ok(code), "aXb2c3");
}

#[test]
fn test_replace_all_matches() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceAll(pattern, "a1b2c3", "X")
    "#;
    assert_eq!(eval_ok(code), "aXbXcX");
}

#[test]
fn test_replace_with_capture_group_refs() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)"));
        regexReplace(pattern, "a123b", "[$1]")
    "#;
    assert_eq!(eval_ok(code), "a[123]b");
}

#[test]
fn test_replace_all_with_capture_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)"));
        regexReplaceAll(pattern, "a1b22c333", "[$1]")
    "#;
    assert_eq!(eval_ok(code), "a[1]b[22]c[333]");
}

#[test]
fn test_replace_special_refs_full_match() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplace(pattern, "a123b", "[$0]")
    "#;
    assert_eq!(eval_ok(code), "a[123]b");
}

#[test]
fn test_replace_empty_replacement() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceAll(pattern, "a1b2c3", "")
    "#;
    assert_eq!(eval_ok(code), "abc");
}

#[test]
fn test_replace_no_match_returns_original() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplace(pattern, "abc", "X")
    "#;
    assert_eq!(eval_ok(code), "abc");
}

#[test]
fn test_replace_unicode() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplace(pattern, "こんにちは123世界", "★")
    "#;
    assert_eq!(eval_ok(code), "こんにちは★世界");
}

#[test]
fn test_replace_multiple_capture_groups() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d+)-(\\w+)"));
        regexReplace(pattern, "abc 123-xyz def", "[$2:$1]")
    "#;
    assert_eq!(eval_ok(code), "abc [xyz:123] def");
}

#[test]
fn test_replace_at_boundaries() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceAll(pattern, "123abc456", "X")
    "#;
    assert_eq!(eval_ok(code), "XabcX");
}

// ============================================================================
// Callback Replacement Tests (8 tests)
// ============================================================================

#[test]
fn test_replace_with_calls_callback_first_match() {
    let code = r#"
        fn bracketize(m: HashMap) -> string {
            return "[" + unwrap(hashMapGet(m, "text")) + "]";
        }
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceWith(pattern, "a1b2c3", bracketize)
    "#;
    assert_eq!(eval_ok(code), "a[1]b2c3");
}

#[test]
fn test_replace_all_with_calls_callback_all_matches() {
    let code = r#"
        fn bracketize(m: HashMap) -> string {
            return "[" + unwrap(hashMapGet(m, "text")) + "]";
        }
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceAllWith(pattern, "a1b2c3", bracketize)
    "#;
    assert_eq!(eval_ok(code), "a[1]b[2]c[3]");
}

#[test]
fn test_callback_receives_correct_match_data() {
    let code = r#"
        fn formatter(m: HashMap) -> string {
            let text = unwrap(hashMapGet(m, "text"));
            let start = unwrap(hashMapGet(m, "start"));
            let end_pos = unwrap(hashMapGet(m, "end"));
            return "[" + text + "@" + toString(start) + "-" + toString(end_pos) + "]";
        }
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceWith(pattern, "hello123world", formatter)
    "#;
    assert_eq!(eval_ok(code), "hello[123@5-8]world");
}

#[test]
fn test_callback_return_value_used_as_replacement() {
    let code = r#"
        fn doubler(m: HashMap) -> string {
            let num = unwrap(hashMapGet(m, "text"));
            return toString(toNumber(num) * 2);
        }
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceWith(pattern, "value:42", doubler)
    "#;
    assert_eq!(eval_ok(code), "value:84");
}

#[test]
fn test_callback_with_capture_groups() {
    let code = r#"
        fn swapper(m: HashMap) -> string {
            let groups = unwrap(hashMapGet(m, "groups"));
            let num = groups[1];
            let word = groups[2];
            return word + ":" + num;
        }
        let pattern = unwrap(regexNew("(\\d+)-(\\w+)"));
        regexReplaceWith(pattern, "abc 123-xyz def", swapper)
    "#;
    assert_eq!(eval_ok(code), "abc xyz:123 def");
}

#[test]
fn test_callback_can_use_match_positions() {
    let code = r#"
        fn firstOrOther(m: HashMap) -> string {
            let start = unwrap(hashMapGet(m, "start"));
            if (start == 0) {
                return "FIRST";
            } else {
                return "OTHER";
            }
        }
        let pattern = unwrap(regexNew("\\w+"));
        regexReplaceWith(pattern, "hello world", firstOrOther)
    "#;
    assert_eq!(eval_ok(code), "FIRST world");
}

#[test]
fn test_callback_can_access_groups_array() {
    let code = r#"
        fn extractCapture(m: HashMap) -> string {
            let groups = unwrap(hashMapGet(m, "groups"));
            let captured = groups[1];
            return "[" + captured + "]";
        }
        let pattern = unwrap(regexNew("(\\d+)"));
        regexReplaceWith(pattern, "test123", extractCapture)
    "#;
    assert_eq!(eval_ok(code), "test[123]");
}

#[test]
fn test_replace_all_with_processes_all_matches() {
    let code = r#"
        fn bracketize(m: HashMap) -> string {
            let num = unwrap(hashMapGet(m, "text"));
            return "[" + num + "]";
        }
        let pattern = unwrap(regexNew("\\d+"));
        regexReplaceAllWith(pattern, "1a2b3c", bracketize)
    "#;
    assert_eq!(eval_ok(code), "[1]a[2]b[3]c");
}

// ============================================================================
// Splitting Tests (8 tests)
// ============================================================================

#[test]
fn test_split_divides_at_matches() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplit(pattern, "a,b,c");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_split_includes_empty_strings() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplit(pattern, "a,b,,c");
        parts[2]
    "#;
    assert_eq!(eval_ok(code), "");
}

#[test]
fn test_split_no_matches_returns_single_element() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplit(pattern, "abc");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "1");
}

#[test]
fn test_split_n_limits_splits() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplitN(pattern, "a,b,c,d", 2);
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3"); // Splits into 3 parts: a, b, c,d
}

#[test]
fn test_split_n_with_limit_zero_returns_empty() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplitN(pattern, "a,b,c", 0);
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_split_on_complex_pattern() {
    let code = r#"
        let pattern = unwrap(regexNew("\\s+"));
        let parts = regexSplit(pattern, "hello   world  test");
        len(parts)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_split_preserves_unicode() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplit(pattern, "こんにちは,世界,テスト");
        parts[1]
    "#;
    assert_eq!(eval_ok(code), "世界");
}

#[test]
fn test_split_with_zero_width_matches() {
    let code = r#"
        let pattern = unwrap(regexNew(""));
        let parts = regexSplit(pattern, "abc");
        len(parts)
    "#;
    // Empty pattern splits between every character including boundaries
    assert_eq!(eval_ok(code), "5"); // "", "a", "b", "c", ""
}

// ============================================================================
// Advanced Features Tests (8 tests)
// ============================================================================

#[test]
fn test_match_indices_returns_positions() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let indices = regexMatchIndices(pattern, "a1b22c333");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "3");
}

#[test]
fn test_match_indices_returns_start_end_pairs() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let indices = regexMatchIndices(pattern, "hello123world");
        let first = indices[0];
        first[0]
    "#;
    assert_eq!(eval_ok(code), "5"); // start position
}

#[test]
fn test_match_indices_no_matches_returns_empty() {
    let code = r#"
        let pattern = unwrap(regexNew("\\d+"));
        let indices = regexMatchIndices(pattern, "hello");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "0");
}

#[test]
fn test_regex_test_convenience_function() {
    let code = r#"
        regexTest("\\d+", "hello123")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_regex_test_returns_false_no_match() {
    let code = r#"
        regexTest("\\d+", "hello")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_regex_test_returns_false_on_compile_error() {
    let code = r#"
        regexTest("[invalid", "test")
    "#;
    assert_eq!(eval_ok(code), "false");
}

#[test]
fn test_match_indices_with_overlapping_pattern() {
    let code = r#"
        let pattern = unwrap(regexNew("\\w+"));
        let indices = regexMatchIndices(pattern, "hello world");
        len(indices)
    "#;
    assert_eq!(eval_ok(code), "2"); // "hello" and "world"
}

#[test]
fn test_regex_test_with_complex_pattern() {
    let code = r#"
        regexTest("[a-z]+@[a-z]+\\.[a-z]+", "user@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

// ============================================================================
// Integration Tests (6 tests)
// ============================================================================

#[test]
fn test_integration_email_validation() {
    let code = r#"
        let email_pattern = unwrap(regexNew("[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"));
        regexIsMatch(email_pattern, "test.user+tag@example.com")
    "#;
    assert_eq!(eval_ok(code), "true");
}

#[test]
fn test_integration_url_extraction() {
    let code = r#"
        let url_pattern = unwrap(regexNew("https?://[^\\s]+"));
        let text = "Visit https://example.com or http://test.org for info";
        let matches = regexFindAll(url_pattern, text);
        len(matches)
    "#;
    assert_eq!(eval_ok(code), "2");
}

#[test]
fn test_integration_phone_formatting() {
    let code = r#"
        let pattern = unwrap(regexNew("(\\d{3})(\\d{3})(\\d{4})"));
        regexReplace(pattern, "Phone: 5551234567", "($1) $2-$3")
    "#;
    assert_eq!(eval_ok(code), "Phone: (555) 123-4567");
}

#[test]
fn test_integration_html_tag_stripping() {
    let code = r#"
        let tag_pattern = unwrap(regexNew("<[^>]+>"));
        regexReplaceAll(tag_pattern, "<p>Hello <b>World</b></p>", "")
    "#;
    assert_eq!(eval_ok(code), "Hello World");
}

#[test]
fn test_integration_csv_parsing() {
    let code = r#"
        let pattern = unwrap(regexNew(","));
        let parts = regexSplit(pattern, "John,Doe,30,Engineer");
        parts[3]
    "#;
    assert_eq!(eval_ok(code), "Engineer");
}

#[test]
fn test_integration_text_processing_pipeline() {
    let code = r#"
        fn uppercase_numbers(m: HashMap) -> string {
            let num = unwrap(hashMapGet(m, "text"));
            return "[" + num + "]";
        }
        let digit_pattern = unwrap(regexNew("\\d+"));
        let text = "Error 404: Page 500 not found";
        let processed = regexReplaceAllWith(digit_pattern, text, uppercase_numbers);
        let word_pattern = unwrap(regexNew("\\s+"));
        let words = regexSplit(word_pattern, processed);
        len(words)
    "#;
    assert_eq!(eval_ok(code), "6"); // "Error", "[404]:", "Page", "[500]", "not", "found"
}
