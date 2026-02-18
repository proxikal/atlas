//! Type conversion between Rust and Atlas values
//!
//! Provides traits and implementations for bidirectional conversion:
//! - `ToAtlas` - Convert Rust types to Atlas `Value`
//! - `FromAtlas` - Convert Atlas `Value` to Rust types
//!
//! # Examples
//!
//! ```
//! use atlas_runtime::api::{ToAtlas, FromAtlas};
//! use atlas_runtime::Value;
//!
//! // Rust to Atlas
//! let atlas_value: Value = 42.0.to_atlas();
//! let atlas_string: Value = "hello".to_string().to_atlas();
//!
//! // Atlas to Rust
//! let rust_number: f64 = FromAtlas::from_atlas(&atlas_value).unwrap();
//! let rust_string: String = FromAtlas::from_atlas(&atlas_string).unwrap();
//! ```

use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Error type for value conversion failures
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionError {
    /// Type mismatch during conversion
    TypeMismatch { expected: String, found: String },
    /// Array element type mismatch
    ArrayElementTypeMismatch {
        index: usize,
        expected: String,
        found: String,
    },
    /// Object value type mismatch
    ObjectValueTypeMismatch {
        key: String,
        expected: String,
        found: String,
    },
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            ConversionError::ArrayElementTypeMismatch {
                index,
                expected,
                found,
            } => write!(
                f,
                "Array element type mismatch at index {}: expected {}, found {}",
                index, expected, found
            ),
            ConversionError::ObjectValueTypeMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "Object value type mismatch for key '{}': expected {}, found {}",
                key, expected, found
            ),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Trait for converting Atlas `Value` to Rust types
pub trait FromAtlas: Sized {
    /// Convert from Atlas `Value` to Rust type
    ///
    /// # Errors
    ///
    /// Returns `ConversionError` if the value cannot be converted to the target type.
    fn from_atlas(value: &Value) -> Result<Self, ConversionError>;
}

/// Trait for converting Rust types to Atlas `Value`
pub trait ToAtlas {
    /// Convert from Rust type to Atlas `Value`
    fn to_atlas(self) -> Value;
}

// Helper function to get type name for error messages
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Function(_) => "function",
        Value::Builtin(_) => "builtin",
        Value::NativeFunction(_) => "function",
        Value::JsonValue(_) => "json",
        Value::Option(_) => "option",
        Value::Result(_) => "result",
        Value::HashMap(_) => "hashmap",
        Value::HashSet(_) => "hashset",
        Value::Queue(_) => "queue",
        Value::Stack(_) => "stack",
        Value::Regex(_) => "regex",
        Value::DateTime(_) => "datetime",
        Value::HttpRequest(_) => "HttpRequest",
        Value::HttpResponse(_) => "HttpResponse",
        Value::Future(_) => "future",
        Value::TaskHandle(_) => "TaskHandle",
        Value::ChannelSender(_) => "ChannelSender",
        Value::ChannelReceiver(_) => "ChannelReceiver",
        Value::AsyncMutex(_) => "AsyncMutex",
    }
}

// Implementations for f64 (number)

impl FromAtlas for f64 {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Number(n) => Ok(*n),
            _ => Err(ConversionError::TypeMismatch {
                expected: "number".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for f64 {
    fn to_atlas(self) -> Value {
        Value::Number(self)
    }
}

// Implementations for String

impl FromAtlas for String {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::String(s) => Ok(s.to_string()),
            _ => Err(ConversionError::TypeMismatch {
                expected: "string".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for String {
    fn to_atlas(self) -> Value {
        Value::String(Arc::from(self))
    }
}

// Implementations for bool

impl FromAtlas for bool {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Bool(b) => Ok(*b),
            _ => Err(ConversionError::TypeMismatch {
                expected: "bool".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for bool {
    fn to_atlas(self) -> Value {
        Value::Bool(self)
    }
}

// Implementations for () (null)

impl FromAtlas for () {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Null => Ok(()),
            _ => Err(ConversionError::TypeMismatch {
                expected: "null".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl ToAtlas for () {
    fn to_atlas(self) -> Value {
        Value::Null
    }
}

// Implementations for Option<T>

impl<T: FromAtlas> FromAtlas for Option<T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Null => Ok(None),
            _ => Ok(Some(T::from_atlas(value)?)),
        }
    }
}

impl<T: ToAtlas> ToAtlas for Option<T> {
    fn to_atlas(self) -> Value {
        match self {
            None => Value::Null,
            Some(v) => v.to_atlas(),
        }
    }
}

// Implementations for Vec<T> (array)

impl<T: FromAtlas> FromAtlas for Vec<T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        match value {
            Value::Array(arr) => {
                let arr_borrow = arr.lock().unwrap();
                let mut result = Vec::with_capacity(arr_borrow.len());
                for (index, elem) in arr_borrow.iter().enumerate() {
                    match T::from_atlas(elem) {
                        Ok(converted) => result.push(converted),
                        Err(ConversionError::TypeMismatch { expected, found }) => {
                            return Err(ConversionError::ArrayElementTypeMismatch {
                                index,
                                expected,
                                found,
                            });
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ok(result)
            }
            _ => Err(ConversionError::TypeMismatch {
                expected: "array".to_string(),
                found: type_name(value).to_string(),
            }),
        }
    }
}

impl<T: ToAtlas> ToAtlas for Vec<T> {
    fn to_atlas(self) -> Value {
        let values: Vec<Value> = self.into_iter().map(|v| v.to_atlas()).collect();
        Value::Array(Arc::new(Mutex::new(values)))
    }
}

// Implementations for HashMap<String, T> (object)

impl<T: FromAtlas> FromAtlas for HashMap<String, T> {
    fn from_atlas(value: &Value) -> Result<Self, ConversionError> {
        // Atlas doesn't have a native object type (other than JsonValue)
        // This is primarily for JsonValue object conversion
        // For now, return type mismatch error
        Err(ConversionError::TypeMismatch {
            expected: "object".to_string(),
            found: type_name(value).to_string(),
        })
    }
}

impl<T: ToAtlas> ToAtlas for HashMap<String, T> {
    fn to_atlas(self) -> Value {
        // Atlas doesn't have a native object/map type
        // We can't convert HashMap to Value directly
        // This would require creating an array of key-value pairs or using JsonValue
        // For v0.2, we'll create a JsonValue object
        use crate::json_value::JsonValue as JV;

        let mut obj = HashMap::new();
        for (key, value) in self {
            // Convert value to Atlas Value first, then try to convert to JsonValue
            let atlas_value = value.to_atlas();
            let json_value = match atlas_value {
                Value::Null => JV::Null,
                Value::Bool(b) => JV::Bool(b),
                Value::Number(n) => JV::Number(n),
                Value::String(s) => JV::String(s.to_string()),
                Value::Array(arr) => {
                    // Convert array to JSON array
                    let arr_borrow = arr.lock().unwrap();
                    let json_arr: Vec<JV> = arr_borrow
                        .iter()
                        .map(|v| match v {
                            Value::Null => JV::Null,
                            Value::Bool(b) => JV::Bool(*b),
                            Value::Number(n) => JV::Number(*n),
                            Value::String(s) => JV::String(s.to_string()),
                            _ => JV::Null, // Can't convert functions, nested arrays, etc.
                        })
                        .collect();
                    JV::Array(json_arr)
                }
                _ => JV::Null, // Can't convert functions, JsonValue, etc.
            };
            obj.insert(key, json_value);
        }
        Value::JsonValue(Arc::new(JV::Object(obj)))
    }
}

// Convenience implementations for reference types

impl ToAtlas for &str {
    fn to_atlas(self) -> Value {
        Value::String(Arc::new(self.to_string()))
    }
}

impl ToAtlas for &String {
    fn to_atlas(self) -> Value {
        Value::String(Arc::new(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // f64 conversion tests

    #[test]
    fn test_f64_to_atlas() {
        let value = 42.0.to_atlas();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_f64_from_atlas() {
        let value = Value::Number(42.0);
        let result: f64 = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_f64_from_atlas_wrong_type() {
        let value = Value::String(Arc::new("hello".to_string()));
        let result: Result<f64, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::TypeMismatch { expected, found } => {
                assert_eq!(expected, "number");
                assert_eq!(found, "string");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    // String conversion tests

    #[test]
    fn test_string_to_atlas() {
        let value = "hello".to_string().to_atlas();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_string_from_atlas() {
        let value = Value::String(Arc::new("hello".to_string()));
        let result: String = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_string_from_atlas_wrong_type() {
        let value = Value::Number(42.0);
        let result: Result<String, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    // bool conversion tests

    #[test]
    fn test_bool_to_atlas() {
        let value = true.to_atlas();
        assert!(matches!(value, Value::Bool(true)));
    }

    #[test]
    fn test_bool_from_atlas() {
        let value = Value::Bool(true);
        let result: bool = FromAtlas::from_atlas(&value).unwrap();
        assert!(result);
    }

    #[test]
    fn test_bool_from_atlas_wrong_type() {
        let value = Value::Null;
        let result: Result<bool, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    // () (null) conversion tests

    #[test]
    fn test_unit_to_atlas() {
        let value = ().to_atlas();
        assert!(matches!(value, Value::Null));
    }

    #[test]
    fn test_unit_from_atlas() {
        let value = Value::Null;
        let result: () = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn test_unit_from_atlas_wrong_type() {
        let value = Value::Number(42.0);
        let result: Result<(), _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    // Option<T> conversion tests

    #[test]
    fn test_option_some_to_atlas() {
        let value = Some(42.0).to_atlas();
        assert!(matches!(value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_option_none_to_atlas() {
        let value: Option<f64> = None;
        let atlas_value = value.to_atlas();
        assert!(matches!(atlas_value, Value::Null));
    }

    #[test]
    fn test_option_some_from_atlas() {
        let value = Value::Number(42.0);
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, Some(42.0));
    }

    #[test]
    fn test_option_none_from_atlas() {
        let value = Value::Null;
        let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, None);
    }

    // Vec<T> conversion tests

    #[test]
    fn test_vec_to_atlas() {
        let vec = vec![1.0, 2.0, 3.0];
        let value = vec.to_atlas();
        match value {
            Value::Array(arr) => {
                let arr_borrow = arr.lock().unwrap();
                assert_eq!(arr_borrow.len(), 3);
                assert!(matches!(arr_borrow[0], Value::Number(n) if n == 1.0));
                assert!(matches!(arr_borrow[1], Value::Number(n) if n == 2.0));
                assert!(matches!(arr_borrow[2], Value::Number(n) if n == 3.0));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_vec_from_atlas() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let value = Value::Array(Arc::new(Mutex::new(arr)));
        let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vec_from_atlas_wrong_type() {
        let value = Value::Number(42.0);
        let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_from_atlas_element_type_mismatch() {
        let arr = vec![
            Value::Number(1.0),
            Value::String(Arc::new("oops".to_string())),
        ];
        let value = Value::Array(Arc::new(Mutex::new(arr)));
        let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::ArrayElementTypeMismatch {
                index,
                expected,
                found,
            } => {
                assert_eq!(index, 1);
                assert_eq!(expected, "number");
                assert_eq!(found, "string");
            }
            _ => panic!("Expected ArrayElementTypeMismatch error"),
        }
    }

    // Nested conversion tests

    #[test]
    fn test_nested_vec_option_string() {
        let data = vec![Some("hello".to_string()), None, Some("world".to_string())];
        let value = data.to_atlas();

        // Convert back
        let result: Vec<Option<String>> = FromAtlas::from_atlas(&value).unwrap();
        assert_eq!(result[0], Some("hello".to_string()));
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some("world".to_string()));
    }

    // HashMap conversion tests

    #[test]
    fn test_hashmap_to_atlas_creates_json_object() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Alice".to_string());
        map.insert("age".to_string(), "30".to_string());

        let value = map.to_atlas();
        assert!(matches!(value, Value::JsonValue(_)));
    }

    // &str convenience conversion tests

    #[test]
    fn test_str_ref_to_atlas() {
        let value = "hello".to_atlas();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_string_ref_to_atlas() {
        let s = "hello".to_string();
        let value = (&s).to_atlas();
        assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
    }
}
