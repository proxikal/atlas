//! Integration tests for Stack collection
//!
//! Tests LIFO semantics, Option returns, and reference semantics.

use atlas_runtime::security::SecurityContext;
use atlas_runtime::span::Span;
use atlas_runtime::stdlib::call_builtin;
use atlas_runtime::value::Value;

fn dummy_span() -> Span {
    Span::dummy()
}

fn security() -> SecurityContext {
    SecurityContext::allow_all()
}

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
    let size = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
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

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));
}

#[test]
fn test_pop_lifo_order() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let third = call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(third, Value::Option(Some(Box::new(Value::Number(3.0)))));

    let second = call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(second, Value::Option(Some(Box::new(Value::Number(2.0)))));

    let first = call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(first, Value::Option(Some(Box::new(Value::Number(1.0)))));
}

#[test]
fn test_pop_from_empty_returns_none() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    let result = call_builtin("stackPop", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(result, Value::Option(None));
}

#[test]
fn test_push_after_pop() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));

    let result = call_builtin("stackPop", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(result, Value::Option(Some(Box::new(Value::Number(2.0)))));
}

#[test]
fn test_stack_accepts_any_value_type() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(42.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::string("hello")],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Bool(true)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Null],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(4.0));
}

// ============================================================================
// Peek Tests
// ============================================================================

#[test]
fn test_peek_returns_top_without_removing() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(42.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let peeked = call_builtin("stackPeek", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(peeked, Value::Option(Some(Box::new(Value::Number(42.0)))));

    let size = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));
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

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size_before =
        call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    call_builtin("stackPeek", &[stack.clone()], dummy_span(), &security()).unwrap();
    let size_after = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();

    assert_eq!(size_before, size_after);
}

// ============================================================================
// Size and IsEmpty Tests
// ============================================================================

#[test]
fn test_size_reflects_element_count() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    let size0 = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size0, Value::Number(0.0));

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    let size1 = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size1, Value::Number(1.0));

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    let size2 = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(size2, Value::Number(2.0));
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

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(false));
}

#[test]
fn test_is_empty_true_after_popping_all() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
    call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();

    let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

// ============================================================================
// Clear Tests
// ============================================================================

#[test]
fn test_clear_removes_all_elements() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    call_builtin("stackClear", &[stack.clone()], dummy_span(), &security()).unwrap();

    let size = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(0.0));

    let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

#[test]
fn test_clear_on_empty_stack_is_safe() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    let result = call_builtin("stackClear", &[stack.clone()], dummy_span(), &security());
    assert!(result.is_ok());

    let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

// ============================================================================
// ToArray Tests
// ============================================================================

#[test]
fn test_to_array_returns_bottom_to_top_order() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let array = call_builtin("stackToArray", &[stack], dummy_span(), &security()).unwrap();

    if let Value::Array(arr) = array {
        let borrowed = arr.lock().unwrap();
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

    call_builtin(
        "stackPush",
        &[stack.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size_before =
        call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    call_builtin("stackToArray", &[stack.clone()], dummy_span(), &security()).unwrap();
    let size_after = call_builtin("stackSize", &[stack], dummy_span(), &security()).unwrap();

    assert_eq!(size_before, size_after);
}

#[test]
fn test_to_array_on_empty_stack() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    let array = call_builtin("stackToArray", &[stack], dummy_span(), &security()).unwrap();

    if let Value::Array(arr) = array {
        assert_eq!(arr.lock().unwrap().len(), 0);
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

    call_builtin(
        "stackPush",
        &[stack1.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "stackPush",
        &[stack2.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size1 = call_builtin("stackSize", &[stack1], dummy_span(), &security()).unwrap();
    let size2 = call_builtin("stackSize", &[stack2], dummy_span(), &security()).unwrap();

    assert_eq!(size1, Value::Number(1.0));
    assert_eq!(size2, Value::Number(1.0));
}

#[test]
fn test_large_stack_performance() {
    let stack = call_builtin("stackNew", &[], dummy_span(), &security()).unwrap();

    // Push 1000 elements
    for i in 0..1000 {
        call_builtin(
            "stackPush",
            &[stack.clone(), Value::Number(i as f64)],
            dummy_span(),
            &security(),
        )
        .unwrap();
    }

    let size = call_builtin("stackSize", &[stack.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1000.0));

    // Pop all elements (reverse order)
    for i in (0..1000).rev() {
        let result = call_builtin("stackPop", &[stack.clone()], dummy_span(), &security()).unwrap();
        assert_eq!(
            result,
            Value::Option(Some(Box::new(Value::Number(i as f64))))
        );
    }

    let empty = call_builtin("stackIsEmpty", &[stack], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}
