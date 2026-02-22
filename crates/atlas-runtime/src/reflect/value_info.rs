//! Value inspection and introspection
//!
//! Provides runtime inspection of value contents, field access, and deep
//! introspection for nested structures.

use crate::value::Value;
use std::sync::Arc;

/// Value information for runtime introspection
#[derive(Debug, Clone)]
pub struct ValueInfo {
    /// The value being inspected
    value: Value,
}

impl ValueInfo {
    /// Create a new ValueInfo from a value
    pub fn new(value: Value) -> Self {
        ValueInfo { value }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &str {
        self.value.type_name()
    }

    /// Get the length of arrays or strings
    pub fn get_length(&self) -> Option<usize> {
        match &self.value {
            Value::Array(arr) => Some(arr.len()),
            Value::String(s) => Some(s.len()),
            _ => None,
        }
    }

    /// Check if a collection is empty
    pub fn is_empty(&self) -> bool {
        match &self.value {
            Value::Array(arr) => arr.is_empty(),
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }

    /// Get field names for object-like values
    /// Currently returns empty vec (no struct types yet)
    pub fn get_field_names(&self) -> Vec<String> {
        // TODO: Implement when struct types are added
        vec![]
    }

    /// Get a field value by name
    /// Currently returns None (no struct types yet)
    pub fn get_field(&self, _name: &str) -> Option<Value> {
        // TODO: Implement when struct types are added
        None
    }

    /// Check if a field exists
    /// Currently returns false (no struct types yet)
    pub fn has_field(&self, _name: &str) -> bool {
        // TODO: Implement when struct types are added
        false
    }

    /// Get array elements
    pub fn get_array_elements(&self) -> Option<Vec<Value>> {
        match &self.value {
            Value::Array(arr) => Some(arr.as_slice().to_vec()),
            _ => None,
        }
    }

    /// Get string value
    pub fn get_string(&self) -> Option<Arc<String>> {
        match &self.value {
            Value::String(s) => Some(Arc::clone(s)),
            _ => None,
        }
    }

    /// Get number value
    pub fn get_number(&self) -> Option<f64> {
        match &self.value {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get boolean value
    pub fn get_bool(&self) -> Option<bool> {
        match &self.value {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self.value, Value::Null)
    }

    /// Check if value is a function
    pub fn is_function(&self) -> bool {
        matches!(
            self.value,
            Value::Function(_) | Value::Builtin(_) | Value::NativeFunction(_)
        )
    }

    /// Check if value is an array
    pub fn is_array(&self) -> bool {
        matches!(self.value, Value::Array(_))
    }

    /// Check if value is a string
    pub fn is_string(&self) -> bool {
        matches!(self.value, Value::String(_))
    }

    /// Check if value is a number
    pub fn is_number(&self) -> bool {
        matches!(self.value, Value::Number(_))
    }

    /// Check if value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self.value, Value::Bool(_))
    }

    /// Get the underlying value
    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_info_type_name() {
        let info = ValueInfo::new(Value::Number(42.0));
        assert_eq!(info.type_name(), "number");

        let info = ValueInfo::new(Value::string("hello"));
        assert_eq!(info.type_name(), "string");
    }

    #[test]
    fn test_value_info_length() {
        let arr = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let info = ValueInfo::new(arr);
        assert_eq!(info.get_length(), Some(2));

        let str_val = Value::string("hello");
        let info = ValueInfo::new(str_val);
        assert_eq!(info.get_length(), Some(5));

        let num = Value::Number(42.0);
        let info = ValueInfo::new(num);
        assert_eq!(info.get_length(), None);
    }

    #[test]
    fn test_value_info_is_empty() {
        let arr = Value::array(vec![]);
        let info = ValueInfo::new(arr);
        assert!(info.is_empty());

        let arr = Value::array(vec![Value::Number(1.0)]);
        let info = ValueInfo::new(arr);
        assert!(!info.is_empty());
    }

    #[test]
    fn test_value_info_type_checks() {
        let num_info = ValueInfo::new(Value::Number(42.0));
        assert!(num_info.is_number());
        assert!(!num_info.is_string());
        assert!(!num_info.is_bool());

        let str_info = ValueInfo::new(Value::string("test"));
        assert!(str_info.is_string());
        assert!(!str_info.is_number());
    }

    #[test]
    fn test_value_info_get_values() {
        let num = Value::Number(42.5);
        let info = ValueInfo::new(num);
        assert_eq!(info.get_number(), Some(42.5));
        assert_eq!(info.get_string(), None);

        let bool_val = Value::Bool(true);
        let info = ValueInfo::new(bool_val);
        assert_eq!(info.get_bool(), Some(true));
        assert_eq!(info.get_number(), None);
    }

    #[test]
    fn test_value_info_is_null() {
        let null_info = ValueInfo::new(Value::Null);
        assert!(null_info.is_null());

        let num_info = ValueInfo::new(Value::Number(0.0));
        assert!(!num_info.is_null());
    }

    #[test]
    fn test_value_info_array_elements() {
        let arr = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let info = ValueInfo::new(arr);

        let elements = info.get_array_elements().unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0], Value::Number(1.0));
        assert_eq!(elements[1], Value::Number(2.0));
    }
}
