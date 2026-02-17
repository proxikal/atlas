//! Type error fix suggestions
//!
//! Provides actionable suggestions for common type errors to improve
//! the developer experience with clear, helpful fix recommendations.

use crate::types::Type;

/// Suggest a fix for a type mismatch (expected vs found).
///
/// Returns `Some(suggestion)` if a common fix is known, `None` otherwise.
pub fn suggest_type_mismatch(expected: &Type, found: &Type) -> Option<String> {
    let expected_norm = expected.normalized();
    let found_norm = found.normalized();
    match (&expected_norm, &found_norm) {
        // number expected, string found → suggest num()
        (Type::Number, Type::String) => {
            Some("convert the string to a number with `num(value)`".to_string())
        }
        // string expected, number found → suggest str()
        (Type::String, Type::Number) => {
            Some("convert the number to a string with `str(value)`".to_string())
        }
        // bool expected, number found → suggest comparison
        (Type::Bool, Type::Number) => {
            Some("use a comparison operator: `value > 0`, `value == 0`, etc.".to_string())
        }
        // bool expected, string found → suggest comparison
        (Type::Bool, Type::String) => {
            Some("use a comparison: `value == \"expected\"` or `len(value) > 0`".to_string())
        }
        // number expected, bool found → suggest ternary/if
        (Type::Number, Type::Bool) => Some(
            "use a conditional expression to convert: `if (value) { 1 } else { 0 }`".to_string(),
        ),
        // Array expected, non-array → suggest wrapping
        (Type::Array(_), other) if !matches!(other, Type::Array(_)) => {
            Some("wrap the value in an array: `[value]`".to_string())
        }
        // Non-array expected, array found → suggest indexing
        (_, Type::Array(elem)) if elem.is_assignable_to(expected) => {
            Some("use an index to access an element: `arr[0]`".to_string())
        }
        _ => None,
    }
}

/// Suggest a fix for a binary operator type error.
pub fn suggest_binary_operator_fix(op: &str, left: &Type, right: &Type) -> Option<String> {
    let left_norm = left.normalized();
    let right_norm = right.normalized();
    match (op, &left_norm, &right_norm) {
        // number + string → suggest str() conversion
        ("+", Type::Number, Type::String) => {
            Some("convert the number to string first: `str(left) + right`".to_string())
        }
        // string + number → suggest str() conversion
        ("+", Type::String, Type::Number) => {
            Some("convert the number to string first: `left + str(right)`".to_string())
        }
        // string - string → suggest wrong operator
        ("-" | "*" | "/" | "%", Type::String, Type::String) => Some(
            "arithmetic operators only work with numbers; for strings, use string methods"
                .to_string(),
        ),
        // Comparison with different types
        ("==" | "!=", _, _) if left_norm != right_norm => Some(format!(
            "both sides must have the same type; found {} and {}",
            left.display_name(),
            right.display_name()
        )),
        // Logical operators with non-bool
        ("and" | "or", _, _) => {
            let mut fixes = Vec::new();
            if *left != Type::Bool {
                fixes.push(format!(
                    "convert left operand from {} to bool",
                    left.display_name()
                ));
            }
            if *right != Type::Bool {
                fixes.push(format!(
                    "convert right operand from {} to bool",
                    right.display_name()
                ));
            }
            if fixes.is_empty() {
                None
            } else {
                Some(fixes.join("; "))
            }
        }
        _ => None,
    }
}

/// Suggest a fix when a condition is not bool.
pub fn suggest_condition_fix(found: &Type) -> String {
    let found_norm = found.normalized();
    match &found_norm {
        Type::Number => "use a comparison: `value != 0` or `value > 0`".to_string(),
        Type::String => "use a comparison: `len(value) > 0` or `value != \"\"`".to_string(),
        Type::Null => "null is not a boolean; use an explicit check".to_string(),
        Type::Array(_) => "use `len(arr) > 0` to check if array is non-empty".to_string(),
        _ => format!(
            "expected bool, found {}; use a comparison to get a bool",
            found.display_name()
        ),
    }
}

/// Suggest a fix for calling a non-function.
pub fn suggest_not_callable(found: &Type) -> String {
    let found_norm = found.normalized();
    match &found_norm {
        Type::String => {
            "strings are not callable; did you mean to use a string method?".to_string()
        }
        Type::Number => "numbers are not callable; remove the parentheses".to_string(),
        Type::Array(_) => {
            "arrays are not callable; use indexing `arr[i]` to access elements".to_string()
        }
        _ => format!(
            "type {} is not callable; expected a function",
            found.display_name()
        ),
    }
}

/// Suggest a fix for wrong number of arguments.
pub fn suggest_arity_fix(expected: usize, found: usize, func_type: &Type) -> String {
    let direction = if found > expected { "remove" } else { "add" };
    let diff = found.abs_diff(expected);

    format!(
        "{} {} argument{}; function signature: {}",
        direction,
        diff,
        if diff == 1 { "" } else { "s" },
        func_type.display_name()
    )
}

/// Suggest a fix for return type mismatch.
pub fn suggest_return_fix(expected: &Type, found: &Type) -> String {
    if *found == Type::Void && *expected != Type::Void {
        return format!(
            "add a return value of type {}; missing return statement?",
            expected.display_name()
        );
    }

    if let Some(conversion) = suggest_type_mismatch(expected, found) {
        return format!("in return value: {}", conversion);
    }

    format!(
        "change the return expression to type {} or update the function's return type to {}",
        expected.display_name(),
        found.display_name()
    )
}

/// Suggest a fix for immutable variable assignment.
pub fn suggest_mutability_fix(var_name: &str) -> String {
    format!(
        "declare '{}' as mutable: `var {} = ...` instead of `let {} = ...`",
        var_name, var_name, var_name
    )
}

/// Find the most similar name from a list of known names (for typo suggestions).
///
/// Uses Levenshtein distance. Returns `None` if no name is close enough.
pub fn suggest_similar_name<'a>(
    unknown: &str,
    known_names: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let max_distance = match unknown.len() {
        0..=2 => 1,
        3..=5 => 2,
        _ => 3,
    };

    let mut best: Option<(&str, usize)> = None;

    for name in known_names {
        let dist = levenshtein_distance(unknown, name);
        if dist <= max_distance && (best.is_none() || dist < best.unwrap().1) {
            best = Some((name, dist));
        }
    }

    best.map(|(name, _)| format!("did you mean '{}'?", name))
}

/// Compute Levenshtein edit distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for (j, item) in prev.iter_mut().enumerate().take(n + 1) {
        *item = j;
    }

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Suggest a fix for for-in with non-array.
pub fn suggest_for_in_fix(found: &Type) -> String {
    match found {
        Type::String => {
            "strings are not directly iterable; use `split(str, \"\")` to iterate over characters"
                .to_string()
        }
        Type::Number => {
            "numbers are not iterable; use `range(0, n)` to iterate over a range".to_string()
        }
        _ => format!(
            "for-in requires an array, found {}; wrap in an array or use a different loop",
            found.display_name()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_number_string_mismatch() {
        let suggestion = suggest_type_mismatch(&Type::Number, &Type::String);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("num("));
    }

    #[test]
    fn test_suggest_string_number_mismatch() {
        let suggestion = suggest_type_mismatch(&Type::String, &Type::Number);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("str("));
    }

    #[test]
    fn test_suggest_bool_from_number() {
        let suggestion = suggest_type_mismatch(&Type::Bool, &Type::Number);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("comparison"));
    }

    #[test]
    fn test_suggest_wrap_in_array() {
        let suggestion = suggest_type_mismatch(&Type::Array(Box::new(Type::Number)), &Type::Number);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("[value]"));
    }

    #[test]
    fn test_suggest_binary_string_plus_number() {
        let suggestion = suggest_binary_operator_fix("+", &Type::String, &Type::Number);
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("str("));
    }

    #[test]
    fn test_suggest_condition_number() {
        let suggestion = suggest_condition_fix(&Type::Number);
        assert!(suggestion.contains("!="));
    }

    #[test]
    fn test_suggest_similar_name() {
        let names = vec!["print", "println", "len", "str"];
        let suggestion = suggest_similar_name("prnt", names.iter().copied());
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("print"));
    }

    #[test]
    fn test_suggest_similar_name_no_match() {
        let names = vec!["print", "len", "str"];
        let suggestion = suggest_similar_name("xyzabc", names.iter().copied());
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_suggest_return_missing_value() {
        let suggestion = suggest_return_fix(&Type::Number, &Type::Void);
        assert!(suggestion.contains("missing return"));
    }

    #[test]
    fn test_suggest_mutability() {
        let suggestion = suggest_mutability_fix("x");
        assert!(suggestion.contains("var x"));
    }

    #[test]
    fn test_suggest_not_callable_string() {
        let suggestion = suggest_not_callable(&Type::String);
        assert!(suggestion.contains("not callable"));
    }

    #[test]
    fn test_suggest_for_in_string() {
        let suggestion = suggest_for_in_fix(&Type::String);
        assert!(suggestion.contains("split"));
    }

    #[test]
    fn test_suggest_arity_too_many() {
        let func = Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Void),
        };
        let suggestion = suggest_arity_fix(1, 3, &func);
        assert!(suggestion.contains("remove"));
        assert!(suggestion.contains("2"));
    }

    #[test]
    fn test_suggest_arity_too_few() {
        let func = Type::Function {
            type_params: vec![],
            params: vec![Type::Number, Type::String],
            return_type: Box::new(Type::Void),
        };
        let suggestion = suggest_arity_fix(2, 0, &func);
        assert!(suggestion.contains("add"));
    }
}
