//! Standard library functions

pub mod array;
pub mod io;
pub mod json;
pub mod math;
pub mod string;
pub mod types;

use crate::security::SecurityContext;
use crate::value::{RuntimeError, Value};

/// Check if a function name is a builtin (stdlib function, not intrinsic)
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print" | "len" | "str"
            // String functions
            | "split" | "join" | "trim" | "trimStart" | "trimEnd"
            | "indexOf" | "lastIndexOf" | "includes"
            | "toUpperCase" | "toLowerCase" | "substring" | "charAt" | "repeat" | "replace"
            | "padStart" | "padEnd" | "startsWith" | "endsWith"
            // Array functions (pure)
            | "pop" | "shift" | "unshift" | "reverse" | "concat" | "flatten"
            | "arrayIndexOf" | "arrayLastIndexOf" | "arrayIncludes" | "slice"
            // Math functions
            | "abs" | "floor" | "ceil" | "round" | "min" | "max"
            | "sqrt" | "pow" | "log"
            | "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
            | "clamp" | "sign" | "random"
            // JSON functions
            | "parseJSON" | "toJSON" | "isValidJSON" | "prettifyJSON" | "minifyJSON"
            // JSON extraction functions (called via method syntax)
            | "jsonAsString" | "jsonAsNumber" | "jsonAsBool" | "jsonIsNull"
            // Type checking functions
            | "typeof" | "isString" | "isNumber" | "isBool" | "isNull" | "isArray" | "isFunction"
            // Type conversion functions
            | "toString" | "toNumber" | "toBool" | "parseInt" | "parseFloat"
            // Option functions
            | "Some" | "None" | "is_some" | "is_none"
            // Result functions
            | "Ok" | "Err" | "is_ok" | "is_err"
            // Generic unwrap functions (work with both Option and Result)
            | "unwrap" | "unwrap_or"
            // File I/O functions
            | "readFile" | "writeFile" | "appendFile" | "fileExists"
            | "readDir" | "createDir" | "removeFile" | "removeDir"
            | "fileInfo" | "pathJoin"
    )
}

/// Check if a function name is an array intrinsic (handled in interpreter/VM)
pub fn is_array_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "map"
            | "filter"
            | "reduce"
            | "forEach"
            | "find"
            | "findIndex"
            | "flatMap"
            | "some"
            | "every"
            | "sort"
            | "sortBy"
    )
}

/// Extract string from value
fn extract_string(value: &Value, span: crate::span::Span) -> Result<&str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_ref()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract number from value
fn extract_number(value: &Value, span: crate::span::Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Extract array from value
fn extract_array(value: &Value, span: crate::span::Span) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(arr) => Ok(arr.borrow().clone()),
        _ => Err(RuntimeError::InvalidStdlibArgument { span }),
    }
}

/// Call a builtin function
///
/// The `call_span` parameter should be the span of the function call expression
/// in the source code, used for error reporting.
/// The `security` parameter is used for permission checks in I/O operations.
pub fn call_builtin(
    name: &str,
    args: &[Value],
    call_span: crate::span::Span,
    security: &SecurityContext,
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

        // String functions - Core Operations
        "split" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let sep = extract_string(&args[1], call_span)?;
            string::split(s, sep, call_span)
        }
        "join" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            let sep = extract_string(&args[1], call_span)?;
            let result = string::join(&arr, sep, call_span)?;
            Ok(Value::string(result))
        }
        "trim" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim(s)))
        }
        "trimStart" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim_start(s)))
        }
        "trimEnd" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::trim_end(s)))
        }

        // String functions - Search Operations
        "indexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Number(string::index_of(s, search)))
        }
        "lastIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Number(string::last_index_of(s, search)))
        }
        "includes" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::includes(s, search)))
        }

        // String functions - Transformation
        "toUpperCase" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::to_upper_case(s)))
        }
        "toLowerCase" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            Ok(Value::string(string::to_lower_case(s)))
        }
        "substring" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let start = extract_number(&args[1], call_span)?;
            let end = extract_number(&args[2], call_span)?;
            let result = string::substring(s, start, end, call_span)?;
            Ok(Value::string(result))
        }
        "charAt" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let index = extract_number(&args[1], call_span)?;
            let result = string::char_at(s, index, call_span)?;
            Ok(Value::string(result))
        }
        "repeat" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let count = extract_number(&args[1], call_span)?;
            let result = string::repeat(s, count, call_span)?;
            Ok(Value::string(result))
        }
        "replace" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let search = extract_string(&args[1], call_span)?;
            let replacement = extract_string(&args[2], call_span)?;
            Ok(Value::string(string::replace(s, search, replacement)))
        }

        // String functions - Formatting
        "padStart" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let length = extract_number(&args[1], call_span)?;
            let fill = extract_string(&args[2], call_span)?;
            let result = string::pad_start(s, length, fill, call_span)?;
            Ok(Value::string(result))
        }
        "padEnd" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let length = extract_number(&args[1], call_span)?;
            let fill = extract_string(&args[2], call_span)?;
            let result = string::pad_end(s, length, fill, call_span)?;
            Ok(Value::string(result))
        }
        "startsWith" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let prefix = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::starts_with(s, prefix)))
        }
        "endsWith" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let s = extract_string(&args[0], call_span)?;
            let suffix = extract_string(&args[1], call_span)?;
            Ok(Value::Bool(string::ends_with(s, suffix)))
        }

        // Array functions - Core Operations
        "pop" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::pop(&arr, call_span)
        }
        "shift" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::shift(&arr, call_span)
        }
        "unshift" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(array::unshift(&arr, args[1].clone()))
        }
        "reverse" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(array::reverse(&arr))
        }
        "concat" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr1 = extract_array(&args[0], call_span)?;
            let arr2 = extract_array(&args[1], call_span)?;
            Ok(array::concat(&arr1, &arr2))
        }
        "flatten" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            array::flatten(&arr, call_span)
        }

        // Array functions - Search Operations
        "arrayIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Number(array::index_of(&arr, &args[1])))
        }
        "arrayLastIndexOf" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Number(array::last_index_of(&arr, &args[1])))
        }
        "arrayIncludes" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            Ok(Value::Bool(array::includes(&arr, &args[1])))
        }

        // Array functions - Slicing
        "slice" => {
            if args.len() != 3 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let arr = extract_array(&args[0], call_span)?;
            let start = extract_number(&args[1], call_span)?;
            let end = extract_number(&args[2], call_span)?;
            array::slice(&arr, start, end, call_span)
        }

        // Math functions - Basic Operations
        "abs" => math::abs(args, call_span),
        "floor" => math::floor(args, call_span),
        "ceil" => math::ceil(args, call_span),
        "round" => math::round(args, call_span),
        "min" => math::min(args, call_span),
        "max" => math::max(args, call_span),

        // Math functions - Exponential/Power
        "sqrt" => math::sqrt(args, call_span),
        "pow" => math::pow(args, call_span),
        "log" => math::log(args, call_span),

        // Math functions - Trigonometry
        "sin" => math::sin(args, call_span),
        "cos" => math::cos(args, call_span),
        "tan" => math::tan(args, call_span),
        "asin" => math::asin(args, call_span),
        "acos" => math::acos(args, call_span),
        "atan" => math::atan(args, call_span),

        // Math functions - Utilities
        "clamp" => math::clamp(args, call_span),
        "sign" => math::sign(args, call_span),
        "random" => math::random(args, call_span),

        // JSON functions
        "parseJSON" => json::parse_json(args, call_span),
        "toJSON" => json::to_json(args, call_span),
        "isValidJSON" => json::is_valid_json(args, call_span),
        "prettifyJSON" => json::prettify_json(args, call_span),
        "minifyJSON" => json::minify_json(args, call_span),

        // JSON extraction functions (called via method syntax)
        "jsonAsString" => json::json_as_string(args, call_span),
        "jsonAsNumber" => json::json_as_number(args, call_span),
        "jsonAsBool" => json::json_as_bool(args, call_span),
        "jsonIsNull" => json::json_is_null(args, call_span),

        // Type checking functions
        "typeof" => types::type_of(args, call_span),
        "isString" => types::is_string(args, call_span),
        "isNumber" => types::is_number(args, call_span),
        "isBool" => types::is_bool(args, call_span),
        "isNull" => types::is_null(args, call_span),
        "isArray" => types::is_array(args, call_span),
        "isFunction" => types::is_function(args, call_span),

        // Type conversion functions
        "toString" => types::to_string(args, call_span),
        "toNumber" => types::to_number(args, call_span),
        "toBool" => types::to_bool(args, call_span),
        "parseInt" => types::parse_int(args, call_span),
        "parseFloat" => types::parse_float(args, call_span),

        // Option<T> constructors
        "Some" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::some(args[0].clone()))
        }
        "None" => {
            if !args.is_empty() {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::none())
        }

        // Option<T> helpers
        "is_some" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_some = types::is_some(&args[0], call_span)?;
            Ok(Value::Bool(is_some))
        }
        "is_none" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_none = types::is_none(&args[0], call_span)?;
            Ok(Value::Bool(is_none))
        }

        // Result<T,E> constructors
        "Ok" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::ok(args[0].clone()))
        }
        "Err" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            Ok(types::err(args[0].clone()))
        }

        // Result<T,E> helpers
        "is_ok" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_ok = types::is_ok(&args[0], call_span)?;
            Ok(Value::Bool(is_ok))
        }
        "is_err" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            let is_err = types::is_err(&args[0], call_span)?;
            Ok(Value::Bool(is_err))
        }

        // File I/O functions
        "readFile" => io::read_file(args, call_span, security),
        "writeFile" => io::write_file(args, call_span, security),
        "appendFile" => io::append_file(args, call_span, security),
        "fileExists" => io::file_exists(args, call_span, security),
        "readDir" => io::read_dir(args, call_span, security),
        "createDir" => io::create_dir(args, call_span, security),
        "removeFile" => io::remove_file(args, call_span, security),
        "removeDir" => io::remove_dir(args, call_span, security),
        "fileInfo" => io::file_info(args, call_span, security),
        "pathJoin" => io::path_join(args, call_span, security),

        // Generic unwrap (works with both Option and Result)
        "unwrap" => {
            if args.len() != 1 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_option(&args[0], call_span),
                Value::Result(_) => types::unwrap_result(&args[0], call_span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap() requires Option or Result value".to_string(),
                    span: call_span,
                }),
            }
        }
        "unwrap_or" => {
            if args.len() != 2 {
                return Err(RuntimeError::InvalidStdlibArgument { span: call_span });
            }
            match &args[0] {
                Value::Option(_) => types::unwrap_or_option(&args[0], args[1].clone(), call_span),
                Value::Result(_) => types::unwrap_or_result(&args[0], args[1].clone(), call_span),
                _ => Err(RuntimeError::TypeError {
                    msg: "unwrap_or() requires Option or Result value".to_string(),
                    span: call_span,
                }),
            }
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
        assert_eq!(str(&Value::Number(2.5), Span::dummy()).unwrap(), "2.5");
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
        let security = SecurityContext::allow_all();
        let result = call_builtin("print", &[Value::string("test")], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_call_builtin_len() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("len", &[Value::string("hello")], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_call_builtin_str() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("str", &[Value::Number(42.0)], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::string("42"));
    }

    #[test]
    fn test_call_builtin_wrong_arg_count() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("print", &[], Span::dummy(), &security);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_call_builtin_unknown_function() {
        let security = SecurityContext::allow_all();
        let result = call_builtin("unknown", &[Value::Null], Span::dummy(), &security);
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
        let security = SecurityContext::allow_all();
        // print() should accept string, number, bool, null per spec
        assert!(call_builtin("print", &[Value::string("test")], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Number(42.0)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Bool(true)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("print", &[Value::Null], Span::dummy(), &security).is_ok());
    }

    #[test]
    fn test_print_rejects_array() {
        let security = SecurityContext::allow_all();
        // print() should reject arrays per spec
        let result = call_builtin(
            "print",
            &[Value::array(vec![Value::Number(1.0)])],
            Span::dummy(),
            &security,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_print_null_displays_correctly() {
        let security = SecurityContext::allow_all();
        // Verify that null prints as "null" per spec
        // This is a behavioral test - actual stdout not captured in unit test
        let result = call_builtin("print", &[Value::Null], Span::dummy(), &security);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    #[test]
    fn test_str_rejects_string() {
        let security = SecurityContext::allow_all();
        // str() should only accept number|bool|null, not strings
        let result = call_builtin(
            "str",
            &[Value::string("already a string")],
            Span::dummy(),
            &security,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_rejects_array() {
        let security = SecurityContext::allow_all();
        // str() should only accept number|bool|null, not arrays
        let result = call_builtin(
            "str",
            &[Value::array(vec![Value::Number(1.0)])],
            Span::dummy(),
            &security,
        );
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidStdlibArgument { .. }
        ));
    }

    #[test]
    fn test_str_accepts_all_valid_types() {
        let security = SecurityContext::allow_all();
        // str() should accept number, bool, null per spec
        assert!(call_builtin("str", &[Value::Number(42.0)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("str", &[Value::Bool(true)], Span::dummy(), &security).is_ok());
        assert!(call_builtin("str", &[Value::Null], Span::dummy(), &security).is_ok());
    }
}
