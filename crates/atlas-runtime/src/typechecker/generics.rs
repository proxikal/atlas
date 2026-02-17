//! Generic type inference and monomorphization
//!
//! Implements:
//! - Hindley-Milner style type inference for generic functions
//! - Monomorphization (generating specialized versions for each type instantiation)
//! - Unification, occurs check, and type parameter substitution

use crate::types::Type;
use std::collections::HashMap;

/// Type inference error
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceError {
    /// Type mismatch during unification
    TypeMismatch { expected: Type, actual: Type },
    /// Infinite type detected (occurs check failed)
    InfiniteType { param: String, ty: Type },
    /// Insufficient information to infer type
    CannotInfer,
}

/// Type inference engine using unification
pub struct TypeInferer {
    /// Substitutions map: type parameter name -> concrete type
    substitutions: HashMap<String, Type>,
}

impl TypeInferer {
    /// Create a new type inferer
    pub fn new() -> Self {
        Self {
            substitutions: HashMap::new(),
        }
    }

    /// Unify two types, building substitution map
    ///
    /// Returns Ok(()) if unification succeeds, Err otherwise.
    #[allow(clippy::result_large_err)]
    pub fn unify(&mut self, expected: &Type, actual: &Type) -> Result<(), InferenceError> {
        // Apply existing substitutions to both types before unifying
        let expected = self.apply_substitutions(expected).normalized();
        let actual = self.apply_substitutions(actual).normalized();

        match (&expected, &actual) {
            // Type parameter unifies with anything (binds to it)
            (Type::TypeParameter { name }, actual_type) => {
                self.add_substitution(name, actual_type.clone())
            }
            (expected_type, Type::TypeParameter { name }) => {
                self.add_substitution(name, expected_type.clone())
            }

            // Unknown unifies with anything (error recovery)
            (Type::Unknown, _) | (_, Type::Unknown) => Ok(()),

            // Concrete types must match exactly
            (Type::Number, Type::Number) => Ok(()),
            (Type::String, Type::String) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),
            (Type::Void, Type::Void) => Ok(()),
            (Type::Null, Type::Null) => Ok(()),
            (Type::JsonValue, Type::JsonValue) => Ok(()),

            // Arrays must have compatible element types
            (Type::Array(e1), Type::Array(e2)) => self.unify(e1, e2),

            // Functions must have compatible signatures
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
                // Type params must match in count (but not necessarily names)
                if tp1.len() != tp2.len() {
                    return Err(InferenceError::TypeMismatch {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }

                // Parameter counts must match
                if p1.len() != p2.len() {
                    return Err(InferenceError::TypeMismatch {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }

                // Unify each parameter type
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2)?;
                }

                // Unify return types
                self.unify(r1, r2)
            }

            // Generic types must have same name and compatible type arguments
            (
                Type::Generic {
                    name: n1,
                    type_args: args1,
                },
                Type::Generic {
                    name: n2,
                    type_args: args2,
                },
            ) => {
                if n1 != n2 {
                    return Err(InferenceError::TypeMismatch {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }

                if args1.len() != args2.len() {
                    return Err(InferenceError::TypeMismatch {
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }

                // Unify each type argument
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    self.unify(arg1, arg2)?;
                }

                Ok(())
            }

            // Different types cannot unify
            _ => Err(InferenceError::TypeMismatch {
                expected: expected.clone(),
                actual: actual.clone(),
            }),
        }
    }

    /// Add a substitution for a type parameter
    #[allow(clippy::result_large_err)]
    fn add_substitution(&mut self, param: &str, ty: Type) -> Result<(), InferenceError> {
        // If we already have a substitution, unify with existing
        if let Some(existing) = self.substitutions.get(param).cloned() {
            return self.unify(&existing, &ty);
        }

        // Occurs check: prevent infinite types like T = Option<T>
        if self.occurs_in(param, &ty) {
            return Err(InferenceError::InfiniteType {
                param: param.to_string(),
                ty,
            });
        }

        // Add the substitution
        self.substitutions.insert(param.to_string(), ty);
        Ok(())
    }

    /// Check if a type parameter occurs in a type (occurs check)
    fn occurs_in(&self, param: &str, ty: &Type) -> bool {
        match ty {
            Type::TypeParameter { name } => {
                // If it's the same parameter, we have a cycle
                if name == param {
                    return true;
                }
                // If it's a different parameter that has a substitution, check recursively
                if let Some(substituted) = self.substitutions.get(name) {
                    return self.occurs_in(param, substituted);
                }
                false
            }
            Type::Array(elem) => self.occurs_in(param, elem),
            Type::Function {
                params,
                return_type,
                ..
            } => {
                params.iter().any(|p| self.occurs_in(param, p))
                    || self.occurs_in(param, return_type)
            }
            Type::Generic { type_args, .. } => {
                type_args.iter().any(|arg| self.occurs_in(param, arg))
            }
            Type::Alias {
                type_args, target, ..
            } => {
                type_args.iter().any(|arg| self.occurs_in(param, arg))
                    || self.occurs_in(param, target)
            }
            _ => false,
        }
    }

    /// Apply all substitutions to a type
    pub fn apply_substitutions(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeParameter { name } => {
                // Look up substitution
                if let Some(substituted) = self.substitutions.get(name) {
                    // Recursively apply substitutions in case substitution contains type params
                    self.apply_substitutions(substituted)
                } else {
                    // No substitution found, return as-is
                    ty.clone()
                }
            }
            Type::Array(elem) => Type::Array(Box::new(self.apply_substitutions(elem))),
            Type::Function {
                type_params,
                params,
                return_type,
            } => Type::Function {
                type_params: type_params.clone(),
                params: params.iter().map(|p| self.apply_substitutions(p)).collect(),
                return_type: Box::new(self.apply_substitutions(return_type)),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args
                    .iter()
                    .map(|arg| self.apply_substitutions(arg))
                    .collect(),
            },
            Type::Alias {
                name,
                type_args,
                target,
            } => Type::Alias {
                name: name.clone(),
                type_args: type_args
                    .iter()
                    .map(|arg| self.apply_substitutions(arg))
                    .collect(),
                target: Box::new(self.apply_substitutions(target)),
            },
            // Other types don't contain type parameters
            _ => ty.clone(),
        }
    }

    /// Get the substitution for a type parameter
    pub fn get_substitution(&self, param: &str) -> Option<&Type> {
        self.substitutions.get(param)
    }

    /// Check if all type parameters have been inferred
    pub fn all_inferred(&self, type_params: &[String]) -> bool {
        type_params
            .iter()
            .all(|param| self.substitutions.contains_key(param))
    }
}

impl Default for TypeInferer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Monomorphization
// ============================================================================

/// Monomorphization error
#[derive(Debug, Clone, PartialEq)]
pub enum MonomorphizeError {
    /// Type parameter count mismatch
    ArityMismatch { expected: usize, actual: usize },
    /// Failed to substitute types
    SubstitutionFailed(String),
}

/// Key for caching monomorphized function instances
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MonomorphicKey {
    function_name: String,
    /// Concrete types for each type parameter (using display names for hashing)
    type_args: Vec<String>,
}

/// Monomorphization engine
///
/// Generates specialized versions of generic functions for each unique
/// set of type arguments (Rust-style monomorphization).
///
/// For runtime execution, we primarily need to track type substitutions
/// rather than generate completely new AST nodes.
pub struct Monomorphizer {
    /// Cache of type substitutions for each monomorphic instance
    /// Maps (function_name, type_args) -> substitution map
    type_substitutions: HashMap<MonomorphicKey, HashMap<String, Type>>,
}

impl Monomorphizer {
    /// Create a new monomorphizer
    pub fn new() -> Self {
        Self {
            type_substitutions: HashMap::new(),
        }
    }

    /// Get or create type substitutions for a monomorphic instance
    ///
    /// Returns the substitution map for type parameters. This is used by
    /// the interpreter and VM to properly type-check values during execution.
    pub fn get_substitutions(
        &mut self,
        function_name: &str,
        type_params: &[String],
        type_args: &[Type],
    ) -> Result<HashMap<String, Type>, MonomorphizeError> {
        // Verify arity
        if type_params.len() != type_args.len() {
            return Err(MonomorphizeError::ArityMismatch {
                expected: type_params.len(),
                actual: type_args.len(),
            });
        }

        // Build cache key
        let key = MonomorphicKey {
            function_name: function_name.to_string(),
            type_args: type_args.iter().map(|t| t.display_name()).collect(),
        };

        // Check cache
        if let Some(subst) = self.type_substitutions.get(&key) {
            return Ok(subst.clone());
        }

        // Build substitution map
        let mut subst = HashMap::new();
        for (param_name, concrete_type) in type_params.iter().zip(type_args.iter()) {
            subst.insert(param_name.clone(), concrete_type.clone());
        }

        // Cache and return
        self.type_substitutions.insert(key, subst.clone());
        Ok(subst)
    }

    /// Generate mangled name for a monomorphic instance
    ///
    /// Example: identity<number> -> "identity$number"
    /// Example: map<string, number> -> "map$string$number"
    pub fn mangle_name(function_name: &str, type_args: &[Type]) -> String {
        if type_args.is_empty() {
            return function_name.to_string();
        }

        let args_str = type_args
            .iter()
            .map(|t| t.display_name())
            .collect::<Vec<_>>()
            .join("$");

        format!("{}${}", function_name, args_str)
    }

    /// Get the number of cached monomorphic instances
    pub fn instance_count(&self) -> usize {
        self.type_substitutions.len()
    }

    /// Clear the cache (mainly for testing)
    pub fn clear_cache(&mut self) {
        self.type_substitutions.clear();
    }
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_concrete_types() {
        let mut inferer = TypeInferer::new();

        // Same concrete types unify
        assert!(inferer.unify(&Type::Number, &Type::Number).is_ok());
        assert!(inferer.unify(&Type::String, &Type::String).is_ok());

        // Different concrete types don't unify
        assert!(inferer.unify(&Type::Number, &Type::String).is_err());
    }

    #[test]
    fn test_unify_type_parameter() {
        let mut inferer = TypeInferer::new();

        // Type parameter unifies with concrete type
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        assert!(inferer.unify(&t, &Type::Number).is_ok());

        // Check substitution was recorded
        assert_eq!(inferer.get_substitution("T"), Some(&Type::Number));
    }

    #[test]
    fn test_unify_array() {
        let mut inferer = TypeInferer::new();

        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let arr_t = Type::Array(Box::new(t.clone()));
        let arr_number = Type::Array(Box::new(Type::Number));

        assert!(inferer.unify(&arr_t, &arr_number).is_ok());
        assert_eq!(inferer.get_substitution("T"), Some(&Type::Number));
    }

    #[test]
    fn test_occurs_check() {
        let mut inferer = TypeInferer::new();

        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        // Try to create T = Option<T>
        let option_t = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![t.clone()],
        };

        // Should fail with infinite type error
        let result = inferer.unify(&t, &option_t);
        assert!(matches!(result, Err(InferenceError::InfiniteType { .. })));
    }

    #[test]
    fn test_apply_substitutions() {
        let mut inferer = TypeInferer::new();

        let t = Type::TypeParameter {
            name: "T".to_string(),
        };

        // Add substitution T -> number
        inferer.unify(&t, &Type::Number).unwrap();

        // Apply to Array<T> should give Array<number>
        let arr_t = Type::Array(Box::new(t));
        let result = inferer.apply_substitutions(&arr_t);

        assert_eq!(result, Type::Array(Box::new(Type::Number)));
    }

    #[test]
    fn test_nested_substitution() {
        let mut inferer = TypeInferer::new();

        // T -> U, U -> number
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let u = Type::TypeParameter {
            name: "U".to_string(),
        };

        inferer.unify(&t, &u).unwrap();
        inferer.unify(&u, &Type::Number).unwrap();

        // Applying substitutions to T should give number
        let result = inferer.apply_substitutions(&t);
        assert_eq!(result, Type::Number);
    }

    // Monomorphizer tests
    #[test]
    fn test_monomorphizer_get_substitutions() {
        let mut mono = Monomorphizer::new();

        let type_params = vec!["T".to_string()];
        let type_args = vec![Type::Number];

        let subst = mono
            .get_substitutions("identity", &type_params, &type_args)
            .unwrap();

        assert_eq!(subst.len(), 1);
        assert_eq!(subst.get("T"), Some(&Type::Number));
    }

    #[test]
    fn test_monomorphizer_multiple_params() {
        let mut mono = Monomorphizer::new();

        let type_params = vec!["T".to_string(), "E".to_string()];
        let type_args = vec![Type::String, Type::Number];

        let subst = mono
            .get_substitutions("map", &type_params, &type_args)
            .unwrap();

        assert_eq!(subst.len(), 2);
        assert_eq!(subst.get("T"), Some(&Type::String));
        assert_eq!(subst.get("E"), Some(&Type::Number));
    }

    #[test]
    fn test_monomorphizer_arity_mismatch() {
        let mut mono = Monomorphizer::new();

        let type_params = vec!["T".to_string()];
        let type_args = vec![Type::Number, Type::String]; // Too many

        let result = mono.get_substitutions("foo", &type_params, &type_args);

        assert!(matches!(
            result,
            Err(MonomorphizeError::ArityMismatch { .. })
        ));
    }

    #[test]
    fn test_monomorphizer_caching() {
        let mut mono = Monomorphizer::new();

        let type_params = vec!["T".to_string()];
        let type_args = vec![Type::Number];

        // First call
        mono.get_substitutions("identity", &type_params, &type_args)
            .unwrap();
        assert_eq!(mono.instance_count(), 1);

        // Second call with same args - should hit cache
        mono.get_substitutions("identity", &type_params, &type_args)
            .unwrap();
        assert_eq!(mono.instance_count(), 1); // Still 1

        // Different type args - new instance
        let type_args2 = vec![Type::String];
        mono.get_substitutions("identity", &type_params, &type_args2)
            .unwrap();
        assert_eq!(mono.instance_count(), 2);
    }

    #[test]
    fn test_mangle_name_no_args() {
        let name = Monomorphizer::mangle_name("foo", &[]);
        assert_eq!(name, "foo");
    }

    #[test]
    fn test_mangle_name_one_arg() {
        let name = Monomorphizer::mangle_name("identity", &[Type::Number]);
        assert_eq!(name, "identity$number");
    }

    #[test]
    fn test_mangle_name_multiple_args() {
        let name = Monomorphizer::mangle_name("map", &[Type::String, Type::Number]);
        assert_eq!(name, "map$string$number");
    }

    #[test]
    fn test_mangle_name_complex_types() {
        let array_type = Type::Array(Box::new(Type::Number));
        let name = Monomorphizer::mangle_name("process", &[array_type]);
        assert_eq!(name, "process$number[]");
    }
}
