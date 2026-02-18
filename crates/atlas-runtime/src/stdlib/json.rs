//! JSON parsing and serialization functions

use crate::json_value::JsonValue;
use crate::span::Span;
use crate::value::{RuntimeError, Value};
use std::collections::HashSet;
use std::sync::Arc;

// ============================================================================
// JSON Functions
// ============================================================================

/// Parse JSON string into JsonValue
///
/// Converts JSON text to Atlas JsonValue type with proper type mapping:
/// - JSON null → JsonValue::Null
/// - JSON boolean → JsonValue::Bool
/// - JSON number → JsonValue::Number
/// - JSON string → JsonValue::String
/// - JSON array → JsonValue::Array
/// - JSON object → JsonValue::Object
///
/// Returns error if JSON is malformed.
pub fn parse_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let json_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "parseJSON() requires string argument".to_string(),
                span,
            })
        }
    };

    // Parse using serde_json
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| RuntimeError::TypeError {
            msg: format!("Invalid JSON: {}", e),
            span,
        })?;

    // Convert serde_json::Value to JsonValue
    let json_value = serde_to_atlas_json(parsed);
    Ok(Value::JsonValue(Arc::new(json_value)))
}

/// Convert JsonValue to JSON string
///
/// Serializes Atlas JsonValue to JSON text. Detects circular references
/// using pointer tracking. Functions cannot be serialized and return error.
///
/// Returns compact JSON string (no whitespace).
pub fn to_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    // Track visited pointers to detect circular references
    let mut visited = HashSet::new();
    let json_str = value_to_json(&args[0], &mut visited, span)?;
    Ok(Value::string(json_str))
}

/// Check if string is valid JSON without parsing
///
/// Returns true if the string is valid JSON, false otherwise.
/// More efficient than parseJSON when you only need validation.
pub fn is_valid_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let json_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "isValidJSON() requires string argument".to_string(),
                span,
            })
        }
    };

    let is_valid = serde_json::from_str::<serde_json::Value>(json_str).is_ok();
    Ok(Value::Bool(is_valid))
}

/// Format JSON string with indentation
///
/// Takes JSON string and indentation size (number of spaces).
/// Returns prettified JSON with proper formatting.
pub fn prettify_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let json_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "prettifyJSON() requires string as first argument".to_string(),
                span,
            })
        }
    };

    let indent_size = match &args[1] {
        Value::Number(n) => {
            if *n < 0.0 || n.fract() != 0.0 {
                return Err(RuntimeError::TypeError {
                    msg: "prettifyJSON() indent must be non-negative integer".to_string(),
                    span,
                });
            }
            *n as usize
        }
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "prettifyJSON() requires number as second argument".to_string(),
                span,
            })
        }
    };

    // Parse JSON
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| RuntimeError::TypeError {
            msg: format!("Invalid JSON: {}", e),
            span,
        })?;

    // Format with custom indentation
    let indent = " ".repeat(indent_size);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
    let mut buf = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

    serde::Serialize::serialize(&parsed, &mut serializer).map_err(|e| RuntimeError::TypeError {
        msg: format!("JSON serialization failed: {}", e),
        span,
    })?;

    let pretty_json = String::from_utf8(buf).map_err(|e| RuntimeError::TypeError {
        msg: format!("UTF-8 conversion failed: {}", e),
        span,
    })?;

    Ok(Value::string(pretty_json))
}

/// Minify JSON string (remove all whitespace)
///
/// Takes JSON string and returns compact version with no extra whitespace.
pub fn minify_json(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let json_str = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(RuntimeError::TypeError {
                msg: "minifyJSON() requires string argument".to_string(),
                span,
            })
        }
    };

    // Parse and re-serialize without formatting
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| RuntimeError::TypeError {
            msg: format!("Invalid JSON: {}", e),
            span,
        })?;

    let minified = serde_json::to_string(&parsed).map_err(|e| RuntimeError::TypeError {
        msg: format!("JSON serialization failed: {}", e),
        span,
    })?;

    Ok(Value::string(minified))
}

// ============================================================================
// JSON Extraction Functions (Phase 17)
// ============================================================================
// These functions extract typed values from JsonValue.
// Called via method syntax (desugared at runtime):
//   json.as_string()  → jsonAsString(json)
//   json.as_number()  → jsonAsNumber(json)
//   json.as_bool()    → jsonAsBool(json)
//   json.is_null()    → jsonIsNull(json)

/// Extract string from JsonValue
///
/// Returns error if JsonValue is not a string.
pub fn json_as_string(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::JsonValue(json) => match json.as_ref() {
            JsonValue::String(s) => Ok(Value::string(s.clone())),
            _ => Err(RuntimeError::TypeError {
                msg: format!(
                    "Cannot extract string from JSON value of type '{}'",
                    json_type_name(json.as_ref())
                ),
                span,
            }),
        },
        _ => Err(RuntimeError::TypeError {
            msg: "as_string() requires json argument".to_string(),
            span,
        }),
    }
}

/// Extract number from JsonValue
///
/// Returns error if JsonValue is not a number.
pub fn json_as_number(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::JsonValue(json) => match json.as_ref() {
            JsonValue::Number(n) => Ok(Value::Number(*n)),
            _ => Err(RuntimeError::TypeError {
                msg: format!(
                    "Cannot extract number from JSON value of type '{}'",
                    json_type_name(json.as_ref())
                ),
                span,
            }),
        },
        _ => Err(RuntimeError::TypeError {
            msg: "as_number() requires json argument".to_string(),
            span,
        }),
    }
}

/// Extract boolean from JsonValue
///
/// Returns error if JsonValue is not a boolean.
pub fn json_as_bool(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::JsonValue(json) => match json.as_ref() {
            JsonValue::Bool(b) => Ok(Value::Bool(*b)),
            _ => Err(RuntimeError::TypeError {
                msg: format!(
                    "Cannot extract bool from JSON value of type '{}'",
                    json_type_name(json.as_ref())
                ),
                span,
            }),
        },
        _ => Err(RuntimeError::TypeError {
            msg: "as_bool() requires json argument".to_string(),
            span,
        }),
    }
}

/// Check if JsonValue is null
///
/// Returns true if JsonValue is null, false otherwise.
pub fn json_is_null(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::JsonValue(json) => Ok(Value::Bool(matches!(json.as_ref(), JsonValue::Null))),
        _ => Err(RuntimeError::TypeError {
            msg: "is_null() requires json argument".to_string(),
            span,
        }),
    }
}

/// Helper: Get type name of JsonValue for error messages
fn json_type_name(json: &JsonValue) -> &'static str {
    match json {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert serde_json::Value to Atlas JsonValue
fn serde_to_atlas_json(value: serde_json::Value) -> JsonValue {
    match value {
        serde_json::Value::Null => JsonValue::Null,
        serde_json::Value::Bool(b) => JsonValue::Bool(b),
        serde_json::Value::Number(n) => {
            // Convert to f64 (JSON numbers are always floats in our system)
            JsonValue::Number(n.as_f64().unwrap_or(0.0))
        }
        serde_json::Value::String(s) => JsonValue::String(s),
        serde_json::Value::Array(arr) => {
            JsonValue::Array(arr.into_iter().map(serde_to_atlas_json).collect())
        }
        serde_json::Value::Object(obj) => JsonValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, serde_to_atlas_json(v)))
                .collect(),
        ),
    }
}

/// Convert Atlas Value to JSON string with circular reference detection
fn value_to_json(
    value: &Value,
    visited: &mut HashSet<usize>,
    span: Span,
) -> Result<String, RuntimeError> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => {
            // Handle special float values
            if n.is_nan() || n.is_infinite() {
                return Err(RuntimeError::TypeError {
                    msg: "Cannot serialize NaN or Infinity to JSON".to_string(),
                    span,
                });
            }
            // Format number (remove unnecessary decimals)
            if n.fract() == 0.0 && n.abs() < 1e15 {
                Ok(format!("{:.0}", n))
            } else {
                Ok(n.to_string())
            }
        }
        Value::String(s) => {
            // Use serde_json to properly escape the string
            Ok(serde_json::to_string(s.as_ref()).unwrap())
        }
        Value::Array(arr_ref) => {
            // Check for circular reference using pointer address
            let ptr = Arc::as_ptr(arr_ref) as usize;
            if !visited.insert(ptr) {
                return Err(RuntimeError::TypeError {
                    msg: "Circular reference detected in array".to_string(),
                    span,
                });
            }

            let arr = arr_ref.lock().unwrap();
            let elements: Result<Vec<String>, RuntimeError> = arr
                .iter()
                .map(|v| value_to_json(v, visited, span))
                .collect();

            visited.remove(&ptr);

            let elements = elements?;
            Ok(format!("[{}]", elements.join(",")))
        }
        Value::JsonValue(json) => {
            // Serialize JsonValue directly
            json_value_to_string(json, span)
        }
        Value::Function(_) | Value::Builtin(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize function to JSON".to_string(),
            span,
        }),
        Value::NativeFunction(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize native function to JSON".to_string(),
            span,
        }),
        Value::Option(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Option to JSON".to_string(),
            span,
        }),
        Value::Result(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Result to JSON".to_string(),
            span,
        }),
        Value::HashMap(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize HashMap to JSON".to_string(),
            span,
        }),
        Value::HashSet(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize HashSet to JSON".to_string(),
            span,
        }),
        Value::Queue(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Queue to JSON".to_string(),
            span,
        }),
        Value::Stack(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Stack to JSON".to_string(),
            span,
        }),
        Value::Regex(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Regex to JSON".to_string(),
            span,
        }),
        Value::DateTime(dt) => {
            // Serialize DateTime as ISO 8601 string
            Ok(serde_json::to_string(&dt.to_rfc3339()).unwrap())
        }
        Value::HttpRequest(_) | Value::HttpResponse(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize HttpRequest/HttpResponse to JSON".to_string(),
            span,
        }),
        Value::Future(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize Future to JSON".to_string(),
            span,
        }),
        Value::TaskHandle(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize TaskHandle to JSON".to_string(),
            span,
        }),
        Value::ChannelSender(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize ChannelSender to JSON".to_string(),
            span,
        }),
        Value::ChannelReceiver(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize ChannelReceiver to JSON".to_string(),
            span,
        }),
        Value::AsyncMutex(_) => Err(RuntimeError::TypeError {
            msg: "Cannot serialize AsyncMutex to JSON".to_string(),
            span,
        }),
    }
}

/// Convert JsonValue to JSON string
fn json_value_to_string(json: &JsonValue, span: Span) -> Result<String, RuntimeError> {
    match json {
        JsonValue::Null => Ok("null".to_string()),
        JsonValue::Bool(b) => Ok(b.to_string()),
        JsonValue::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                return Err(RuntimeError::TypeError {
                    msg: "Cannot serialize NaN or Infinity to JSON".to_string(),
                    span,
                });
            }
            if n.fract() == 0.0 && n.abs() < 1e15 {
                Ok(format!("{:.0}", n))
            } else {
                Ok(n.to_string())
            }
        }
        JsonValue::String(s) => Ok(serde_json::to_string(s).unwrap()),
        JsonValue::Array(arr) => {
            let elements: Result<Vec<String>, RuntimeError> =
                arr.iter().map(|v| json_value_to_string(v, span)).collect();
            let elements = elements?;
            Ok(format!("[{}]", elements.join(",")))
        }
        JsonValue::Object(obj) => {
            let mut pairs = Vec::new();
            for (key, value) in obj.iter() {
                let key_json = serde_json::to_string(key).unwrap();
                let value_json = json_value_to_string(value, span)?;
                pairs.push(format!("{}:{}", key_json, value_json));
            }
            Ok(format!("{{{}}}", pairs.join(",")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_primitives() {
        let span = Span::dummy();

        // Null
        let result = parse_json(&[Value::string("null")], span).unwrap();
        assert!(matches!(result, Value::JsonValue(_)));

        // Boolean
        let result = parse_json(&[Value::string("true")], span).unwrap();
        if let Value::JsonValue(json) = result {
            assert!(matches!(&*json, JsonValue::Bool(true)));
        }

        // Number
        let result = parse_json(&[Value::string("42")], span).unwrap();
        if let Value::JsonValue(json) = result {
            assert!(matches!(&*json, JsonValue::Number(n) if (*n - 42.0).abs() < 1e-10));
        }

        // String
        let result = parse_json(&[Value::string(r#""hello""#)], span).unwrap();
        if let Value::JsonValue(json) = result {
            assert!(matches!(&*json, JsonValue::String(s) if s == "hello"));
        }
    }

    #[test]
    fn test_parse_json_array() {
        let span = Span::dummy();
        let result = parse_json(&[Value::string(r#"[1,2,3]"#)], span).unwrap();

        if let Value::JsonValue(json) = result {
            if let JsonValue::Array(arr) = &*json {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected array");
            }
        }
    }

    #[test]
    fn test_parse_json_object() {
        let span = Span::dummy();
        let result = parse_json(&[Value::string(r#"{"name":"Alice","age":30}"#)], span).unwrap();

        if let Value::JsonValue(json) = result {
            if let JsonValue::Object(obj) = &*json {
                assert_eq!(obj.len(), 2);
                assert!(obj.contains_key("name"));
                assert!(obj.contains_key("age"));
            } else {
                panic!("Expected object");
            }
        }
    }

    #[test]
    fn test_parse_json_invalid() {
        let span = Span::dummy();
        let result = parse_json(&[Value::string("{invalid}")], span);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json_primitives() {
        let span = Span::dummy();

        assert_eq!(
            to_json(&[Value::Null], span).unwrap(),
            Value::string("null")
        );
        assert_eq!(
            to_json(&[Value::Bool(true)], span).unwrap(),
            Value::string("true")
        );
        assert_eq!(
            to_json(&[Value::Number(42.0)], span).unwrap(),
            Value::string("42")
        );
        assert_eq!(
            to_json(&[Value::string("hello")], span).unwrap(),
            Value::string(r#""hello""#)
        );
    }

    #[test]
    fn test_to_json_array() {
        let span = Span::dummy();
        let arr = Value::array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = to_json(&[arr], span).unwrap();
        assert_eq!(result, Value::string("[1,2,3]"));
    }

    #[test]
    fn test_to_json_function_error() {
        let span = Span::dummy();
        let func = Value::Function(crate::value::FunctionRef {
            name: "print".to_string(),
            arity: 1,
            bytecode_offset: 0,
            local_count: 1,
        });
        let result = to_json(&[func], span);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_json() {
        let span = Span::dummy();

        assert_eq!(
            is_valid_json(&[Value::string(r#"{"valid":true}"#)], span).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            is_valid_json(&[Value::string("{invalid}")], span).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_prettify_json() {
        let span = Span::dummy();
        let compact = r#"{"name":"Alice","age":30}"#;
        let result = prettify_json(&[Value::string(compact), Value::Number(2.0)], span).unwrap();

        if let Value::String(s) = result {
            assert!(s.contains("  ")); // Should have 2-space indentation
            assert!(s.contains('\n')); // Should have newlines
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_minify_json() {
        let span = Span::dummy();
        let pretty = r#"{
  "name": "Alice",
  "age": 30
}"#;
        let result = minify_json(&[Value::string(pretty)], span).unwrap();
        assert_eq!(result, Value::string(r#"{"age":30,"name":"Alice"}"#));
    }
}
