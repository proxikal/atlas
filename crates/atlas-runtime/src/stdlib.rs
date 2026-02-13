//! Standard library functions

use crate::value::{RuntimeError, Value};

/// Check if a function name is a builtin
pub fn is_builtin(name: &str) -> bool {
    matches!(name, "print" | "len" | "str")
}

/// Call a builtin function
///
/// The `call_span` parameter should be the span of the function call expression
/// in the source code, used for error reporting.
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    match name {
        "print" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            print(&args[0], call_span)?;
            Ok(Value::Null)
        }
        "len" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let length = len(&args[0], call_span)?;
            Ok(Value::Number(length))
        }
        "str" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = str(&args[0], call_span)?;
            Ok(Value::string(s))
        }
        _ => Err(RuntimeError::UnknownFunction {
            name: name.to_string(),
            span: call_span,
        }),
    }
}

/// Print a value to stdout
///
/// Only accepts string, number, bool, or null per stdlib specification.
pub fn print(value: &Value, span: crate::span::Span) -> Result<(), RuntimeError> {
    match value {
        Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null => {
            println!("{}", value.to_display_string());
            Ok(())
        }
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Get the length of a string or array
///
/// For strings, returns Unicode scalar count (not byte length).
/// For arrays, returns element count.
pub fn len(value: &Value, span: crate::span::Span) -> Result<f64, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.chars().count() as f64), // Unicode scalar count
        Value::Array(arr) => Ok(arr.borrow().len() as f64),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Convert a value to a string
///
/// Only accepts number, bool, or null per stdlib specification.
pub fn str(value: &Value, span: crate::span::Span) -> Result<String, RuntimeError> {
    match value {
        Value::Number(_) | Value::Bool(_) | Value::Null => Ok(value.to_display_string()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_len_string() {
        let val = Value::string("hello");
        assert_eq!(len(&val, Span::dummy()).unwrap() as i64, 5);
    }

    #[test]
    fn test_len_array() {
        let val = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(len(&val, Span::dummy()).unwrap() as i64, 2);
    }

    #[test]
    fn test_str() {
        let val = Value::Number(42.0);
        assert_eq!(str(&val, Span::dummy()).unwrap(), "42");
    }

    #[test]
    fn test_len_unicode_string() {
        // Test Unicode scalar count vs byte length
        let val = Value::string("hello");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 5.0); // 5 chars, 5 bytes

        let val = Value::string("héllo");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 5.0); // 5 chars, 6 bytes

        let val = Value::string("你好");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 2.0); // 2 chars, 6 bytes

        let val = Value::string("🎉");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 1.0); // 1 char (emoji), 4 bytes
    }

    #[test]
    fn test_len_empty_string() {
        let val = Value::string("");
        assert_eq!(len(&val, Span::dummy()).unwrap(), 0.0);
    }

    #[test]
    fn test_len_empty_array() {
        let val = Value::array(vec![]);
        assert_eq!(len(&val, Span::dummy()).unwrap(), 0.0);
    }

    #[test]
    fn test_len_invalid_type() {
        let val = Value::Number(42.0);
        assert!(len(&val, Span::dummy()).is_err());
        assert!(matches!(
            len(&val, Span::dummy()).unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_number() {
        assert_eq!(str(&Value::Number(42.0), Span::dummy()).unwrap(), "42");
        assert_eq!(str(&Value::Number(3.14), Span::dummy()).unwrap(), "3.14");
        assert_eq!(str(&Value::Number(-10.0), Span::dummy()).unwrap(), "-10");
    }

    #[test]
    fn test_str_bool() {
        assert_eq!(str(&Value::Bool(true), Span::dummy()).unwrap(), "true");
        assert_eq!(str(&Value::Bool(false), Span::dummy()).unwrap(), "false");
    }

    #[test]
    fn test_str_null() {
        assert_eq!(str(&Value::Null, Span::dummy()).unwrap(), "null");
    }

    #[test]
    fn test_call_builtin_print() {
        let result = call_builtin("print", &[Value::string("test")], Span::dummy());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_call_builtin_len() {
        let result = call_builtin("len", &[Value::string("hello")], Span::dummy());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_call_builtin_str() {
        let result = call_builtin("str", &[Value::Number(42.0)], Span::dummy());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::string("42"));
    }

    #[test]
    fn test_call_builtin_wrong_arg_count() {
        let result = call_builtin("print", &[], Span::dummy());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_call_builtin_unknown_function() {
        let result = call_builtin("unknown", &[Value::Null], Span::dummy());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::UnknownFunction { .. }
        ));
    }

    #[test]
    fn test_is_builtin() {
        assert!(is_builtin("print"));
        assert!(is_builtin("len"));
        assert!(is_builtin("str"));
        assert!(!is_builtin("unknown"));
        assert!(!is_builtin("foo"));
    }

    // ========================================================================
    // Type Restriction Tests (Spec Compliance)
    // ========================================================================

    #[test]
    fn test_print_accepts_all_valid_types() {
        // print() should accept string, number, bool, null per spec
        assert!(call_builtin("print", &[Value::string("test")], Span::dummy()).is_ok());
        assert!(call_builtin("print", &[Value::Number(42.0)], Span::dummy()).is_ok());
        assert!(call_builtin("print", &[Value::Bool(true)], Span::dummy()).is_ok());
        assert!(call_builtin("print", &[Value::Null], Span::dummy()).is_ok());
    }

    #[test]
    fn test_print_rejects_array() {
        // print() should reject arrays per spec
        let result =
            call_builtin("print", &[Value::array(vec![Value::Number(1.0)])], Span::dummy());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_print_null_displays_correctly() {
        // Verify that null prints as "null" per spec
        // This is a behavioral test - actual stdout not captured in unit test
        let result = call_builtin("print", &[Value::Null], Span::dummy());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_str_rejects_string() {
        // str() should only accept number|bool|null, not strings
        let result =
            call_builtin("str", &[Value::string("already a string")], Span::dummy());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_rejects_array() {
        // str() should only accept number|bool|null, not arrays
        let result = call_builtin("str", &[Value::array(vec![Value::Number(1.0)])], Span::dummy());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_accepts_all_valid_types() {
        // str() should accept number, bool, null per spec
        assert!(call_builtin("str", &[Value::Number(42.0)], Span::dummy()).is_ok());
        assert!(call_builtin("str", &[Value::Bool(true)], Span::dummy()).is_ok());
        assert!(call_builtin("str", &[Value::Null], Span::dummy()).is_ok());
    }
}
