//! JSON Value Type
//!
//! Isolated dynamic type for JSON interop. This is the ONLY exception to
//! Atlas's strict typing - designed specifically for API responses and config files.
//!
//! Design follows Rust's serde_json pattern:
//! - Natural indexing: data["user"]["name"]
//! - Explicit extraction: .as_string(), .as_number()
//! - Safe defaults: missing keys return Null, not errors
//!
//! CRITICAL: JsonValue is isolated - cannot be assigned to non-json variables
//! without explicit extraction.

use std::collections::HashMap;
use std::fmt;

/// JSON value type - isolated dynamic type for JSON interop only
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// JSON null
    Null,
    /// JSON boolean
    Bool(bool),
    /// JSON number (IEEE 754 double-precision)
    Number(f64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<JsonValue>),
    /// JSON object (key-value map)
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    /// Create a new JSON object
    pub fn object(map: HashMap<String, JsonValue>) -> Self {
        JsonValue::Object(map)
    }

    /// Create a new JSON array
    pub fn array(values: Vec<JsonValue>) -> Self {
        JsonValue::Array(values)
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Check if this value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Check if this value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Check if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Check if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Extract as boolean, returns None if not a bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Extract as number, returns None if not a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Extract as string reference, returns None if not a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract as array reference, returns None if not an array
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Extract as object reference, returns None if not an object
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Index into an object with a string key
    /// Returns JsonValue::Null if key doesn't exist or value is not an object
    pub fn index_str(&self, key: &str) -> JsonValue {
        match self {
            JsonValue::Object(obj) => obj.get(key).cloned().unwrap_or(JsonValue::Null),
            _ => JsonValue::Null,
        }
    }

    /// Index into an array with a numeric index
    /// Returns JsonValue::Null if index out of bounds or value is not an array
    pub fn index_num(&self, index: f64) -> JsonValue {
        // Convert f64 to usize (truncate, must be non-negative integer)
        if index < 0.0 || index.fract() != 0.0 {
            return JsonValue::Null;
        }

        let idx = index as usize;

        match self {
            JsonValue::Array(arr) => arr.get(idx).cloned().unwrap_or(JsonValue::Null),
            _ => JsonValue::Null,
        }
    }

    /// Get the length of an array or object
    /// Returns None if value is neither array nor object
    pub fn len(&self) -> Option<usize> {
        match self {
            JsonValue::Array(arr) => Some(arr.len()),
            JsonValue::Object(obj) => Some(obj.len()),
            _ => None,
        }
    }

    /// Check if array or object is empty
    /// Returns true for null and non-array/object types
    pub fn is_empty(&self) -> bool {
        match self {
            JsonValue::Array(arr) => arr.is_empty(),
            JsonValue::Object(obj) => obj.is_empty(),
            _ => true,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => {
                // Format numbers without trailing .0 for integers
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonValue::String(s) => write!(f, "\"{}\"", s),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, val) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_null() {
        let val = JsonValue::Null;
        assert!(val.is_null());
        assert!(!val.is_bool());
        assert!(!val.is_number());
        assert!(!val.is_string());
        assert!(!val.is_array());
        assert!(!val.is_object());
    }

    #[test]
    fn test_json_bool() {
        let val = JsonValue::Bool(true);
        assert!(!val.is_null());
        assert!(val.is_bool());
        assert_eq!(val.as_bool(), Some(true));
        assert_eq!(val.to_string(), "true");
    }

    #[test]
    fn test_json_number() {
        let val = JsonValue::Number(42.0);
        assert!(val.is_number());
        assert_eq!(val.as_number(), Some(42.0));
        assert_eq!(val.to_string(), "42");

        let val = JsonValue::Number(3.14);
        assert_eq!(val.to_string(), "3.14");
    }

    #[test]
    fn test_json_string() {
        let val = JsonValue::String("hello".to_string());
        assert!(val.is_string());
        assert_eq!(val.as_string(), Some("hello"));
        assert_eq!(val.to_string(), "\"hello\"");
    }

    #[test]
    fn test_json_array() {
        let val = JsonValue::array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert!(val.is_array());
        assert_eq!(val.len(), Some(3));
        assert_eq!(val.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_json_object() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Alice".to_string()));
        map.insert("age".to_string(), JsonValue::Number(30.0));

        let val = JsonValue::object(map);
        assert!(val.is_object());
        assert_eq!(val.len(), Some(2));
    }

    #[test]
    fn test_index_object() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Bob".to_string()));
        map.insert("age".to_string(), JsonValue::Number(25.0));

        let val = JsonValue::object(map);

        // Existing key
        let name = val.index_str("name");
        assert_eq!(name, JsonValue::String("Bob".to_string()));

        // Missing key returns Null
        let missing = val.index_str("missing");
        assert_eq!(missing, JsonValue::Null);

        // Non-object returns Null
        let num = JsonValue::Number(42.0);
        assert_eq!(num.index_str("key"), JsonValue::Null);
    }

    #[test]
    fn test_index_array() {
        let arr = JsonValue::array(vec![
            JsonValue::Number(10.0),
            JsonValue::Number(20.0),
            JsonValue::Number(30.0),
        ]);

        // Valid indices
        assert_eq!(arr.index_num(0.0), JsonValue::Number(10.0));
        assert_eq!(arr.index_num(1.0), JsonValue::Number(20.0));
        assert_eq!(arr.index_num(2.0), JsonValue::Number(30.0));

        // Out of bounds returns Null
        assert_eq!(arr.index_num(3.0), JsonValue::Null);
        assert_eq!(arr.index_num(100.0), JsonValue::Null);

        // Negative index returns Null
        assert_eq!(arr.index_num(-1.0), JsonValue::Null);

        // Fractional index returns Null
        assert_eq!(arr.index_num(1.5), JsonValue::Null);

        // Non-array returns Null
        let num = JsonValue::Number(42.0);
        assert_eq!(num.index_num(0.0), JsonValue::Null);
    }

    #[test]
    fn test_nested_indexing() {
        let mut user = HashMap::new();
        user.insert("name".to_string(), JsonValue::String("Charlie".to_string()));
        user.insert("age".to_string(), JsonValue::Number(35.0));

        let mut data = HashMap::new();
        data.insert("user".to_string(), JsonValue::object(user));

        let json = JsonValue::object(data);

        // Nested object access
        let user_obj = json.index_str("user");
        let name = user_obj.index_str("name");
        assert_eq!(name, JsonValue::String("Charlie".to_string()));

        // Missing nested key
        let missing = json.index_str("user").index_str("missing");
        assert_eq!(missing, JsonValue::Null);
    }

    #[test]
    fn test_extraction_methods() {
        let val = JsonValue::Bool(false);
        assert_eq!(val.as_bool(), Some(false));
        assert_eq!(val.as_number(), None);
        assert_eq!(val.as_string(), None);

        let val = JsonValue::Number(99.5);
        assert_eq!(val.as_number(), Some(99.5));
        assert_eq!(val.as_bool(), None);

        let val = JsonValue::String("test".to_string());
        assert_eq!(val.as_string(), Some("test"));
        assert_eq!(val.as_number(), None);
    }

    #[test]
    fn test_is_empty() {
        assert!(JsonValue::Null.is_empty());
        assert!(!JsonValue::array(vec![JsonValue::Null]).is_empty());
        assert!(JsonValue::array(vec![]).is_empty());
        assert!(JsonValue::object(HashMap::new()).is_empty());
    }
}
