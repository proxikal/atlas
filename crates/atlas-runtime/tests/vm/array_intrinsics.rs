//! VM tests for array intrinsics (callback-based)
//!
//! Tests: map, filter, reduce, forEach, find, findIndex,  
//!        flatMap, some, every, sort, sortBy

use crate::vm::{execute_vm_err, execute_vm_ok};
use atlas_runtime::value::Value;

// ============================================================================
// map() tests
// ============================================================================

#[test]
fn test_map_double() {
    let result = execute_vm_ok(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let arr = [1, 2, 3];
        map(arr, double);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(6.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_map_empty_array() {
    let result = execute_vm_ok(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        let arr: number[] = [];
        map(arr, double);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_map_single_element() {
    let result = execute_vm_ok(r#"
        fn triple(x: number) -> number {
            return x * 3;
        }
        let arr = [5];
        map(arr, triple);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(borrowed[0], Value::Number(15.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_map_large_array() {
    let result = execute_vm_ok(r#"
        fn triple(x: number) -> number {
            return x * 3;
        }
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = map(arr, triple);
        len(result);
    "#);
    assert_eq!(result, Value::Number(10.0));
}

// ============================================================================
// filter() tests
// ============================================================================

#[test]
fn test_filter_evens() {
    let result = execute_vm_ok(r#"
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr = [1, 2, 3, 4, 5, 6];
        filter(arr, isEven);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(2.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_filter_empty_array() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        let arr: number[] = [];
        filter(arr, isPositive);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_filter_all_match() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        let arr = [1, 2, 3, 4, 5];
        filter(arr, isPositive);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 5);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_filter_none_match() {
    let result = execute_vm_ok(r#"
        fn isNegative(x: number) -> bool {
            return x < 0;
        }
        let arr = [1, 2, 3];
        filter(arr, isNegative);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_filter_chain() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr = [-2, -1, 0, 1, 2, 3, 4];
        let positive = filter(arr, isPositive);
        filter(positive, isEven);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 2);
            assert_eq!(borrowed[0], Value::Number(2.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_filter_with_find() {
    let result = execute_vm_ok(r#"
        fn isLarge(x: number) -> bool {
            return x > 5;
        }
        let arr = [1, 10, 3, 8, 2];
        let filtered = filter(arr, isLarge);
        find(filtered, isLarge);
    "#);
    assert_eq!(result, Value::Number(10.0));
}

// ============================================================================
// reduce() tests
// ============================================================================

#[test]
fn test_reduce_sum() {
    let result = execute_vm_ok(r#"
        fn add(acc: number, x: number) -> number {
            return acc + x;
        }
        let arr = [1, 2, 3, 4, 5];
        reduce(arr, add, 0);
    "#);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_reduce_empty_array() {
    let result = execute_vm_ok(r#"
        fn add(acc: number, x: number) -> number {
            return acc + x;
        }
        let arr: number[] = [];
        reduce(arr, add, 10);
    "#);
    assert_eq!(result, Value::Number(10.0)); // Returns initial value
}

#[test]
fn test_reduce_single_element() {
    let result = execute_vm_ok(r#"
        fn add(acc: number, x: number) -> number {
            return acc + x;
        }
        let arr = [5];
        reduce(arr, add, 10);
    "#);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_reduce_product() {
    let result = execute_vm_ok(r#"
        fn multiply(acc: number, x: number) -> number {
            return acc * x;
        }
        let arr = [2, 3, 4];
        reduce(arr, multiply, 1);
    "#);
    assert_eq!(result, Value::Number(24.0));
}

#[test]
fn test_reduce_max() {
    let result = execute_vm_ok(r#"
        fn max(acc: number, x: number) -> number {
            if (x > acc) {
                return x;
            } else {
                return acc;
            }
        }
        let arr = [3, 7, 2, 9, 1];
        reduce(arr, max, 0);
    "#);
    assert_eq!(result, Value::Number(9.0));
}

#[test]
fn test_map_then_reduce() {
    let result = execute_vm_ok(r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn add(acc: number, x: number) -> number {
            return acc + x;
        }
        let arr = [1, 2, 3];
        let doubled = map(arr, double);
        reduce(doubled, add, 0);
    "#);
    assert_eq!(result, Value::Number(12.0));
}

// ============================================================================
// find() / findIndex() tests
// ============================================================================

#[test]
fn test_find_first_match() {
    let result = execute_vm_ok(r#"
        fn isLarge(x: number) -> bool {
            return x > 10;
        }
        let arr = [1, 5, 15, 20];
        find(arr, isLarge);
    "#);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_find_no_match() {
    let result = execute_vm_ok(r#"
        fn isLarge(x: number) -> bool {
            return x > 100;
        }
        let arr = [1, 5, 15];
        find(arr, isLarge);
    "#);
    assert_eq!(result, Value::Null);
}

#[test]
fn test_find_empty_array() {
    let result = execute_vm_ok(r#"
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr: number[] = [];
        find(arr, isEven);
    "#);
    assert_eq!(result, Value::Null);
}

#[test]
fn test_find_first_of_many() {
    let result = execute_vm_ok(r#"
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr = [1, 2, 3, 4, 5, 6];
        find(arr, isEven);
    "#);
    assert_eq!(result, Value::Number(2.0)); // First even number
}

#[test]
fn test_find_with_complex_predicate() {
    let result = execute_vm_ok(r#"
        fn isDivisibleBy3And5(x: number) -> bool {
            if (x % 3 == 0) {
                if (x % 5 == 0) {
                    return true;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        let arr = [1, 5, 10, 15, 20, 30];
        find(arr, isDivisibleBy3And5);
    "#);
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_find_index() {
    let result = execute_vm_ok(r#"
        fn isLarge(x: number) -> bool {
            return x > 10;
        }
        let arr = [1, 5, 15, 20];
        findIndex(arr, isLarge);
    "#);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_find_index_empty() {
    let result = execute_vm_ok(r#"
        fn alwaysTrue(x: number) -> bool {
            return x == x;
        }
        let arr: number[] = [];
        findIndex(arr, alwaysTrue);
    "#);
    assert_eq!(result, Value::Number(-1.0));
}

#[test]
fn test_find_index_not_found() {
    let result = execute_vm_ok(r#"
        fn isNegative(x: number) -> bool {
            return x < 0;
        }
        let arr = [1, 2, 3];
        findIndex(arr, isNegative);
    "#);
    assert_eq!(result, Value::Number(-1.0));
}

// ============================================================================
// flatMap() tests
// ============================================================================

#[test]
fn test_flat_map() {
    let result = execute_vm_ok(r#"
        fn duplicate(x: number) -> number[] {
            return [x, x];
        }
        let arr = [1, 2, 3];
        flatMap(arr, duplicate);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 6);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(1.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flat_map_empty() {
    let result = execute_vm_ok(r#"
        fn duplicate(x: number) -> number[] {
            return [x, x];
        }
        let arr: number[] = [];
        flatMap(arr, duplicate);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_flat_map_empty_results() {
    let result = execute_vm_ok(r#"
        fn makeEmpty(x: number) -> number[] {
            let check = x + 1;
            if (check > 0) {
                let empty: number[] = [];
                return empty;
            } else {
                let empty: number[] = [];
                return empty;
            }
        }
        let arr = [1, 2, 3];
        flatMap(arr, makeEmpty);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// some() / every() tests
// ============================================================================

#[test]
fn test_some_true() {
    let result = execute_vm_ok(r#"
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr = [1, 3, 4, 5];
        some(arr, isEven);
    "#);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_some_false() {
    let result = execute_vm_ok(r#"
        fn isEven(x: number) -> bool {
            return x % 2 == 0;
        }
        let arr = [1, 3, 5];
        some(arr, isEven);
    "#);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_some_empty_array() {
    let result = execute_vm_ok(r#"
        fn alwaysTrue(x: number) -> bool {
            return x == x;
        }
        let arr: number[] = [];
        some(arr, alwaysTrue);
    "#);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_some_first_element() {
    let result = execute_vm_ok(r#"
        fn isOne(x: number) -> bool {
            return x == 1;
        }
        let arr = [1, 2, 3];
        some(arr, isOne);
    "#);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_some_short_circuit() {
    let result = execute_vm_ok(r#"
        fn isHundred(x: number) -> bool {
            return x == 100;
        }
        let arr = [1, 2, 100, 4];
        some(arr, isHundred);
    "#);
    assert_eq!(result, Value::Bool(true)); // Should stop at 100
}

#[test]
fn test_every_true() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        let arr = [1, 2, 3];
        every(arr, isPositive);
    "#);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_every_false() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        let arr = [1, -1, 3];
        every(arr, isPositive);
    "#);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_every_empty_array() {
    let result = execute_vm_ok(r#"
        fn alwaysTrue(x: number) -> bool {
            return x == x;
        }
        let arr: number[] = [];
        every(arr, alwaysTrue);
    "#);
    assert_eq!(result, Value::Bool(true)); // Vacuously true
}

#[test]
fn test_every_single_element_true() {
    let result = execute_vm_ok(r#"
        fn isPositive(x: number) -> bool {
            return x > 0;
        }
        let arr = [5];
        every(arr, isPositive);
    "#);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_every_single_element_false() {
    let result = execute_vm_ok(r#"
        fn isNegative(x: number) -> bool {
            return x < 0;
        }
        let arr = [5];
        every(arr, isNegative);
    "#);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_every_short_circuit() {
    let result = execute_vm_ok(r#"
        fn isSmall(x: number) -> bool {
            return x < 10;
        }
        let arr = [1, 2, 100, 4];
        every(arr, isSmall);
    "#);
    assert_eq!(result, Value::Bool(false)); // Should stop at 100
}

// ============================================================================
// sort() / sortBy() tests
// ============================================================================

#[test]
fn test_sort_ascending() {
    let result = execute_vm_ok(r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let arr = [3, 1, 4, 1, 5];
        sort(arr, compare);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[4], Value::Number(5.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_descending() {
    let result = execute_vm_ok(r#"
        fn compareDesc(a: number, b: number) -> number {
            return b - a;
        }
        let arr = [1, 5, 3, 2, 4];
        sort(arr, compareDesc);
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
fn test_sort_empty_array() {
    let result = execute_vm_ok(r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let arr: number[] = [];
        sort(arr, compare);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_single_element() {
    let result = execute_vm_ok(r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let arr = [42];
        sort(arr, compare);
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
fn test_sort_stability() {
    let result = execute_vm_ok(r#"
        fn alwaysZero(a: number, b: number) -> number {
            let sum = a + b;
            return 0 - sum + sum;
        }
        let arr = [3, 1, 4, 1, 5];
        sort(arr, alwaysZero);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            // Should maintain original order when comparator returns 0
            assert_eq!(borrowed[0], Value::Number(3.0));
            assert_eq!(borrowed[1], Value::Number(1.0));
            assert_eq!(borrowed[2], Value::Number(4.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_with_equal_elements() {
    let result = execute_vm_ok(r#"
        fn compare(a: number, b: number) -> number {
            return a - b;
        }
        let arr = [3, 1, 2, 1, 3];
        sort(arr, compare);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(1.0));
            assert_eq!(borrowed[4], Value::Number(3.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_by_numeric_key() {
    let result = execute_vm_ok(r#"
        fn getAbs(x: number) -> number {
            if (x < 0) {
                return 0 - x;
            } else {
                return x;
            }
        }
        let arr = [3, -5, 2, -1];
        sortBy(arr, getAbs);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            // Should be sorted by absolute value: [-1, 2, 3, -5]
            assert_eq!(borrowed[0], Value::Number(-1.0));
            assert_eq!(borrowed[3], Value::Number(-5.0));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_by_empty() {
    let result = execute_vm_ok(r#"
        fn identity(x: number) -> number {
            return x;
        }
        let arr: number[] = [];
        sortBy(arr, identity);
    "#);
    
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 0);
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_sort_by_single() {
    let result = execute_vm_ok(r#"
        fn identity(x: number) -> number {
            return x;
        }
        let arr = [99];
        sortBy(arr, identity);
    "#);
    
    match result {
        Value::Array(arr) => {
            let borrowed = arr.lock().unwrap();
            assert_eq!(borrowed.len(), 1);
            assert_eq!(borrowed[0], Value::Number(99.0));
        }
        _ => panic!("Expected array"),
    }
}
