//! Integration tests for Queue collection
//!
//! Tests FIFO semantics, Option returns, and reference semantics.

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
fn test_create_empty_queue() {
    let result = call_builtin("queueNew", &[], dummy_span(), &security());
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Queue(_)));
}

#[test]
fn test_new_queue_has_size_zero() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
    let size = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
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

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));
}

#[test]
fn test_dequeue_fifo_order() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let first = call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(first, Value::Option(Some(Box::new(Value::Number(1.0)))));

    let second = call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(second, Value::Option(Some(Box::new(Value::Number(2.0)))));

    let third = call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(third, Value::Option(Some(Box::new(Value::Number(3.0)))));
}

#[test]
fn test_dequeue_from_empty_returns_none() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    let result = call_builtin("queueDequeue", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(result, Value::Option(None));
}

#[test]
fn test_enqueue_after_dequeue() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));

    let result = call_builtin("queueDequeue", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(result, Value::Option(Some(Box::new(Value::Number(2.0)))));
}

#[test]
fn test_queue_accepts_any_value_type() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(42.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::string("hello")],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Bool(true)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Null],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(4.0));
}

// ============================================================================
// Peek Tests
// ============================================================================

#[test]
fn test_peek_returns_front_without_removing() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(42.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let peeked = call_builtin("queuePeek", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(peeked, Value::Option(Some(Box::new(Value::Number(42.0)))));

    let size = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1.0));
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

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size_before =
        call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    call_builtin("queuePeek", &[queue.clone()], dummy_span(), &security()).unwrap();
    let size_after = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();

    assert_eq!(size_before, size_after);
}

// ============================================================================
// Size and IsEmpty Tests
// ============================================================================

#[test]
fn test_size_reflects_element_count() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    let size0 = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size0, Value::Number(0.0));

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    let size1 = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size1, Value::Number(1.0));

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    let size2 = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(size2, Value::Number(2.0));
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

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(false));
}

#[test]
fn test_is_empty_true_after_dequeuing_all() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
    call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();

    let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

// ============================================================================
// Clear Tests
// ============================================================================

#[test]
fn test_clear_removes_all_elements() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    call_builtin("queueClear", &[queue.clone()], dummy_span(), &security()).unwrap();

    let size = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(0.0));

    let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

#[test]
fn test_clear_on_empty_queue_is_safe() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    let result = call_builtin("queueClear", &[queue.clone()], dummy_span(), &security());
    assert!(result.is_ok());

    let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}

// ============================================================================
// ToArray Tests
// ============================================================================

#[test]
fn test_to_array_returns_fifo_order() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(3.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let array = call_builtin("queueToArray", &[queue], dummy_span(), &security()).unwrap();

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
fn test_to_array_doesnt_modify_queue() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size_before =
        call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    call_builtin("queueToArray", &[queue.clone()], dummy_span(), &security()).unwrap();
    let size_after = call_builtin("queueSize", &[queue], dummy_span(), &security()).unwrap();

    assert_eq!(size_before, size_after);
}

#[test]
fn test_to_array_on_empty_queue() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    let array = call_builtin("queueToArray", &[queue], dummy_span(), &security()).unwrap();

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
fn test_multiple_queues_are_independent() {
    let queue1 = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();
    let queue2 = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    call_builtin(
        "queueEnqueue",
        &[queue1.clone(), Value::Number(1.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();
    call_builtin(
        "queueEnqueue",
        &[queue2.clone(), Value::Number(2.0)],
        dummy_span(),
        &security(),
    )
    .unwrap();

    let size1 = call_builtin("queueSize", &[queue1], dummy_span(), &security()).unwrap();
    let size2 = call_builtin("queueSize", &[queue2], dummy_span(), &security()).unwrap();

    assert_eq!(size1, Value::Number(1.0));
    assert_eq!(size2, Value::Number(1.0));
}

#[test]
fn test_large_queue_performance() {
    let queue = call_builtin("queueNew", &[], dummy_span(), &security()).unwrap();

    // Enqueue 1000 elements
    for i in 0..1000 {
        call_builtin(
            "queueEnqueue",
            &[queue.clone(), Value::Number(i as f64)],
            dummy_span(),
            &security(),
        )
        .unwrap();
    }

    let size = call_builtin("queueSize", &[queue.clone()], dummy_span(), &security()).unwrap();
    assert_eq!(size, Value::Number(1000.0));

    // Dequeue all elements
    for i in 0..1000 {
        let result =
            call_builtin("queueDequeue", &[queue.clone()], dummy_span(), &security()).unwrap();
        assert_eq!(
            result,
            Value::Option(Some(Box::new(Value::Number(i as f64))))
        );
    }

    let empty = call_builtin("queueIsEmpty", &[queue], dummy_span(), &security()).unwrap();
    assert_eq!(empty, Value::Bool(true));
}
