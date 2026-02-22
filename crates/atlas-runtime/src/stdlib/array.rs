//! Array manipulation functions
//!
//! Pure array functions (no callbacks). For callback-based operations like map/filter/reduce,
//! see interpreter/VM intrinsics.

use crate::span::Span;
use crate::value::{RuntimeError, Value};

// ============================================================================
// Core Operations
// ============================================================================

/// Append element to array
///
/// Returns new array with element appended at the end
pub fn push(arr: &[Value], element: Value) -> Value {
    let mut new_arr = arr.to_vec();
    new_arr.push(element);
    Value::array(new_arr)
}

/// Sort array by natural order (numbers ascending, strings lexicographic)
///
/// Returns new sorted array; original is not modified.
pub fn sort_natural(arr: &[Value]) -> Value {
    let mut new_arr = arr.to_vec();
    new_arr.sort_by(compare_values_natural);
    Value::array(new_arr)
}

/// Natural comparison for sort: numbers by value, everything else by debug repr
fn compare_values_natural(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(x), Value::String(y)) => x.as_ref().cmp(y.as_ref()),
        _ => format!("{a:?}").cmp(&format!("{b:?}")),
    }
}

/// Remove and return last element from array
///
/// Returns two-element array: [removed_element, new_array]
/// Returns error if array is empty
pub fn pop(arr: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if arr.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Cannot pop from empty array".to_string(),
            span,
        });
    }

    let mut new_arr = arr.to_vec();
    let removed = new_arr.pop().unwrap(); // Safe: checked non-empty

    // Return [removed_element, new_array]
    Ok(Value::array(vec![removed, Value::array(new_arr)]))
}

/// Remove and return first element from array
///
/// Returns two-element array: [removed_element, new_array]
/// Returns error if array is empty
pub fn shift(arr: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if arr.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Cannot shift from empty array".to_string(),
            span,
        });
    }

    let mut new_arr = arr.to_vec();
    let removed = new_arr.remove(0);

    // Return [removed_element, new_array]
    Ok(Value::array(vec![removed, Value::array(new_arr)]))
}

/// Prepend element to array
///
/// Returns new array with element at the beginning
pub fn unshift(arr: &[Value], element: Value) -> Value {
    let mut new_arr = Vec::with_capacity(arr.len() + 1);
    new_arr.push(element);
    new_arr.extend_from_slice(arr);
    Value::array(new_arr)
}

/// Reverse array elements
///
/// Returns new array with elements in reverse order
pub fn reverse(arr: &[Value]) -> Value {
    let mut new_arr = arr.to_vec();
    new_arr.reverse();
    Value::array(new_arr)
}

/// Concatenate arrays
///
/// Returns new array containing all elements from both arrays
pub fn concat(arr1: &[Value], arr2: &[Value]) -> Value {
    let mut new_arr = Vec::with_capacity(arr1.len() + arr2.len());
    new_arr.extend_from_slice(arr1);
    new_arr.extend_from_slice(arr2);
    Value::array(new_arr)
}

/// Flatten array by one level
///
/// Returns new array with nested arrays flattened one level deep
pub fn flatten(arr: &[Value], _span: Span) -> Result<Value, RuntimeError> {
    let mut result = Vec::new();

    for elem in arr {
        match elem {
            Value::Array(nested) => {
                result.extend(nested.as_slice().iter().cloned());
            }
            other => {
                result.push(other.clone());
            }
        }
    }

    Ok(Value::array(result))
}

// ============================================================================
// Search Operations
// ============================================================================

/// Find first index of element in array
///
/// Returns index as number, or -1 if not found
pub fn index_of(arr: &[Value], search: &Value) -> f64 {
    for (i, elem) in arr.iter().enumerate() {
        if values_equal(elem, search) {
            return i as f64;
        }
    }
    -1.0
}

/// Find last index of element in array
///
/// Returns index as number, or -1 if not found
pub fn last_index_of(arr: &[Value], search: &Value) -> f64 {
    for (i, elem) in arr.iter().enumerate().rev() {
        if values_equal(elem, search) {
            return i as f64;
        }
    }
    -1.0
}

/// Check if array contains element
///
/// Returns true if element is found, false otherwise
pub fn includes(arr: &[Value], search: &Value) -> bool {
    arr.iter().any(|elem| values_equal(elem, search))
}

// ============================================================================
// Slicing
// ============================================================================

/// Extract slice of array
///
/// Returns new array containing elements from start (inclusive) to end (exclusive)
pub fn slice(arr: &[Value], start: f64, end: f64, span: Span) -> Result<Value, RuntimeError> {
    let len = arr.len() as i64;

    // Convert to integer indices
    let start_idx = start as i64;
    let end_idx = end as i64;

    // Handle negative indices (not supported in v0.2 - just clamp to 0)
    let start_idx = start_idx.max(0).min(len) as usize;
    let end_idx = end_idx.max(0).min(len) as usize;

    // Validate range
    if start_idx > end_idx {
        return Err(RuntimeError::TypeError {
            msg: format!("Invalid slice range: start {} > end {}", start_idx, end_idx),
            span,
        });
    }

    let sliced = arr[start_idx..end_idx].to_vec();
    Ok(Value::array(sliced))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if two values are equal (for indexOf/includes)
///
/// Uses Value's PartialEq implementation which handles:
/// - Numbers, strings, bools, null by value
/// - Arrays by reference identity (not deep equality)
/// - Functions by name equality
fn values_equal(a: &Value, b: &Value) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_normal() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let result = pop(&arr, Span::dummy()).unwrap();

        // Should return [removed_element, new_array]
        match result {
            Value::Array(result_arr) => {
                let s = result_arr.as_slice();
                assert_eq!(s.len(), 2);
                assert_eq!(s[0], Value::Number(3.0)); // Removed element
                                                      // s[1] is the new array
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_pop_empty() {
        let arr: Vec<Value> = vec![];
        assert!(pop(&arr, Span::dummy()).is_err());
    }

    #[test]
    fn test_shift_normal() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let result = shift(&arr, Span::dummy()).unwrap();

        // Should return [removed_element, new_array]
        match result {
            Value::Array(result_arr) => {
                let s = result_arr.as_slice();
                assert_eq!(s.len(), 2);
                assert_eq!(s[0], Value::Number(1.0)); // Removed element
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_unshift() {
        let arr = vec![Value::Number(2.0), Value::Number(3.0)];
        let result = unshift(&arr, Value::Number(1.0));

        match result {
            Value::Array(new_arr) => {
                let s = new_arr.as_slice();
                assert_eq!(s.len(), 3);
                assert_eq!(s[0], Value::Number(1.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_reverse() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
        let result = reverse(&arr);

        match result {
            Value::Array(new_arr) => {
                let s = new_arr.as_slice();
                assert_eq!(s[0], Value::Number(3.0));
                assert_eq!(s[2], Value::Number(1.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_concat() {
        let arr1 = vec![Value::Number(1.0), Value::Number(2.0)];
        let arr2 = vec![Value::Number(3.0), Value::Number(4.0)];
        let result = concat(&arr1, &arr2);

        match result {
            Value::Array(new_arr) => {
                assert_eq!(new_arr.len(), 4);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_flatten() {
        let nested = Value::array(vec![Value::Number(2.0), Value::Number(3.0)]);
        let arr = vec![Value::Number(1.0), nested, Value::Number(4.0)];
        let result = flatten(&arr, Span::dummy()).unwrap();

        match result {
            Value::Array(new_arr) => {
                assert_eq!(new_arr.len(), 4);
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_index_of_found() {
        let arr = vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ];
        assert_eq!(index_of(&arr, &Value::Number(20.0)), 1.0);
    }

    #[test]
    fn test_index_of_not_found() {
        let arr = vec![Value::Number(10.0), Value::Number(20.0)];
        assert_eq!(index_of(&arr, &Value::Number(99.0)), -1.0);
    }

    #[test]
    fn test_includes_true() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0)];
        assert!(includes(&arr, &Value::Number(2.0)));
    }

    #[test]
    fn test_includes_false() {
        let arr = vec![Value::Number(1.0), Value::Number(2.0)];
        assert!(!includes(&arr, &Value::Number(3.0)));
    }

    #[test]
    fn test_slice_normal() {
        let arr = vec![
            Value::Number(0.0),
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
        ];
        let result = slice(&arr, 1.0, 4.0, Span::dummy()).unwrap();

        match result {
            Value::Array(new_arr) => {
                let s = new_arr.as_slice();
                assert_eq!(s.len(), 3);
                assert_eq!(s[0], Value::Number(1.0));
            }
            _ => panic!("Expected array"),
        }
    }
}
