//! Reflection and introspection API
//!
//! Provides runtime type information and value inspection capabilities
//! for metaprogramming, serialization, and dynamic tooling.
//!
//! # Examples
//!
//! ```
//! use atlas_runtime::reflect::{TypeInfo, ValueInfo, get_value_type_info};
//! use atlas_runtime::value::Value;
//! use atlas_runtime::types::Type;
//!
//! // Get type information from a Type
//! let num_type = Type::Number;
//! let type_info = TypeInfo::from_type(&num_type);
//! assert_eq!(type_info.name, "number");
//! assert!(type_info.is_primitive());
//!
//! // Inspect a value
//! let value = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
//! let value_info = ValueInfo::new(value.clone());
//! assert_eq!(value_info.get_length(), Some(2));
//! assert!(value_info.is_array());
//!
//! // Get type info from a value
//! let type_info = get_value_type_info(&value);
//! assert_eq!(type_info.name, "array");
//! ```

pub mod type_info;
pub mod value_info;

pub use type_info::{FieldInfo, TypeInfo, TypeKind};
pub use value_info::ValueInfo;

use crate::types::Type;
use crate::value::Value;

/// Get TypeInfo from a runtime value
///
/// This function infers type information from a value at runtime.
/// For complex types like functions and arrays, it reconstructs the
/// type structure from the runtime representation.
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::get_value_type_info;
/// use atlas_runtime::value::Value;
///
/// let num = Value::Number(42.0);
/// let type_info = get_value_type_info(&num);
/// assert_eq!(type_info.name, "number");
///
/// let arr = Value::array(vec![Value::Number(1.0)]);
/// let type_info = get_value_type_info(&arr);
/// assert_eq!(type_info.name, "array");
/// ```
pub fn get_value_type_info(value: &Value) -> TypeInfo {
    match value {
        Value::Number(_) => TypeInfo::from_type(&Type::Number),
        Value::String(_) => TypeInfo::from_type(&Type::String),
        Value::Bool(_) => TypeInfo::from_type(&Type::Bool),
        Value::Null => TypeInfo::from_type(&Type::Null),

        Value::Array(_) => {
            // For arrays, we create a generic "array" type
            // We could inspect elements to determine element type,
            // but that would be expensive. For now, just report "array".
            TypeInfo {
                name: "array".to_string(),
                kind: TypeKind::Array,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None, // Could inspect first element if needed
                type_args: vec![],
            }
        }

        Value::Function(_) | Value::NativeFunction(_) => {
            // Functions at runtime don't carry full type information
            // Report generic "function" type
            TypeInfo {
                name: "function".to_string(),
                kind: TypeKind::Function,
                fields: vec![],
                parameters: vec![], // Runtime doesn't track param types
                return_type: None,  // Runtime doesn't track return type
                element_type: None,
                type_args: vec![],
            }
        }

        Value::JsonValue(_) => TypeInfo::from_type(&Type::JsonValue),

        Value::Option(_) => {
            // Generic Option type
            TypeInfo {
                name: "Option".to_string(),
                kind: TypeKind::Option,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect inner value
            }
        }

        Value::Result(_) => {
            // Generic Result type
            TypeInfo {
                name: "Result".to_string(),
                kind: TypeKind::Result,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect Ok/Err values
            }
        }

        Value::HashMap(_) => {
            // Generic HashMap type
            TypeInfo {
                name: "HashMap".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect keys/values
            }
        }

        Value::HashSet(_) => {
            // Generic HashSet type
            TypeInfo {
                name: "HashSet".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect elements
            }
        }

        Value::Queue(_) => {
            // Generic Queue type
            TypeInfo {
                name: "Queue".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect elements
            }
        }

        Value::Stack(_) => {
            // Generic Stack type
            TypeInfo {
                name: "Stack".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![], // Could inspect elements
            }
        }

        Value::Regex(_) => {
            // Regex type
            TypeInfo {
                name: "Regex".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
            }
        }

        Value::DateTime(_) => {
            // DateTime type
            TypeInfo {
                name: "DateTime".to_string(),
                kind: TypeKind::Generic,
                fields: vec![],
                parameters: vec![],
                return_type: None,
                element_type: None,
                type_args: vec![],
            }
        }
        Value::HttpRequest(_) => TypeInfo {
            name: "HttpRequest".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::HttpResponse(_) => TypeInfo {
            name: "HttpResponse".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::Future(_) => TypeInfo {
            name: "Future".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::TaskHandle(_) => TypeInfo {
            name: "TaskHandle".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::ChannelSender(_) => TypeInfo {
            name: "ChannelSender".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::ChannelReceiver(_) => TypeInfo {
            name: "ChannelReceiver".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
        Value::AsyncMutex(_) => TypeInfo {
            name: "AsyncMutex".to_string(),
            kind: TypeKind::Generic,
            fields: vec![],
            parameters: vec![],
            return_type: None,
            element_type: None,
            type_args: vec![],
        },
    }
}

/// Check if a value matches a given type
///
/// Performs runtime type checking to verify if a value is compatible
/// with the specified type.
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::value_is_type;
/// use atlas_runtime::value::Value;
/// use atlas_runtime::types::Type;
///
/// let num = Value::Number(42.0);
/// assert!(value_is_type(&num, &Type::Number));
/// assert!(!value_is_type(&num, &Type::String));
/// ```
pub fn value_is_type(value: &Value, ty: &Type) -> bool {
    match (value, ty) {
        (Value::Number(_), Type::Number) => true,
        (Value::String(_), Type::String) => true,
        (Value::Bool(_), Type::Bool) => true,
        (Value::Null, Type::Null) => true,
        (Value::Array(_), Type::Array(_)) => true, // Could check element types
        (Value::Function(_), Type::Function { .. }) => true,
        (Value::NativeFunction(_), Type::Function { .. }) => true,
        (Value::JsonValue(_), Type::JsonValue) => true,
        (Value::Option(_), Type::Generic { name, .. }) if name == "Option" => true,
        (Value::Result(_), Type::Generic { name, .. }) if name == "Result" => true,
        _ => false,
    }
}

/// Get a human-readable type name from a value
///
/// This is a convenience function that returns the type name string
/// directly from a value.
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::get_type_name;
/// use atlas_runtime::value::Value;
///
/// assert_eq!(get_type_name(&Value::Number(42.0)), "number");
/// assert_eq!(get_type_name(&Value::string("hello")), "string");
/// assert_eq!(get_type_name(&Value::Bool(true)), "bool");
/// ```
pub fn get_type_name(value: &Value) -> &str {
    value.type_name()
}

/// Check if a value is a primitive type
///
/// Primitive types are: number, string, bool, and null.
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::is_primitive_value;
/// use atlas_runtime::value::Value;
///
/// assert!(is_primitive_value(&Value::Number(42.0)));
/// assert!(is_primitive_value(&Value::string("test")));
/// assert!(is_primitive_value(&Value::Bool(false)));
/// assert!(is_primitive_value(&Value::Null));
/// assert!(!is_primitive_value(&Value::array(vec![])));
/// ```
pub fn is_primitive_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Number(_) | Value::String(_) | Value::Bool(_) | Value::Null
    )
}

/// Check if a value is callable (function or native function)
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::is_callable;
/// use atlas_runtime::value::Value;
///
/// assert!(!is_callable(&Value::Number(42.0)));
/// // Functions would return true
/// ```
pub fn is_callable(value: &Value) -> bool {
    matches!(value, Value::Function(_) | Value::NativeFunction(_))
}

/// Compare two values for type equality
///
/// Checks if two values have the same runtime type.
///
/// # Examples
///
/// ```
/// use atlas_runtime::reflect::same_type;
/// use atlas_runtime::value::Value;
///
/// let num1 = Value::Number(1.0);
/// let num2 = Value::Number(2.0);
/// let str1 = Value::string("hello");
///
/// assert!(same_type(&num1, &num2));
/// assert!(!same_type(&num1, &str1));
/// ```
pub fn same_type(a: &Value, b: &Value) -> bool {
    a.type_name() == b.type_name()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value_type_info_primitives() {
        let num = Value::Number(42.0);
        let info = get_value_type_info(&num);
        assert_eq!(info.name, "number");
        assert_eq!(info.kind, TypeKind::Number);

        let str_val = Value::string("test");
        let info = get_value_type_info(&str_val);
        assert_eq!(info.name, "string");
        assert_eq!(info.kind, TypeKind::String);
    }

    #[test]
    fn test_get_value_type_info_array() {
        let arr = Value::array(vec![Value::Number(1.0)]);
        let info = get_value_type_info(&arr);
        assert_eq!(info.name, "array");
        assert_eq!(info.kind, TypeKind::Array);
    }

    #[test]
    fn test_value_is_type() {
        let num = Value::Number(42.0);
        assert!(value_is_type(&num, &Type::Number));
        assert!(!value_is_type(&num, &Type::String));

        let arr = Value::array(vec![]);
        assert!(value_is_type(&arr, &Type::Array(Box::new(Type::Number))));
    }

    #[test]
    fn test_get_type_name() {
        assert_eq!(get_type_name(&Value::Number(1.0)), "number");
        assert_eq!(get_type_name(&Value::string("hi")), "string");
        assert_eq!(get_type_name(&Value::Bool(true)), "bool");
        assert_eq!(get_type_name(&Value::Null), "null");
    }

    #[test]
    fn test_is_primitive_value() {
        assert!(is_primitive_value(&Value::Number(1.0)));
        assert!(is_primitive_value(&Value::string("test")));
        assert!(is_primitive_value(&Value::Bool(true)));
        assert!(is_primitive_value(&Value::Null));
        assert!(!is_primitive_value(&Value::array(vec![])));
    }

    #[test]
    fn test_is_callable() {
        assert!(!is_callable(&Value::Number(42.0)));
        assert!(!is_callable(&Value::string("test")));
        assert!(!is_callable(&Value::array(vec![])));
        // Function values would return true
    }

    #[test]
    fn test_same_type() {
        let num1 = Value::Number(1.0);
        let num2 = Value::Number(999.0);
        let str1 = Value::string("hello");

        assert!(same_type(&num1, &num2));
        assert!(!same_type(&num1, &str1));
        assert!(same_type(&str1, &Value::string("world")));
    }
}
