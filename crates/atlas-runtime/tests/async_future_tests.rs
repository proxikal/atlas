//! Integration tests for Future/Promise type and async foundation
//!
//! Tests Phase-11a: Async Foundation - Future Type & Runtime
//! - Future value type
//! - Future state machine
//! - Future constructor functions
//! - Future combinators (futureAll, futureRace)
//! - Runtime integration
//! - Error handling

use atlas_runtime::api::Runtime;
use atlas_runtime::*;

/// Helper to evaluate code with the interpreter
fn eval(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new(api::ExecutionMode::Interpreter);
    Ok(runtime.eval(code)?)
}

/// Helper to evaluate code expecting success
fn eval_ok(code: &str) -> Value {
    eval(code).unwrap()
}

/// Helper to evaluate code with VM
fn eval_vm(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut runtime = Runtime::new(api::ExecutionMode::VM);
    Ok(runtime.eval(code)?)
}

// ============================================================================
// Future Type Tests
// ============================================================================

#[test]
fn test_future_resolve_creates_resolved_future() {
    let result = eval_ok("futureResolve(42)");
    assert_eq!(result.type_name(), "future");

    // Check it's resolved
    let is_resolved = eval_ok("futureIsResolved(futureResolve(42))");
    assert_eq!(is_resolved, Value::Bool(true));
}

#[test]
fn test_future_reject_creates_rejected_future() {
    let result = eval_ok("futureReject(\"error\")");
    assert_eq!(result.type_name(), "future");

    // Check it's rejected
    let is_rejected = eval_ok("futureIsRejected(futureReject(\"error\"))");
    assert_eq!(is_rejected, Value::Bool(true));
}

#[test]
fn test_future_new_creates_pending_future() {
    let result = eval_ok("futureNew()");
    assert_eq!(result.type_name(), "future");

    // Check it's pending
    let is_pending = eval_ok("futureIsPending(futureNew())");
    assert_eq!(is_pending, Value::Bool(true));
}

#[test]
fn test_future_clone_shares_state() {
    let code = r#"
        let f1 = futureResolve(100);
        let f2 = f1;
        futureIsResolved(f2)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_display_format() {
    // Futures should have a display format
    let result = eval_ok("toString(futureResolve(42))");
    match result {
        Value::String(s) => {
            assert!(s.contains("Future"));
        }
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_future_type_name() {
    let result = eval_ok("typeof(futureResolve(42))");
    assert_eq!(result, Value::string("future"));
}

#[test]
fn test_future_in_array() {
    let result = eval_ok("[futureResolve(1), futureResolve(2), futureResolve(3)]");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.lock().unwrap().len(), 3);
            for val in arr.lock().unwrap().iter() {
                assert_eq!(val.type_name(), "future");
            }
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_future_multiple_references() {
    let code = r#"
        let f = futureResolve(42);
        let arr = [f, f, f];
        len(arr)
    "#;
    assert_eq!(eval_ok(code), Value::Number(3.0));
}

// ============================================================================
// Future State Machine Tests
// ============================================================================

#[test]
fn test_pending_to_resolved_transition() {
    // In phase-11a, futures created with futureNew stay pending
    // This test verifies pending futures remain pending
    let result = eval_ok("futureIsPending(futureNew())");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_resolved_state_is_final() {
    let code = r#"
        let f = futureResolve(42);
        futureIsResolved(f)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_rejected_state_is_final() {
    let code = r#"
        let f = futureReject("error");
        futureIsRejected(f)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_state_check_resolved() {
    let code = r#"
        let f = futureResolve(100);
        [futureIsResolved(f), futureIsRejected(f), futureIsPending(f)]
    "#;
    let result = eval_ok(code);
    match result {
        Value::Array(arr) => {
            let values = arr.lock().unwrap();
            assert_eq!(values[0], Value::Bool(true)); // isResolved
            assert_eq!(values[1], Value::Bool(false)); // isRejected
            assert_eq!(values[2], Value::Bool(false)); // isPending
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_future_state_check_rejected() {
    let code = r#"
        let f = futureReject("fail");
        [futureIsResolved(f), futureIsRejected(f), futureIsPending(f)]
    "#;
    let result = eval_ok(code);
    match result {
        Value::Array(arr) => {
            let values = arr.lock().unwrap();
            assert_eq!(values[0], Value::Bool(false)); // isResolved
            assert_eq!(values[1], Value::Bool(true)); // isRejected
            assert_eq!(values[2], Value::Bool(false)); // isPending
        }
        _ => panic!("Expected array"),
    }
}

// ============================================================================
// Future Combinators Tests
// ============================================================================

#[test]
fn test_future_all_with_all_resolved() {
    let code = r#"
        let futures = [
            futureResolve(1),
            futureResolve(2),
            futureResolve(3)
        ];
        let result = futureAll(futures);
        futureIsResolved(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_all_with_one_rejected() {
    let code = r#"
        let futures = [
            futureResolve(1),
            futureReject("error"),
            futureResolve(3)
        ];
        let result = futureAll(futures);
        futureIsRejected(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_all_empty_array() {
    let code = r#"
        let result = futureAll([]);
        futureIsResolved(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_all_with_pending() {
    let code = r#"
        let futures = [
            futureResolve(1),
            futureNew(),  // pending
            futureResolve(3)
        ];
        let result = futureAll(futures);
        futureIsPending(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_race_first_resolved() {
    let code = r#"
        let futures = [
            futureResolve(1),
            futureNew(),  // pending
            futureResolve(3)
        ];
        let result = futureRace(futures);
        futureIsResolved(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_race_first_rejected() {
    let code = r#"
        let f1 = futureReject("error");
        let f2 = futureNew();
        let futures = [f1, f2];
        let result = futureRace(futures);
        futureIsRejected(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_race_all_pending() {
    let code = r#"
        let futures = [
            futureNew(),
            futureNew()
        ];
        let result = futureRace(futures);
        futureIsPending(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

// ============================================================================
// Runtime Integration Tests
// ============================================================================

#[test]
fn test_tokio_runtime_initializes() {
    // Just creating a future should initialize the runtime
    eval_ok("futureResolve(42)");
    // No panic = success
}

#[test]
fn test_multiple_concurrent_futures() {
    let code = r#"
        let f1 = futureResolve(1);
        let f2 = futureResolve(2);
        let f3 = futureResolve(3);
        let f4 = futureResolve(4);
        let f5 = futureResolve(5);
        let futures = [f1, f2, f3, f4, f5];
        let result = futureAll(futures);
        futureIsResolved(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_future_reject_propagates_error() {
    let code = r#"
        let f = futureReject("something went wrong");
        futureIsRejected(f)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_all_rejects_on_any_error() {
    let code = r#"
        let futures = [
            futureResolve(1),
            futureResolve(2),
            futureReject("error in middle"),
            futureResolve(4)
        ];
        futureIsRejected(futureAll(futures))
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_wrong_type_to_future_is_pending() {
    let result = eval("futureIsPending(42)");
    assert!(result.is_err());
}

#[test]
fn test_future_all_requires_array() {
    let result = eval("futureAll(42)");
    assert!(result.is_err());
}

#[test]
fn test_future_all_requires_array_of_futures() {
    let result = eval("futureAll([1, 2, 3])");
    assert!(result.is_err());
}

// ============================================================================
// Integration & Real-World Patterns Tests
// ============================================================================

#[test]
fn test_future_pattern_conditional_resolution() {
    let code = r#"
        let shouldSucceed = true;
        let future = futureResolve("success");
        futureIsResolved(future)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_pattern_collect_results() {
    let code = r#"
        let f1 = futureResolve(2);
        let f2 = futureResolve(4);
        let f3 = futureResolve(6);
        let futures = [f1, f2, f3];
        futureIsResolved(futureAll(futures))
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_pattern_error_handling() {
    let code = r#"
        let operation = futureReject("network error");
        let isRejected = futureIsRejected(operation);
        isRejected
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_pattern_timeout_simulation() {
    let code = r#"
        // Simulate timeout by racing with immediate rejection
        let operation = futureNew();  // Never completes
        let timeout = futureReject("timeout");
        let result = futureRace([operation, timeout]);
        futureIsRejected(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

#[test]
fn test_future_pattern_fallback_chain() {
    let code = r#"
        let primary = futureReject("primary failed");
        let secondary = futureResolve("secondary success");
        let result = futureRace([primary, secondary]);
        // First to complete (primary rejection) wins
        futureIsRejected(result)
    "#;
    assert_eq!(eval_ok(code), Value::Bool(true));
}

// ============================================================================
// VM Parity Tests
// ============================================================================

#[test]
fn test_vm_future_resolve() {
    let code = "futureIsResolved(futureResolve(42))";
    let result = eval_vm(code).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_future_reject() {
    let code = "futureIsRejected(futureReject(\"error\"))";
    let result = eval_vm(code).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_future_all() {
    let code = r#"
        futureIsResolved(futureAll([futureResolve(1), futureResolve(2)]))
    "#;
    let result = eval_vm(code).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_future_race() {
    let code = r#"
        futureIsResolved(futureRace([futureResolve(1), futureNew()]))
    "#;
    let result = eval_vm(code).unwrap();
    assert_eq!(result, Value::Bool(true));
}
