//! Runtime value representation
//!
//! Shared value representation for interpreter and VM.
//! - Numbers, Bools, Null: Immediate values (stack-allocated)
//! - Strings: Heap-allocated, reference-counted (Rc<String>), immutable
//! - Arrays: Heap-allocated, reference-counted (Rc<RefCell<Vec<Value>>>), mutable
//! - Functions: Reference to bytecode or builtin
//! - NativeFunction: Rust closures callable from Atlas
//! - JsonValue: Isolated dynamic type for JSON interop (Rc<JsonValue>)

use crate::json_value::JsonValue;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use thiserror::Error;

/// Native function type - Rust closure callable from Atlas
///
/// Native functions receive an array of Atlas values and return either a value or a runtime error.
/// Arc provides thread safety and cheap cloning for sharing natives across execution contexts.
pub type NativeFn = Arc<dyn Fn(&[Value]) -> Result<Value, RuntimeError> + Send + Sync>;

/// Runtime value type
#[derive(Clone)]
pub enum Value {
    /// Numeric value (IEEE 754 double-precision)
    Number(f64),
    /// String value (reference-counted, immutable)
    String(Rc<String>),
    /// Boolean value
    Bool(bool),
    /// Null value
    Null,
    /// Array value (reference-counted, mutable through RefCell)
    Array(Rc<RefCell<Vec<Value>>>),
    /// Function reference (bytecode or builtin)
    Function(FunctionRef),
    /// Native function (Rust closure callable from Atlas)
    NativeFunction(NativeFn),
    /// JSON value (isolated dynamic type for JSON interop)
    JsonValue(Rc<JsonValue>),
    /// Option value (Some(value) or None)
    Option(Option<Box<Value>>),
    /// Result value (Ok(value) or Err(error))
    Result(Result<Box<Value>, Box<Value>>),
    /// HashMap collection (key-value pairs)
    HashMap(Rc<RefCell<crate::stdlib::collections::hashmap::AtlasHashMap>>),
    /// HashSet collection (unique values)
    HashSet(Rc<RefCell<crate::stdlib::collections::hashset::AtlasHashSet>>),
    /// Queue collection (FIFO)
    Queue(Rc<RefCell<crate::stdlib::collections::queue::AtlasQueue>>),
    /// Stack collection (LIFO)
    Stack(Rc<RefCell<crate::stdlib::collections::stack::AtlasStack>>),
    /// Regular expression pattern
    Regex(Rc<regex::Regex>),
    /// DateTime value (UTC timezone)
    DateTime(Rc<chrono::DateTime<chrono::Utc>>),
}

/// Function reference
#[derive(Debug, Clone)]
pub struct FunctionRef {
    /// Function name
    pub name: String,
    /// Number of parameters
    pub arity: usize,
    /// Bytecode offset (for VM) or builtin ID
    pub bytecode_offset: usize,
    /// Total number of local variables (parameters + locals)
    /// Used by VM to properly allocate stack space
    pub local_count: usize,
}

impl Value {
    /// Create a new string value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Rc::new(s.into()))
    }

    /// Create a new array value
    pub fn array(values: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(values)))
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Array(_) => "array",
            Value::Function(_) => "function",
            Value::NativeFunction(_) => "function",
            Value::JsonValue(_) => "json",
            Value::Option(_) => "Option",
            Value::Result(_) => "Result",
            Value::HashMap(_) => "hashmap",
            Value::HashSet(_) => "hashset",
            Value::Queue(_) => "queue",
            Value::Stack(_) => "stack",
            Value::Regex(_) => "regex",
            Value::DateTime(_) => "datetime",
        }
    }

    /// Check if this value is truthy
    /// In Atlas, only `true` is truthy - no implicit conversions
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false, // Only bool values are truthy in Atlas
        }
    }

    /// Get a display string representation (alias for to_string for backwards compatibility)
    pub fn to_display_string(&self) -> String {
        self.to_string()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            // Arrays use reference identity, not deep equality
            (Value::Array(a), Value::Array(b)) => Rc::ptr_eq(a, b),
            // Functions are equal if they have the same name
            (Value::Function(a), Value::Function(b)) => a.name == b.name,
            // Native functions use pointer equality
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Arc::ptr_eq(a, b),
            // JsonValue uses structural equality
            (Value::JsonValue(a), Value::JsonValue(b)) => a == b,
            // Option uses deep equality
            (Value::Option(a), Value::Option(b)) => a == b,
            // Result uses deep equality
            (Value::Result(a), Value::Result(b)) => a == b,
            // HashMap uses reference identity (like arrays)
            (Value::HashMap(a), Value::HashMap(b)) => Rc::ptr_eq(a, b),
            // HashSet uses reference identity (like arrays)
            (Value::HashSet(a), Value::HashSet(b)) => Rc::ptr_eq(a, b),
            // Queue uses reference identity (like arrays)
            (Value::Queue(a), Value::Queue(b)) => Rc::ptr_eq(a, b),
            // Stack uses reference identity (like arrays)
            (Value::Stack(a), Value::Stack(b)) => Rc::ptr_eq(a, b),
            // Regex uses reference identity (like arrays)
            (Value::Regex(a), Value::Regex(b)) => Rc::ptr_eq(a, b),
            // DateTime uses value equality (compare timestamps)
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                // Format number nicely (no trailing .0 for whole numbers)
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{:.0}", n)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s.as_ref()),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                let elements: Vec<String> = borrowed.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::NativeFunction(_) => write!(f, "<native fn>"),
            Value::JsonValue(json) => write!(f, "{}", json),
            Value::Option(opt) => match opt {
                Some(val) => write!(f, "Some({})", val),
                None => write!(f, "None"),
            },
            Value::Result(res) => match res {
                Ok(val) => write!(f, "Ok({})", val),
                Err(err) => write!(f, "Err({})", err),
            },
            Value::HashMap(map) => write!(f, "<HashMap size={}>", map.borrow().len()),
            Value::HashSet(set) => write!(f, "<HashSet size={}>", set.borrow().len()),
            Value::Queue(queue) => write!(f, "<Queue size={}>", queue.borrow().len()),
            Value::Stack(stack) => write!(f, "<Stack size={}>", stack.borrow().len()),
            Value::Regex(r) => write!(f, "<Regex /{}/>", r.as_str()),
            Value::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Null => write!(f, "Null"),
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                write!(f, "Array({:?})", &*borrowed)
            }
            Value::Function(func) => write!(f, "Function({:?})", func),
            Value::NativeFunction(_) => write!(f, "NativeFunction(<closure>)"),
            Value::JsonValue(json) => write!(f, "JsonValue({:?})", json),
            Value::Option(opt) => write!(f, "Option({:?})", opt),
            Value::Result(res) => write!(f, "Result({:?})", res),
            Value::HashMap(map) => write!(f, "HashMap(size={})", map.borrow().len()),
            Value::HashSet(set) => write!(f, "HashSet(size={})", set.borrow().len()),
            Value::Queue(queue) => write!(f, "Queue(size={})", queue.borrow().len()),
            Value::Stack(stack) => write!(f, "Stack(size={})", stack.borrow().len()),
            Value::Regex(r) => write!(f, "Regex(/{}/)", r.as_str()),
            Value::DateTime(dt) => write!(f, "DateTime({})", dt.to_rfc3339()),
        }
    }
}

/// Runtime error type with source span information
#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuntimeError {
    /// Type error
    #[error("Type error: {msg}")]
    TypeError {
        msg: String,
        span: crate::span::Span,
    },
    /// Undefined variable
    #[error("Undefined variable: {name}")]
    UndefinedVariable {
        name: String,
        span: crate::span::Span,
    },
    /// Division by zero
    #[error("Division by zero")]
    DivideByZero { span: crate::span::Span },
    /// Array index out of bounds
    #[error("Array index out of bounds")]
    OutOfBounds { span: crate::span::Span },
    /// Invalid numeric result (NaN, Infinity)
    #[error("Invalid numeric result")]
    InvalidNumericResult { span: crate::span::Span },
    /// Unknown opcode (VM error)
    #[error("Unknown opcode")]
    UnknownOpcode { span: crate::span::Span },
    /// Stack underflow (VM error)
    #[error("Stack underflow")]
    StackUnderflow { span: crate::span::Span },
    /// Unknown function
    #[error("Unknown function: {name}")]
    UnknownFunction {
        name: String,
        span: crate::span::Span,
    },
    /// Invalid stdlib argument
    #[error("Invalid stdlib argument")]
    InvalidStdlibArgument { span: crate::span::Span },
    /// Invalid index (non-integer)
    #[error("Invalid index: expected number")]
    InvalidIndex { span: crate::span::Span },
    /// Permission denied - filesystem
    #[error("Permission denied: {operation} access to {path}")]
    FilesystemPermissionDenied {
        operation: String,
        path: String,
        span: crate::span::Span,
    },
    /// Permission denied - network
    #[error("Permission denied: network access to {host}")]
    NetworkPermissionDenied {
        host: String,
        span: crate::span::Span,
    },
    /// Permission denied - process
    #[error("Permission denied: process execution of {command}")]
    ProcessPermissionDenied {
        command: String,
        span: crate::span::Span,
    },
    /// Permission denied - environment
    #[error("Permission denied: environment variable {var}")]
    EnvironmentPermissionDenied {
        var: String,
        span: crate::span::Span,
    },
    /// I/O error (file operations)
    #[error("I/O error: {message}")]
    IoError {
        message: String,
        span: crate::span::Span,
    },
    /// Unhashable type (collections)
    #[error("Cannot hash type {type_name} - only number, string, bool, null are hashable")]
    UnhashableType {
        type_name: String,
        span: crate::span::Span,
    },
}

impl RuntimeError {
    /// Get the source span for this error
    pub fn span(&self) -> crate::span::Span {
        match self {
            RuntimeError::TypeError { span, .. } => *span,
            RuntimeError::UndefinedVariable { span, .. } => *span,
            RuntimeError::DivideByZero { span } => *span,
            RuntimeError::OutOfBounds { span } => *span,
            RuntimeError::InvalidNumericResult { span } => *span,
            RuntimeError::UnknownOpcode { span } => *span,
            RuntimeError::StackUnderflow { span } => *span,
            RuntimeError::UnknownFunction { span, .. } => *span,
            RuntimeError::InvalidStdlibArgument { span } => *span,
            RuntimeError::InvalidIndex { span } => *span,
            RuntimeError::FilesystemPermissionDenied { span, .. } => *span,
            RuntimeError::NetworkPermissionDenied { span, .. } => *span,
            RuntimeError::ProcessPermissionDenied { span, .. } => *span,
            RuntimeError::EnvironmentPermissionDenied { span, .. } => *span,
            RuntimeError::IoError { span, .. } => *span,
            RuntimeError::UnhashableType { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let val = Value::Number(42.0);
        assert_eq!(val.to_display_string(), "42");
    }

    #[test]
    fn test_string_value() {
        let val = Value::string("hello");
        assert_eq!(val.to_display_string(), "hello");
    }

    #[test]
    fn test_array_value() {
        let val = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(val.to_display_string(), "[1, 2]");
    }

    #[test]
    fn test_type_names() {
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::string("hi").type_name(), "string");
        assert_eq!(Value::Bool(true).type_name(), "bool");
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::array(vec![]).type_name(), "array");
        assert_eq!(
            Value::Function(FunctionRef {
                name: "test".to_string(),
                arity: 0,
                bytecode_offset: 0,
                local_count: 0,
            })
            .type_name(),
            "function"
        );
    }

    #[test]
    fn test_is_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Number(1.0).is_truthy()); // Numbers are not truthy
        assert!(!Value::Null.is_truthy());
        assert!(!Value::string("hello").is_truthy());
    }

    #[test]
    fn test_to_string_number() {
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(2.5).to_string(), "2.5");
        assert_eq!(Value::Number(-5.0).to_string(), "-5");
    }

    #[test]
    fn test_to_string_string() {
        assert_eq!(Value::string("hello").to_string(), "hello");
    }

    #[test]
    fn test_to_string_bool() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_to_string_null() {
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_to_string_array() {
        let arr = Value::array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(arr.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn test_to_string_nested_array() {
        let inner = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let outer = Value::array(vec![inner, Value::Number(3.0)]);
        assert_eq!(outer.to_string(), "[[1, 2], 3]");
    }

    #[test]
    fn test_to_string_function() {
        let func = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 2,
            bytecode_offset: 0,
            local_count: 0,
        });
        assert_eq!(func.to_string(), "<fn test>");
    }

    #[test]
    fn test_equality_numbers() {
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_ne!(Value::Number(42.0), Value::Number(43.0));
    }

    #[test]
    fn test_equality_strings() {
        assert_eq!(Value::string("hello"), Value::string("hello"));
        assert_ne!(Value::string("hello"), Value::string("world"));
    }

    #[test]
    fn test_equality_bools() {
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_ne!(Value::Bool(true), Value::Bool(false));
    }

    #[test]
    fn test_equality_null() {
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn test_equality_different_types() {
        assert_ne!(Value::Number(1.0), Value::Bool(true));
        assert_ne!(Value::Null, Value::Number(0.0));
    }

    #[test]
    fn test_array_reference_identity() {
        let arr1 = Value::array(vec![Value::Number(1.0)]);
        let arr2 = arr1.clone(); // Same reference
        let arr3 = Value::array(vec![Value::Number(1.0)]); // Different reference

        assert_eq!(arr1, arr2); // Same reference
        assert_ne!(arr1, arr3); // Different reference, even with same contents
    }

    #[test]
    fn test_array_mutation_visible_through_references() {
        let arr1 = Value::array(vec![Value::Number(1.0), Value::Number(2.0)]);
        let arr2 = arr1.clone(); // Same reference

        // Mutate through arr1
        if let Value::Array(a) = &arr1 {
            a.borrow_mut()[0] = Value::Number(42.0);
        }

        // Verify arr2 sees the change
        if let Value::Array(a) = &arr2 {
            assert_eq!(a.borrow()[0], Value::Number(42.0));
        }
    }

    #[test]
    fn test_function_equality() {
        let func1 = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 0,
            bytecode_offset: 0,
            local_count: 0,
        });
        let func2 = Value::Function(FunctionRef {
            name: "test".to_string(),
            arity: 1,
            bytecode_offset: 100,
            local_count: 0,
        });
        let func3 = Value::Function(FunctionRef {
            name: "other".to_string(),
            arity: 0,
            bytecode_offset: 0,
            local_count: 0,
        });

        assert_eq!(func1, func2); // Same name, different arity/offset
        assert_ne!(func1, func3); // Different name
    }

    #[test]
    fn test_runtime_errors() {
        use crate::span::Span;

        let err1 = RuntimeError::DivideByZero {
            span: Span::dummy(),
        };
        let err2 = RuntimeError::OutOfBounds {
            span: Span::dummy(),
        };
        let err3 = RuntimeError::UnknownFunction {
            name: "foo".to_string(),
            span: Span::dummy(),
        };

        assert_eq!(err1.to_string(), "Division by zero");
        assert_eq!(err2.to_string(), "Array index out of bounds");
        assert_eq!(err3.to_string(), "Unknown function: foo");
    }
}
