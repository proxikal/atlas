//! Interpreter tests for pure array functions
//!
//! Tests: pop, shift, unshift, reverse, concat, flatten,
//!        arrayIndexOf, arrayLastIndexOf, arrayIncludes, slice

use crate::stdlib::{eval_err, eval_ok};
use atlas_runtime::value::Value;

// ============================================================================
// pop() tests
// ============================================================================

#[test]
fn test_pop_normal() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        pop(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 2);
            assert_eq!(borrowed[0], Value::Number(3.0)); // Removed element
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_pop_single_element() {
    let result = eval_ok(r#"
        let arr = [42];
        pop(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(42.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_pop_empty_error() {
    let _err = eval_err(r#"
        let arr: number[] = [];
        pop(arr);
    "#);
}

// ============================================================================
// shift() tests  
// ============================================================================

#[test]
fn test_shift_normal() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        shift(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_shift_single_element() {
    let result = eval_ok(r#"
        let arr = [99];
        shift(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(99.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_shift_empty_error() {
    let _err = eval_err(r#"
        let arr: number[] = [];
        shift(arr);
    "#);
}

// ============================================================================
// unshift() tests
// ============================================================================

#[test]
fn test_unshift_normal() {
    let result = eval_ok(r#"
        let arr = [2, 3];
        unshift(arr, 1);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_unshift_multiple() {
    let result = eval_ok(r#"
        let arr = [3, 4];
        let arr2 = unshift(arr, 2);
        unshift(arr2, 1);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 4);
            assert_eq!(borrowed[0], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// reverse() tests
// ============================================================================

#[test]
fn test_reverse_normal() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3, 4, 5];
        reverse(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(5.0));
            assert_eq!(borrowed[4], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_empty() {
    let result = eval_ok(r#"
        let arr: number[] = [];
        reverse(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_single() {
    let result = eval_ok(r#"
        let arr = [42];
        reverse(arr);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(borrowed[0], Value::Number(42.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_reverse_preserves_original() {
    let result = eval_ok(r#"
        let original = [1, 2, 3];
        let reversed = reverse(original);
        arrayIndexOf(original, 1);
    "#);
    assert_eq!(result, Value::Number(0.0));
}

// ============================================================================
// concat() tests
// ============================================================================

#[test]
fn test_concat_normal() {
    let result = eval_ok(r#"
        let arr1 = [1, 2];
        let arr2 = [3, 4];
        concat(arr1, arr2);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 4);
            assert_eq!(borrowed[2], Value::Number(3.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_concat_empty_arrays() {
    let result = eval_ok(r#"
        let arr1: number[] = [];
        let arr2: number[] = [];
        concat(arr1, arr2);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_concat_with_empty() {
    let result = eval_ok(r#"
        let arr1 = [1, 2, 3];
        let arr2: number[] = [];
        concat(arr1, arr2);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 3);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_concat_multiple_chains() {
    let result = eval_ok(r#"
        let a = [1];
        let b = [2, 3];
        let c = [4, 5, 6];
        let ab = concat(a, b);
        concat(ab, c);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 6);
            assert_eq!(borrowed[5], Value::Number(6.0));
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// flatten() tests
// ============================================================================

#[test]
fn test_flatten_normal() {
    let result = eval_ok(r#"
        let nested: number[][] = [[1], [2, 3], [4]];
        let flat: number[] = flatten(nested);
        len(flat);
    "#);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_flatten_empty() {
    let result = eval_ok(r#"
        let nested: number[][] = [];
        flatten(nested);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_with_empty_inner() {
    let result = eval_ok(r#"
        let nested: number[][] = [[], [1], [], [2, 3]];
        flatten(nested);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flatten_single_nested() {
    let result = eval_ok(r#"
        let nested: number[][] = [[1, 2, 3]];
        flatten(nested);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 3);
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// arrayIndexOf() / arrayLastIndexOf() tests
// ============================================================================

#[test]
fn test_array_index_of_found() {
    let result = eval_ok(r#"
        let arr = [10, 20, 30];
        arrayIndexOf(arr, 20);
    "#);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_array_index_of_not_found() {
    let result = eval_ok(r#"
        let arr = [10, 20];
        arrayIndexOf(arr, 99);
    "#);
    assert_eq!(result, Value::Number(-1.0));
}

#[test]
fn test_array_index_of_duplicates() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3, 2, 4];
        arrayIndexOf(arr, 2);
    "#);
    assert_eq!(result, Value::Number(1.0)); // First occurrence
}

#[test]
fn test_array_last_index_of_duplicates() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3, 2, 4];
        arrayLastIndexOf(arr, 2);
    "#);
    assert_eq!(result, Value::Number(3.0)); // Last occurrence
}

#[test]
fn test_array_last_index_of_not_found() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        arrayLastIndexOf(arr, 99);
    "#);
    assert_eq!(result, Value::Number(-1.0));
}

// ============================================================================
// arrayIncludes() tests
// ============================================================================

#[test]
fn test_array_includes_true() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        arrayIncludes(arr, 2);
    "#);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_array_includes_false() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        arrayIncludes(arr, 99);
    "#);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_array_includes_boundary() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        let first = arrayIncludes(arr, 1);
        let last = arrayIncludes(arr, 3);
        if (first) {
            if (last) {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    "#);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// slice() tests
// ============================================================================

#[test]
fn test_slice_normal() {
    let result = eval_ok(r#"
        let arr = [0, 1, 2, 3, 4];
        slice(arr, 1, 4);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_entire_array() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3, 4];
        slice(arr, 0, 4);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 4);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_single_element() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3, 4];
        slice(arr, 2, 3);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(borrowed[0], Value::Number(3.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_slice_empty_range() {
    let result = eval_ok(r#"
        let arr = [1, 2, 3];
        slice(arr, 2, 2);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}
