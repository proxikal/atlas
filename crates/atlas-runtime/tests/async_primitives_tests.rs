//! Comprehensive async primitives tests (Phase-11c)
//!
//! Tests for task spawning, channels, timers, mutex, and timeout operations.
//! Tests the async primitives from Atlas code (not direct Rust API calls).

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

// ============================================================================
// Task Spawning Tests (10 tests)
// ============================================================================

#[test]
fn test_spawn_simple_task() {
    let code = r#"
        let future = futureResolve(42);
        let handle = spawn(future, "simple_task");
        taskStatus(handle)
    "#;
    let result = eval_ok(code);
    // Should return a status string
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_task_returns_value() {
    let code = r#"
        let future = futureResolve(100);
        let handle = spawn(future, null);
        taskId(handle)
    "#;
    let result = eval_ok(code);
    // Just verify we get a task ID (number)
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_multiple_concurrent_tasks() {
    let code = r#"
        let h1 = spawn(async { 1 }, "task1");
        let h2 = spawn(async { 2 }, "task2");
        let h3 = spawn(async { 3 }, "task3");

        // Get task names
        taskName(h1)
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "task1"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_task_with_sleep() {
    let code = r#"
        let handle = spawn(async {
            await sleep(50);
            42
        }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_named_tasks() {
    let code = r#"
        let handle = spawn(async { 1 }, "named_task_123");
        taskName(handle)
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "named_task_123"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_task_id_uniqueness() {
    let code = r#"
        let h1 = spawn(async { 1 }, null);
        let h2 = spawn(async { 2 }, null);
        let id1 = taskId(h1);
        let id2 = taskId(h2);
        id1 != id2
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_spawn_with_string_result() {
    let code = r#"
        let handle = spawn(async { "hello" }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => "error"
        }
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_spawn_with_bool_result() {
    let code = r#"
        let handle = spawn(async { true }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => false
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_spawn_with_array_result() {
    let code = r#"
        let handle = spawn(async { [1, 2, 3] }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => len(val),
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_spawn_multiple_and_join_all() {
    let code = r#"
        let h1 = spawn(async { 10 }, null);
        let h2 = spawn(async { 20 }, null);
        let h3 = spawn(async { 30 }, null);

        let results = await joinAll([h1, h2, h3]);
        len(results)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(3.0));
}

// ============================================================================
// Channel Tests (12 tests)
// ============================================================================

#[test]
fn test_create_bounded_channel() {
    let code = r#"
        let channel = channelBounded(10);
        len(channel)
    "#;
    let result = eval_ok(code);
    // Should return array of [sender, receiver]
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_create_unbounded_channel() {
    let code = r#"
        let channel = channelUnbounded();
        len(channel)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_send_and_receive_message() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, 42);
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_multiple_messages_in_order() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, 1);
        channelSend(sender, 2);
        channelSend(sender, 3);

        let v1 = await channelReceive(receiver);
        let v2 = await channelReceive(receiver);
        let v3 = await channelReceive(receiver);

        v1 + v2 + v3
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_bounded_channel_capacity() {
    let code = r#"
        let channel = channelBounded(2);
        let sender = channel[0];

        channelSend(sender, 1);
        channelSend(sender, 2);
        channelSend(sender, 3);

        // Should succeed
        true
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_channel_with_string_messages() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, "hello");
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_channel_with_bool_messages() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, true);
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_channel_with_array_messages() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, [1, 2]);
        let result = await channelReceive(receiver);
        len(result)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_channel_multiple_sends_before_receive() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Send many messages
        for (var i: number = 0; i < 10; i = i + 1) {
            channelSend(sender, i);
        }

        // Receive first message
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_channel_interleaved_send_receive() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        var sum: number = 0;
        for (var i: number = 0; i < 5; i = i + 1) {
            channelSend(sender, i);
            let val = await channelReceive(receiver);
            sum = sum + val;
        }

        sum
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(10.0)); // 0+1+2+3+4 = 10
}

#[test]
fn test_channel_with_null_messages() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, null);
        await channelReceive(receiver)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Null);
}

#[test]
fn test_channel_send_receive_types() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        channelSend(sender, 42);
        channelSend(sender, "test");
        channelSend(sender, true);

        let v1 = await channelReceive(receiver);
        let v2 = await channelReceive(receiver);
        let v3 = await channelReceive(receiver);

        // All received
        true
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Sleep and Timer Tests (8 tests)
// ============================================================================

#[test]
fn test_sleep_for_duration() {
    let code = r#"
        await sleep(10);
        42
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_sleep_zero_duration() {
    let code = r#"
        await sleep(0);
        42
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_sleep_short_duration() {
    let code = r#"
        await sleep(5);
        123
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(123.0));
}

#[test]
fn test_sleep_in_task() {
    let code = r#"
        let handle = spawn(async {
            await sleep(10);
            42
        }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_sleep_sequence() {
    let code = r#"
        await sleep(5);
        await sleep(5);
        await sleep(5);
        42
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_sleep_with_value_return() {
    let code = r#"
        fn sleepAndReturn(value: number) -> Future<number> {
            return async {
                await sleep(5);
                value
            };
        }

        await sleepAndReturn(123)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(123.0));
}

#[test]
fn test_timer_basic() {
    let code = r#"
        await timer(10);
        42
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_interval_basic() {
    let code = r#"
        let handle = interval(10, async { 42 });
        // Cancel immediately
        cancelTimer(handle);
        true
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Timeout Tests (6 tests)
// ============================================================================

#[test]
fn test_timeout_completes_before_limit() {
    let code = r#"
        let result = await timeout(async { 42 }, 100);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_timeout_immediate_completion() {
    let code = r#"
        let result = await timeout(async { 42 }, 50);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_timeout_with_task() {
    let code = r#"
        let handle = spawn(async {
            let result = await timeout(async { 42 }, 50);
            match result {
                Ok(val) => val,
                Err(msg) => -1
            }
        }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => -2
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_timeout_preserves_value() {
    let code = r#"
        let result = await timeout(async { "test_value" }, 50);
        match result {
            Ok(val) => val,
            Err(msg) => "error"
        }
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "test_value"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_timeout_with_sleep() {
    let code = r#"
        let result = await timeout(async {
            await sleep(5);
            42
        }, 50);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_timeout_returns_result() {
    let code = r#"
        let result = await timeout(async { 100 }, 50);
        // Result should be a Result type
        match result {
            Ok(val) => true,
            Err(msg) => false
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// Async Mutex Tests (6 tests)
// ============================================================================

#[test]
fn test_create_async_mutex() {
    let code = r#"
        let mutex = asyncMutex(42);
        // Should create successfully
        true
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_async_mutex_lock_and_get() {
    let code = r#"
        let mutex = asyncMutex(42);
        let guard = await asyncMutexLock(mutex);
        asyncMutexGet(guard)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_async_mutex_update() {
    let code = r#"
        let mutex = asyncMutex(0);
        let guard = await asyncMutexLock(mutex);
        asyncMutexSet(guard, 42);
        asyncMutexGet(guard)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_async_mutex_with_string() {
    let code = r#"
        let mutex = asyncMutex("hello");
        let guard = await asyncMutexLock(mutex);
        asyncMutexGet(guard)
    "#;
    let result = eval_ok(code);
    match result {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("Expected String, got {:?}", other),
    }
}

#[test]
fn test_async_mutex_sequential_locks() {
    let code = r#"
        let mutex = asyncMutex(0);

        var guard: AsyncMutexGuard = await asyncMutexLock(mutex);
        asyncMutexSet(guard, 1);
        asyncMutexUnlock(guard);

        guard = await asyncMutexLock(mutex);
        asyncMutexGet(guard)
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_async_mutex_in_task() {
    let code = r#"
        let mutex = asyncMutex(0);
        let handle = spawn(async {
            let guard = await asyncMutexLock(mutex);
            asyncMutexSet(guard, 42);
            asyncMutexGet(guard)
        }, null);
        let result = await taskJoin(handle);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

// ============================================================================
// Integration Tests (5 tests)
// ============================================================================

#[test]
fn test_task_with_channel_communication() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        let producer = spawn(async {
            channelSend(sender, 10);
            channelSend(sender, 20);
            channelSend(sender, 30);
        }, "producer");

        let consumer = spawn(async {
            let v1 = await channelReceive(receiver);
            let v2 = await channelReceive(receiver);
            let v3 = await channelReceive(receiver);
            v1 + v2 + v3
        }, "consumer");

        let result = await taskJoin(consumer);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(60.0));
}

#[test]
fn test_timeout_with_channel() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Send after a delay
        spawn(async {
            await sleep(5);
            channelSend(sender, 42);
        }, null);

        // Try to receive with timeout
        let result = await timeout(channelReceive(receiver), 50);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_mutex_shared_between_tasks() {
    let code = r#"
        let mutex = asyncMutex(0);

        let t1 = spawn(async {
            let guard = await asyncMutexLock(mutex);
            let val = asyncMutexGet(guard);
            asyncMutexSet(guard, val + 10);
        }, null);

        let t2 = spawn(async {
            let guard = await asyncMutexLock(mutex);
            let val = asyncMutexGet(guard);
            asyncMutexSet(guard, val + 20);
        }, null);

        await taskJoin(t1);
        await taskJoin(t2);

        let guard = await asyncMutexLock(mutex);
        asyncMutexGet(guard)
    "#;
    let result = eval_ok(code);
    // Should be either 10 or 30 depending on task order
    assert!(matches!(result, Value::Number(n) if n == 10.0 || n == 30.0));
}

#[test]
fn test_complex_async_workflow() {
    let code = r#"
        let channel = channelUnbounded();
        let sender = channel[0];
        let receiver = channel[1];

        // Producer task
        spawn(async {
            for (var i: number = 1; i <= 5; i = i + 1) {
                await sleep(2);
                channelSend(sender, i);
            }
        }, "producer");

        // Consumer task
        var sum: number = 0;
        for (var i: number = 0; i < 5; i = i + 1) {
            let val = await channelReceive(receiver);
            sum = sum + val;
        }

        sum
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(15.0)); // 1+2+3+4+5
}

#[test]
fn test_retry_with_timeout_pattern() {
    let code = r#"
        fn operation() -> Future<number> {
            return async {
                await sleep(5);
                42
            };
        }

        let result = await retryWithTimeout(operation, 3, 50);
        match result {
            Ok(val) => val,
            Err(msg) => -1
        }
    "#;
    let result = eval_ok(code);
    assert_eq!(result, Value::Number(42.0));
}
