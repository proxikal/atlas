//! Type system representation

use serde::{Deserialize, Serialize};

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Number type (unified int/float)
    Number,
    /// String type
    String,
    /// Boolean type
    Bool,
    /// Null type
    Null,
    /// Void type (for functions that return nothing)
    Void,
    /// Array type
    Array(Box<Type>),
    /// Function type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Unknown type (for error recovery)
    Unknown,
}

impl Type {
    /// Check if this type is compatible with another type
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        // Unknown type is assignable to anything (error recovery)
        if matches!(self, Type::Unknown) || matches!(other, Type::Unknown) {
            return true;
        }

        match (self, other) {
            // Same type is always assignable
            (a, b) if a == b => true,

            // Array types must have compatible element types
            (Type::Array(a), Type::Array(b)) => a.is_assignable_to(b),

            // Function types must have compatible signatures
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(a, b)| a.is_assignable_to(b))
                    && r1.is_assignable_to(r2)
            }

            // No other types are assignable
            _ => false,
        }
    }

    /// Get a human-readable name for this type
    pub fn display_name(&self) -> String {
        match self {
            Type::Number => "number".to_string(),
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Null => "null".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(inner) => format!("{}[]", inner.display_name()),
            Type::Function { .. } => "function".to_string(),
            Type::Unknown => "?".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Number.display_name(), "number");
        assert_eq!(Type::String.display_name(), "string");
        assert_eq!(Type::Bool.display_name(), "bool");
        assert_eq!(Type::Null.display_name(), "null");
        assert_eq!(Type::Void.display_name(), "void");
    }

    #[test]
    fn test_array_type() {
        let arr_type = Type::Array(Box::new(Type::Number));
        assert_eq!(arr_type.display_name(), "number[]");
    }

    #[test]
    fn test_function_type() {
        let func_type = Type::Function {
            params: vec![Type::Number, Type::String],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(func_type.display_name(), "function");
    }
}
