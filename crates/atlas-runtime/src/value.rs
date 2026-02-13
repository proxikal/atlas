//! Runtime value representation
//!
//! Shared value representation for interpreter and VM.
//! - Numbers, Bools, Null: Immediate values (stack-allocated)
//! - Strings: Heap-allocated, reference-counted (Rc<String>), immutable
//! - Arrays: Heap-allocated, reference-counted (Rc<RefCell<Vec<Value>>>), mutable
//! - Functions: Reference to bytecode or builtin

use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

/// Runtime value type
#[derive(Debug, Clone)]
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

    /// Convert value to string representation
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                // Format number nicely (no trailing .0 for whole numbers)
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.as_ref().clone(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let borrowed = arr.borrow();
                let elements: Vec<String> = borrowed.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            Value::Function(f) => format!("<fn {}>", f.name),
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
            _ => false,
        }
    }
}

impl Eq for Value {}

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
    DivideByZero {
        span: crate::span::Span,
    },
    /// Array index out of bounds
    #[error("Array index out of bounds")]
    OutOfBounds {
        span: crate::span::Span,
    },
    /// Invalid numeric result (NaN, Infinity)
    #[error("Invalid numeric result")]
    InvalidNumericResult {
        span: crate::span::Span,
    },
    /// Unknown opcode (VM error)
    #[error("Unknown opcode")]
    UnknownOpcode {
        span: crate::span::Span,
    },
    /// Stack underflow (VM error)
    #[error("Stack underflow")]
    StackUnderflow {
        span: crate::span::Span,
    },
    /// Unknown function
    #[error("Unknown function: {name}")]
    UnknownFunction {
        name: String,
        span: crate::span::Span,
    },
    /// Invalid stdlib argument
    #[error("Invalid stdlib argument")]
    InvalidStdlibArgument {
        span: crate::span::Span,
    },
    /// Invalid index (non-integer)
    #[error("Invalid index: expected number")]
    InvalidIndex {
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
        assert_eq!(Value::Number(3.14).to_string(), "3.14");
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
        let arr = Value::array(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]);
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

        let err1 = RuntimeError::DivideByZero { span: Span::dummy() };
        let err2 = RuntimeError::OutOfBounds { span: Span::dummy() };
        let err3 = RuntimeError::UnknownFunction {
            name: "foo".to_string(),
            span: Span::dummy(),
        };

        assert_eq!(err1.to_string(), "Division by zero");
        assert_eq!(err2.to_string(), "Array index out of bounds");
        assert_eq!(err3.to_string(), "Unknown function: foo");
    }
}
