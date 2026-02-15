//! Hash function infrastructure for Atlas collections
//!
//! Provides deterministic hashing for Atlas values using Rust's DefaultHasher.
//! Only primitive types (number, string, bool, null) are hashable.

use crate::span::Span;
use crate::value::{RuntimeError, Value};
use ordered_float::OrderedFloat;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Wrapper type for hashable Atlas values
///
/// Only Number, String, Bool, Null can be hashed.
/// Arrays, functions, JsonValue, Option, Result are not hashable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    /// Number value with IEEE 754 canonicalization
    Number(OrderedFloat<f64>),
    /// String value (reference-counted)
    String(Rc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
}

impl HashKey {
    /// Create HashKey from Value, returns error if not hashable
    ///
    /// # Errors
    /// Returns `RuntimeError::UnhashableType` if value cannot be hashed
    pub fn from_value(value: &Value, span: Span) -> Result<Self, RuntimeError> {
        match value {
            Value::Number(n) => {
                // Canonicalize NaN to ensure consistent hashing
                // All NaN values hash to the same value
                let normalized = if n.is_nan() { f64::NAN } else { *n };
                Ok(HashKey::Number(OrderedFloat(normalized)))
            }
            Value::String(s) => Ok(HashKey::String(Rc::clone(s))),
            Value::Bool(b) => Ok(HashKey::Bool(*b)),
            Value::Null => Ok(HashKey::Null),
            _ => Err(RuntimeError::UnhashableType {
                type_name: value.type_name().to_string(),
                span,
            }),
        }
    }

    /// Convert HashKey back to Value
    pub fn to_value(&self) -> Value {
        match self {
            HashKey::Number(n) => Value::Number(n.0),
            HashKey::String(s) => Value::String(Rc::clone(s)),
            HashKey::Bool(b) => Value::Bool(*b),
            HashKey::Null => Value::Null,
        }
    }
}

/// Compute deterministic hash for a HashKey
///
/// Uses Rust's DefaultHasher for reproducible hash values.
/// Same input always produces the same output (AI-friendly testing).
pub fn compute_hash(key: &HashKey) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_number() {
        let val = Value::Number(42.0);
        let key = HashKey::from_value(&val, Span::dummy()).unwrap();
        assert_eq!(key, HashKey::Number(OrderedFloat(42.0)));
    }

    #[test]
    fn test_hash_string() {
        let val = Value::string("hello");
        let key = HashKey::from_value(&val, Span::dummy()).unwrap();
        match key {
            HashKey::String(s) => assert_eq!(s.as_str(), "hello"),
            _ => panic!("Expected HashKey::String"),
        }
    }

    #[test]
    fn test_hash_bool() {
        let val = Value::Bool(true);
        let key = HashKey::from_value(&val, Span::dummy()).unwrap();
        assert_eq!(key, HashKey::Bool(true));
    }

    #[test]
    fn test_hash_null() {
        let val = Value::Null;
        let key = HashKey::from_value(&val, Span::dummy()).unwrap();
        assert_eq!(key, HashKey::Null);
    }

    #[test]
    fn test_hash_nan_canonicalization() {
        // All NaN values should hash to the same value
        let nan1 = Value::Number(f64::NAN);
        let nan2 = Value::Number(f64::NAN);
        let key1 = HashKey::from_value(&nan1, Span::dummy()).unwrap();
        let key2 = HashKey::from_value(&nan2, Span::dummy()).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_unhashable_array() {
        let val = Value::array(vec![Value::Number(1.0)]);
        let result = HashKey::from_value(&val, Span::dummy());
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::UnhashableType { type_name, .. } => {
                assert_eq!(type_name, "array");
            }
            _ => panic!("Expected UnhashableType error"),
        }
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let key = HashKey::Number(OrderedFloat(42.0));
        let hash1 = compute_hash(&key);
        let hash2 = compute_hash(&key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_values() {
        let key1 = HashKey::Number(OrderedFloat(1.0));
        let key2 = HashKey::Number(OrderedFloat(2.0));
        let hash1 = compute_hash(&key1);
        let hash2 = compute_hash(&key2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_key_to_value() {
        let key = HashKey::Number(OrderedFloat(42.0));
        let val = key.to_value();
        assert_eq!(val, Value::Number(42.0));
    }
}
