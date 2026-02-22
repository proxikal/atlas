//! Future/Promise implementation for Atlas
//!
//! Provides a Future type representing pending asynchronous computations.
//! Similar to JavaScript Promises, Atlas Futures can be in three states:
//! - Pending: computation in progress
//! - Resolved: computation completed successfully
//! - Rejected: computation failed with an error
//!
//! Futures support chaining via `then` and `catch` methods, and combinators
//! like `futureAll` and `futureRace` for working with multiple futures.

use crate::value::Value;
use std::fmt;
use std::sync::{Arc, Mutex};

/// Future state representing the status of an async computation
#[derive(Clone)]
pub enum FutureState {
    /// Computation is in progress
    Pending,
    /// Computation completed successfully with a value
    Resolved(Value),
    /// Computation failed with an error
    Rejected(Value),
}

impl fmt::Debug for FutureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FutureState::Pending => write!(f, "Pending"),
            FutureState::Resolved(_) => write!(f, "Resolved"),
            FutureState::Rejected(_) => write!(f, "Rejected"),
        }
    }
}

/// Atlas Future - represents a pending asynchronous computation
///
/// Futures are the foundation of Atlas's async I/O system. They represent
/// values that may not be available yet but will be computed asynchronously.
///
/// # State Machine
/// - Pending → Resolved (success)
/// - Pending → Rejected (error)
/// - Once Resolved or Rejected, state is final
///
/// # Example Usage (Atlas code)
/// ```atlas
/// let f = futureResolve(42);
/// let f2 = then(f, |x| x * 2);
/// // f2 will resolve to 84
/// ```
#[derive(Clone)]
pub struct AtlasFuture {
    state: Arc<Mutex<FutureState>>,
}

impl AtlasFuture {
    /// Create a new pending future
    pub fn new_pending() -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Pending)),
        }
    }

    /// Create an immediately resolved future
    pub fn resolved(value: Value) -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Resolved(value))),
        }
    }

    /// Create an immediately rejected future
    pub fn rejected(error: Value) -> Self {
        Self {
            state: Arc::new(Mutex::new(FutureState::Rejected(error))),
        }
    }

    /// Check if the future is pending
    pub fn is_pending(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Pending)
    }

    /// Check if the future is resolved
    pub fn is_resolved(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Resolved(_))
    }

    /// Check if the future is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(*self.state.lock().unwrap(), FutureState::Rejected(_))
    }

    /// Get the current state (cloned)
    pub fn get_state(&self) -> FutureState {
        self.state.lock().unwrap().clone()
    }

    /// Resolve the future with a value
    ///
    /// This transitions the future from Pending to Resolved.
    /// If the future is already resolved or rejected, this is a no-op.
    pub fn resolve(&self, value: Value) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, FutureState::Pending) {
            *state = FutureState::Resolved(value);
        }
    }

    /// Reject the future with an error
    ///
    /// This transitions the future from Pending to Rejected.
    /// If the future is already resolved or rejected, this is a no-op.
    pub fn reject(&self, error: Value) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, FutureState::Pending) {
            *state = FutureState::Rejected(error);
        }
    }

    /// Apply a transformation to a resolved future
    ///
    /// Creates a new future that will contain the result of applying
    /// the handler to this future's value (when it resolves).
    ///
    /// If this future rejects, the error propagates to the new future.
    pub fn then<F>(&self, handler: F) -> Self
    where
        F: FnOnce(Value) -> Value + 'static,
    {
        match self.get_state() {
            FutureState::Resolved(value) => {
                // Future already resolved, apply handler immediately
                let result = handler(value);
                Self::resolved(result)
            }
            FutureState::Rejected(error) => {
                // Future rejected, propagate error
                Self::rejected(error)
            }
            FutureState::Pending => {
                // For now, pending futures can't be chained dynamically
                // This would require a more complex executor with callback queues
                // For phase-11a, we focus on immediately resolved/rejected futures
                Self::new_pending()
            }
        }
    }

    /// Handle a rejected future
    ///
    /// Creates a new future that will contain the result of applying
    /// the error handler to this future's error (when it rejects).
    ///
    /// If this future resolves successfully, the value propagates unchanged.
    pub fn catch<F>(&self, handler: F) -> Self
    where
        F: FnOnce(Value) -> Value + 'static,
    {
        match self.get_state() {
            FutureState::Resolved(value) => {
                // Future resolved, propagate value
                Self::resolved(value)
            }
            FutureState::Rejected(error) => {
                // Future rejected, apply error handler
                let result = handler(error);
                Self::resolved(result)
            }
            FutureState::Pending => {
                // Pending future - would need executor support
                Self::new_pending()
            }
        }
    }
}

impl fmt::Debug for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Future({:?})", *self.state.lock().unwrap())
    }
}

impl fmt::Display for AtlasFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.state.lock().unwrap() {
            FutureState::Pending => write!(f, "Future(pending)"),
            FutureState::Resolved(_) => write!(f, "Future(resolved)"),
            FutureState::Rejected(_) => write!(f, "Future(rejected)"),
        }
    }
}

/// Combine multiple futures into one that resolves when all resolve
///
/// Returns a future containing an array of all results.
/// If any future rejects, the combined future rejects immediately.
pub fn future_all(futures: Vec<AtlasFuture>) -> AtlasFuture {
    let mut results = Vec::new();

    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => results.push(value),
            FutureState::Rejected(error) => {
                // Any rejection causes immediate rejection
                return AtlasFuture::rejected(error);
            }
            FutureState::Pending => {
                // If any future is pending, result is pending
                return AtlasFuture::new_pending();
            }
        }
    }

    // All futures resolved successfully
    AtlasFuture::resolved(Value::array(results))
}

/// Return the first future to complete (resolve or reject)
///
/// Creates a future that adopts the state of the first future to complete.
pub fn future_race(futures: Vec<AtlasFuture>) -> AtlasFuture {
    for future in futures {
        match future.get_state() {
            FutureState::Resolved(value) => {
                return AtlasFuture::resolved(value);
            }
            FutureState::Rejected(error) => {
                return AtlasFuture::rejected(error);
            }
            FutureState::Pending => {
                // Continue checking other futures
            }
        }
    }

    // All futures still pending
    AtlasFuture::new_pending()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolved_future() {
        let f = AtlasFuture::resolved(Value::Number(42.0));
        assert!(f.is_resolved());
        assert!(!f.is_pending());
        assert!(!f.is_rejected());

        match f.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected resolved future"),
        }
    }

    #[test]
    fn test_rejected_future() {
        let f = AtlasFuture::rejected(Value::string("error"));
        assert!(f.is_rejected());
        assert!(!f.is_pending());
        assert!(!f.is_resolved());

        match f.get_state() {
            FutureState::Rejected(Value::String(s)) => assert_eq!(&**s, "error"),
            _ => panic!("Expected rejected future"),
        }
    }

    #[test]
    fn test_pending_future() {
        let f = AtlasFuture::new_pending();
        assert!(f.is_pending());
        assert!(!f.is_resolved());
        assert!(!f.is_rejected());
    }

    #[test]
    fn test_resolve_pending_future() {
        let f = AtlasFuture::new_pending();
        assert!(f.is_pending());

        f.resolve(Value::Number(100.0));
        assert!(f.is_resolved());

        match f.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 100.0),
            _ => panic!("Expected resolved future"),
        }
    }

    #[test]
    fn test_reject_pending_future() {
        let f = AtlasFuture::new_pending();
        assert!(f.is_pending());

        f.reject(Value::string("failed"));
        assert!(f.is_rejected());

        match f.get_state() {
            FutureState::Rejected(Value::String(s)) => assert_eq!(&**s, "failed"),
            _ => panic!("Expected rejected future"),
        }
    }

    #[test]
    fn test_resolve_idempotent() {
        let f = AtlasFuture::resolved(Value::Number(1.0));
        f.resolve(Value::Number(2.0)); // Should be ignored

        match f.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 1.0),
            _ => panic!("Expected original resolved value"),
        }
    }

    #[test]
    fn test_then_on_resolved() {
        let f = AtlasFuture::resolved(Value::Number(10.0));
        let f2 = f.then(|_v| {
            if let Value::Number(n) = _v {
                Value::Number(n * 2.0)
            } else {
                _v
            }
        });

        match f2.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 20.0),
            _ => panic!("Expected resolved future with doubled value"),
        }
    }

    #[test]
    fn test_then_on_rejected() {
        let f = AtlasFuture::rejected(Value::string("error"));
        let f2 = f.then(|_v| Value::Number(42.0)); // Should not be called

        assert!(f2.is_rejected());
        match f2.get_state() {
            FutureState::Rejected(Value::String(s)) => assert_eq!(&**s, "error"),
            _ => panic!("Expected error to propagate"),
        }
    }

    #[test]
    fn test_catch_on_rejected() {
        let f = AtlasFuture::rejected(Value::string("error"));
        let f2 = f.catch(|_err| Value::Number(0.0)); // Recover from error

        assert!(f2.is_resolved());
        match f2.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 0.0),
            _ => panic!("Expected recovered value"),
        }
    }

    #[test]
    fn test_catch_on_resolved() {
        let f = AtlasFuture::resolved(Value::Number(42.0));
        let f2 = f.catch(|_err| Value::Number(0.0)); // Should not be called

        assert!(f2.is_resolved());
        match f2.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected value to propagate"),
        }
    }

    #[test]
    fn test_future_all_success() {
        let futures = vec![
            AtlasFuture::resolved(Value::Number(1.0)),
            AtlasFuture::resolved(Value::Number(2.0)),
            AtlasFuture::resolved(Value::Number(3.0)),
        ];

        let result = future_all(futures);
        assert!(result.is_resolved());

        match result.get_state() {
            FutureState::Resolved(Value::Array(arr)) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected array of results"),
        }
    }

    #[test]
    fn test_future_all_with_rejection() {
        let futures = vec![
            AtlasFuture::resolved(Value::Number(1.0)),
            AtlasFuture::rejected(Value::string("error")),
            AtlasFuture::resolved(Value::Number(3.0)),
        ];

        let result = future_all(futures);
        assert!(result.is_rejected());

        match result.get_state() {
            FutureState::Rejected(Value::String(s)) => assert_eq!(&**s, "error"),
            _ => panic!("Expected rejection"),
        }
    }

    #[test]
    fn test_future_all_empty() {
        let result = future_all(vec![]);
        assert!(result.is_resolved());

        match result.get_state() {
            FutureState::Resolved(Value::Array(arr)) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("Expected empty array"),
        }
    }

    #[test]
    fn test_future_race_first_resolved() {
        let futures = vec![
            AtlasFuture::resolved(Value::Number(1.0)),
            AtlasFuture::new_pending(),
            AtlasFuture::resolved(Value::Number(3.0)),
        ];

        let result = future_race(futures);
        assert!(result.is_resolved());

        match result.get_state() {
            FutureState::Resolved(Value::Number(n)) => assert_eq!(n, 1.0),
            _ => panic!("Expected first resolved value"),
        }
    }

    #[test]
    fn test_future_race_first_rejected() {
        let futures = vec![
            AtlasFuture::rejected(Value::string("error")),
            AtlasFuture::new_pending(),
        ];

        let result = future_race(futures);
        assert!(result.is_rejected());
    }

    #[test]
    fn test_future_race_all_pending() {
        let futures = vec![AtlasFuture::new_pending(), AtlasFuture::new_pending()];

        let result = future_race(futures);
        assert!(result.is_pending());
    }

    #[test]
    fn test_clone_future() {
        let f1 = AtlasFuture::resolved(Value::Number(42.0));
        let f2 = f1.clone();

        assert!(f1.is_resolved());
        assert!(f2.is_resolved());

        // Both should share the same state
        match (f1.get_state(), f2.get_state()) {
            (
                FutureState::Resolved(Value::Number(n1)),
                FutureState::Resolved(Value::Number(n2)),
            ) => {
                assert_eq!(n1, 42.0);
                assert_eq!(n2, 42.0);
            }
            _ => panic!("Expected both to be resolved with same value"),
        }
    }
}
