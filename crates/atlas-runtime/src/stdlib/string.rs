//! String manipulation functions
//!
//! Complete string API with Unicode support

use crate::span::Span;
use crate::value::{RuntimeError, Value};

/// Maximum repeat count to prevent memory abuse
const MAX_REPEAT_COUNT: i64 = 1_000_000;

// ============================================================================
// Core Operations
// ============================================================================

/// Split a string by separator
///
/// Returns an array of string parts. If separator is empty, returns array of individual characters.
pub fn split(s: &str, separator: &str, _span: Span) -> Result<Value, RuntimeError> {
    if separator.is_empty() {
        // Split into individual characters
        let chars: Vec<Value> = s.chars().map(|c| Value::string(c.to_string())).collect();
        Ok(Value::array(chars))
    } else {
        let parts: Vec<Value> = s
            .split(separator)
            .map(|part| Value::string(part.to_string()))
            .collect();
        Ok(Value::array(parts))
    }
}

/// Join an array of strings with separator
pub fn join(parts: &[Value], separator: &str, span: Span) -> Result<String, RuntimeError> {
    let mut strings = Vec::with_capacity(parts.len());

    for part in parts {
        match part {
            Value::String(s) => strings.push(s.as_ref().clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: "join() requires array of strings".to_string(),
                    span,
                })
            }
        }
    }

    Ok(strings.join(separator))
}

/// Trim leading and trailing whitespace (Unicode-aware)
pub fn trim(s: &str) -> String {
    s.trim().to_string()
}

/// Trim leading whitespace (Unicode-aware)
pub fn trim_start(s: &str) -> String {
    s.trim_start().to_string()
}

/// Trim trailing whitespace (Unicode-aware)
pub fn trim_end(s: &str) -> String {
    s.trim_end().to_string()
}

// ============================================================================
// Search Operations
// ============================================================================

/// Find first occurrence index
///
/// Returns -1 if not found, index of first occurrence otherwise.
pub fn index_of(haystack: &str, needle: &str) -> f64 {
    if needle.is_empty() {
        return 0.0; // Empty string is at index 0
    }

    haystack.find(needle).map(|idx| idx as f64).unwrap_or(-1.0)
}

/// Find last occurrence index
///
/// Returns -1 if not found, index of last occurrence otherwise.
pub fn last_index_of(haystack: &str, needle: &str) -> f64 {
    if needle.is_empty() {
        return haystack.len() as f64; // Empty string is at the end
    }

    haystack.rfind(needle).map(|idx| idx as f64).unwrap_or(-1.0)
}

/// Check if string contains substring
pub fn includes(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

// ============================================================================
// Transformation
// ============================================================================

/// Convert to uppercase (Unicode-aware)
pub fn to_upper_case(s: &str) -> String {
    s.to_uppercase()
}

/// Convert to lowercase (Unicode-aware)
pub fn to_lower_case(s: &str) -> String {
    s.to_lowercase()
}

/// Extract substring from start to end (UTF-8 boundary safe)
///
/// Returns substring from start (inclusive) to end (exclusive).
/// Validates UTF-8 boundaries and checks bounds.
pub fn substring(s: &str, start: f64, end: f64, span: Span) -> Result<String, RuntimeError> {
    // Validate indices are integers
    if start.fract() != 0.0 || end.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "substring() indices must be integers".to_string(),
            span,
        });
    }

    let start_idx = start as usize;
    let end_idx = end as usize;
    let byte_len = s.len();

    // Validate bounds
    if start_idx > end_idx {
        return Err(RuntimeError::OutOfBounds { span });
    }

    if start_idx > byte_len || end_idx > byte_len {
        return Err(RuntimeError::OutOfBounds { span });
    }

    // Validate UTF-8 boundaries
    if !s.is_char_boundary(start_idx) || !s.is_char_boundary(end_idx) {
        return Err(RuntimeError::TypeError {
            msg: "substring() indices must be on UTF-8 character boundaries".to_string(),
            span,
        });
    }

    Ok(s[start_idx..end_idx].to_string())
}

/// Get character at index (returns grapheme cluster, not byte)
///
/// Returns single character string at the given index.
pub fn char_at(s: &str, index: f64, span: Span) -> Result<String, RuntimeError> {
    // Validate index is integer
    if index.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "charAt() index must be an integer".to_string(),
            span,
        });
    }

    let idx = index as usize;

    // Get character at index
    s.chars()
        .nth(idx)
        .map(|c| c.to_string())
        .ok_or(RuntimeError::OutOfBounds { span })
}

/// Repeat string count times
///
/// Limits count to prevent memory abuse.
pub fn repeat(s: &str, count: f64, span: Span) -> Result<String, RuntimeError> {
    // Validate count is integer
    if count.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "repeat() count must be an integer".to_string(),
            span,
        });
    }

    let count_i64 = count as i64;

    // Negative count is error
    if count_i64 < 0 {
        return Err(RuntimeError::TypeError {
            msg: "repeat() count cannot be negative".to_string(),
            span,
        });
    }

    // Limit count to prevent memory abuse
    if count_i64 > MAX_REPEAT_COUNT {
        return Err(RuntimeError::InvalidNumericResult { span });
    }

    Ok(s.repeat(count_i64 as usize))
}

/// Replace first occurrence
///
/// Replaces only the first occurrence of search with replacement.
pub fn replace(s: &str, search: &str, replacement: &str) -> String {
    if search.is_empty() {
        // Empty search returns original string
        return s.to_string();
    }

    s.replacen(search, replacement, 1)
}

// ============================================================================
// Formatting
// ============================================================================

/// Pad start to reach target length
///
/// If string is already >= length, returns original string.
/// Fill string is repeated as needed.
pub fn pad_start(s: &str, length: f64, fill: &str, span: Span) -> Result<String, RuntimeError> {
    // Validate length is integer
    if length.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "padStart() length must be an integer".to_string(),
            span,
        });
    }

    let target_len = length as usize;
    let current_len = s.chars().count();

    if current_len >= target_len {
        return Ok(s.to_string());
    }

    if fill.is_empty() {
        return Ok(s.to_string());
    }

    let padding_needed = target_len - current_len;
    let fill_chars: Vec<char> = fill.chars().collect();
    let fill_len = fill_chars.len();

    let mut result = String::with_capacity(target_len);

    // Add padding
    for i in 0..padding_needed {
        result.push(fill_chars[i % fill_len]);
    }

    // Add original string
    result.push_str(s);

    Ok(result)
}

/// Pad end to reach target length
///
/// If string is already >= length, returns original string.
/// Fill string is repeated as needed.
pub fn pad_end(s: &str, length: f64, fill: &str, span: Span) -> Result<String, RuntimeError> {
    // Validate length is integer
    if length.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            msg: "padEnd() length must be an integer".to_string(),
            span,
        });
    }

    let target_len = length as usize;
    let current_len = s.chars().count();

    if current_len >= target_len {
        return Ok(s.to_string());
    }

    if fill.is_empty() {
        return Ok(s.to_string());
    }

    let padding_needed = target_len - current_len;
    let fill_chars: Vec<char> = fill.chars().collect();
    let fill_len = fill_chars.len();

    let mut result = String::with_capacity(target_len);

    // Add original string
    result.push_str(s);

    // Add padding
    for i in 0..padding_needed {
        result.push(fill_chars[i % fill_len]);
    }

    Ok(result)
}

/// Check if string starts with prefix
pub fn starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

/// Check if string ends with suffix
pub fn ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_basic() {
        let result = split("a,b,c", ",", Span::dummy()).unwrap();
        if let Value::Array(arr) = result {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::string("a"));
            assert_eq!(borrowed[1], Value::string("b"));
            assert_eq!(borrowed[2], Value::string("c"));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_split_empty_separator() {
        let result = split("abc", "", Span::dummy()).unwrap();
        if let Value::Array(arr) = result {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::string("a"));
            assert_eq!(borrowed[1], Value::string("b"));
            assert_eq!(borrowed[2], Value::string("c"));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_join_basic() {
        let parts = vec![Value::string("a"), Value::string("b"), Value::string("c")];
        let result = join(&parts, ",", Span::dummy()).unwrap();
        assert_eq!(result, "a,b,c");
    }

    #[test]
    fn test_trim_functions() {
        assert_eq!(trim("  hello  "), "hello");
        assert_eq!(trim_start("  hello"), "hello");
        assert_eq!(trim_end("hello  "), "hello");
    }

    #[test]
    fn test_index_of() {
        assert_eq!(index_of("hello", "ll"), 2.0);
        assert_eq!(index_of("hello", "x"), -1.0);
        assert_eq!(index_of("hello", ""), 0.0);
    }

    #[test]
    fn test_last_index_of() {
        assert_eq!(last_index_of("hello", "l"), 3.0);
        assert_eq!(last_index_of("hello", "x"), -1.0);
    }

    #[test]
    fn test_includes() {
        assert!(includes("hello", "ll"));
        assert!(!includes("hello", "x"));
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_upper_case("hello"), "HELLO");
        assert_eq!(to_lower_case("HELLO"), "hello");
    }

    #[test]
    fn test_substring() {
        let result = substring("hello", 1.0, 4.0, Span::dummy()).unwrap();
        assert_eq!(result, "ell");
    }

    #[test]
    fn test_char_at() {
        let result = char_at("hello", 0.0, Span::dummy()).unwrap();
        assert_eq!(result, "h");
    }

    #[test]
    fn test_repeat() {
        let result = repeat("ha", 3.0, Span::dummy()).unwrap();
        assert_eq!(result, "hahaha");
    }

    #[test]
    fn test_replace() {
        assert_eq!(replace("hello", "l", "L"), "heLlo");
    }

    #[test]
    fn test_pad_start() {
        let result = pad_start("5", 3.0, "0", Span::dummy()).unwrap();
        assert_eq!(result, "005");
    }

    #[test]
    fn test_pad_end() {
        let result = pad_end("5", 3.0, "0", Span::dummy()).unwrap();
        assert_eq!(result, "500");
    }

    #[test]
    fn test_starts_with() {
        assert!(starts_with("hello", "he"));
        assert!(!starts_with("hello", "x"));
    }

    #[test]
    fn test_ends_with() {
        assert!(ends_with("hello", "lo"));
        assert!(!ends_with("hello", "x"));
    }
}
