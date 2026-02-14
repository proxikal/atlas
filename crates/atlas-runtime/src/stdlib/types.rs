//! Built-in generic types: Option<T> and Result<T,E>

use crate::span::Span;
use crate::value::{RuntimeError, Value};

// ============================================================================
// Option<T> Functions
// ============================================================================

/// Construct Some(value) - Option<T> with a value
pub fn some(value: Value) -> Value {
    Value::Option(Some(Box::new(value)))
}

/// Construct None - Option<T> without a value
pub fn none() -> Value {
    Value::Option(None)
}

/// Check if Option has a value (is Some)
pub fn is_some(opt: &Value, span: Span) -> Result<bool, RuntimeError> {
    match opt {
        Value::Option(opt) => Ok(opt.is_some()),
        _ => Err(RuntimeError::TypeError {
            msg: "is_some() requires Option value".to_string(),
            span,
        }),
    }
}

/// Check if Option is None
pub fn is_none(opt: &Value, span: Span) -> Result<bool, RuntimeError> {
    match opt {
        Value::Option(opt) => Ok(opt.is_none()),
        _ => Err(RuntimeError::TypeError {
            msg: "is_none() requires Option value".to_string(),
            span,
        }),
    }
}

/// Unwrap Option value, panic if None
pub fn unwrap_option(opt: &Value, span: Span) -> Result<Value, RuntimeError> {
    match opt {
        Value::Option(Some(val)) => Ok((**val).clone()),
        Value::Option(None) => Err(RuntimeError::TypeError {
            msg: "unwrap() called on None".to_string(),
            span,
        }),
        _ => Err(RuntimeError::TypeError {
            msg: "unwrap() requires Option value".to_string(),
            span,
        }),
    }
}

/// Unwrap Option value with default
pub fn unwrap_or_option(opt: &Value, default: Value, span: Span) -> Result<Value, RuntimeError> {
    match opt {
        Value::Option(Some(val)) => Ok((**val).clone()),
        Value::Option(None) => Ok(default),
        _ => Err(RuntimeError::TypeError {
            msg: "unwrap_or() requires Option value".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Result<T,E> Functions
// ============================================================================

/// Construct Ok(value) - successful Result
pub fn ok(value: Value) -> Value {
    Value::Result(Ok(Box::new(value)))
}

/// Construct Err(error) - failed Result
pub fn err(error: Value) -> Value {
    Value::Result(Err(Box::new(error)))
}

/// Check if Result is Ok
pub fn is_ok(res: &Value, span: Span) -> Result<bool, RuntimeError> {
    match res {
        Value::Result(res) => Ok(res.is_ok()),
        _ => Err(RuntimeError::TypeError {
            msg: "is_ok() requires Result value".to_string(),
            span,
        }),
    }
}

/// Check if Result is Err
pub fn is_err(res: &Value, span: Span) -> Result<bool, RuntimeError> {
    match res {
        Value::Result(res) => Ok(res.is_err()),
        _ => Err(RuntimeError::TypeError {
            msg: "is_err() requires Result value".to_string(),
            span,
        }),
    }
}

/// Unwrap Result value, panic if Err
pub fn unwrap_result(res: &Value, span: Span) -> Result<Value, RuntimeError> {
    match res {
        Value::Result(Ok(val)) => Ok((**val).clone()),
        Value::Result(Err(err)) => Err(RuntimeError::TypeError {
            msg: format!("unwrap() called on Err({})", err),
            span,
        }),
        _ => Err(RuntimeError::TypeError {
            msg: "unwrap() requires Result value".to_string(),
            span,
        }),
    }
}

/// Unwrap Result value with default
pub fn unwrap_or_result(res: &Value, default: Value, span: Span) -> Result<Value, RuntimeError> {
    match res {
        Value::Result(Ok(val)) => Ok((**val).clone()),
        Value::Result(Err(_)) => Ok(default),
        _ => Err(RuntimeError::TypeError {
            msg: "unwrap_or() requires Result value".to_string(),
            span,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_none() {
        let some_val = some(Value::Number(42.0));
        assert!(matches!(some_val, Value::Option(Some(_))));

        let none_val = none();
        assert!(matches!(none_val, Value::Option(None)));
    }

    #[test]
    fn test_is_some_is_none() {
        let some_val = some(Value::Number(42.0));
        let none_val = none();

        assert_eq!(is_some(&some_val, Span::dummy()).unwrap(), true);
        assert_eq!(is_none(&some_val, Span::dummy()).unwrap(), false);

        assert_eq!(is_some(&none_val, Span::dummy()).unwrap(), false);
        assert_eq!(is_none(&none_val, Span::dummy()).unwrap(), true);
    }

    #[test]
    fn test_unwrap_option() {
        let some_val = some(Value::Number(42.0));
        let unwrapped = unwrap_option(&some_val, Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(42.0));

        let none_val = none();
        assert!(unwrap_option(&none_val, Span::dummy()).is_err());
    }

    #[test]
    fn test_unwrap_or_option() {
        let some_val = some(Value::Number(42.0));
        let unwrapped = unwrap_or_option(&some_val, Value::Number(0.0), Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(42.0));

        let none_val = none();
        let unwrapped = unwrap_or_option(&none_val, Value::Number(99.0), Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(99.0));
    }

    #[test]
    fn test_ok_err() {
        let ok_val = ok(Value::Number(42.0));
        assert!(matches!(ok_val, Value::Result(Ok(_))));

        let err_val = err(Value::string("error"));
        assert!(matches!(err_val, Value::Result(Err(_))));
    }

    #[test]
    fn test_is_ok_is_err() {
        let ok_val = ok(Value::Number(42.0));
        let err_val = err(Value::string("error"));

        assert_eq!(is_ok(&ok_val, Span::dummy()).unwrap(), true);
        assert_eq!(is_err(&ok_val, Span::dummy()).unwrap(), false);

        assert_eq!(is_ok(&err_val, Span::dummy()).unwrap(), false);
        assert_eq!(is_err(&err_val, Span::dummy()).unwrap(), true);
    }

    #[test]
    fn test_unwrap_result() {
        let ok_val = ok(Value::Number(42.0));
        let unwrapped = unwrap_result(&ok_val, Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(42.0));

        let err_val = err(Value::string("error"));
        assert!(unwrap_result(&err_val, Span::dummy()).is_err());
    }

    #[test]
    fn test_unwrap_or_result() {
        let ok_val = ok(Value::Number(42.0));
        let unwrapped = unwrap_or_result(&ok_val, Value::Number(0.0), Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(42.0));

        let err_val = err(Value::string("error"));
        let unwrapped = unwrap_or_result(&err_val, Value::Number(99.0), Span::dummy()).unwrap();
        assert_eq!(unwrapped, Value::Number(99.0));
    }
}
