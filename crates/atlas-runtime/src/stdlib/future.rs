//! Future/Promise stdlib functions for asynchronous operations
//!
//! This module provides Atlas stdlib functions for working with Futures:
//! - futureResolve: Create a resolved future
//! - futureReject: Create a rejected future
//! - futureNew: Create a pending future (with executor)
//! - futureThen: Chain a success handler
//! - futureCatch: Chain an error handler
//! - futureAll: Combine multiple futures
//! - futureRace: Get first completed future
//! - futureIsPending, futureIsResolved, futureIsRejected: Status checks

use crate::async_runtime::{future_all, future_race, AtlasFuture};
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::sync::Arc;

/// Create a resolved future with a value
///
/// Atlas signature: `futureResolve(value: T) -> Future<T>`
pub fn future_resolve(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let future = AtlasFuture::resolved(args[0].clone());
    Ok(Value::Future(Arc::new(future)))
}

/// Create a rejected future with an error
///
/// Atlas signature: `futureReject(error: T) -> Future<never>`
pub fn future_reject(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let future = AtlasFuture::rejected(args[0].clone());
    Ok(Value::Future(Arc::new(future)))
}

/// Create a new pending future
///
/// Note: Without an executor callback (which would require closures), this creates
/// a pending future that stays pending. This is mainly useful for testing.
/// In phase-11b/11c, we'll add proper executor support.
///
/// Atlas signature: `futureNew() -> Future<T>`
pub fn future_new(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let future = AtlasFuture::new_pending();
    Ok(Value::Future(Arc::new(future)))
}

/// Check if a future is pending
///
/// Atlas signature: `futureIsPending(future: Future<T>) -> bool`
pub fn future_is_pending(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_pending())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Check if a future is resolved
///
/// Atlas signature: `futureIsResolved(future: Future<T>) -> bool`
pub fn future_is_resolved(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_resolved())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Check if a future is rejected
///
/// Atlas signature: `futureIsRejected(future: Future<T>) -> bool`
pub fn future_is_rejected(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Future(f) => Ok(Value::Bool(f.is_rejected())),
        _ => Err(RuntimeError::TypeError {
            msg: format!("Expected Future, got {}", args[0].type_name()),
            span,
        }),
    }
}

/// Chain a success handler to a future
///
/// Note: In phase-11a, this only works with immediately resolved/rejected futures.
/// Dynamic chaining with pending futures will be added in phase-11b/11c with executor support.
///
/// Atlas signature: `futureThen(future: Future<T>, handler: fn(T) -> U) -> Future<U>`
pub fn future_then(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let _future = match &args[0] {
        Value::Future(f) => f,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // For phase-11a, we can only handle immediately resolved/rejected futures
    // with a simple transformation. Full callback support requires changes
    // to how we pass functions to stdlib (need FunctionRef, not Value)
    // For now, return error indicating this is not yet fully implemented
    Err(RuntimeError::TypeError {
        msg: "futureThen with dynamic handlers requires full executor support (phase-11b/11c)"
            .to_string(),
        span,
    })
}

/// Chain an error handler to a future
///
/// Note: In phase-11a, this only works with immediately resolved/rejected futures.
/// Dynamic chaining with pending futures will be added in phase-11b/11c with executor support.
///
/// Atlas signature: `futureCatch(future: Future<T>, handler: fn(E) -> T) -> Future<T>`
pub fn future_catch(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let _future = match &args[0] {
        Value::Future(f) => f,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Future, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // For phase-11a, we can only handle immediately resolved/rejected futures
    // with a simple transformation. Full callback support requires changes
    // to how we pass functions to stdlib (need FunctionRef, not Value)
    // For now, return error indicating this is not yet fully implemented
    Err(RuntimeError::TypeError {
        msg: "futureCatch with dynamic handlers requires full executor support (phase-11b/11c)"
            .to_string(),
        span,
    })
}

/// Combine multiple futures into one
///
/// Returns a future that resolves when all input futures resolve,
/// with an array of all results. Rejects if any future rejects.
///
/// Atlas signature: `futureAll(futures: Array<Future<T>>) -> Future<Array<T>>`
pub fn future_all_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract futures from array
    let mut futures = Vec::new();
    for value in futures_array.lock().unwrap().iter() {
        match value {
            Value::Future(f) => futures.push((**f).clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    // Combine futures
    let result = future_all(futures);
    Ok(Value::Future(Arc::new(result)))
}

/// Return the first future to complete
///
/// Creates a future that adopts the state of the first future to complete
/// (either resolved or rejected).
///
/// Atlas signature: `futureRace(futures: Array<Future<T>>) -> Future<T>`
pub fn future_race_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let futures_array = match &args[0] {
        Value::Array(arr) => arr,
        _ => {
            return Err(RuntimeError::TypeError {
                msg: format!("Expected Array, got {}", args[0].type_name()),
                span,
            })
        }
    };

    // Extract futures from array
    let mut futures = Vec::new();
    for value in futures_array.lock().unwrap().iter() {
        match value {
            Value::Future(f) => futures.push((**f).clone()),
            _ => {
                return Err(RuntimeError::TypeError {
                    msg: format!("Expected array of Futures, got {}", value.type_name()),
                    span,
                })
            }
        }
    }

    // Race futures
    let result = future_race(futures);
    Ok(Value::Future(Arc::new(result)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    #[test]
    fn test_future_resolve() {
        let result = future_resolve(&[Value::Number(42.0)], span()).unwrap();
        match result {
            Value::Future(f) => {
                assert!(f.is_resolved());
            }
            _ => panic!("Expected Future value"),
        }
    }

    #[test]
    fn test_future_reject() {
        let result = future_reject(&[Value::string("error")], span()).unwrap();
        match result {
            Value::Future(f) => {
                assert!(f.is_rejected());
            }
            _ => panic!("Expected Future value"),
        }
    }

    #[test]
    fn test_future_new() {
        let result = future_new(&[], span()).unwrap();
        match result {
            Value::Future(f) => {
                assert!(f.is_pending());
            }
            _ => panic!("Expected Future value"),
        }
    }

    #[test]
    fn test_future_is_pending() {
        let future = future_new(&[], span()).unwrap();
        let result = future_is_pending(&[future], span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_future_is_resolved() {
        let future = future_resolve(&[Value::Number(42.0)], span()).unwrap();
        let result = future_is_resolved(&[future], span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_future_is_rejected() {
        let future = future_reject(&[Value::string("error")], span()).unwrap();
        let result = future_is_rejected(&[future], span()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_future_all_success() {
        let f1 = future_resolve(&[Value::Number(1.0)], span()).unwrap();
        let f2 = future_resolve(&[Value::Number(2.0)], span()).unwrap();
        let f3 = future_resolve(&[Value::Number(3.0)], span()).unwrap();

        let futures = Value::array(vec![f1, f2, f3]);
        let result = future_all_fn(&[futures], span()).unwrap();

        match result {
            Value::Future(f) => {
                assert!(f.is_resolved());
            }
            _ => panic!("Expected Future value"),
        }
    }

    #[test]
    fn test_future_all_rejection() {
        let f1 = future_resolve(&[Value::Number(1.0)], span()).unwrap();
        let f2 = future_reject(&[Value::string("error")], span()).unwrap();
        let f3 = future_resolve(&[Value::Number(3.0)], span()).unwrap();

        let futures = Value::array(vec![f1, f2, f3]);
        let result = future_all_fn(&[futures], span()).unwrap();

        match result {
            Value::Future(f) => {
                assert!(f.is_rejected());
            }
            _ => panic!("Expected Future value"),
        }
    }

    #[test]
    fn test_future_race() {
        let f1 = future_resolve(&[Value::Number(1.0)], span()).unwrap();
        let f2 = future_new(&[], span()).unwrap();
        let f3 = future_resolve(&[Value::Number(3.0)], span()).unwrap();

        let futures = Value::array(vec![f1, f2, f3]);
        let result = future_race_fn(&[futures], span()).unwrap();

        match result {
            Value::Future(f) => {
                assert!(f.is_resolved());
            }
            _ => panic!("Expected Future value"),
        }
    }
}
