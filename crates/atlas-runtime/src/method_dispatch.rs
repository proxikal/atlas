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
    // Future types added here as stdlib phases add method support:
    // String, Array, HashMap, HashSet, DateTime, Regex, ...
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match type_tag {
        TypeTag::JsonValue => Some(format!("json{}", capitalize_first(method_name))),
    }
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
