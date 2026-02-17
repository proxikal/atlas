//! Task spawning and management for Atlas
//!
//! Provides task spawning, cancellation, and status tracking.
//! Tasks run concurrently on the tokio runtime and can be managed
//! through TaskHandle values.

use crate::async_runtime::AtlasFuture;
use crate::value::Value;
use futures_util::FutureExt;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::task::JoinHandle;

/// Global task ID counter
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task was cancelled
    Cancelled,
    /// Task failed with an error
    Failed,
}

/// Inner task state shared between TaskHandle and the actual task
pub(crate) struct TaskState {
    id: u64,
    name: Option<String>,
    status: StdMutex<TaskStatus>,
    cancelled: AtomicBool,
    result: StdMutex<Option<Result<Value, String>>>,
}

/// Handle to a spawned task
///
/// Provides control over a running task including status checking,
/// cancellation, and awaiting completion.
pub struct TaskHandle {
    state: Arc<TaskState>,
    // We store the handle but can't use it directly due to Send requirements
    // Instead we track completion via the state
    _marker: std::marker::PhantomData<JoinHandle<()>>,
}

impl TaskHandle {
    /// Create a new task handle
    fn new(name: Option<String>) -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            state: Arc::new(TaskState {
                id,
                name,
                status: StdMutex::new(TaskStatus::Running),
                cancelled: AtomicBool::new(false),
                result: StdMutex::new(None),
            }),
            _marker: std::marker::PhantomData,
        }
    }

    /// Get task ID
    pub fn id(&self) -> u64 {
        self.state.id
    }

    /// Get task name
    pub fn name(&self) -> Option<&str> {
        self.state.name.as_deref()
    }

    /// Get current task status
    pub fn status(&self) -> TaskStatus {
        *self.state.status.lock().unwrap()
    }

    /// Check if task is pending (running)
    pub fn is_pending(&self) -> bool {
        matches!(self.status(), TaskStatus::Running)
    }

    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status(), TaskStatus::Completed)
    }

    /// Check if task was cancelled
    pub fn is_cancelled(&self) -> bool {
        matches!(self.status(), TaskStatus::Cancelled)
    }

    /// Check if task failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status(), TaskStatus::Failed)
    }

    /// Cancel the task
    pub fn cancel(&self) {
        self.state.cancelled.store(true, Ordering::SeqCst);
        let mut status = self.state.status.lock().unwrap();
        if *status == TaskStatus::Running {
            *status = TaskStatus::Cancelled;
        }
    }

    /// Check if cancellation was requested
    pub fn is_cancellation_requested(&self) -> bool {
        self.state.cancelled.load(Ordering::SeqCst)
    }

    /// Wait for task completion and get result
    ///
    /// Returns a Future that resolves to the task's result value.
    pub fn join(&self) -> AtlasFuture {
        // Check if already complete
        let result = self.state.result.lock().unwrap().clone();

        if let Some(res) = result {
            match res {
                Ok(value) => return AtlasFuture::resolved(value),
                Err(error) => return AtlasFuture::rejected(Value::string(error)),
            }
        }

        // Task still running - return pending future
        // Note: In a full implementation, this would poll the task
        // For now, we return pending and rely on external polling
        AtlasFuture::new_pending()
    }

    /// Mark task as completed with result
    #[allow(dead_code)]
    fn complete(&self, result: Result<Value, String>) {
        let mut status = self.state.status.lock().unwrap();
        if *status == TaskStatus::Running {
            *status = if result.is_ok() {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
        }
        *self.state.result.lock().unwrap() = Some(result);
    }

    /// Clone the task state reference
    pub(crate) fn state_ref(&self) -> Arc<TaskState> {
        Arc::clone(&self.state)
    }
}

impl Clone for TaskHandle {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            _marker: std::marker::PhantomData,
        }
    }
}

impl fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskHandle")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("status", &self.status())
            .finish()
    }
}

impl fmt::Display for TaskHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "Task({}, \"{}\")", self.id(), name)
        } else {
            write!(f, "Task({})", self.id())
        }
    }
}

/// Spawn a new async task
///
/// The task executes concurrently on the tokio runtime.
/// Returns a TaskHandle that can be used to check status or await completion.
///
/// # Arguments
/// * `future` - The async computation to run
/// * `name` - Optional task name for debugging
///
/// # Example
/// ```ignore
/// let handle = spawn_task(async { 42 }, Some("my-task"));
/// let result = handle.join(); // Returns Future<number>
/// ```
pub fn spawn_task<F>(future: F, name: Option<String>) -> TaskHandle
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    let handle = TaskHandle::new(name);
    let state = handle.state_ref();

    // Spawn the task execution in a LocalSet context
    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        crate::async_runtime::block_on(async move {
            // Spawn on LocalSet
            tokio::task::spawn_local(async move {
                // Check for cancellation before starting
                if state_clone.cancelled.load(Ordering::SeqCst) {
                    let mut status = state_clone.status.lock().unwrap();
                    *status = TaskStatus::Cancelled;
                    return;
                }

                // Execute the future
                let result = std::panic::AssertUnwindSafe(future).catch_unwind().await;

                match result {
                    Ok(value) => {
                        // Task completed successfully
                        let mut status = state_clone.status.lock().unwrap();
                        if *status == TaskStatus::Running {
                            *status = TaskStatus::Completed;
                            *state_clone.result.lock().unwrap() = Some(Ok(value));
                        }
                    }
                    Err(panic_err) => {
                        // Task panicked
                        let error_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Task panicked".to_string()
                        };

                        let mut status = state_clone.status.lock().unwrap();
                        if *status == TaskStatus::Running {
                            *status = TaskStatus::Failed;
                            *state_clone.result.lock().unwrap() = Some(Err(error_msg));
                        }
                    }
                }
            })
            .await
            .ok(); // Ignore join errors
        });
    });

    handle
}

/// Spawn a task and immediately await its result
///
/// This is a convenience function that spawns a task and blocks until completion.
/// Useful for simple async operations that don't need concurrent management.
pub fn spawn_and_await<F>(future: F) -> Result<Value, String>
where
    F: std::future::Future<Output = Value> + Send + 'static,
{
    let handle = spawn_task(future, None);

    // Poll for completion
    // In a full implementation, this would be more efficient
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);

    loop {
        if handle.is_completed() {
            let result = handle.state.result.lock().unwrap().clone();
            return result.unwrap_or(Ok(Value::Null));
        } else if handle.is_failed() || handle.is_cancelled() {
            let result = handle.state.result.lock().unwrap().clone();
            return result.unwrap_or(Err("Task failed".to_string()));
        }

        if start.elapsed() > timeout {
            return Err("Task timeout".to_string());
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Join multiple tasks
///
/// Returns a Future that resolves when all tasks complete.
/// Results are returned in the same order as the input handles.
pub fn join_all(handles: Vec<TaskHandle>) -> AtlasFuture {
    let mut results = Vec::new();
    let mut all_complete = true;

    for handle in handles {
        let result = handle.state.result.lock().unwrap().clone();

        if let Some(res) = result {
            match res {
                Ok(value) => results.push(value),
                Err(error) => {
                    // Any error causes immediate rejection
                    return AtlasFuture::rejected(Value::string(error));
                }
            }
        } else {
            // At least one task still pending
            all_complete = false;
            break;
        }
    }

    if all_complete {
        AtlasFuture::resolved(Value::array(results))
    } else {
        AtlasFuture::new_pending()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_simple_task() {
        let handle = spawn_task(async { Value::Number(42.0) }, None);
        assert!(handle.id() > 0);
        assert!(handle.name().is_none());

        // Wait a bit for task to complete
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_completed());
    }

    #[test]
    fn test_named_task() {
        let handle = spawn_task(
            async { Value::string("result") },
            Some("test-task".to_string()),
        );

        assert_eq!(handle.name(), Some("test-task"));
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(handle.is_completed());
    }

    #[test]
    fn test_task_cancellation() {
        let handle = spawn_task(
            async {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                Value::Null
            },
            None,
        );

        assert!(handle.is_pending());
        handle.cancel();
        assert!(handle.is_cancelled());
    }

    #[test]
    fn test_spawn_and_await() {
        let result = spawn_and_await(async { Value::Number(100.0) });
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Number(n) => assert_eq!(n, 100.0),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_join_all_success() {
        let handles = vec![
            spawn_task(async { Value::Number(1.0) }, None),
            spawn_task(async { Value::Number(2.0) }, None),
            spawn_task(async { Value::Number(3.0) }, None),
        ];

        std::thread::sleep(std::time::Duration::from_millis(200));

        let future = join_all(handles);
        assert!(future.is_resolved());
    }
}
