//! Built-in generic types: Option<T> and Result<T,E>
//! Type checking and conversion utilities

use crate::json_value::JsonValue;
use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
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

/// Unwrap Result value with custom error message, panic if Err
pub fn expect_result(res: &Value, message: &str, span: Span) -> Result<Value, RuntimeError> {
    match res {
        Value::Result(Ok(val)) => Ok((**val).clone()),
        Value::Result(Err(err)) => Err(RuntimeError::TypeError {
            msg: format!("{}: {}", message, err),
            span,
        }),
        _ => Err(RuntimeError::TypeError {
            msg: "expect() requires Result value".to_string(),
            span,
        }),
    }
}

/// Unwrap Option value with custom error message, panic if None
pub fn expect_option(opt: &Value, message: &str, span: Span) -> Result<Value, RuntimeError> {
    match opt {
        Value::Option(Some(val)) => Ok((**val).clone()),
        Value::Option(None) => Err(RuntimeError::TypeError {
            msg: message.to_string(),
            span,
        }),
        _ => Err(RuntimeError::TypeError {
            msg: "expect() requires Option value".to_string(),
            span,
        }),
    }
}

// Note: map, map_err, and_then, or_else are implemented as intrinsics
// in interpreter/expr.rs and vm/mod.rs because they require callback support

/// Convert Result to Option, dropping Err
pub fn result_ok(res: &Value, span: Span) -> Result<Value, RuntimeError> {
    match res {
        Value::Result(Ok(val)) => Ok(Value::Option(Some(val.clone()))),
        Value::Result(Err(_)) => Ok(Value::Option(None)),
        _ => Err(RuntimeError::TypeError {
            msg: "ok() requires Result value".to_string(),
            span,
        }),
    }
}

/// Convert Result to Option, extracting Err and dropping Ok
pub fn result_err(res: &Value, span: Span) -> Result<Value, RuntimeError> {
    match res {
        Value::Result(Ok(_)) => Ok(Value::Option(None)),
        Value::Result(Err(err)) => Ok(Value::Option(Some(err.clone()))),
        _ => Err(RuntimeError::TypeError {
            msg: "err() requires Result value".to_string(),
            span,
        }),
    }
}

// ============================================================================
// Type Checking Functions
// ============================================================================

/// Get the type name of a value as a string
///
/// Returns one of: "null", "bool", "number", "string", "array", "function",
/// "json", "option", "result"
pub fn type_of(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let type_name = match &args[0] {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Function(_) => "function",
        Value::Builtin(_) => "builtin",
        Value::NativeFunction(_) => "function",
        Value::JsonValue(_) => "json",
        Value::Option(_) => "option",
        Value::Result(_) => "result",
        Value::HashMap(_) => "hashmap",
        Value::HashSet(_) => "hashset",
        Value::Queue(_) => "queue",
        Value::Stack(_) => "stack",
        Value::Regex(_) => "regex",
        Value::Future(_) => "future",
        Value::DateTime(_) => "datetime",
        Value::HttpRequest(_) => "HttpRequest",
        Value::HttpResponse(_) => "HttpResponse",
        Value::TaskHandle(_) => "TaskHandle",
        Value::ChannelSender(_) => "ChannelSender",
        Value::ChannelReceiver(_) => "ChannelReceiver",
        Value::AsyncMutex(_) => "AsyncMutex",
    };

    Ok(Value::string(type_name))
}

/// Check if value is a string
pub fn is_string(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// Check if value is a number (including NaN)
pub fn is_number(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(args[0], Value::Number(_))))
}

/// Check if value is a boolean
pub fn is_bool(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

/// Check if value is null
pub fn is_null(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(args[0], Value::Null)))
}

/// Check if value is an array
pub fn is_array(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(args[0], Value::Array(_))))
}

/// Check if value is a function
pub fn is_function(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(matches!(
        args[0],
        Value::Function(_) | Value::Builtin(_)
    )))
}

/// Check if value is a JSON object
pub fn is_object(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(
        matches!(&args[0], Value::JsonValue(json) if json.is_object()),
    ))
}

/// Check if value matches a specific runtime type name
pub fn is_type(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let expected_type = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "isType() requires type name string as second argument".to_string(),
                span,
            })
        }
    };

    if expected_type == "object" {
        return Ok(Value::Bool(matches!(
            &args[0],
            Value::JsonValue(json) if json.is_object()
        )));
    }

    let actual = type_name(&args[0]);
    Ok(Value::Bool(actual == expected_type))
}

/// Check if value has a field/key with the given name
pub fn has_field(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let field = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hasField() requires field name string as second argument".to_string(),
                span,
            })
        }
    };

    match &args[0] {
        Value::JsonValue(json) => Ok(Value::Bool(
            json.as_object()
                .map(|obj| obj.contains_key(field))
                .unwrap_or(false),
        )),
        Value::HashMap(map) => {
            let key = HashKey::from_value(&Value::string(field), span)?;
            let exists = map.lock().unwrap().contains_key(&key);
            Ok(Value::Bool(exists))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// Check if value has a method with the given name
pub fn has_method(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let field = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hasMethod() requires method name string as second argument".to_string(),
                span,
            })
        }
    };

    match &args[0] {
        Value::JsonValue(json) => Ok(Value::Bool(
            json.as_object()
                .map(|obj| obj.contains_key(field))
                .unwrap_or(false),
        )),
        Value::HashMap(map) => {
            let key = HashKey::from_value(&Value::string(field), span)?;
            let exists = map.lock().unwrap().contains_key(&key);
            Ok(Value::Bool(exists))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// Check if value has a tag field matching the given tag value
pub fn has_tag(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let tag_value = match &args[1] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "hasTag() requires tag value string as second argument".to_string(),
                span,
            })
        }
    };

    match &args[0] {
        Value::JsonValue(json) => {
            if let Some(obj) = json.as_object() {
                if let Some(JsonValue::String(value)) = obj.get("tag") {
                    return Ok(Value::Bool(value == tag_value));
                }
            }
            Ok(Value::Bool(false))
        }
        Value::HashMap(map) => {
            let key = HashKey::from_value(&Value::string("tag"), span)?;
            if let Some(Value::String(value)) = map.lock().unwrap().get(&key) {
                return Ok(Value::Bool(value.as_ref() == tag_value));
            }
            Ok(Value::Bool(false))
        }
        _ => Ok(Value::Bool(false)),
    }
}

// ============================================================================
// Type Conversion Functions
// ============================================================================

/// Convert any value to string representation
///
/// Conversion rules:
/// - null → "null"
/// - bool → "true" or "false"
/// - number → string representation (e.g., "42", "3.14")
/// - string → same string (identity)
/// - array → "[Array]"
/// - function → "[Function]"
/// - json → JSON string representation
/// - option → "Some(value)" or "None"
/// - result → "Ok(value)" or "Err(error)"
pub fn to_string(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let string_value = match &args[0] {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // Format number (remove unnecessary decimals)
            if n.is_nan() {
                "NaN".to_string()
            } else if n.is_infinite() {
                if *n > 0.0 {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.as_ref().clone(),
        Value::Array(_) => "[Array]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::Builtin(name) => format!("[Builtin {}]", name),
        Value::NativeFunction(_) => "[Native Function]".to_string(),
        Value::JsonValue(_) => "[JSON]".to_string(),
        Value::Option(Some(v)) => format!("Some({})", value_to_display_string(v)),
        Value::Option(None) => "None".to_string(),
        Value::Result(Ok(v)) => format!("Ok({})", value_to_display_string(v)),
        Value::Result(Err(e)) => format!("Err({})", value_to_display_string(e)),
        Value::HashMap(_) => "[HashMap]".to_string(),
        Value::HashSet(_) => "[HashSet]".to_string(),
        Value::Queue(_) => "[Queue]".to_string(),
        Value::Stack(_) => "[Stack]".to_string(),
        Value::Regex(r) => format!("[Regex /{}/ ]", r.as_str()),
        Value::DateTime(dt) => dt.to_rfc3339(),
        Value::HttpRequest(req) => format!("<HttpRequest {} {}>", req.method(), req.url()),
        Value::HttpResponse(res) => format!("<HttpResponse {}>", res.status()),
        Value::Future(f) => f.to_string(),
        Value::TaskHandle(h) => format!("[TaskHandle #{}]", h.lock().unwrap().id()),
        Value::ChannelSender(_) => "[ChannelSender]".to_string(),
        Value::ChannelReceiver(_) => "[ChannelReceiver]".to_string(),
        Value::AsyncMutex(_) => "[AsyncMutex]".to_string(),
    };

    Ok(Value::string(string_value))
}

/// Convert value to number
///
/// Conversion rules:
/// - number → same number (identity)
/// - bool → true=1.0, false=0.0
/// - string → parsed as number (error if invalid)
/// - null → error
/// - other types → error
pub fn to_number(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Err(RuntimeError::TypeError {
                    msg: "Cannot convert empty string to number".to_string(),
                    span,
                });
            }
            trimmed
                .parse::<f64>()
                .map(Value::Number)
                .map_err(|_| RuntimeError::TypeError {
                    msg: format!("Cannot convert '{}' to number", s),
                    span,
                })
        }
        _ => Err(RuntimeError::TypeError {
            msg: format!("Cannot convert {} to number", type_name(&args[0])),
            span,
        }),
    }
}

/// Convert value to boolean
///
/// Conversion rules (JavaScript-like):
/// - bool → same bool (identity)
/// - number → false if 0, NaN; true otherwise
/// - string → false if empty; true otherwise
/// - null → false
/// - array, function, json, option, result → true
pub fn to_bool(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let bool_value = match &args[0] {
        Value::Bool(b) => *b,
        Value::Number(n) => !(*n == 0.0 || n.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Null => false,
        Value::Array(_)
        | Value::Function(_)
        | Value::Builtin(_)
        | Value::NativeFunction(_)
        | Value::JsonValue(_)
        | Value::Option(_)
        | Value::Result(_)
        | Value::HashMap(_)
        | Value::HashSet(_)
        | Value::Queue(_)
        | Value::Stack(_)
        | Value::Regex(_)
        | Value::DateTime(_)
        | Value::HttpRequest(_)
        | Value::HttpResponse(_)
        | Value::Future(_)
        | Value::TaskHandle(_)
        | Value::ChannelSender(_)
        | Value::ChannelReceiver(_)
        | Value::AsyncMutex(_) => true,
    };

    Ok(Value::Bool(bool_value))
}

/// Parse string as integer with specified radix
///
/// Radix must be between 2 and 36 (inclusive).
/// String is trimmed and can have optional +/- prefix.
/// Returns error if string is invalid for the given radix.
pub fn parse_int(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let string = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "parseInt() requires string as first argument".to_string(),
                span,
            })
        }
    };

    let radix = match &args[1] {
        Value::Number(n) => {
            if n.fract() != 0.0 || *n < 2.0 || *n > 36.0 {
                return Err(RuntimeError::TypeError {
                    msg: "parseInt() radix must be integer between 2 and 36".to_string(),
                    span,
                });
            }
            *n as u32
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "parseInt() requires number as second argument".to_string(),
                span,
            })
        }
    };

    let trimmed = string.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Cannot parse empty string as integer".to_string(),
            span,
        });
    }

    // Handle sign
    let (sign, digits) = if let Some(stripped) = trimmed.strip_prefix('-') {
        (-1.0, stripped)
    } else if let Some(stripped) = trimmed.strip_prefix('+') {
        (1.0, stripped)
    } else {
        (1.0, trimmed)
    };

    if digits.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: format!("Invalid integer for radix {}", radix),
            span,
        });
    }

    // Parse with radix
    let parsed = i64::from_str_radix(digits, radix).map_err(|_| RuntimeError::TypeError {
        msg: format!("Invalid integer '{}' for radix {}", string, radix),
        span,
    })?;

    Ok(Value::Number(sign * parsed as f64))
}

/// Parse string as floating-point number
///
/// Supports decimal notation and scientific notation (e.g., "1.5e-3").
/// String is trimmed before parsing.
/// Returns error if string is not a valid number.
pub fn parse_float(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let string = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "parseFloat() requires string argument".to_string(),
                span,
            })
        }
    };

    let trimmed = string.trim();
    if trimmed.is_empty() {
        return Err(RuntimeError::TypeError {
            msg: "Cannot parse empty string as float".to_string(),
            span,
        });
    }

    trimmed
        .parse::<f64>()
        .map(Value::Number)
        .map_err(|_| RuntimeError::TypeError {
            msg: format!("Cannot parse '{}' as float", string),
            span,
        })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get type name for error messages
fn type_name(value: &Value) -> &str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Function(_) => "function",
        Value::Builtin(_) => "builtin",
        Value::NativeFunction(_) => "function",
        Value::JsonValue(_) => "json",
        Value::Option(_) => "option",
        Value::Result(_) => "result",
        Value::HashMap(_) => "hashmap",
        Value::HashSet(_) => "hashset",
        Value::Queue(_) => "queue",
        Value::Stack(_) => "stack",
        Value::Regex(_) => "regex",
        Value::DateTime(_) => "datetime",
        Value::HttpRequest(_) => "HttpRequest",
        Value::HttpResponse(_) => "HttpResponse",
        Value::Future(_) => "future",
        Value::TaskHandle(_) => "TaskHandle",
        Value::ChannelSender(_) => "ChannelSender",
        Value::ChannelReceiver(_) => "ChannelReceiver",
        Value::AsyncMutex(_) => "AsyncMutex",
    }
}

/// Convert value to display string (for debugging)
fn value_to_display_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{:.0}", n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s),
        Value::Array(_) => "[Array]".to_string(),
        Value::Function(_) => "[Function]".to_string(),
        Value::Builtin(name) => format!("[Builtin {}]", name),
        Value::NativeFunction(_) => "[Native Function]".to_string(),
        Value::JsonValue(_) => "[JSON]".to_string(),
        Value::Option(_) => "[Option]".to_string(),
        Value::Result(_) => "[Result]".to_string(),
        Value::HashMap(_) => "[HashMap]".to_string(),
        Value::HashSet(_) => "[HashSet]".to_string(),
        Value::Queue(_) => "[Queue]".to_string(),
        Value::Stack(_) => "[Stack]".to_string(),
        Value::Regex(r) => format!("[Regex /{}/ ]", r.as_str()),
        Value::DateTime(dt) => format!("[DateTime {}]", dt.to_rfc3339()),
        Value::HttpRequest(req) => format!("[HttpRequest {} {}]", req.method(), req.url()),
        Value::HttpResponse(res) => format!("[HttpResponse {}]", res.status()),
        Value::Future(f) => format!("[{}]", f.as_ref()),
        Value::TaskHandle(h) => format!("[TaskHandle #{}]", h.lock().unwrap().id()),
        Value::ChannelSender(_) => "[ChannelSender]".to_string(),
        Value::ChannelReceiver(_) => "[ChannelReceiver]".to_string(),
        Value::AsyncMutex(_) => "[AsyncMutex]".to_string(),
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

        assert!(is_some(&some_val, Span::dummy()).unwrap());
        assert!(!is_none(&some_val, Span::dummy()).unwrap());

        assert!(!is_some(&none_val, Span::dummy()).unwrap());
        assert!(is_none(&none_val, Span::dummy()).unwrap());
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

        assert!(is_ok(&ok_val, Span::dummy()).unwrap());
        assert!(!is_err(&ok_val, Span::dummy()).unwrap());

        assert!(!is_ok(&err_val, Span::dummy()).unwrap());
        assert!(is_err(&err_val, Span::dummy()).unwrap());
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
