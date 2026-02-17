//! Async runtime infrastructure for Atlas
//!
//! This module provides the foundation for asynchronous I/O operations in Atlas:
//! - Future type for representing pending computations
//! - Tokio runtime integration for executing async operations
//! - Task management and spawning
//! - Channels for message passing
//! - Async primitives (sleep, timers, mutex, timeout)
//!
//! The async runtime enables non-blocking I/O operations without requiring
//! language-level async/await syntax (reserved for future versions).

pub mod channel;
pub mod future;
pub mod primitives;
pub mod task;

pub use channel::{
    channel_bounded, channel_select, channel_unbounded, ChannelReceiver, ChannelSender,
};
pub use future::{future_all, future_race, AtlasFuture, FutureState};
pub use primitives::{interval, retry_with_timeout, sleep, timeout, timer, AsyncMutex};
pub use task::{join_all, spawn_and_await, spawn_task, TaskHandle, TaskStatus};

use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;

/// Global tokio runtime for async operations
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Thread-local LocalSet for spawning !Send futures
thread_local! {
    static LOCAL_SET: std::cell::RefCell<Option<LocalSet>> = const { std::cell::RefCell::new(None) };
}

/// Initialize the global tokio runtime
///
/// This must be called before any async operations. It creates a multi-threaded
/// tokio runtime that will be used for all async operations in Atlas.
///
/// # Panics
/// Panics if the runtime fails to initialize
pub fn init_runtime() {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    });
}

/// Get a reference to the global tokio runtime
///
/// Initializes the runtime if it hasn't been initialized yet.
pub fn runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    })
}

/// Spawn a !Send task on the local set
///
/// This allows spawning futures that contain Rc/RefCell (our Value type).
/// Tasks run on the same thread but provide true async concurrency.
pub fn spawn_local<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + 'static,
    F::Output: 'static,
{
    // Initialize LocalSet if needed
    LOCAL_SET.with(|cell| {
        let mut local_set = cell.lock().unwrap();
        if local_set.is_none() {
            *local_set = Some(LocalSet::new());
        }
    });

    // Spawn on the LocalSet
    tokio::task::spawn_local(future)
}

/// Block on a future until it completes
///
/// This bridges the sync/async boundary by blocking the current thread
/// until the future completes. Uses LocalSet for !Send futures.
pub fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    // Create a new LocalSet for this block_on call
    // This ensures each block_on has its own execution context
    let local_set = LocalSet::new();
    runtime().block_on(local_set.run_until(future))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_initialization() {
        // Runtime should initialize successfully
        let _ = runtime();
    }

    #[test]
    fn test_block_on() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_spawn_local() {
        let handle = spawn_local(async { "hello" });
        let result = block_on(handle).unwrap();
        assert_eq!(result, "hello");
    }
}
