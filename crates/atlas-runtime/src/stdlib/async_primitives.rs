//! Async primitives stdlib functions
//!
//! This module provides Atlas stdlib functions for async task management, channels,
//! timers, and synchronization primitives.
//!
//! Task spawning and management:
//! - spawn: Spawn an async task
//! - taskJoin: Await task completion
//! - taskStatus: Check task status
//! - taskCancel: Cancel a running task
//! - taskId: Get task ID
//! - taskName: Get task name
//! - joinAll: Join multiple tasks
//!
//! Channels for message passing:
//! - channelBounded: Create bounded channel
//! - channelUnbounded: Create unbounded channel
//! - channelSend: Send message to channel
//! - channelReceive: Receive message from channel
//! - channelSelect: Select from multiple channels
//! - channelIsClosed: Check if channel is closed
//!
//! Timers and sleep:
//! - sleep: Sleep for milliseconds
//! - timer: Create a timer
//! - interval: Create repeating interval
//!
//! Timeout operations:
//! - timeout: Wrap future with timeout
//! - retryWithTimeout: Retry operation with timeout
//!
//! Async mutex:
//! - asyncMutex: Create async mutex
//! - asyncMutexLock: Lock mutex (returns future)
//! - asyncMutexGet: Get value from mutex
//! - asyncMutexSet: Set value in mutex

use crate::async_runtime;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ============================================================================
// Task Spawning and Management
// ============================================================================

/// Spawn an async task
///
/// Atlas signature: `spawn(future: Future<T>, name: string | null) -> TaskHandle`
pub fn spawn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    // Extract future
    let future = match &args[0] {
        Value::Future(f) => Arc::clone(f),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract name (optional)
    let name = match &args[1] {
        Value::Null => None,
        Value::String(s) => Some(s.as_ref().clone()),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected string or null for name, got {}",
                    args[1].type_name()
                ),
                span,
            })
        }
    };

    // Spawn task using async_runtime
    // Poll the AtlasFuture until it completes
    let handle = async_runtime::spawn_task(
        async move {
            // Poll the future state until resolved or rejected
            loop {
                match future.get_state() {
                    async_runtime::FutureState::Resolved(value) => return value,
                    async_runtime::FutureState::Rejected(error) => return error,
                    async_runtime::FutureState::Pending => {
                        // Wait a bit before polling again
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        },
        name,
    );

    Ok(Value::TaskHandle(Arc::new(Mutex::new(handle))))
}

/// Join a task (await its completion)
///
/// Atlas signature: `taskJoin(handle: TaskHandle) -> Future<T>`
pub fn task_join(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handle = match &args[0] {
        Value::TaskHandle(h) => h.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected TaskHandle, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Get join future from handle
    let future = handle.join();
    Ok(Value::Future(Arc::new(future)))
}

/// Get task status
///
/// Atlas signature: `taskStatus(handle: TaskHandle) -> string`
pub fn task_status(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handle = match &args[0] {
        Value::TaskHandle(h) => h.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected TaskHandle, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let status = match handle.status() {
        async_runtime::TaskStatus::Running => "Running",
        async_runtime::TaskStatus::Completed => "Completed",
        async_runtime::TaskStatus::Cancelled => "Cancelled",
        async_runtime::TaskStatus::Failed => "Failed",
    };

    Ok(Value::string(status))
}

/// Cancel a task
///
/// Atlas signature: `taskCancel(handle: TaskHandle) -> null`
pub fn task_cancel(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handle = match &args[0] {
        Value::TaskHandle(h) => h.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected TaskHandle, got {}", args[0].type_name()),
                span,
            })
        }
    };

    handle.cancel();
    Ok(Value::Null)
}

/// Get task ID
///
/// Atlas signature: `taskId(handle: TaskHandle) -> number`
pub fn task_id(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handle = match &args[0] {
        Value::TaskHandle(h) => h.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected TaskHandle, got {}", args[0].type_name()),
                span,
            })
        }
    };

    Ok(Value::Number(handle.id() as f64))
}

/// Get task name
///
/// Atlas signature: `taskName(handle: TaskHandle) -> string | null`
pub fn task_name(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handle = match &args[0] {
        Value::TaskHandle(h) => h.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected TaskHandle, got {}", args[0].type_name()),
                span,
            })
        }
    };

    match handle.name() {
        Some(name) => Ok(Value::string(name)),
        None => Ok(Value::Null),
    }
}

/// Join all tasks
///
/// Atlas signature: `joinAll(handles: TaskHandle[]) -> Future<T[]>`
pub fn join_all(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let handles_array = match &args[0] {
        Value::Array(arr) => arr.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected array of TaskHandles, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract TaskHandles from array
    let mut handles = Vec::new();
    for val in handles_array.iter() {
        match val {
            Value::TaskHandle(h) => {
                handles.push(h.lock().unwrap().clone());
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected TaskHandle in array, got {}", val.type_name()),
                    span,
                })
            }
        }
    }

    // Use async_runtime::join_all
    let future = async_runtime::join_all(handles);
    Ok(Value::Future(Arc::new(future)))
}

// ============================================================================
// Channels
// ============================================================================

/// Create an unbounded channel
///
/// Atlas signature: `channelUnbounded() -> [ChannelSender, ChannelReceiver]`
pub fn channel_unbounded(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let (sender, receiver) = async_runtime::channel_unbounded();

    Ok(Value::array(vec![
        Value::ChannelSender(Arc::new(Mutex::new(sender))),
        Value::ChannelReceiver(Arc::new(Mutex::new(receiver))),
    ]))
}

/// Create a bounded channel
///
/// Atlas signature: `channelBounded(capacity: number) -> [ChannelSender, ChannelReceiver]`
pub fn channel_bounded(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let capacity = match &args[0] {
        Value::Number(n) if *n >= 1.0 && n.fract() == 0.0 => *n as usize,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Channel capacity must be a positive integer".to_string(),
                span,
            })
        }
    };

    let (sender, receiver) = async_runtime::channel_bounded(capacity);

    Ok(Value::array(vec![
        Value::ChannelSender(Arc::new(Mutex::new(sender))),
        Value::ChannelReceiver(Arc::new(Mutex::new(receiver))),
    ]))
}

/// Send a message through a channel
///
/// Atlas signature: `channelSend(sender: ChannelSender, value: T) -> bool`
pub fn channel_send(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let sender = match &args[0] {
        Value::ChannelSender(s) => s.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected ChannelSender, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let value = args[1].clone();
    let success = sender.send(value);

    Ok(Value::Bool(success))
}

/// Receive a message from a channel
///
/// Atlas signature: `channelReceive(receiver: ChannelReceiver) -> Future<T>`
pub fn channel_receive(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let receiver = match &args[0] {
        Value::ChannelReceiver(r) => r.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected ChannelReceiver, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let future = receiver.receive();
    Ok(Value::Future(Arc::new(future)))
}

/// Select from multiple channels
///
/// Atlas signature: `channelSelect(receivers: ChannelReceiver[]) -> Future<[value, index]>`
pub fn channel_select(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let receivers_array = match &args[0] {
        Value::Array(arr) => arr.lock().unwrap(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!(
                    "Expected array of ChannelReceivers, got {}",
                    args[0].type_name()
                ),
                span,
            })
        }
    };

    // Extract ChannelReceivers from array
    let mut receivers = Vec::new();
    for val in receivers_array.iter() {
        match val {
            Value::ChannelReceiver(r) => {
                receivers.push(r.lock().unwrap().clone());
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected ChannelReceiver in array, got {}", val.type_name()),
                    span,
                })
            }
        }
    }

    let future = async_runtime::channel_select(receivers);
    Ok(Value::Future(Arc::new(future)))
}

/// Check if a channel sender is closed
///
/// Atlas signature: `channelIsClosed(sender: ChannelSender) -> bool`
pub fn channel_is_closed(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::ChannelSender(s) => Ok(Value::Bool(s.lock().unwrap().is_closed())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected ChannelSender, got {}", args[0].type_name()),
            span,
        }),
    }
}

// ============================================================================
// Sleep and Timers
// ============================================================================

/// Sleep for milliseconds
///
/// Atlas signature: `sleep(milliseconds: number) -> Future<null>`
pub fn sleep_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let milliseconds = match &args[0] {
        Value::Number(n) if *n >= 0.0 => *n as u64,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Sleep duration must be a non-negative number".to_string(),
                span,
            })
        }
    };

    let future = async_runtime::sleep(milliseconds);
    Ok(Value::Future(Arc::new(future)))
}

/// Create a timer
///
/// Atlas signature: `timer(milliseconds: number) -> Future<null>`
pub fn timer_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let milliseconds = match &args[0] {
        Value::Number(n) if *n >= 0.0 => *n as u64,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Timer duration must be a non-negative number".to_string(),
                span,
            })
        }
    };

    let future = async_runtime::timer(milliseconds);
    Ok(Value::Future(Arc::new(future)))
}

/// Create a repeating interval
///
/// Atlas signature: `interval(milliseconds: number) -> Future<null>`
pub fn interval_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let milliseconds = match &args[0] {
        Value::Number(n) if *n > 0.0 => *n as u64,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Interval duration must be a positive number".to_string(),
                span,
            })
        }
    };

    let future = async_runtime::interval(milliseconds);
    Ok(Value::Future(Arc::new(future)))
}

// ============================================================================
// Timeout Operations
// ============================================================================

/// Wrap a future with a timeout
///
/// Atlas signature: `timeout(future: Future<T>, milliseconds: number) -> Future<T>`
pub fn timeout_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let future = match &args[0] {
        Value::Future(f) => Arc::clone(f),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let milliseconds = match &args[1] {
        Value::Number(n) if *n >= 0.0 => *n as u64,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "Timeout duration must be a non-negative number".to_string(),
                span,
            })
        }
    };

    let timeout_future = async_runtime::timeout((*future).clone(), milliseconds);
    Ok(Value::Future(Arc::new(timeout_future)))
}

// ============================================================================
// Async Mutex
// ============================================================================

/// Create an async mutex
///
/// Atlas signature: `asyncMutex(value: T) -> AsyncMutex`
pub fn async_mutex_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let value = args[0].clone();
    let mutex = Arc::new(tokio::sync::Mutex::new(value));

    Ok(Value::AsyncMutex(mutex))
}

/// Get value from async mutex (blocking)
///
/// Atlas signature: `asyncMutexGet(mutex: AsyncMutex) -> T`
pub fn async_mutex_get(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let mutex = match &args[0] {
        Value::AsyncMutex(m) => Arc::clone(m),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected AsyncMutex, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Block on lock acquisition
    let value = async_runtime::block_on(async move {
        let guard = mutex.lock().await;
        guard.clone()
    });

    Ok(value)
}

/// Set value in async mutex (blocking)
///
/// Atlas signature: `asyncMutexSet(mutex: AsyncMutex, value: T) -> null`
pub fn async_mutex_set(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let mutex = match &args[0] {
        Value::AsyncMutex(m) => Arc::clone(m),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected AsyncMutex, got {}", args[0].type_name()),
                span,
            })
        }
    };

    let new_value = args[1].clone();

    // Block on lock acquisition and update
    async_runtime::block_on(async move {
        let mut guard = mutex.lock().await;
        *guard = new_value;
    });

    Ok(Value::Null)
}
