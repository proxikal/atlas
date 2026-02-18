//! Bytecode serialization and deserialization

use crate::span::Span;
use crate::value::Value;

/// Serialize a Value to bytes
pub(super) fn serialize_value(value: &Value, bytes: &mut Vec<u8>) {
    match value {
        Value::Null => {
            bytes.push(0x00); // Type tag
        }
        Value::Bool(b) => {
            bytes.push(0x01); // Type tag
            bytes.push(if *b { 1 } else { 0 });
        }
        Value::Number(n) => {
            bytes.push(0x02); // Type tag
            bytes.extend_from_slice(&n.to_be_bytes());
        }
        Value::String(s) => {
            bytes.push(0x03); // Type tag
            let s_bytes = s.as_bytes();
            bytes.extend_from_slice(&(s_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(s_bytes);
        }
        Value::Function(func) => {
            bytes.push(0x04); // Type tag
                              // Serialize function name
            let name_bytes = func.name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(name_bytes);
            // Serialize arity
            bytes.push(func.arity as u8);
            // Serialize bytecode offset
            bytes.extend_from_slice(&(func.bytecode_offset as u32).to_be_bytes());
        }
        Value::Builtin(name) => {
            bytes.push(0x05); // Type tag for Builtin
            let name_bytes = name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(name_bytes);
        }
        Value::NativeFunction(_) => {
            // Native functions cannot be serialized in constant pool
            // They are runtime-only closures
            panic!("Cannot serialize native functions in bytecode constants");
        }
        Value::Array(_) => {
            // Arrays cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize array values in bytecode constants");
        }
        Value::JsonValue(_) => {
            // JSON values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize JSON values in bytecode constants");
        }
        Value::Option(_) => {
            // Option values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize Option values in bytecode constants");
        }
        Value::Result(_) => {
            // Result values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize Result values in bytecode constants");
        }
        Value::HashMap(_) => {
            // HashMap values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize HashMap values in bytecode constants");
        }
        Value::HashSet(_) => {
            // HashSet values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize HashSet values in bytecode constants");
        }
        Value::Queue(_) => {
            // Queue values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize Queue values in bytecode constants");
        }
        Value::Stack(_) => {
            // Stack values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize Stack values in bytecode constants");
        }
        Value::Regex(_) => {
            // Regex values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize Regex values in bytecode constants");
        }
        Value::DateTime(_) => {
            // DateTime values cannot be serialized in constant pool
            // They are runtime-only values
            panic!("Cannot serialize DateTime values in bytecode constants");
        }
        Value::HttpRequest(_) => {
            panic!("Cannot serialize HttpRequest values in bytecode constants");
        }
        Value::HttpResponse(_) => {
            panic!("Cannot serialize HttpResponse values in bytecode constants");
        }
        Value::Future(_) => {
            // Future values cannot be serialized in bytecode constants
            // They are runtime-only values
            panic!("Cannot serialize Future values in bytecode constants");
        }
        Value::TaskHandle(_) => {
            panic!("Cannot serialize TaskHandle values in bytecode constants");
        }
        Value::ChannelSender(_) => {
            panic!("Cannot serialize ChannelSender values in bytecode constants");
        }
        Value::ChannelReceiver(_) => {
            panic!("Cannot serialize ChannelReceiver values in bytecode constants");
        }
        Value::AsyncMutex(_) => {
            panic!("Cannot serialize AsyncMutex values in bytecode constants");
        }
    }
}

/// Deserialize a Value from bytes, returns (Value, bytes_consumed)
pub(super) fn deserialize_value(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
        return Err("Unexpected end of data while reading value".to_string());
    }

    let tag = bytes[0];
    match tag {
        0x00 => Ok((Value::Null, 1)),
        0x01 => {
            if bytes.len() < 2 {
                return Err("Truncated bool value".to_string());
            }
            Ok((Value::Bool(bytes[1] != 0), 2))
        }
        0x02 => {
            if bytes.len() < 9 {
                return Err("Truncated number value".to_string());
            }
            let num_bytes: [u8; 8] = bytes[1..9].try_into().unwrap();
            Ok((Value::Number(f64::from_be_bytes(num_bytes)), 9))
        }
        0x03 => {
            if bytes.len() < 5 {
                return Err("Truncated string value".to_string());
            }
            let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
            if bytes.len() < 5 + len {
                return Err("Truncated string data".to_string());
            }
            let s = String::from_utf8(bytes[5..5 + len].to_vec())
                .map_err(|e| format!("Invalid UTF-8 in string: {}", e))?;
            Ok((Value::string(&s), 5 + len))
        }
        0x04 => {
            if bytes.len() < 5 {
                return Err("Truncated function value".to_string());
            }
            let name_len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
            if bytes.len() < 5 + name_len + 1 + 4 {
                return Err("Truncated function data".to_string());
            }
            let name = String::from_utf8(bytes[5..5 + name_len].to_vec())
                .map_err(|e| format!("Invalid UTF-8 in function name: {}", e))?;
            let arity = bytes[5 + name_len] as usize;
            let offset = u32::from_be_bytes([
                bytes[6 + name_len],
                bytes[7 + name_len],
                bytes[8 + name_len],
                bytes[9 + name_len],
            ]) as usize;
            Ok((
                Value::Function(crate::value::FunctionRef {
                    name,
                    arity,
                    bytecode_offset: offset,
                    local_count: 0, // Deserialized from old format, will be set correctly on recompile
                }),
                10 + name_len,
            ))
        }
        0x05 => {
            if bytes.len() < 5 {
                return Err("Truncated builtin value".to_string());
            }
            let name_len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
            if bytes.len() < 5 + name_len {
                return Err("Truncated builtin name".to_string());
            }
            let name = String::from_utf8(bytes[5..5 + name_len].to_vec())
                .map_err(|e| format!("Invalid UTF-8 in builtin name: {}", e))?;
            Ok((
                Value::Builtin(std::sync::Arc::from(name.as_str())),
                5 + name_len,
            ))
        }
        _ => Err(format!("Unknown value type tag: {:#x}", tag)),
    }
}

/// Serialize a Span to bytes
pub(super) fn serialize_span(span: &Span, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(span.start as u32).to_be_bytes());
    bytes.extend_from_slice(&(span.end as u32).to_be_bytes());
}

/// Deserialize a Span from bytes, returns (Span, bytes_consumed)
pub(super) fn deserialize_span(bytes: &[u8]) -> Result<(Span, usize), String> {
    if bytes.len() < 8 {
        return Err("Truncated span data".to_string());
    }
    let start = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let end = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    Ok((Span { start, end }, 8))
}
