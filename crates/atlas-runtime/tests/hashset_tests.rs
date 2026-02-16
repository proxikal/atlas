//! HashSet Tests - Comprehensive Test Suite

mod common;
use atlas_runtime::{Atlas, Value};

fn eval(code: &str) -> Value {
    let runtime = Atlas::new();
    runtime.eval(code).expect("Interpretation failed")
}

fn eval_expect_error(code: &str) -> bool {
    let runtime = Atlas::new();
    runtime.eval(code).is_err()
}

// Creation Tests

#[test]
fn test_hashset_new() {
    let result = eval("hashSetSize(hashSetNew())");
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_hashset_from_array() {
    let result = eval("hashSetSize(hashSetFromArray([1, 2, 3]))");
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_from_array_removes_duplicates() {
    let result = eval("hashSetSize(hashSetFromArray([1, 2, 2, 3, 3, 3]))");
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_from_array_unhashable() {
    assert!(eval_expect_error("hashSetFromArray([[1, 2]])"));
}

#[test]
fn test_hashset_empty_is_empty() {
    let result = eval("hashSetIsEmpty(hashSetNew())");
    assert_eq!(result, Value::Bool(true));
}

// Add and Remove Tests

#[test]
fn test_hashset_add_increases_size() {
    let result = eval(
        r#"
        let set = hashSetNew();
        hashSetAdd(set, 42);
        hashSetSize(set)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_hashset_add_duplicate_idempotent() {
    let result = eval(
        r#"
        let set = hashSetNew();
        hashSetAdd(set, 42);
        hashSetAdd(set, 42);
        hashSetSize(set)
    "#,
    );
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_hashset_add_different_types() {
    let result = eval(
        r#"
        let set = hashSetNew();
        hashSetAdd(set, 42);
        hashSetAdd(set, "hello");
        hashSetAdd(set, true);
        hashSetAdd(set, null);
        hashSetSize(set)
    "#,
    );
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_hashset_remove_existing() {
    let result = eval(
        r#"
        let set = hashSetFromArray([1, 2, 3]);
        hashSetRemove(set, 2)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_remove_nonexistent() {
    let result = eval(
        r#"
        let set = hashSetFromArray([1, 2, 3]);
        hashSetRemove(set, 99)
    "#,
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_hashset_add_unhashable() {
    assert!(eval_expect_error("let set = hashSetNew(); hashSetAdd(set, [1, 2])"));
}

// Has Tests

#[test]
fn test_hashset_has_existing() {
    let result = eval("hashSetHas(hashSetFromArray([1, 2, 3]), 2)");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_has_nonexistent() {
    let result = eval("hashSetHas(hashSetFromArray([1, 2, 3]), 99)");
    assert_eq!(result, Value::Bool(false));
}

// Size and IsEmpty Tests

#[test]
fn test_hashset_size_reflects_count() {
    let result = eval(
        r#"
        let set = hashSetNew();
        hashSetAdd(set, 1);
        hashSetAdd(set, 2);
        hashSetAdd(set, 3);
        hashSetSize(set)
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_is_empty_with_elements() {
    let result = eval("hashSetIsEmpty(hashSetFromArray([1, 2, 3]))");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_hashset_is_empty_after_clear() {
    let result = eval(
        r#"
        let set = hashSetFromArray([1, 2, 3]);
        hashSetClear(set);
        hashSetIsEmpty(set)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// Union Tests

#[test]
fn test_hashset_union_disjoint() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2]);
        let b = hashSetFromArray([3, 4]);
        hashSetSize(hashSetUnion(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_hashset_union_overlapping() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([2, 3, 4]);
        hashSetSize(hashSetUnion(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_hashset_union_with_empty() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetNew();
        hashSetSize(hashSetUnion(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

// Intersection Tests

#[test]
fn test_hashset_intersection_overlapping() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([2, 3, 4]);
        hashSetSize(hashSetIntersection(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_intersection_disjoint() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2]);
        let b = hashSetFromArray([3, 4]);
        hashSetSize(hashSetIntersection(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

// Difference Tests

#[test]
fn test_hashset_difference() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([2, 3, 4]);
        let d = hashSetDifference(a, b);
        hashSetHas(d, 1)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_difference_disjoint() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2]);
        let b = hashSetFromArray([3, 4]);
        hashSetSize(hashSetDifference(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

// Symmetric Difference Tests

#[test]
fn test_hashset_symmetric_difference() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([2, 3, 4]);
        hashSetSize(hashSetSymmetricDifference(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_hashset_symmetric_difference_identical() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        let b = hashSetFromArray([1, 2, 3]);
        hashSetSize(hashSetSymmetricDifference(a, b))
    "#,
    );
    assert_eq!(result, Value::Number(0.0));
}

// Subset and Superset Tests

#[test]
fn test_hashset_empty_is_subset() {
    let result = eval(
        r#"
        let a = hashSetNew();
        let b = hashSetFromArray([1, 2, 3]);
        hashSetIsSubset(a, b)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_set_is_subset_of_itself() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        hashSetIsSubset(a, a)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_proper_subset() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2]);
        let b = hashSetFromArray([1, 2, 3]);
        hashSetIsSubset(a, b)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_empty_not_superset_of_nonempty() {
    let result = eval(
        r#"
        let a = hashSetNew();
        let b = hashSetFromArray([1, 2, 3]);
        hashSetIsSuperset(a, b)
    "#,
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_hashset_set_is_superset_of_itself() {
    let result = eval(
        r#"
        let a = hashSetFromArray([1, 2, 3]);
        hashSetIsSuperset(a, a)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

// Integration Tests

#[test]
fn test_hashset_reference_semantics() {
    let result = eval(
        r#"
        let a = hashSetNew();
        let b = a;
        hashSetAdd(b, 42);
        hashSetHas(a, 42)
    "#,
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_hashset_to_array_preserves_elements() {
    let result = eval(
        r#"
        let set = hashSetFromArray([1, 2, 3]);
        len(hashSetToArray(set))
    "#,
    );
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_hashset_large_set() {
    // Test with a reasonable number of elements
    let code = r#"
let set = hashSetNew();
hashSetAdd(set, 1);
hashSetAdd(set, 2);
hashSetAdd(set, 3);
hashSetAdd(set, 4);
hashSetAdd(set, 5);
hashSetAdd(set, 6);
hashSetAdd(set, 7);
hashSetAdd(set, 8);
hashSetAdd(set, 9);
hashSetAdd(set, 10);
hashSetSize(set)
"#;
    let result = eval(code);
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_hashset_mixed_types() {
    let result = eval(
        r#"
        let set = hashSetNew();
        hashSetAdd(set, 42);
        hashSetAdd(set, "hello");
        hashSetAdd(set, true);
        hashSetAdd(set, false);
        hashSetAdd(set, null);
        hashSetAdd(set, 3.14);
        hashSetSize(set)
    "#,
    );
    assert_eq!(result, Value::Number(6.0));
}
