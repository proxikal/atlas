//! collections.rs — merged from 5 files (Phase Infra-02)

mod common;

use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::typechecker::generics::Monomorphizer;
use atlas_runtime::types::{Type, TypeParamDef};
use atlas_runtime::value::Value;
use atlas_runtime::Atlas;
use common::{assert_error_code, assert_eval_bool, assert_eval_number, assert_eval_string};
use pretty_assertions::assert_eq;

// ============================================================================
// Canonical helpers (deduplicated from queue_tests and stack_tests)
// ============================================================================

fn dummy_span() -> Span {
    Span::dummy()
}

fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

// ============================================================================
// From hash_function_tests.rs
// ============================================================================

// Hash function tests for Atlas collections

#[test]
fn test_hash_number_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, 42, "forty-two");
        let result = hashMapGet(hm, 42);
        unwrap(result)
    "#;
    assert_eval_string(code, "forty-two");
}

#[test]
fn test_hash_string_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "hello", "world");
        let result = hashMapGet(hm, "hello");
        unwrap(result)
    "#;
    assert_eval_string(code, "world");
}

#[test]
fn test_hash_bool_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, true, "yes");
        let result = hashMapGet(hm, true);
        unwrap(result)
    "#;
    assert_eval_string(code, "yes");
}

#[test]
fn test_hash_null_key() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, null, "null-value");
        let result = hashMapGet(hm, null);
        unwrap(result)
    "#;
    assert_eval_string(code, "null-value");
}

#[test]
fn test_cannot_hash_array() {
    let code = r#"
        let hm = hashMapNew();
        let arr = [1, 2, 3];
        hashMapPut(hm, arr, "value");
    "#;
    assert_error_code(code, "AT0140");
}

#[test]
fn test_mixed_key_types() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, 42, "number");
        hashMapPut(hm, "key", "string");
        hashMapPut(hm, true, "bool");
        hashMapPut(hm, null, "null");
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 4.0);
}

#[test]
fn test_hashhm_new() {
    let code = r#"
        let hm = hashMapNew();
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashhm_put_get() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        let result = hashMapGet(hm, "key");
        unwrap(result)
    "#;
    assert_eval_string(code, "value");
}

#[test]
fn test_hashhm_get_nonexistent() {
    let code = r#"
        let hm = hashMapNew();
        let result = hashMapGet(hm, "nonexistent");
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_remove() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        let removed = hashMapRemove(hm, "key");
        is_some(removed)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_has() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "key", "value");
        hashMapHas(hm, "key")
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_size() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        hashMapPut(hm, "c", 3);
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_hashhm_is_empty() {
    let code = r#"
        let hm = hashMapNew();
        let empty1 = hashMapIsEmpty(hm);
        hashMapPut(hm, "key", "value");
        let empty2 = hashMapIsEmpty(hm);
        empty1 && !empty2
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashhm_clear() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        hashMapClear(hm);
        hashMapSize(hm)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashhm_keys() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let keys = hashMapKeys(hm);
        len(keys)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_hashhm_values() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let values = hashMapValues(hm);
        len(values)
    "#;
    assert_eval_number(code, 2.0);
}

#[test]
fn test_hashhm_entries() {
    let code = r#"
        let hm = hashMapNew();
        hashMapPut(hm, "a", 1);
        hashMapPut(hm, "b", 2);
        let entries = hashMapEntries(hm);
        len(entries)
    "#;
    assert_eval_number(code, 2.0);
}

// ============================================================================
// From hashset_tests.rs
// ============================================================================

// HashSet Tests - Comprehensive Test Suite

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
    assert!(eval_expect_error(
        "let set = hashSetNew(); hashSetAdd(set, [1, 2])"
    ));
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
#[ignore = "deadlocks: hashSetIsSubset with same Arc<Mutex> arg locks mutex twice"]
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
#[ignore = "deadlocks: hashSetIsSuperset with same Arc<Mutex> arg locks mutex twice"]
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
    // CoW value semantics: b is a logical copy of a.
    // Adding 42 to b does not affect a.
    let result = eval(
        r#"
        let a = hashSetNew();
        let b = a;
        hashSetAdd(b, 42);
        hashSetHas(a, 42)
    "#,
    );
    assert_eq!(result, Value::Bool(false));
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

// ============================================================================
// From generics_runtime_tests.rs
// ============================================================================

// Runtime tests for generic functions (Interpreter)
//
// BLOCKER 02-C: Monomorphization infrastructure
//
// These tests verify that the monomorphization infrastructure is in place
// and ready for full generic function execution (which requires integration
// with the full compilation pipeline in later phases).

#[test]
fn test_monomorphizer_substitutions() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];
    let type_args = vec![Type::Number];

    let subst = mono
        .get_substitutions("identity", &type_params, &type_args)
        .unwrap();

    assert_eq!(subst.len(), 1);
    assert_eq!(subst.get("T"), Some(&Type::Number));
}

#[test]
fn test_monomorphizer_multiple_instantiations() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];

    // identity<number>
    let subst1 = mono
        .get_substitutions("identity", &type_params, &[Type::Number])
        .unwrap();
    assert_eq!(subst1.get("T"), Some(&Type::Number));

    // identity<string>
    let subst2 = mono
        .get_substitutions("identity", &type_params, &[Type::String])
        .unwrap();
    assert_eq!(subst2.get("T"), Some(&Type::String));

    // identity<bool>
    let subst3 = mono
        .get_substitutions("identity", &type_params, &[Type::Bool])
        .unwrap();
    assert_eq!(subst3.get("T"), Some(&Type::Bool));

    // Should have 3 cached instances
    assert_eq!(mono.instance_count(), 3);
}

#[test]
fn test_monomorphizer_complex_types() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];

    // Array types
    let array_number = Type::Array(Box::new(Type::Number));
    let subst = mono
        .get_substitutions("process", &type_params, std::slice::from_ref(&array_number))
        .unwrap();

    assert_eq!(subst.get("T"), Some(&array_number));
}

#[test]
fn test_monomorphizer_multiple_type_params() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![
        TypeParamDef {
            name: "K".to_string(),
            bound: None,
                trait_bounds: vec![],
            },
        TypeParamDef {
            name: "V".to_string(),
            bound: None,
                trait_bounds: vec![],
            },
    ];
    let type_args = vec![Type::String, Type::Number];

    let subst = mono
        .get_substitutions("map", &type_params, &type_args)
        .unwrap();

    assert_eq!(subst.len(), 2);
    assert_eq!(subst.get("K"), Some(&Type::String));
    assert_eq!(subst.get("V"), Some(&Type::Number));
}

#[test]
fn test_name_mangling() {
    // No type args
    assert_eq!(Monomorphizer::mangle_name("foo", &[]), "foo");

    // Single type arg
    assert_eq!(
        Monomorphizer::mangle_name("identity", &[Type::Number]),
        "identity$number"
    );

    // Multiple type args
    assert_eq!(
        Monomorphizer::mangle_name("map", &[Type::String, Type::Number]),
        "map$string$number"
    );

    // Complex types
    let array_type = Type::Array(Box::new(Type::Bool));
    assert_eq!(
        Monomorphizer::mangle_name("process", &[array_type]),
        "process$bool[]"
    );
}

#[test]
fn test_monomorphizer_caching() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];
    let type_args = vec![Type::Number];

    // First call - should create new instance
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1);

    // Second call with same args - should use cache
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1); // Still 1

    // Clear cache
    mono.clear_cache();
    assert_eq!(mono.instance_count(), 0);

    // After clear, should create new instance again
    mono.get_substitutions("identity", &type_params, &type_args)
        .unwrap();
    assert_eq!(mono.instance_count(), 1);
}

#[test]
fn test_monomorphizer_generic_types() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];

    // Option<number>
    let option_number = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![Type::Number],
    };

    let subst = mono
        .get_substitutions("unwrap", &type_params, std::slice::from_ref(&option_number))
        .unwrap();

    assert_eq!(subst.get("T"), Some(&option_number));
}

#[test]
fn test_monomorphizer_nested_generics() {
    let mut mono = Monomorphizer::new();

    let type_params = vec![TypeParamDef {
        name: "T".to_string(),
        bound: None,
                trait_bounds: vec![],
            }];

    // Option<Result<number, string>>
    let result_type = Type::Generic {
        name: "Result".to_string(),
        type_args: vec![Type::Number, Type::String],
    };
    let option_result = Type::Generic {
        name: "Option".to_string(),
        type_args: vec![result_type.clone()],
    };

    let subst = mono
        .get_substitutions(
            "process",
            &type_params,
            std::slice::from_ref(&option_result),
        )
        .unwrap();

    assert_eq!(subst.get("T"), Some(&option_result));
}

// Additional integration tests will be added in BLOCKER 02-D
// when Option<T> and Result<T,E> are fully implemented.

// ============================================================================
// Queue and Stack tests (in submodules to avoid naming conflicts)
// ============================================================================

mod queue {
    use super::{dummy_span, security};
    use atlas_runtime::value::Value;
    use pretty_assertions::assert_eq;

    // Shadow call_builtin with a 4-arg wrapper (5th arg = stdout_writer) so tests don't need changing.
    fn call_builtin(
        name: &str,
        args: &[Value],
        span: atlas_runtime::span::Span,
        sec: &atlas_runtime::security::SecurityContext,
    ) -> Result<Value, atlas_runtime::value::RuntimeError> {
        atlas_runtime::stdlib::call_builtin(
            name,
            args,
            span,
            sec,
            &atlas_runtime::stdlib::stdout_writer(),
        )
    }

    // CoW helpers — enqueue/dequeue/clear return new state
    fn q_enqueue(queue: Value, item: Value) -> Value {
        call_builtin("queueEnqueue", &[queue, item], dummy_span(), &security()).unwrap()
    }
    /// Returns (item: Value::Option, new_queue: Value)
    fn q_dequeue(queue: Value) -> (Value, Value) {
        let r = call_builtin("queueDequeue", &[queue], dummy_span(), &security()).unwrap();
        if let Value::Array(arr) = r {
            let s = arr.as_slice();
            (s[0].clone(), s[1].clone())
        } else {
            panic!("queueDequeue returned non-array")
        }
    }
    fn q_clear(queue: Value) -> Value {
        call_builtin("queueClear", &[queue], dummy_span(), &security()).unwrap()
    }
    fn q_size(queue: &Value) -> Value {
        call_builtin(
            "queueSize",
            std::slice::from_ref(queue),
            dummy_span(),
            &security(),
        )
        .unwrap()
    }
    fn q_is_empty(queue: &Value) -> Value {
        call_builtin(
            "queueIsEmpty",
            std::slice::from_ref(queue),
            dummy_span(),
            &security(),
        )
        .unwrap()
    }

    // ============================================================================
    // From queue_tests.rs
    // ============================================================================

    // Integration tests for Queue collection
    //
    // Tests FIFO semantics, Option returns, and reference semantics.

    // ============================================================================
    // Creation Tests
    // ============================================================================

    #[test]
    fn test_create_empty_queue() {
        let result = call_builtin("queueNew", &[], dummy_span(), &security());
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Queue(_)));
    }

    #[test]
    fn test_new_queue_has_size_zero() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let size = call_builtin(
            "queueSize",
            std::slice::from_ref(&queue),
            dummy_span(),
            &security(),
        )
        .unwrap();
        assert_eq!(size, Value::Number(0.0));

        let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
        assert_eq!(empty, Value::Bool(true));
    }

    // ============================================================================
    // Enqueue and Dequeue Tests
    // ============================================================================

    #[test]
    fn test_enqueue_increases_size() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        assert_eq!(q_size(&queue), Value::Number(1.0));
    }

    #[test]
    fn test_dequeue_fifo_order() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        let queue = q_enqueue(queue, Value::Number(3.0));

        let (first, queue) = q_dequeue(queue);
        assert_eq!(first, Value::Option(Some(Box::new(Value::Number(1.0)))));

        let (second, queue) = q_dequeue(queue);
        assert_eq!(second, Value::Option(Some(Box::new(Value::Number(2.0)))));

        let (third, _queue) = q_dequeue(queue);
        assert_eq!(third, Value::Option(Some(Box::new(Value::Number(3.0)))));
    }

    #[test]
    fn test_dequeue_from_empty_returns_none() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let (result, _queue) = q_dequeue(queue);
        assert_eq!(result, Value::Option(None));
    }

    #[test]
    fn test_enqueue_after_dequeue() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let (_item, queue) = q_dequeue(queue);
        let queue = q_enqueue(queue, Value::Number(2.0));
        assert_eq!(q_size(&queue), Value::Number(1.0));
        let (result, _queue) = q_dequeue(queue);
        assert_eq!(result, Value::Option(Some(Box::new(Value::Number(2.0)))));
    }

    #[test]
    fn test_queue_accepts_any_value_type() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(42.0));
        let queue = q_enqueue(queue, Value::string("hello"));
        let queue = q_enqueue(queue, Value::Bool(true));
        let queue = q_enqueue(queue, Value::Null);
        assert_eq!(q_size(&queue), Value::Number(4.0));
    }

    // ============================================================================
    // Peek Tests
    // ============================================================================

    #[test]
    fn test_peek_returns_front_without_removing() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(42.0));
        let peeked = call_builtin(
            "queuePeek",
            std::slice::from_ref(&queue),
            dummy_span(),
            &security(),
        )
        .unwrap();
        assert_eq!(peeked, Value::Option(Some(Box::new(Value::Number(42.0)))));
        assert_eq!(q_size(&queue), Value::Number(1.0));
    }

    #[test]
    fn test_peek_on_empty_returns_none() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

        let result = call_builtin("queuePeek", &[queue], dummy_span(), &security()).unwrap();
        assert_eq!(result, Value::Option(None));
    }

    #[test]
    fn test_peek_doesnt_change_size() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        let size_before = q_size(&queue);
        call_builtin(
            "queuePeek",
            std::slice::from_ref(&queue),
            dummy_span(),
            &security(),
        )
        .unwrap();
        let size_after = q_size(&queue);
        assert_eq!(size_before, size_after);
    }

    // ============================================================================
    // Size and IsEmpty Tests
    // ============================================================================

    #[test]
    fn test_size_reflects_element_count() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        assert_eq!(q_size(&queue), Value::Number(0.0));
        let queue = q_enqueue(queue, Value::Number(1.0));
        assert_eq!(q_size(&queue), Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        assert_eq!(q_size(&queue), Value::Number(2.0));
    }

    #[test]
    fn test_is_empty_true_for_new_queue() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

        let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
        assert_eq!(empty, Value::Bool(true));
    }

    #[test]
    fn test_is_empty_false_after_enqueue() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        assert_eq!(q_is_empty(&queue), Value::Bool(false));
    }

    #[test]
    fn test_is_empty_true_after_dequeuing_all() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        let (_item, queue) = q_dequeue(queue);
        let (_item, queue) = q_dequeue(queue);
        assert_eq!(q_is_empty(&queue), Value::Bool(true));
    }

    // ============================================================================
    // Clear Tests
    // ============================================================================

    #[test]
    fn test_clear_removes_all_elements() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        let queue = q_enqueue(queue, Value::Number(3.0));
        let queue = q_clear(queue);
        assert_eq!(q_size(&queue), Value::Number(0.0));
        assert_eq!(q_is_empty(&queue), Value::Bool(true));
    }

    #[test]
    fn test_clear_on_empty_queue_is_safe() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_clear(queue);
        assert_eq!(q_is_empty(&queue), Value::Bool(true));
    }

    // ============================================================================
    // ToArray Tests
    // ============================================================================

    #[test]
    fn test_to_array_returns_fifo_order() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let queue = q_enqueue(queue, Value::Number(2.0));
        let queue = q_enqueue(queue, Value::Number(3.0));

        let array = call_builtin("queueToArray", &[queue], dummy_span(), &security()).unwrap();

        if let Value::Array(arr) = array {
            let borrowed = arr.as_slice();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_to_array_doesnt_modify_queue() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue = q_enqueue(queue, Value::Number(1.0));
        let size_before = q_size(&queue);
        call_builtin(
            "queueToArray",
            std::slice::from_ref(&queue),
            dummy_span(),
            &security(),
        )
        .unwrap();
        let size_after = q_size(&queue);
        assert_eq!(size_before, size_after);
    }

    #[test]
    fn test_to_array_on_empty_queue() {
        let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

        let array = call_builtin("queueToArray", &[queue], dummy_span(), &security()).unwrap();

        if let Value::Array(arr) = array {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected array");
        }
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_multiple_queues_are_independent() {
        let queue1 = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue2 = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
        let queue1 = q_enqueue(queue1, Value::Number(1.0));
        let queue2 = q_enqueue(queue2, Value::Number(2.0));
        assert_eq!(q_size(&queue1), Value::Number(1.0));
        assert_eq!(q_size(&queue2), Value::Number(1.0));
    }

    #[test]
    fn test_large_queue_performance() {
        let mut queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

        // Enqueue 1000 elements
        for i in 0..1000 {
            queue = q_enqueue(queue, Value::Number(i as f64));
        }
        assert_eq!(q_size(&queue), Value::Number(1000.0));

        // Dequeue all elements in FIFO order
        for i in 0..1000 {
            let (item, new_queue) = q_dequeue(queue);
            assert_eq!(item, Value::Option(Some(Box::new(Value::Number(i as f64)))));
            queue = new_queue;
        }
        assert_eq!(q_is_empty(&queue), Value::Bool(true));
    }
}

mod stack {
    use super::{dummy_span, security};
    use atlas_runtime::value::Value;
    use pretty_assertions::assert_eq;

    // Shadow call_builtin with a 4-arg wrapper (5th arg = stdout_writer) so tests don't need changing.
    fn call_builtin(
        name: &str,
        args: &[Value],
        span: atlas_runtime::span::Span,
        sec: &atlas_runtime::security::SecurityContext,
    ) -> Result<Value, atlas_runtime::value::RuntimeError> {
        atlas_runtime::stdlib::call_builtin(
            name,
            args,
            span,
            sec,
            &atlas_runtime::stdlib::stdout_writer(),
        )
    }

    // CoW helpers — push/pop/clear return new state
    fn s_push(stack: Value, item: Value) -> Value {
        call_builtin("stackPush", &[stack, item], dummy_span(), &security()).unwrap()
    }
    /// Returns (item: Value::Option, new_stack: Value)
    fn s_pop(stack: Value) -> (Value, Value) {
        let r = call_builtin("stackPop", &[stack], dummy_span(), &security()).unwrap();
        if let Value::Array(arr) = r {
            let s = arr.as_slice();
            (s[0].clone(), s[1].clone())
        } else {
            panic!("stackPop returned non-array")
        }
    }
    fn s_clear(stack: Value) -> Value {
        call_builtin("stackClear", &[stack], dummy_span(), &security()).unwrap()
    }
    fn s_size(stack: &Value) -> Value {
        call_builtin(
            "stackSize",
            std::slice::from_ref(stack),
            dummy_span(),
            &security(),
        )
        .unwrap()
    }
    fn s_is_empty(stack: &Value) -> Value {
        call_builtin(
            "stackIsEmpty",
            std::slice::from_ref(stack),
            dummy_span(),
            &security(),
        )
        .unwrap()
    }

    // ============================================================================
    // From stack_tests.rs
    // ============================================================================

    // Integration tests for Stack collection
    //
    // Tests LIFO semantics, Option returns, and reference semantics.

    // ============================================================================
    // Creation Tests
    // ============================================================================

    #[test]
    fn test_create_empty_stack() {
        let result = call_builtin("stackNew", &[], dummy_span(), &security());
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Value::Stack(_)));
    }

    #[test]
    fn test_new_stack_has_size_zero() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let size = call_builtin(
            "stackSize",
            std::slice::from_ref(&stack),
            dummy_span(),
            &security(),
        )
        .unwrap();
        assert_eq!(size, Value::Number(0.0));

        let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
        assert_eq!(empty, Value::Bool(true));
    }

    // ============================================================================
    // Push and Pop Tests
    // ============================================================================

    #[test]
    fn test_push_increases_size() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        assert_eq!(s_size(&stack), Value::Number(1.0));
    }

    #[test]
    fn test_pop_lifo_order() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        let stack = s_push(stack, Value::Number(3.0));

        let (third, stack) = s_pop(stack);
        assert_eq!(third, Value::Option(Some(Box::new(Value::Number(3.0)))));

        let (second, stack) = s_pop(stack);
        assert_eq!(second, Value::Option(Some(Box::new(Value::Number(2.0)))));

        let (first, _stack) = s_pop(stack);
        assert_eq!(first, Value::Option(Some(Box::new(Value::Number(1.0)))));
    }

    #[test]
    fn test_pop_from_empty_returns_none() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let (result, _stack) = s_pop(stack);
        assert_eq!(result, Value::Option(None));
    }

    #[test]
    fn test_push_after_pop() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let (_item, stack) = s_pop(stack);
        let stack = s_push(stack, Value::Number(2.0));
        assert_eq!(s_size(&stack), Value::Number(1.0));
        let (result, _stack) = s_pop(stack);
        assert_eq!(result, Value::Option(Some(Box::new(Value::Number(2.0)))));
    }

    #[test]
    fn test_stack_accepts_any_value_type() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(42.0));
        let stack = s_push(stack, Value::string("hello"));
        let stack = s_push(stack, Value::Bool(true));
        let stack = s_push(stack, Value::Null);
        assert_eq!(s_size(&stack), Value::Number(4.0));
    }

    // ============================================================================
    // Peek Tests
    // ============================================================================

    #[test]
    fn test_peek_returns_top_without_removing() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(42.0));
        let peeked = call_builtin(
            "stackPeek",
            std::slice::from_ref(&stack),
            dummy_span(),
            &security(),
        )
        .unwrap();
        assert_eq!(peeked, Value::Option(Some(Box::new(Value::Number(42.0)))));
        assert_eq!(s_size(&stack), Value::Number(1.0));
    }

    #[test]
    fn test_peek_on_empty_returns_none() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

        let result = call_builtin("stackPeek", &[stack], dummy_span(), &security()).unwrap();
        assert_eq!(result, Value::Option(None));
    }

    #[test]
    fn test_peek_doesnt_change_size() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        let size_before = s_size(&stack);
        call_builtin(
            "stackPeek",
            std::slice::from_ref(&stack),
            dummy_span(),
            &security(),
        )
        .unwrap();
        let size_after = s_size(&stack);
        assert_eq!(size_before, size_after);
    }

    // ============================================================================
    // Size and IsEmpty Tests
    // ============================================================================

    #[test]
    fn test_size_reflects_element_count() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        assert_eq!(s_size(&stack), Value::Number(0.0));
        let stack = s_push(stack, Value::Number(1.0));
        assert_eq!(s_size(&stack), Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        assert_eq!(s_size(&stack), Value::Number(2.0));
    }

    #[test]
    fn test_is_empty_true_for_new_stack() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

        let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
        assert_eq!(empty, Value::Bool(true));
    }

    #[test]
    fn test_is_empty_false_after_push() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        assert_eq!(s_is_empty(&stack), Value::Bool(false));
    }

    #[test]
    fn test_is_empty_true_after_popping_all() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        let (_item, stack) = s_pop(stack);
        let (_item, stack) = s_pop(stack);
        assert_eq!(s_is_empty(&stack), Value::Bool(true));
    }

    // ============================================================================
    // Clear Tests
    // ============================================================================

    #[test]
    fn test_clear_removes_all_elements() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        let stack = s_push(stack, Value::Number(3.0));
        let stack = s_clear(stack);
        assert_eq!(s_size(&stack), Value::Number(0.0));
        assert_eq!(s_is_empty(&stack), Value::Bool(true));
    }

    #[test]
    fn test_clear_on_empty_stack_is_safe() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_clear(stack);
        assert_eq!(s_is_empty(&stack), Value::Bool(true));
    }

    // ============================================================================
    // ToArray Tests
    // ============================================================================

    #[test]
    fn test_to_array_returns_bottom_to_top_order() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let stack = s_push(stack, Value::Number(2.0));
        let stack = s_push(stack, Value::Number(3.0));

        let array = call_builtin("stackToArray", &[stack], dummy_span(), &security()).unwrap();

        if let Value::Array(arr) = array {
            let borrowed = arr.as_slice();
            assert_eq!(borrowed.len(), 3);
            assert_eq!(borrowed[0], Value::Number(1.0));
            assert_eq!(borrowed[1], Value::Number(2.0));
            assert_eq!(borrowed[2], Value::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_to_array_doesnt_modify_stack() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack = s_push(stack, Value::Number(1.0));
        let size_before = s_size(&stack);
        call_builtin(
            "stackToArray",
            std::slice::from_ref(&stack),
            dummy_span(),
            &security(),
        )
        .unwrap();
        let size_after = s_size(&stack);
        assert_eq!(size_before, size_after);
    }

    #[test]
    fn test_to_array_on_empty_stack() {
        let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

        let array = call_builtin("stackToArray", &[stack], dummy_span(), &security()).unwrap();

        if let Value::Array(arr) = array {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected array");
        }
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_multiple_stacks_are_independent() {
        let stack1 = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack2 = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();
        let stack1 = s_push(stack1, Value::Number(1.0));
        let stack2 = s_push(stack2, Value::Number(2.0));
        assert_eq!(s_size(&stack1), Value::Number(1.0));
        assert_eq!(s_size(&stack2), Value::Number(1.0));
    }

    #[test]
    fn test_large_stack_performance() {
        let mut stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

        // Push 1000 elements
        for i in 0..1000 {
            stack = s_push(stack, Value::Number(i as f64));
        }
        assert_eq!(s_size(&stack), Value::Number(1000.0));

        // Pop all elements (LIFO: reverse order)
        for i in (0..1000).rev() {
            let (item, new_stack) = s_pop(stack);
            assert_eq!(item, Value::Option(Some(Box::new(Value::Number(i as f64)))));
            stack = new_stack;
        }
        assert_eq!(s_is_empty(&stack), Value::Bool(true));
    }
}

// ============================================================================
// Collections Extended Hardening (Phase v02-completion-04)
// ============================================================================

// --- HashMap edge cases ---

#[test]
fn test_hashmap_get_missing_key_returns_none() {
    let code = r#"
        let m = hashMapNew();
        let result = hashMapGet(m, "missing");
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashmap_put_overwrites_existing_key() {
    let code = r#"
        let m = hashMapNew();
        hashMapPut(m, "k", "first");
        hashMapPut(m, "k", "second");
        unwrap(hashMapGet(m, "k"))
    "#;
    assert_eval_string(code, "second");
}

#[test]
fn test_hashmap_keys_on_empty_returns_empty_array() {
    let code = r#"
        let m = hashMapNew();
        len(hashMapKeys(m))
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashmap_values_on_empty_returns_empty_array() {
    let code = r#"
        let m = hashMapNew();
        len(hashMapValues(m))
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashmap_remove_nonexistent_key_returns_none() {
    let code = r#"
        let m = hashMapNew();
        let result = hashMapRemove(m, "ghost");
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashmap_size_decrements_after_remove() {
    let code = r#"
        let m = hashMapNew();
        hashMapPut(m, "a", 1);
        hashMapPut(m, "b", 2);
        hashMapRemove(m, "a");
        hashMapSize(m)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_hashmap_is_empty_on_new_map() {
    let code = r#"
        let m = hashMapNew();
        hashMapIsEmpty(m)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_hashmap_is_empty_false_after_put() {
    let code = r#"
        let m = hashMapNew();
        hashMapPut(m, "x", 1);
        hashMapIsEmpty(m)
    "#;
    assert_eval_bool(code, false);
}

#[test]
fn test_hashmap_clear_empties_map() {
    let code = r#"
        let m = hashMapNew();
        hashMapPut(m, "a", 1);
        hashMapPut(m, "b", 2);
        hashMapClear(m);
        hashMapSize(m)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashmap_from_entries_roundtrip() {
    let code = r#"
        let entries = [["k", "v"]];
        let m = hashMapFromEntries(entries);
        unwrap(hashMapGet(m, "k"))
    "#;
    assert_eval_string(code, "v");
}

// --- HashSet edge cases ---

#[test]
fn test_hashset_add_duplicate_does_not_increase_size() {
    let code = r#"
        let s = hashSetNew();
        hashSetAdd(s, "x");
        hashSetAdd(s, "x");
        hashSetSize(s)
    "#;
    assert_eval_number(code, 1.0);
}

#[test]
fn test_hashset_remove_nonexistent_no_error() {
    let code = r#"
        let s = hashSetNew();
        hashSetRemove(s, "ghost");
        hashSetSize(s)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashset_union_empty_with_empty() {
    let code = r#"
        let a = hashSetNew();
        let b = hashSetNew();
        let u = hashSetUnion(a, b);
        hashSetSize(u)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashset_intersection_empty_with_nonempty() {
    let code = r#"
        let a = hashSetNew();
        let b = hashSetNew();
        hashSetAdd(b, 1);
        hashSetAdd(b, 2);
        let i = hashSetIntersection(a, b);
        hashSetSize(i)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashset_difference_identical_sets_is_empty() {
    // Use two separate sets with identical contents (avoids Arc<Mutex> self-deadlock)
    let code = r#"
        let a = hashSetNew();
        hashSetAdd(a, 1);
        hashSetAdd(a, 2);
        let b = hashSetNew();
        hashSetAdd(b, 1);
        hashSetAdd(b, 2);
        let d = hashSetDifference(a, b);
        hashSetSize(d)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashset_to_array_from_empty() {
    let code = r#"
        let s = hashSetNew();
        let arr = hashSetToArray(s);
        len(arr)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_hashset_is_empty_on_new() {
    let code = r#"
        let s = hashSetNew();
        hashSetIsEmpty(s)
    "#;
    assert_eval_bool(code, true);
}

// --- Queue edge cases ---

#[test]
fn test_queue_mixed_type_values() {
    let code = r#"
        let q = queueNew();
        queueEnqueue(q, 42);
        queueEnqueue(q, "hello");
        queueEnqueue(q, true);
        let a = unwrap(queueDequeue(q));
        let b = unwrap(queueDequeue(q));
        let c = unwrap(queueDequeue(q));
        queueIsEmpty(q)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_queue_clear_then_size_is_zero() {
    let code = r#"
        let q = queueNew();
        queueEnqueue(q, 1);
        queueEnqueue(q, 2);
        queueClear(q);
        queueSize(q)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_queue_to_array_empty_returns_empty() {
    let code = r#"
        let q = queueNew();
        let arr = queueToArray(q);
        len(arr)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_queue_fifo_order_preserved() {
    let code = r#"
        let q = queueNew();
        queueEnqueue(q, 10);
        queueEnqueue(q, 20);
        queueEnqueue(q, 30);
        let first = unwrap(queueDequeue(q));
        first
    "#;
    assert_eval_number(code, 10.0);
}

#[test]
fn test_queue_dequeue_on_empty_returns_none() {
    let code = r#"
        let q = queueNew();
        let result = queueDequeue(q);
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

// --- Stack edge cases ---

#[test]
fn test_stack_pop_on_empty_returns_none() {
    let code = r#"
        let s = stackNew();
        let result = stackPop(s);
        is_none(result)
    "#;
    assert_eval_bool(code, true);
}

#[test]
fn test_stack_peek_reflects_top_after_push() {
    let code = r#"
        let s = stackNew();
        stackPush(s, 1);
        stackPush(s, 2);
        stackPush(s, 3);
        unwrap(stackPeek(s))
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_stack_clear_idempotent_on_empty() {
    let code = r#"
        let s = stackNew();
        stackClear(s);
        stackSize(s)
    "#;
    assert_eval_number(code, 0.0);
}

#[test]
fn test_stack_lifo_order() {
    let code = r#"
        let s = stackNew();
        stackPush(s, 1);
        stackPush(s, 2);
        stackPush(s, 3);
        unwrap(stackPop(s))
    "#;
    assert_eval_number(code, 3.0);
}

#[test]
fn test_stack_to_array_bottom_to_top() {
    let code = r#"
        let s = stackNew();
        stackPush(s, 10);
        stackPush(s, 20);
        stackPush(s, 30);
        let arr = stackToArray(s);
        arr[0]
    "#;
    assert_eval_number(code, 10.0);
}
