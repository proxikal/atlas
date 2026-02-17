//! Reflection and introspection stdlib functions
//!
//! Provides runtime type inspection and value introspection capabilities.

use crate::reflect::{
    get_type_name, get_value_type_info, is_callable, is_primitive_value, same_type,
};
use crate::span::Span;
use crate::value::{RuntimeError, Value};

/// Get the type name of a value as a string
///
/// # Atlas Usage
/// ```atlas
/// let x = 42;
/// print(typeof(x));  // "number"
/// ```
pub fn typeof_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let type_name = get_type_name(&args[0]);
    Ok(Value::string(type_name))
}

/// Check if a value is callable (function or native function)
///
/// # Atlas Usage
/// ```atlas
/// fn foo() { }
/// print(is_callable(foo));  // true
/// print(is_callable(42));   // false
/// ```
pub fn is_callable_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(is_callable(&args[0])))
}

/// Check if a value is a primitive type (number, string, bool, or null)
///
/// # Atlas Usage
/// ```atlas
/// print(is_primitive(42));        // true
/// print(is_primitive("test"));    // true
/// print(is_primitive([1, 2, 3])); // false
/// ```
pub fn is_primitive_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(is_primitive_value(&args[0])))
}

/// Check if two values have the same type
///
/// # Atlas Usage
/// ```atlas
/// print(same_type(42, 99));       // true
/// print(same_type(42, "hello"));  // false
/// ```
pub fn same_type_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::Bool(same_type(&args[0], &args[1])))
}

/// Get the length of a collection (arrays or strings)
///
/// This is an alias for the built-in len() but in the reflect namespace.
///
/// # Atlas Usage
/// ```atlas
/// print(get_length([1, 2, 3]));  // 3
/// print(get_length("hello"));    // 5
/// ```
pub fn get_length_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Array(arr) => Ok(Value::Number(arr.lock().unwrap().len() as f64)),
        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
        _ => Err(RuntimeError::TypeError {
            msg: "get_length() requires array or string".to_string(),
            span,
        }),
    }
}

/// Check if a collection is empty
///
/// # Atlas Usage
/// ```atlas
/// print(is_empty([]));        // true
/// print(is_empty(""));        // true
/// print(is_empty([1, 2]));    // false
/// ```
pub fn is_empty_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Array(arr) => Ok(Value::Bool(arr.lock().unwrap().is_empty())),
        Value::String(s) => Ok(Value::Bool(s.is_empty())),
        _ => Err(RuntimeError::TypeError {
            msg: "is_empty() requires array or string".to_string(),
            span,
        }),
    }
}

/// Get a detailed type description
///
/// Returns a human-readable description of the type.
///
/// # Atlas Usage
/// ```atlas
/// print(type_describe(42));           // "primitive number type"
/// print(type_describe([1, 2, 3]));    // "array type"
/// ```
pub fn type_describe_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let type_info = get_value_type_info(&args[0]);
    Ok(Value::string(type_info.describe()))
}

/// Clone a value (deep copy for arrays, shallow for primitives)
///
/// # Atlas Usage
/// ```atlas
/// let arr = [1, 2, 3];
/// let arr2 = clone(arr);
/// ```
pub fn clone_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    // Value already implements Clone, so this is straightforward
    Ok(args[0].clone())
}

/// Convert any value to its string representation
///
/// Unlike toString(), this works with all types including arrays and functions.
///
/// # Atlas Usage
/// ```atlas
/// print(value_to_string([1, 2, 3]));  // "[1, 2, 3]"
/// ```
pub fn value_to_string_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    Ok(Value::string(args[0].to_string()))
}

/// Check deep equality between two values
///
/// Unlike ==, this performs deep comparison for arrays.
///
/// # Atlas Usage
/// ```atlas
/// let a = [1, 2, 3];
/// let b = [1, 2, 3];
/// print(deep_equals(a, b));  // true (content equality)
/// print(a == b);              // false (reference equality)
/// ```
pub fn deep_equals_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    let result = deep_equals_impl(&args[0], &args[1]);
    Ok(Value::Bool(result))
}

/// Deep equality implementation
fn deep_equals_impl(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,

        (Value::Array(arr_a), Value::Array(arr_b)) => {
            let a_borrowed = arr_a.lock().unwrap();
            let b_borrowed = arr_b.lock().unwrap();

            if a_borrowed.len() != b_borrowed.len() {
                return false;
            }

            a_borrowed
                .iter()
                .zip(b_borrowed.iter())
                .all(|(x, y)| deep_equals_impl(x, y))
        }

        (Value::Function(f_a), Value::Function(f_b)) => f_a.name == f_b.name,
        (Value::JsonValue(j_a), Value::JsonValue(j_b)) => j_a == j_b,
        (Value::Option(o_a), Value::Option(o_b)) => match (o_a, o_b) {
            (Some(x), Some(y)) => deep_equals_impl(x, y),
            (None, None) => true,
            _ => false,
        },
        (Value::Result(r_a), Value::Result(r_b)) => match (r_a, r_b) {
            (Ok(x), Ok(y)) => deep_equals_impl(x, y),
            (Err(x), Err(y)) => deep_equals_impl(x, y),
            _ => false,
        },

        _ => false,
    }
}

/// Get the function name (for function values)
///
/// # Atlas Usage
/// ```atlas
/// fn myFunction() { }
/// print(get_function_name(myFunction));  // "myFunction"
/// ```
pub fn get_function_name_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Function(func) => Ok(Value::string(&func.name)),
        Value::NativeFunction(_) => Ok(Value::string("<native function>")),
        _ => Err(RuntimeError::TypeError {
            msg: "get_function_name() requires function value".to_string(),
            span,
        }),
    }
}

/// Get the function arity (number of parameters)
///
/// # Atlas Usage
/// ```atlas
/// fn add(a, b) { return a + b; }
/// print(get_function_arity(add));  // 2
/// ```
pub fn get_function_arity_fn(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument { span });
    }

    match &args[0] {
        Value::Function(func) => Ok(Value::Number(func.arity as f64)),
        Value::NativeFunction(_) => Err(RuntimeError::TypeError {
            msg: "get_function_arity() not supported for native functions".to_string(),
            span,
        }),
        _ => Err(RuntimeError::TypeError {
            msg: "get_function_arity() requires function value".to_string(),
            span,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typeof_fn() {
        let result = typeof_fn(&[Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("number"));

        let result = typeof_fn(&[Value::string("hello")], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("string"));

        let result = typeof_fn(&[Value::Bool(true)], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("bool"));
    }

    #[test]
    fn test_is_callable_fn() {
        let result = is_callable_fn(&[Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));

        let result = is_callable_fn(&[Value::array(vec![])], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_is_primitive_fn() {
        let result = is_primitive_fn(&[Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = is_primitive_fn(&[Value::string("test")], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = is_primitive_fn(&[Value::array(vec![])], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_same_type_fn() {
        let result =
            same_type_fn(&[Value::Number(1.0), Value::Number(2.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result =
            same_type_fn(&[Value::Number(1.0), Value::string("test")], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_get_length_fn() {
        let result = get_length_fn(
            &[Value::array(vec![Value::Number(1.0), Value::Number(2.0)])],
            Span::dummy(),
        )
        .unwrap();
        assert_eq!(result, Value::Number(2.0));

        let result = get_length_fn(&[Value::string("hello")], Span::dummy()).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_is_empty_fn() {
        let result = is_empty_fn(&[Value::array(vec![])], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = is_empty_fn(&[Value::string("")], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = is_empty_fn(&[Value::array(vec![Value::Number(1.0)])], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_type_describe_fn() {
        let result = type_describe_fn(&[Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("primitive number type"));

        let result = type_describe_fn(&[Value::array(vec![])], Span::dummy()).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_clone_fn() {
        let original = Value::Number(42.0);
        let cloned = clone_fn(&[original.clone()], Span::dummy()).unwrap();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_value_to_string_fn() {
        let result = value_to_string_fn(&[Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("42"));

        let result =
            value_to_string_fn(&[Value::array(vec![Value::Number(1.0)])], Span::dummy()).unwrap();
        assert_eq!(result, Value::string("[1]"));
    }

    #[test]
    fn test_deep_equals_primitives() {
        let result =
            deep_equals_fn(&[Value::Number(42.0), Value::Number(42.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result =
            deep_equals_fn(&[Value::Number(42.0), Value::Number(99.0)], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_deep_equals_arrays() {
        let arr1 = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let arr2 = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);

        let result = deep_equals_fn(&[arr1.clone(), arr2.clone()], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));

        let arr3 = Value::array(vec![Value::Number(1.0), Value::Number(3.0)]);
        let result = deep_equals_fn(&[arr1, arr3], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_deep_equals_nested_arrays() {
        let arr1 = Value::array(vec![
            Value::array(vec![Value::Number(1.0)]),
            Value::Number(2.0),
        ]);
        let arr2 = Value::array(vec![
            Value::array(vec![Value::Number(1.0)]),
            Value::Number(2.0),
        ]);

        let result = deep_equals_fn(&[arr1, arr2], Span::dummy()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}
