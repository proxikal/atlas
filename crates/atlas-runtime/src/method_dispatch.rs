//! Shared method dispatch table for interpreter/VM parity.
//!
//! Both the interpreter and compiler consult this module to map
//! (TypeTag, method_name) → stdlib function name.

use serde::{Deserialize, Serialize};

/// Runtime-stable type tag for method dispatch.
/// Mirrors the types that support method call syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeTag {
    JsonValue,
    Array,
    // Future types added here as stdlib phases add method support:
    // String, HashMap, HashSet, DateTime, Regex, ...
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match type_tag {
        TypeTag::JsonValue => Some(format!("json{}", capitalize_first(method_name))),
        TypeTag::Array => resolve_array_method(method_name),
    }
}

/// Resolve an array method call to its stdlib function name.
fn resolve_array_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Mutating methods — write back to receiver
        "push" => "arrayPush",
        "pop" => "arrayPop",
        "shift" => "arrayShift",
        "unshift" => "arrayUnshift",
        "reverse" => "arrayReverse",
        // Non-mutating — return new value, receiver unchanged
        "sort" => "arraySort",
        "len" => "len",
        "includes" => "arrayIncludes",
        "indexOf" => "arrayIndexOf",
        "lastIndexOf" => "arrayLastIndexOf",
        "slice" => "slice",
        "concat" => "concat",
        "flatten" => "flatten",
        "join" => "join",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Returns true if a stdlib function name is a mutating array method (returns modified collection).
pub fn is_array_mutating_collection(func_name: &str) -> bool {
    matches!(func_name, "arrayPush" | "arrayUnshift" | "arrayReverse")
}

/// Returns true if a stdlib function name is a mutating array method that returns a pair
/// `[extracted_value, new_array]` (pop/shift pattern).
pub fn is_array_mutating_pair(func_name: &str) -> bool {
    matches!(func_name, "arrayPop" | "arrayShift")
}

/// Capitalize first letter of each snake_case segment and join.
///
/// "as_string" → "AsString"
/// "is_null" → "IsNull"
fn capitalize_first(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_array_methods() {
        assert_eq!(
            resolve_method(TypeTag::Array, "push"),
            Some("arrayPush".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "pop"),
            Some("arrayPop".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "shift"),
            Some("arrayShift".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "unshift"),
            Some("arrayUnshift".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "reverse"),
            Some("arrayReverse".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "sort"),
            Some("arraySort".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::Array, "len"),
            Some("len".to_string())
        );
        assert_eq!(resolve_method(TypeTag::Array, "unknown"), None);
    }

    #[test]
    fn test_array_mutating_helpers() {
        assert!(is_array_mutating_collection("arrayPush"));
        assert!(is_array_mutating_collection("arrayUnshift"));
        assert!(is_array_mutating_collection("arrayReverse"));
        assert!(!is_array_mutating_collection("arrayPop"));
        assert!(!is_array_mutating_collection("arraySort"));

        assert!(is_array_mutating_pair("arrayPop"));
        assert!(is_array_mutating_pair("arrayShift"));
        assert!(!is_array_mutating_pair("arrayPush"));
    }

    #[test]
    fn test_resolve_json_methods() {
        assert_eq!(
            resolve_method(TypeTag::JsonValue, "as_string"),
            Some("jsonAsString".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::JsonValue, "as_number"),
            Some("jsonAsNumber".to_string())
        );
        assert_eq!(
            resolve_method(TypeTag::JsonValue, "is_null"),
            Some("jsonIsNull".to_string())
        );
    }
}
