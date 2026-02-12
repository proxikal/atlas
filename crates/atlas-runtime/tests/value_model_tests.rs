//! Value Model Integration Tests
//!
//! Tests for value representation semantics and equality rules.
//! These tests verify that value semantics work correctly through
//! the interpreter and VM runtime APIs.
//!
//! Phase: interpreter/phase-07-value-model-tests.md
//!
//! Test Coverage:
//! - Number, string, bool, null equality
//! - Array reference equality
//! - Mutation visibility across references
//!
//! Note: These tests are currently ignored because the interpreter
//! hasn't been implemented yet (phase-01-interpreter-core.md).
//! Enable these tests once the interpreter is ready.

use atlas_runtime::{Atlas, Value};

// ============================================================================
// Number Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_number_equality_same_values() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = 42;
        let b: int = 42;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "42 should equal 42"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_number_equality_different_values() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = 42;
        let b: int = 43;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(!result, "42 should not equal 43"),
        _ => panic!("Expected Bool(false)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_number_inequality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = 10;
        let b: int = 20;
        a != b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "10 should not equal 20"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_number_zero_equality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = 0;
        let b: int = 0;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "0 should equal 0"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_number_negative_equality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = -5;
        let b: int = -5;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "-5 should equal -5"),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// String Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_string_equality_same_content() {
    let runtime = Atlas::new();
    let code = r#"
        let a: string = "hello";
        let b: string = "hello";
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Strings with same content should be equal"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_string_equality_different_content() {
    let runtime = Atlas::new();
    let code = r#"
        let a: string = "hello";
        let b: string = "world";
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(!result, "Different strings should not be equal"),
        _ => panic!("Expected Bool(false)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_string_empty_equality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: string = "";
        let b: string = "";
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Empty strings should be equal"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_string_assignment_equality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: string = "test";
        let b: string = a;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Assigned strings should be equal"),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// Boolean Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_bool_equality_both_true() {
    let runtime = Atlas::new();
    let code = r#"
        let a: bool = true;
        let b: bool = true;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "true should equal true"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_bool_equality_both_false() {
    let runtime = Atlas::new();
    let code = r#"
        let a: bool = false;
        let b: bool = false;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "false should equal false"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_bool_equality_different() {
    let runtime = Atlas::new();
    let code = r#"
        let a: bool = true;
        let b: bool = false;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(!result, "true should not equal false"),
        _ => panic!("Expected Bool(false)"),
    }
}

// ============================================================================
// Null Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_null_equality() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int? = null;
        let b: int? = null;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "null should equal null"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_null_inequality_with_value() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int? = null;
        let b: int? = 42;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(!result, "null should not equal a value"),
        _ => panic!("Expected Bool(false)"),
    }
}

// ============================================================================
// Type Mismatch Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready (should fail at type checking)
fn test_number_string_type_mismatch() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int = 42;
        let b: string = "42";
        a == b
    "#;

    // This should fail during type checking, not runtime
    // Atlas only allows == between same types
    assert!(runtime.eval(code).is_err(), "Should reject comparing int and string");
}

// ============================================================================
// Array Reference Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_reference_equality_same_reference() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2, 3];
        let b: int[] = a;
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Same array reference should be equal"),
        _ => panic!("Expected Bool(true)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_reference_equality_different_references() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2, 3];
        let b: int[] = [1, 2, 3];
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => {
            assert!(!result, "Different array references should not be equal even with same contents");
        }
        _ => panic!("Expected Bool(false)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_reference_equality_empty_arrays() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [];
        let b: int[] = [];
        a == b
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => {
            assert!(!result, "Different empty array references should not be equal");
        }
        _ => panic!("Expected Bool(false)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_reference_chain() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2];
        let b: int[] = a;
        let c: int[] = b;
        a == c
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Chained references should point to same array"),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// Array Mutation Visibility Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_mutation_visible_through_alias() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2, 3];
        let b: int[] = a;
        a[0] = 42;
        b[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => assert_eq!(n, 42.0, "Mutation should be visible through alias"),
        _ => panic!("Expected Number(42.0)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_mutation_bidirectional() {
    let runtime = Atlas::new();

    // Mutate through first reference, check through second
    let code1 = r#"
        let arr1: int[] = [1, 2, 3];
        let arr2: int[] = arr1;
        arr1[1] = 99;
        arr2[1]
    "#;

    match runtime.eval(code1) {
        Ok(Value::Number(n)) => assert_eq!(n, 99.0, "Mutation via arr1 should be visible in arr2"),
        _ => panic!("Expected Number(99.0)"),
    }

    // Mutate through second reference, check through first
    let code2 = r#"
        let arr1: int[] = [1, 2, 3];
        let arr2: int[] = arr1;
        arr2[2] = 88;
        arr1[2]
    "#;

    match runtime.eval(code2) {
        Ok(Value::Number(n)) => assert_eq!(n, 88.0, "Mutation via arr2 should be visible in arr1"),
        _ => panic!("Expected Number(88.0)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_mutation_multiple_aliases() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [10, 20, 30];
        let b: int[] = a;
        let c: int[] = a;
        let d: int[] = b;

        c[0] = 100;
        d[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 100.0, "Mutation should be visible through all aliases");
        }
        _ => panic!("Expected Number(100.0)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_independent_arrays_no_interference() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2, 3];
        let b: int[] = [1, 2, 3];

        a[0] = 99;
        b[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 1.0, "Independent arrays should not affect each other");
        }
        _ => panic!("Expected Number(1.0)"),
    }
}

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_mutation_preserves_other_elements() {
    let runtime = Atlas::new();
    let code = r#"
        let a: int[] = [1, 2, 3, 4, 5];
        let b: int[] = a;

        a[2] = 999;
        b[4]
    "#;

    match runtime.eval(code) {
        Ok(Value::Number(n)) => {
            assert_eq!(n, 5.0, "Mutation should not affect other elements");
        }
        _ => panic!("Expected Number(5.0)"),
    }
}

// ============================================================================
// String Immutability Tests (Contrast with Arrays)
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready (should fail at type checking)
fn test_string_immutability() {
    let runtime = Atlas::new();
    let code = r#"
        let s: string = "hello";
        s[0] = "H";
    "#;

    // Strings are immutable in Atlas - this should fail during type checking
    // (strings don't support index assignment)
    assert!(runtime.eval(code).is_err(), "Should reject string mutation");
}

// ============================================================================
// Mixed Type Equality Tests
// ============================================================================

#[test]
#[ignore] // Enable when interpreter is ready
fn test_array_content_equality_primitive_types() {
    let runtime = Atlas::new();

    // Test that array elements can be compared
    let code = r#"
        let a: int[] = [1, 2, 3];
        let b: int[] = a;
        a[0] == b[0]
    "#;

    match runtime.eval(code) {
        Ok(Value::Bool(result)) => assert!(result, "Array elements should be equal"),
        _ => panic!("Expected Bool(true)"),
    }
}

// ============================================================================
// Documentation Tests
// ============================================================================

/// This test demonstrates the core value semantics of Atlas:
/// - Primitives (number, string, bool, null) use value equality
/// - Arrays use reference equality
/// - Array mutation is visible through all references
#[test]
#[ignore] // Enable when interpreter is ready
fn test_value_semantics_documentation_example() {
    let runtime = Atlas::new();
    let code = r#"
        // Primitives: value equality
        let n1: int = 42;
        let n2: int = 42;
        let numbers_equal: bool = n1 == n2;

        // Arrays: reference equality
        let arr1: int[] = [1, 2];
        let arr2: int[] = arr1;
        let arr3: int[] = [1, 2];

        let same_ref: bool = arr1 == arr2;
        let diff_ref: bool = arr1 == arr3;

        // Mutation visibility
        arr1[0] = 99;
        let seen_by_arr2: int = arr2[0];
        let not_seen_by_arr3: int = arr3[0];

        numbers_equal
    "#;

    // Just verify it executes without error
    // The actual assertions would be done in the Atlas code with assertions
    // (once we have assert or similar in stdlib)
    match runtime.eval(code) {
        Ok(Value::Bool(b)) => assert!(b),
        _ => panic!("Expected successful execution"),
    }
}
