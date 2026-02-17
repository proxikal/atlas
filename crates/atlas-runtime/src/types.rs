//! Type system representation

use crate::ffi::ExternType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        /// Type parameters (empty for non-generic functions)
        type_params: Vec<String>,
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// JSON value type (isolated dynamic type for JSON interop)
    JsonValue,
    /// Generic type with instantiated arguments (e.g., Result<number, string>)
    Generic { name: String, type_args: Vec<Type> },
    /// Type alias with resolved target type
    Alias {
        name: String,
        type_args: Vec<Type>,
        target: Box<Type>,
    },
    /// Type parameter (unresolved variable, e.g., T in Result<T, E>)
    TypeParameter { name: String },
    /// Unknown type (for error recovery)
    Unknown,
    /// Extern type for FFI (Foreign Function Interface)
    Extern(ExternType),
}

impl Type {
    /// Check if this type is compatible with another type
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        let self_norm = self.normalized();
        let other_norm = other.normalized();

        // Unknown type is assignable to anything (error recovery)
        if matches!(self_norm, Type::Unknown) || matches!(other_norm, Type::Unknown) {
            return true;
        }

        match (&self_norm, &other_norm) {
            // Same type is always assignable
            (a, b) if a == b => true,

            // Array types must have compatible element types
            (Type::Array(a), Type::Array(b)) => a.is_assignable_to(b),

            // Function types must have compatible signatures
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                    type_params: tp1,
                    ..
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                    type_params: tp2,
                    ..
                },
            ) => {
                if p1.len() != p2.len() {
                    return false;
                }

                // Allow generic functions to be assigned to concrete signatures
                if !tp1.is_empty() && tp2.is_empty() {
                    let mut substitutions = HashMap::new();
                    for (actual_param, expected_param) in p1.iter().zip(p2.iter()) {
                        if !match_type_params(actual_param, expected_param, &mut substitutions) {
                            return false;
                        }
                    }
                    return match_type_params(r1, r2, &mut substitutions);
                }

                p1.iter().zip(p2.iter()).all(|(a, b)| a.is_assignable_to(b))
                    && r1.is_assignable_to(r2)
            }

            // CRITICAL: JsonValue is isolated - only json to json
            // Cannot assign json to non-json types (requires explicit extraction)
            (Type::JsonValue, Type::JsonValue) => true,

            // Extern types are assignable if they match
            (Type::Extern(a), Type::Extern(b)) => a == b,

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
            Type::Function {
                params,
                return_type,
                type_params,
            } => {
                let mut result = String::new();
                if !type_params.is_empty() {
                    result.push('<');
                    result.push_str(&type_params.join(", "));
                    result.push('>');
                }
                result.push('(');
                let param_strs: Vec<String> = params.iter().map(|p| p.display_name()).collect();
                result.push_str(&param_strs.join(", "));
                result.push_str(") -> ");
                result.push_str(&return_type.display_name());
                result
            }
            Type::JsonValue => "json".to_string(),
            Type::Generic { name, type_args } => {
                let args = type_args
                    .iter()
                    .map(|t| t.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args)
            }
            Type::Alias {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args = type_args
                        .iter()
                        .map(|t| t.display_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, args)
                }
            }
            Type::TypeParameter { name } => name.clone(),
            Type::Unknown => "?".to_string(),
            Type::Extern(extern_type) => extern_type.display_name().to_string(),
        }
    }

    /// Return a normalized type with aliases fully expanded.
    pub fn normalized(&self) -> Type {
        match self {
            Type::Alias { target, .. } => target.normalized(),
            Type::Array(inner) => Type::Array(Box::new(inner.normalized())),
            Type::Function {
                type_params,
                params,
                return_type,
            } => Type::Function {
                type_params: type_params.clone(),
                params: params.iter().map(|p| p.normalized()).collect(),
                return_type: Box::new(return_type.normalized()),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|t| t.normalized()).collect(),
            },
            other => other.clone(),
        }
    }
}

fn match_type_params(
    template: &Type,
    expected: &Type,
    substitutions: &mut HashMap<String, Type>,
) -> bool {
    let template_norm = template.normalized();
    let expected_norm = expected.normalized();

    match (&template_norm, &expected_norm) {
        (Type::TypeParameter { name }, actual) => {
            if let Some(existing) = substitutions.get(name) {
                existing.normalized() == *actual
            } else {
                substitutions.insert(name.clone(), actual.clone());
                true
            }
        }
        (Type::Array(inner_template), Type::Array(inner_expected)) => {
            match_type_params(inner_template, inner_expected, substitutions)
        }
        (
            Type::Function {
                type_params: tp1,
                params: p1,
                return_type: r1,
            },
            Type::Function {
                type_params: tp2,
                params: p2,
                return_type: r2,
            },
        ) => {
            if tp1.len() != tp2.len() || p1.len() != p2.len() {
                return false;
            }
            for (param1, param2) in p1.iter().zip(p2.iter()) {
                if !match_type_params(param1, param2, substitutions) {
                    return false;
                }
            }
            match_type_params(r1, r2, substitutions)
        }
        (
            Type::Generic {
                name: n1,
                type_args: a1,
            },
            Type::Generic {
                name: n2,
                type_args: a2,
            },
        ) => {
            if n1 != n2 || a1.len() != a2.len() {
                return false;
            }
            for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                if !match_type_params(arg1, arg2, substitutions) {
                    return false;
                }
            }
            true
        }
        (a, b) => a == b,
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
            type_params: vec![],
            params: vec![Type::Number, Type::String],
            return_type: Box::new(Type::Bool),
        };
        assert_eq!(func_type.display_name(), "(number, string) -> bool");
    }
}
