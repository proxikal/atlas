//! Standard library functions

use crate::value::{RuntimeError, Value};

/// Print a value to stdout
pub fn print(value: &Value) {
    println!("{}", value.to_display_string());
}

/// Get the length of a string or array
pub fn len(value: &Value) -> Result<f64, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.len() as f64),
        Value::Array(arr) => Ok(arr.borrow().len() as f64),
        _ => Err(RuntimeError::TypeError(
            "len() requires string or array".to_string(),
        )),
    }
}

/// Convert a value to a string
pub fn str(value: &Value) -> String {
    value.to_display_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_string() {
        let val = Value::string("hello");
        assert_eq!(len(&val).unwrap() as i64, 5);
    }

    #[test]
    fn test_len_array() {
        let val = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(len(&val).unwrap() as i64, 2);
    }

    #[test]
    fn test_str() {
        let val = Value::Number(42.0);
        assert_eq!(str(&val), "42");
    }
}
