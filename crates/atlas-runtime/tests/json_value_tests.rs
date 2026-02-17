//! Integration tests for JsonValue type
//!
//! Tests both interpreter and VM parity for JSON value operations.

use atlas_runtime::{JsonValue, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Helper to create a test JSON object
fn make_test_json() -> Value {
    let mut user = HashMap::new();
    user.insert("name".to_string(), JsonValue::String("Alice".to_string()));
    user.insert("age".to_string(), JsonValue::Number(30.0));

    let mut data = HashMap::new();
    data.insert("user".to_string(), JsonValue::object(user));
    data.insert("active".to_string(), JsonValue::Bool(true));

    Value::JsonValue(Arc::new(JsonValue::object(data)))
}

#[test]
fn test_json_value_type_display() {
    assert_eq!(make_test_json().type_name(), "json");
}

#[test]
fn test_json_value_equality() {
    let json1 = Value::JsonValue(Arc::new(JsonValue::Number(42.0)));
    let json2 = Value::JsonValue(Arc::new(JsonValue::Number(42.0)));
    let json3 = Value::JsonValue(Arc::new(JsonValue::Number(43.0)));

    assert_eq!(json1, json2); // Same value
    assert_ne!(json1, json3); // Different value
}

// ===== Type Declaration Tests =====

// NOTE: Type annotation test skipped - no JSON literal syntax yet
// This will be added in Phase 4: JSON API when json_parse() is implemented
// For now, JsonValue can only be constructed from Rust code, not Atlas code

// #[rstest]
// #[case("let x: json = null; x", "null")]
// fn test_json_type_annotation(#[case] input: &str, #[case] _expected: &str) {
//     // Test that "json" type annotation is recognized
//     let runtime = Atlas::new();
//     let result = runtime.eval(input);
//     assert!(result.is_ok(), "Should accept json type annotation");
// }

// ===== Object Indexing Tests =====

#[test]
fn test_json_object_string_indexing_interpreter() {
    let json = make_test_json();

    // Access nested object
    if let Value::JsonValue(j) = json {
        let user = j.index_str("user");
        assert!(user.is_object());

        let name = user.index_str("name");
        assert_eq!(name, JsonValue::String("Alice".to_string()));

        let age = user.index_str("age");
        assert_eq!(age, JsonValue::Number(30.0));
    } else {
        panic!("Expected JsonValue");
    }
}

#[test]
fn test_json_missing_key_returns_null() {
    let json = make_test_json();

    if let Value::JsonValue(j) = json {
        let missing = j.index_str("nonexistent");
        assert_eq!(missing, JsonValue::Null);
    } else {
        panic!("Expected JsonValue");
    }
}

// ===== Array Indexing Tests =====

#[test]
fn test_json_array_number_indexing() {
    let arr = Value::JsonValue(Arc::new(JsonValue::array(vec![
        JsonValue::Number(10.0),
        JsonValue::Number(20.0),
        JsonValue::Number(30.0),
    ])));

    if let Value::JsonValue(j) = arr {
        assert_eq!(j.index_num(0.0), JsonValue::Number(10.0));
        assert_eq!(j.index_num(1.0), JsonValue::Number(20.0));
        assert_eq!(j.index_num(2.0), JsonValue::Number(30.0));

        // Out of bounds returns null
        assert_eq!(j.index_num(3.0), JsonValue::Null);
        assert_eq!(j.index_num(100.0), JsonValue::Null);

        // Negative index returns null
        assert_eq!(j.index_num(-1.0), JsonValue::Null);

        // Fractional index returns null
        assert_eq!(j.index_num(1.5), JsonValue::Null);
    } else {
        panic!("Expected JsonValue");
    }
}

// ===== Type Extraction Tests =====

#[test]
fn test_json_type_checking_methods() {
    let null_val = JsonValue::Null;
    let bool_val = JsonValue::Bool(true);
    let num_val = JsonValue::Number(42.0);
    let str_val = JsonValue::String("hello".to_string());
    let arr_val = JsonValue::array(vec![]);
    let obj_val = JsonValue::object(HashMap::new());

    assert!(null_val.is_null());
    assert!(!null_val.is_bool());

    assert!(bool_val.is_bool());
    assert_eq!(bool_val.as_bool(), Some(true));

    assert!(num_val.is_number());
    assert_eq!(num_val.as_number(), Some(42.0));

    assert!(str_val.is_string());
    assert_eq!(str_val.as_string(), Some("hello"));

    assert!(arr_val.is_array());
    assert!(obj_val.is_object());
}

#[test]
fn test_json_extraction_returns_none_for_wrong_type() {
    let num = JsonValue::Number(42.0);

    assert_eq!(num.as_number(), Some(42.0));
    assert_eq!(num.as_bool(), None);
    assert_eq!(num.as_string(), None);
    assert_eq!(num.as_array(), None);
    assert_eq!(num.as_object(), None);
}

// ===== Nested Access Tests =====

#[test]
fn test_json_nested_object_access() {
    let mut inner = HashMap::new();
    inner.insert(
        "city".to_string(),
        JsonValue::String("New York".to_string()),
    );

    let mut outer = HashMap::new();
    outer.insert("address".to_string(), JsonValue::object(inner));

    let json = JsonValue::object(outer);

    let address = json.index_str("address");
    let city = address.index_str("city");

    assert_eq!(city, JsonValue::String("New York".to_string()));
}

#[test]
fn test_json_nested_array_object_access() {
    let user1 = {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Bob".to_string()));
        JsonValue::object(map)
    };

    let user2 = {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Carol".to_string()));
        JsonValue::object(map)
    };

    let users = JsonValue::array(vec![user1, user2]);

    let first = users.index_num(0.0);
    let name = first.index_str("name");

    assert_eq!(name, JsonValue::String("Bob".to_string()));
}

// ===== Display/Format Tests =====

#[test]
fn test_json_display_null() {
    assert_eq!(JsonValue::Null.to_string(), "null");
}

#[test]
fn test_json_display_bool() {
    assert_eq!(JsonValue::Bool(true).to_string(), "true");
    assert_eq!(JsonValue::Bool(false).to_string(), "false");
}

#[test]
fn test_json_display_number() {
    assert_eq!(JsonValue::Number(42.0).to_string(), "42");
    assert_eq!(JsonValue::Number(2.5).to_string(), "2.5");
    assert_eq!(JsonValue::Number(-5.0).to_string(), "-5");
}

#[test]
fn test_json_display_string() {
    assert_eq!(
        JsonValue::String("hello".to_string()).to_string(),
        "\"hello\""
    );
}

#[test]
fn test_json_display_array() {
    let arr = JsonValue::array(vec![
        JsonValue::Number(1.0),
        JsonValue::Number(2.0),
        JsonValue::Number(3.0),
    ]);
    assert_eq!(arr.to_string(), "[1, 2, 3]");
}

#[test]
fn test_json_display_object() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), JsonValue::Number(1.0));

    let obj = JsonValue::object(map);
    // Note: HashMap order is not guaranteed, so we just check it contains the right parts
    let display = obj.to_string();
    assert!(display.starts_with('{'));
    assert!(display.ends_with('}'));
    assert!(display.contains("\"a\""));
    assert!(display.contains('1'));
}

// ===== Length Tests =====

#[test]
fn test_json_array_length() {
    let arr = JsonValue::array(vec![
        JsonValue::Number(1.0),
        JsonValue::Number(2.0),
        JsonValue::Number(3.0),
    ]);
    assert_eq!(arr.len(), Some(3));
}

#[test]
fn test_json_object_length() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), JsonValue::Number(1.0));
    map.insert("b".to_string(), JsonValue::Number(2.0));

    let obj = JsonValue::object(map);
    assert_eq!(obj.len(), Some(2));
}

#[test]
fn test_json_non_collection_length_is_none() {
    assert_eq!(JsonValue::Null.len(), None);
    assert_eq!(JsonValue::Bool(true).len(), None);
    assert_eq!(JsonValue::Number(42.0).len(), None);
    assert_eq!(JsonValue::String("hi".to_string()).len(), None);
}

#[test]
fn test_json_is_empty() {
    assert!(JsonValue::Null.is_empty());
    assert!(JsonValue::array(vec![]).is_empty());
    assert!(!JsonValue::array(vec![JsonValue::Null]).is_empty());
    assert!(JsonValue::object(HashMap::new()).is_empty());
}

// ===== Isolation Tests (Type System) =====
// These tests verify that JsonValue is isolated from regular types

#[test]
fn test_json_isolation_in_type_system() {
    use atlas_runtime::Type;

    let json_type = Type::JsonValue;
    let number_type = Type::Number;
    let string_type = Type::String;

    // JsonValue can only assign to JsonValue
    assert!(json_type.is_assignable_to(&json_type));
    assert!(!json_type.is_assignable_to(&number_type));
    assert!(!json_type.is_assignable_to(&string_type));

    // Other types cannot assign to JsonValue
    assert!(!number_type.is_assignable_to(&json_type));
    assert!(!string_type.is_assignable_to(&json_type));
}

#[test]
fn test_json_type_display_name() {
    use atlas_runtime::Type;

    assert_eq!(Type::JsonValue.display_name(), "json");
}
