//! HashMap and HashSet Iteration Tests
//!
//! Comprehensive tests for forEach, map, and filter intrinsics on collections.
//!
//! NOTE: Atlas v0.2 does not support anonymous functions (fn(x) { ... }).
//! All callbacks must be named functions passed by reference.

use atlas_runtime::{Atlas, Value};
use std::sync::Arc;

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

fn eval_expect_error(code: &str) -> bool {
    let runtime = Atlas::new();
    runtime.eval(code).is_err()
}

// =============================================================================
// HashMap Iteration Tests
// =============================================================================

#[test]
fn test_hashmap_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(_v: number, _k: string) -> void {}
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapForEach(hmap, callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashmap_foreach_executes_callback() {
    // Verify callback executes by counting iterations
    let result = eval(
        r#"
        var count: number = 0;
        fn callback(_v: number, _k: string) -> void {
            count = count + 1;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapForEach(hmap, callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_map_transforms_values() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        let mapped = hashMapMap(hmap, double);
        unwrap(hashMapGet(mapped, "a"))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_map_preserves_keys() {
    let result = eval(
        r#"
        fn addFive(v: number, _k: string) -> number {
            return v + 5;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "x", 10);
        hashMapPut(hmap, "y", 20);
        let mapped = hashMapMap(hmap, addFive);
        hashMapHas(mapped, "x") && hashMapHas(mapped, "y")
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashmap_map_preserves_size() {
    let result = eval(
        r#"
        fn times10(v: number, _k: string) -> number {
            return v * 10;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let mapped = hashMapMap(hmap, times10);
        hashMapSize(mapped)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashmap_filter_keeps_matching_entries() {
    let result = eval(
        r#"
        fn greaterThanOne(v: number, _k: string) -> bool {
            return v > 1;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, greaterThanOne);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_with_predicate() {
    let result = eval(
        r#"
        fn isEven(v: number, _k: string) -> bool {
            return v % 2 == 0;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapPut(hmap, "d", 4);
        let filtered = hashMapFilter(hmap, isEven);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(v: number, _k: string) -> bool {
            return v > 10;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, greaterThan10);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_empty_iteration() {
    let result = eval(
        r#"
        fn identity(v: number, _k: string) -> number {
            return v;
        }
        let hmap = hashMapNew();
        let mapped = hashMapMap(hmap, identity);
        hashMapSize(mapped)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashmap_chaining_operations() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        fn greaterThan2(v: number, _k: string) -> bool {
            return v > 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let doubled = hashMapMap(hmap, double);
        let filtered = hashMapFilter(doubled, greaterThan2);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashmap_callback_receives_value_and_key() {
    // Verify callback receives both value and key parameters
    let result = eval(
        r#"
        fn addIfTest(v: number, k: string) -> number {
            if (k == "test") {
                return v + 1;
            } else {
                return v;
            }
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "test", 42);
        let mapped = hashMapMap(hmap, addIfTest);
        unwrap(hashMapGet(mapped, "test"))
    "#,
    );
    assert_eq!(result, Value::Number(43.0));
}

#[test]
fn test_hashmap_large_map() {
    let result = eval(
        r#"
        fn lessThan25(v: number, _k: string) -> bool {
            return v < 25;
        }
        let hmap = hashMapNew();
        var i: number = 0;
        while (i < 50) {
            hashMapPut(hmap, toString(i), i);
            i = i + 1;
        }
        let filtered = hashMapFilter(hmap, lessThan25);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(25.0));
}

// Error Handling Tests

#[test]
fn test_hashmap_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapForEach(hmap, "not a function")
    "#
    ));
}

#[test]
fn test_hashmap_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapMap(hmap, 42)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapFilter(hmap, null)
    "#
    ));
}

#[test]
fn test_hashmap_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(v: number, _k: string) -> number {
            return v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapFilter(hmap, returnValue)
    "#
    ));
}

// =============================================================================
// HashSet Iteration Tests
// =============================================================================

#[test]
fn test_hashset_foreach_returns_null() {
    let result = eval(
        r#"
        fn callback(_elem: number) -> void {}
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetForEach(hset, callback)
    "#,
    );
    assert_eq!(result, Value::Null);
}

#[test]
fn test_hashset_foreach_executes_callback() {
    let result = eval(
        r#"
        var count: number = 0;
        fn callback(_elem: number) -> void {
            count = count + 1;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetForEach(hset, callback);
        count
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_to_array() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        let arr = hashSetMap(hset, double);
        typeof(arr)
    "#,
    );
    assert_eq!(result, Value::String(Arc::new("array".to_string())));
}

#[test]
fn test_hashset_map_array_length() {
    let result = eval(
        r#"
        fn times10(elem: number) -> number {
            return elem * 10;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let arr = hashSetMap(hset, times10);
        len(arr)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_map_transforms_elements() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 5);
        let arr = hashSetMap(hset, double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_hashset_filter_keeps_matching() {
    let result = eval(
        r#"
        fn greaterThan2(elem: number) -> bool {
            return elem > 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetAdd(hset, 4);
        let filtered = hashSetFilter(hset, greaterThan2);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_filter_removes_non_matching() {
    let result = eval(
        r#"
        fn greaterThan10(elem: number) -> bool {
            return elem > 10;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let filtered = hashSetFilter(hset, greaterThan10);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_empty_filter() {
    let result = eval(
        r#"
        fn alwaysTrue(_elem: number) -> bool {
            return true;
        }
        let hset = hashSetNew();
        let filtered = hashSetFilter(hset, alwaysTrue);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_filter_chaining() {
    let result = eval(
        r#"
        fn greaterThan1(elem: number) -> bool {
            return elem > 1;
        }
        fn lessThan4(elem: number) -> bool {
            return elem < 4;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        hashSetAdd(hset, 4);
        let f1 = hashSetFilter(hset, greaterThan1);
        let f2 = hashSetFilter(f1, lessThan4);
        hashSetSize(f2)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_large_set() {
    let result = eval(
        r#"
        fn divisibleBy3(elem: number) -> bool {
            return elem % 3 == 0;
        }
        let hset = hashSetNew();
        var i: number = 0;
        while (i < 30) {
            hashSetAdd(hset, i);
            i = i + 1;
        }
        let filtered = hashSetFilter(hset, divisibleBy3);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

// Error Handling Tests

#[test]
fn test_hashset_foreach_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetForEach(hset, "not a function")
    "#
    ));
}

#[test]
fn test_hashset_map_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetMap(hset, 42)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_function_callback() {
    assert!(eval_expect_error(
        r#"
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetFilter(hset, null)
    "#
    ));
}

#[test]
fn test_hashset_filter_non_bool_return() {
    // Filter predicate must return bool
    assert!(eval_expect_error(
        r#"
        fn returnValue(elem: number) -> number {
            return elem;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetFilter(hset, returnValue)
    "#
    ));
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_integration_hashmap_to_hashset() {
    let result = eval(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        let values = hashMapValues(hmap);
        let hset = hashSetFromArray(values);
        hashSetSize(hset)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_hashset_map_to_array_filter() {
    let result = eval(
        r#"
        fn double(x: number) -> number {
            return x * 2;
        }
        fn greaterThan2(x: number) -> bool {
            return x > 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let arr = hashSetMap(hset, double);
        let filtered = filter(arr, greaterThan2);
        len(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_integration_empty_collections() {
    let result = eval(
        r#"
        fn identity(v: number, _k: string) -> number {
            return v;
        }
        fn alwaysTrue(_x: number) -> bool {
            return true;
        }
        let hm = hashMapNew();
        let hs = hashSetNew();
        let mr = hashMapMap(hm, identity);
        let sr = hashSetFilter(hs, alwaysTrue);
        hashMapSize(mr) + hashSetSize(sr)
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_integration_complex_transformation() {
    let result = eval(
        r#"
        fn double(v: number, _k: string) -> number {
            return v * 2;
        }
        fn greaterOrEqual4(v: number, _k: string) -> bool {
            return v >= 4;
        }
        var sum: number = 0;
        fn addToSum(v: number) -> void {
            sum = sum + v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        hashMapPut(hmap, "d", 4);
        let doubled = hashMapMap(hmap, double);
        let filtered = hashMapFilter(doubled, greaterOrEqual4);
        let values = hashMapValues(filtered);
        forEach(values, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(18.0)); // 4 + 6 + 8 = 18
}

#[test]
fn test_integration_hashmap_keys_to_hashset() {
    let result = eval(
        r#"
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let keys = hashMapKeys(hmap);
        let hset = hashSetFromArray(keys);
        hashSetSize(hset)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

// =============================================================================
// Parity Tests (ensure interpreter/VM consistency)
// =============================================================================

#[test]
fn test_parity_hashmap_foreach() {
    let result = eval(
        r#"
        var sum: number = 0;
        fn addToSum(v: number, _k: string) -> void {
            sum = sum + v;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "x", 5);
        hashMapForEach(hmap, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_parity_hashmap_map() {
    let result = eval(
        r#"
        fn triple(v: number, _k: string) -> number {
            return v * 3;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "test", 5);
        let mapped = hashMapMap(hmap, triple);
        unwrap(hashMapGet(mapped, "test"))
    "#,
    );
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_parity_hashmap_filter() {
    let result = eval(
        r#"
        fn notEqual2(v: number, _k: string) -> bool {
            return v != 2;
        }
        let hmap = hashMapNew();
        hashMapPut(hmap, "a", 1);
        hashMapPut(hmap, "b", 2);
        hashMapPut(hmap, "c", 3);
        let filtered = hashMapFilter(hmap, notEqual2);
        hashMapSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_parity_hashset_foreach() {
    let result = eval(
        r#"
        var sum: number = 0;
        fn addToSum(elem: number) -> void {
            sum = sum + elem;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 10);
        hashSetForEach(hset, addToSum);
        sum
    "#,
    );
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_parity_hashset_map() {
    let result = eval(
        r#"
        fn double(elem: number) -> number {
            return elem * 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 7);
        let arr = hashSetMap(hset, double);
        arr[0]
    "#,
    );
    assert_eq!(result, Value::Number(14.0));
}

#[test]
fn test_parity_hashset_filter() {
    let result = eval(
        r#"
        fn lessOrEqual2(elem: number) -> bool {
            return elem <= 2;
        }
        let hset = hashSetNew();
        hashSetAdd(hset, 1);
        hashSetAdd(hset, 2);
        hashSetAdd(hset, 3);
        let filtered = hashSetFilter(hset, lessOrEqual2);
        hashSetSize(filtered)
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}
