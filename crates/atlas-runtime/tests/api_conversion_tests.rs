//! Integration tests for value conversion API
//!
//! Tests ToAtlas and FromAtlas traits for bidirectional conversion
//! between Rust and Atlas types.

use atlas_runtime::api::{ConversionError, FromAtlas, ToAtlas};
use atlas_runtime::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// f64 Conversion Tests

#[test]
fn test_f64_to_atlas() {
    let value = 42.5.to_atlas();
    assert!(matches!(value, Value::Number(n) if n == 42.5));
}

#[test]
fn test_f64_from_atlas_success() {
    let value = Value::Number(42.5);
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, 42.5);
}

#[test]
fn test_f64_from_atlas_type_mismatch() {
    let value = Value::String(Rc::new("hello".to_string()));
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

#[test]
fn test_f64_zero() {
    let value = 0.0.to_atlas();
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn test_f64_negative() {
    let value = (-123.456).to_atlas();
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, -123.456);
}

#[test]
fn test_f64_large_number() {
    let value = 1.7976931348623157e308.to_atlas();
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, 1.7976931348623157e308);
}

// String Conversion Tests

#[test]
fn test_string_to_atlas() {
    let value = "hello world".to_string().to_atlas();
    assert!(matches!(value, Value::String(s) if s.as_ref() == "hello world"));
}

#[test]
fn test_string_from_atlas_success() {
    let value = Value::String(Rc::new("hello".to_string()));
    let result: String = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_string_from_atlas_type_mismatch() {
    let value = Value::Number(42.0);
    let result: Result<String, _> = FromAtlas::from_atlas(&value);
    assert!(result.is_err());
}

#[test]
fn test_string_empty() {
    let value = String::new().to_atlas();
    let result: String = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_string_unicode() {
    let value = "Hello, 世界! 🌍".to_string().to_atlas();
    let result: String = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, "Hello, 世界! 🌍");
}

#[test]
fn test_string_ref_to_atlas() {
    let s = "test";
    let value = s.to_atlas();
    assert!(matches!(value, Value::String(ref rs) if rs.as_ref() == "test"));
}

// bool Conversion Tests

#[test]
fn test_bool_true_to_atlas() {
    let value = true.to_atlas();
    assert!(matches!(value, Value::Bool(true)));
}

#[test]
fn test_bool_false_to_atlas() {
    let value = false.to_atlas();
    assert!(matches!(value, Value::Bool(false)));
}

#[test]
fn test_bool_from_atlas_true() {
    let value = Value::Bool(true);
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert!(result);
}

#[test]
fn test_bool_from_atlas_false() {
    let value = Value::Bool(false);
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert!(!result);
}

#[test]
fn test_bool_from_atlas_type_mismatch() {
    let value = Value::Null;
    let result: Result<bool, _> = FromAtlas::from_atlas(&value);
    assert!(result.is_err());
}

// () (null) Conversion Tests

#[test]
fn test_unit_to_atlas() {
    let value = ().to_atlas();
    assert!(matches!(value, Value::Null));
}

#[test]
fn test_unit_from_atlas_success() {
    let value = Value::Null;
    let result: () = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, ());
}

#[test]
fn test_unit_from_atlas_type_mismatch() {
    let value = Value::Number(0.0);
    let result: Result<(), _> = FromAtlas::from_atlas(&value);
    assert!(result.is_err());
}

// Option<T> Conversion Tests

#[test]
fn test_option_some_number_to_atlas() {
    let value = Some(42.0).to_atlas();
    assert!(matches!(value, Value::Number(n) if n == 42.0));
}

#[test]
fn test_option_some_string_to_atlas() {
    let value = Some("hello".to_string()).to_atlas();
    assert!(matches!(value, Value::String(s) if s.as_ref() == "hello"));
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

#[test]
fn test_option_string_some_from_atlas() {
    let value = Value::String(Rc::new("test".to_string()));
    let result: Option<String> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, Some("test".to_string()));
}

#[test]
fn test_option_string_none_from_atlas() {
    let value = Value::Null;
    let result: Option<String> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, None);
}

// Vec<T> Conversion Tests

#[test]
fn test_vec_f64_to_atlas() {
    let vec = vec![1.0, 2.0, 3.0];
    let value = vec.to_atlas();
    match value {
        Value::Array(arr) => {
            let arr_borrow = arr.borrow();
            assert_eq!(arr_borrow.len(), 3);
            assert!(matches!(arr_borrow[0], Value::Number(n) if n == 1.0));
            assert!(matches!(arr_borrow[1], Value::Number(n) if n == 2.0));
            assert!(matches!(arr_borrow[2], Value::Number(n) if n == 3.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_vec_f64_from_atlas() {
    let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let value = Value::Array(Rc::new(RefCell::new(arr)));
    let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_vec_string_to_atlas() {
    let vec = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let value = vec.to_atlas();
    match value {
        Value::Array(arr) => {
            let arr_borrow = arr.borrow();
            assert_eq!(arr_borrow.len(), 3);
            assert!(matches!(&arr_borrow[0], Value::String(s) if s.as_ref() == "a"));
            assert!(matches!(&arr_borrow[1], Value::String(s) if s.as_ref() == "b"));
            assert!(matches!(&arr_borrow[2], Value::String(s) if s.as_ref() == "c"));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_vec_string_from_atlas() {
    let arr = vec![
        Value::String(Rc::new("x".to_string())),
        Value::String(Rc::new("y".to_string())),
    ];
    let value = Value::Array(Rc::new(RefCell::new(arr)));
    let result: Vec<String> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, vec!["x".to_string(), "y".to_string()]);
}

#[test]
fn test_vec_empty_to_atlas() {
    let vec: Vec<f64> = vec![];
    let value = vec.to_atlas();
    match value {
        Value::Array(arr) => {
            let arr_borrow = arr.borrow();
            assert_eq!(arr_borrow.len(), 0);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_vec_empty_from_atlas() {
    let value = Value::Array(Rc::new(RefCell::new(vec![])));
    let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result.len(), 0);
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
        Value::String(Rc::new("oops".to_string())),
    ];
    let value = Value::Array(Rc::new(RefCell::new(arr)));
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

// Nested Conversion Tests

#[test]
fn test_nested_vec_option_f64() {
    let data = vec![Some(1.0), None, Some(3.0)];
    let value = data.to_atlas();

    // Convert back
    let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, vec![Some(1.0), None, Some(3.0)]);
}

#[test]
fn test_nested_vec_option_string() {
    let data = vec![Some("hello".to_string()), None, Some("world".to_string())];
    let value = data.to_atlas();

    // Convert back
    let result: Vec<Option<String>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(
        result,
        vec![Some("hello".to_string()), None, Some("world".to_string())]
    );
}

#[test]
fn test_nested_option_vec_f64() {
    let data: Option<Vec<f64>> = Some(vec![1.0, 2.0, 3.0]);
    let value = data.to_atlas();

    // Convert back
    let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, Some(vec![1.0, 2.0, 3.0]));
}

#[test]
fn test_nested_option_vec_none() {
    let data: Option<Vec<f64>> = None;
    let value = data.to_atlas();

    // Convert back
    let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, None);
}

// HashMap Conversion Tests

#[test]
fn test_hashmap_to_atlas_creates_json() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), 1.0);
    map.insert("y".to_string(), 2.0);

    let value = map.to_atlas();
    assert!(matches!(value, Value::JsonValue(_)));
}

#[test]
fn test_hashmap_string_to_atlas() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Alice".to_string());
    map.insert("city".to_string(), "Boston".to_string());

    let value = map.to_atlas();
    assert!(matches!(value, Value::JsonValue(_)));
}

// Bidirectional Roundtrip Tests

#[test]
fn test_roundtrip_f64() {
    let original = 42.5;
    let value = original.to_atlas();
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_string() {
    let original = "hello world".to_string();
    let value = original.clone().to_atlas();
    let result: String = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_bool_true() {
    let original = true;
    let value = original.to_atlas();
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_bool_false() {
    let original = false;
    let value = original.to_atlas();
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_option_some() {
    let original = Some(42.0);
    let value = original.to_atlas();
    let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_option_none() {
    let original: Option<f64> = None;
    let value = original.to_atlas();
    let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_f64() {
    let original = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let value = original.clone().to_atlas();
    let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_string() {
    let original = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let value = original.clone().to_atlas();
    let result: Vec<String> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_option_f64() {
    let original = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
    let value = original.clone().to_atlas();
    let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

// Error Message Quality Tests

#[test]
fn test_conversion_error_display_type_mismatch() {
    let error = ConversionError::TypeMismatch {
        expected: "number".to_string(),
        found: "string".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("number"));
    assert!(message.contains("string"));
    assert!(message.contains("mismatch"));
}

#[test]
fn test_conversion_error_display_array_element() {
    let error = ConversionError::ArrayElementTypeMismatch {
        index: 5,
        expected: "number".to_string(),
        found: "bool".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("5"));
    assert!(message.contains("number"));
    assert!(message.contains("bool"));
    assert!(message.contains("Array"));
}

#[test]
fn test_conversion_error_display_object_value() {
    let error = ConversionError::ObjectValueTypeMismatch {
        key: "name".to_string(),
        expected: "string".to_string(),
        found: "number".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("name"));
    assert!(message.contains("string"));
    assert!(message.contains("number"));
    assert!(message.contains("Object"));
}
