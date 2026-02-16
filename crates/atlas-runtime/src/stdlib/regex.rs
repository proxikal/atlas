//! Regular expression operations
//!
//! This module provides regex pattern matching, searching, and capture group extraction
//! using the Rust `regex` crate for efficient pattern compilation and matching.
//!
//! # Functions
//! - `regexNew(pattern: string) -> Result<Regex, string>` - Compile a regex pattern
//! - `regexNewWithFlags(pattern: string, flags: string) -> Result<Regex, string>` - Compile with flags
//! - `regexEscape(text: string) -> string` - Escape special regex characters
//! - `regexIsMatch(regex: Regex, text: string) -> boolean` - Test if pattern matches
//! - `regexFind(regex: Regex, text: string) -> Option<HashMap>` - Find first match
//! - `regexFindAll(regex: Regex, text: string) -> Array<HashMap>` - Find all matches
//! - `regexCaptures(regex: Regex, text: string) -> Option<Array>` - Extract capture groups by index
//! - `regexCapturesNamed(regex: Regex, text: string) -> Option<HashMap>` - Extract named capture groups

use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::collections::hashmap::AtlasHashMap;
use crate::value::{RuntimeError, Value};
use regex::{Regex, RegexBuilder};
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// Construction Functions
// ============================================================================

/// Compile a regular expression pattern
///
/// # Arguments
/// - `pattern`: The regex pattern string
///
/// # Returns
/// - `Ok(Regex)` if the pattern compiles successfully
/// - `Err(string)` with error message if the pattern is invalid
///
/// # Example
/// ```atlas
/// let pattern = regexNew("\\d+");
/// ```
pub fn regex_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let pattern_str = expect_string(&args[0], span, "pattern")?;

    match Regex::new(pattern_str) {
        Ok(regex) => {
            let regex_value = Value::Regex(Rc::new(regex));
            // Return Result::Ok(regex)
            Ok(Value::Result(Ok(Box::new(regex_value))))
        }
        Err(err) => {
            // Return Result::Err(error message)
            let err_msg = Value::string(err.to_string());
            Ok(Value::Result(Err(Box::new(err_msg))))
        }
    }
}

/// Compile a regular expression with flags
///
/// # Arguments
/// - `pattern`: The regex pattern string
/// - `flags`: String containing flag characters (i, m, s, x)
///   - `i` - case insensitive
///   - `m` - multi-line mode (^ and $ match line boundaries)
///   - `s` - dot matches newline
///   - `x` - extended mode (ignore whitespace and allow comments)
///
/// # Returns
/// - `Ok(Regex)` if the pattern compiles successfully
/// - `Err(string)` with error message if the pattern is invalid or flags are invalid
///
/// # Example
/// ```atlas
/// let pattern = regexNewWithFlags("hello", "i"); // Case-insensitive
/// ```
pub fn regex_new_with_flags(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let pattern_str = expect_string(&args[0], span, "pattern")?;
    let flags_str = expect_string(&args[1], span, "flags")?;

    let mut builder = RegexBuilder::new(pattern_str);

    // Parse flags
    for ch in flags_str.chars() {
        match ch {
            'i' => {
                builder.case_insensitive(true);
            }
            'm' => {
                builder.multi_line(true);
            }
            's' => {
                builder.dot_matches_new_line(true);
            }
            'x' => {
                builder.ignore_whitespace(true);
            }
            _ => {
                let err_msg = Value::string(format!("Invalid regex flag: '{}'", ch));
                return Ok(Value::Result(Err(Box::new(err_msg))));
            }
        }
    }

    match builder.build() {
        Ok(regex) => {
            let regex_value = Value::Regex(Rc::new(regex));
            Ok(Value::Result(Ok(Box::new(regex_value))))
        }
        Err(err) => {
            let err_msg = Value::string(err.to_string());
            Ok(Value::Result(Err(Box::new(err_msg))))
        }
    }
}

/// Escape special regex characters in a string
///
/// Escapes all regex metacharacters: . * + ? ^ $ ( ) [ ] { } | \
///
/// # Arguments
/// - `text`: The string to escape
///
/// # Returns
/// - String with all regex metacharacters escaped
///
/// # Example
/// ```atlas
/// let escaped = regexEscape("hello.world"); // Returns "hello\\.world"
/// ```
pub fn regex_escape(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let text = expect_string(&args[0], span, "text")?;
    let escaped = regex::escape(text);

    Ok(Value::string(escaped))
}

// ============================================================================
// Pattern Matching Functions
// ============================================================================

/// Test if a string matches a regex pattern
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to test
///
/// # Returns
/// - `true` if the pattern matches anywhere in the text
/// - `false` if no match is found
///
/// # Example
/// ```atlas
/// let pattern = regexNew("\\d+").unwrap();
/// let matches = regexIsMatch(pattern, "hello123"); // Returns true
/// ```
pub fn regex_is_match(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    let is_match = regex.is_match(text);

    Ok(Value::Bool(is_match))
}

/// Find the first match of a pattern in a string
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to search
///
/// # Returns
/// - `Some(HashMap)` with keys: `text` (matched text), `start` (byte index), `end` (byte index)
/// - `None` if no match is found
///
/// # Example
/// ```atlas
/// let pattern = regexNew("\\d+").unwrap();
/// let match_data = regexFind(pattern, "hello123world");
/// // Returns Some({ text: "123", start: 5, end: 8 })
/// ```
pub fn regex_find(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    if let Some(mat) = regex.find(text) {
        let mut map = AtlasHashMap::new();
        map.insert(
            HashKey::String(Rc::new("text".to_string())),
            Value::string(mat.as_str()),
        );
        map.insert(
            HashKey::String(Rc::new("start".to_string())),
            Value::Number(mat.start() as f64),
        );
        map.insert(
            HashKey::String(Rc::new("end".to_string())),
            Value::Number(mat.end() as f64),
        );

        let hashmap_value = Value::HashMap(Rc::new(RefCell::new(map)));
        Ok(Value::Option(Some(Box::new(hashmap_value))))
    } else {
        Ok(Value::Option(None))
    }
}

/// Find all matches of a pattern in a string
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to search
///
/// # Returns
/// - Array of HashMaps, each with keys: `text`, `start`, `end`
/// - Empty array if no matches are found
///
/// # Example
/// ```atlas
/// let pattern = regexNew("\\d+").unwrap();
/// let matches = regexFindAll(pattern, "a1 b22 c333");
/// // Returns [{ text: "1", start: 1, end: 2 }, { text: "22", start: 4, end: 6 }, ...]
/// ```
pub fn regex_find_all(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    let mut matches = Vec::new();

    for mat in regex.find_iter(text) {
        let mut map = AtlasHashMap::new();
        map.insert(
            HashKey::String(Rc::new("text".to_string())),
            Value::string(mat.as_str()),
        );
        map.insert(
            HashKey::String(Rc::new("start".to_string())),
            Value::Number(mat.start() as f64),
        );
        map.insert(
            HashKey::String(Rc::new("end".to_string())),
            Value::Number(mat.end() as f64),
        );

        matches.push(Value::HashMap(Rc::new(RefCell::new(map))));
    }

    Ok(Value::array(matches))
}

// ============================================================================
// Capture Group Functions
// ============================================================================

/// Extract capture groups from the first match
///
/// # Arguments
/// - `regex`: The compiled regex pattern (with capture groups)
/// - `text`: The string to match against
///
/// # Returns
/// - `Some(Array)` where index 0 is the full match, index 1+ are capture groups
/// - `None` if no match is found
///
/// # Example
/// ```atlas
/// let pattern = regexNew("(\\d+)-(\\w+)").unwrap();
/// let groups = regexCaptures(pattern, "123-abc");
/// // Returns Some(["123-abc", "123", "abc"])
/// ```
pub fn regex_captures(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    if let Some(caps) = regex.captures(text) {
        let mut groups = Vec::new();

        // Index 0 is the full match, 1+ are capture groups
        for i in 0..caps.len() {
            if let Some(group) = caps.get(i) {
                groups.push(Value::string(group.as_str()));
            } else {
                // Optional group that didn't match
                groups.push(Value::Null);
            }
        }

        Ok(Value::Option(Some(Box::new(Value::array(groups)))))
    } else {
        Ok(Value::Option(None))
    }
}

/// Extract named capture groups from the first match
///
/// # Arguments
/// - `regex`: The compiled regex pattern (with named capture groups)
/// - `text`: The string to match against
///
/// # Returns
/// - `Some(HashMap)` with named group mappings
/// - `None` if no match is found
///
/// # Example
/// ```atlas
/// let pattern = regexNew("(?P<num>\\d+)-(?P<word>\\w+)").unwrap();
/// let groups = regexCapturesNamed(pattern, "123-abc");
/// // Returns Some({ num: "123", word: "abc" })
/// ```
pub fn regex_captures_named(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    if let Some(caps) = regex.captures(text) {
        let mut map = AtlasHashMap::new();

        // Iterate over named groups
        for name in regex.capture_names().flatten() {
            let key = HashKey::String(Rc::new(name.to_string()));
            if let Some(group) = caps.name(name) {
                map.insert(key, Value::string(group.as_str()));
            } else {
                // Named group exists but didn't match
                map.insert(key, Value::Null);
            }
        }

        let hashmap_value = Value::HashMap(Rc::new(RefCell::new(map)));
        Ok(Value::Option(Some(Box::new(hashmap_value))))
    } else {
        Ok(Value::Option(None))
    }
}

// ============================================================================
// Replacement Functions
// ============================================================================

/// Replace the first match of a pattern with a replacement string
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to search
/// - `replacement`: The replacement string (supports capture group references)
///
/// # Replacement Syntax
/// - `$1`, `$2`, etc. - Numbered capture groups
/// - `$&` - The entire match
/// - `$`` - Everything before the match
/// - `$'` - Everything after the match
///
/// # Returns
/// - String with the first match replaced
///
/// # Example
/// ```atlas
/// let pattern = regexNew("(\\d+)").unwrap();
/// let result = regexReplace(pattern, "a1b2c3", "[$1]");
/// // Returns "a[1]b2c3"
/// ```
pub fn regex_replace(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;
    let replacement = expect_string(&args[2], span, "replacement")?;

    let result = regex.replace(text, replacement);

    Ok(Value::string(result.into_owned()))
}

/// Replace all matches of a pattern with a replacement string
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to search
/// - `replacement`: The replacement string (supports capture group references)
///
/// # Replacement Syntax
/// - `$1`, `$2`, etc. - Numbered capture groups
/// - `$&` - The entire match
/// - `$`` - Everything before the match
/// - `$'` - Everything after the match
///
/// # Returns
/// - String with all matches replaced
///
/// # Example
/// ```atlas
/// let pattern = regexNew("(\\d+)").unwrap();
/// let result = regexReplaceAll(pattern, "a1b2c3", "[$1]");
/// // Returns "a[1]b[2]c[3]"
/// ```
pub fn regex_replace_all(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;
    let replacement = expect_string(&args[2], span, "replacement")?;

    let result = regex.replace_all(text, replacement);

    Ok(Value::string(result.into_owned()))
}

// NOTE: regexReplaceWith and regexReplaceAllWith are implemented as intrinsics
// in the interpreter and VM (see interpreter/expr.rs and vm/mod.rs)

// ============================================================================
// String Splitting Functions
// ============================================================================

/// Split a string at regex matches
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to split
///
/// # Returns
/// - Array of substrings (empty strings included)
///
/// # Example
/// ```atlas
/// let pattern = unwrap(regexNew(","));
/// regexSplit(pattern, "a,b,,c") // Returns ["a", "b", "", "c"]
/// ```
pub fn regex_split(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    let parts: Vec<Value> = regex.split(text).map(Value::string).collect();

    Ok(Value::array(parts))
}

/// Split a string at regex matches with a limit
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to split
/// - `limit`: Maximum number of splits (0 = no limit)
///
/// # Returns
/// - Array of substrings (empty strings included)
/// - If limit is 0, returns empty array
///
/// # Example
/// ```atlas
/// let pattern = unwrap(regexNew(","));
/// regexSplitN(pattern, "a,b,c,d", 2) // Returns ["a", "b", "c,d"]
/// ```
pub fn regex_split_n(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;
    let limit = match &args[2] {
        Value::Number(n) => *n as usize,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Expected number for limit".to_string(),
                span,
            })
        }
    };

    if limit == 0 {
        return Ok(Value::array(vec![]));
    }

    let parts: Vec<Value> = regex.splitn(text, limit + 1).map(Value::string).collect();

    Ok(Value::array(parts))
}

// ============================================================================
// Advanced Features
// ============================================================================

/// Get all match positions as [start, end] pairs
///
/// # Arguments
/// - `regex`: The compiled regex pattern
/// - `text`: The string to search
///
/// # Returns
/// - Array of [start, end] arrays (byte indices)
///
/// # Example
/// ```atlas
/// let pattern = unwrap(regexNew("\\d+"));
/// regexMatchIndices(pattern, "a1b22c") // Returns [[1, 2], [3, 5]]
/// ```
pub fn regex_match_indices(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let regex = expect_regex(&args[0], span)?;
    let text = expect_string(&args[1], span, "text")?;

    let indices: Vec<Value> = regex
        .find_iter(text)
        .map(|mat| {
            Value::array(vec![
                Value::Number(mat.start() as f64),
                Value::Number(mat.end() as f64),
            ])
        })
        .collect();

    Ok(Value::array(indices))
}

/// Convenience function: compile and test in one call
///
/// # Arguments
/// - `pattern`: The regex pattern string
/// - `text`: The string to test
///
/// # Returns
/// - `true` if pattern matches, `false` if no match or compile error
///
/// # Example
/// ```atlas
/// regexTest("\\d+", "hello123") // Returns true
/// regexTest("[invalid", "test") // Returns false (compile error)
/// ```
pub fn regex_test(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let pattern_str = expect_string(&args[0], span, "pattern")?;
    let text = expect_string(&args[1], span, "text")?;

    match Regex::new(pattern_str) {
        Ok(regex) => Ok(Value::Bool(regex.is_match(text))),
        Err(_) => Ok(Value::Bool(false)), // Return false on compile error
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Expect a string argument
fn expect_string<'a>(
    value: &'a Value,
    span: Span,
    arg_name: &str,
) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(RuntimeError::TypeError {
            msg: format!(
                "Expected string for {}, got {}",
                arg_name,
                value.type_name()
            ),
            span,
        }),
    }
}

/// Expect a regex argument
fn expect_regex(value: &Value, span: Span) -> Result<&Regex, RuntimeError> {
    match value {
        Value::Regex(r) => Ok(r.as_ref()),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected regex, got {}", value.type_name()),
            span,
        }),
    }
}
