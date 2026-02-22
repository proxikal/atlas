//! Advanced unification algorithm for type inference
//!
//! Provides constraint-based type inference with:
//! - Constraint accumulation and batch solving
//! - Occurs check to prevent infinite types
//! - Structural unification for compound types
//! - Constraint-aware unification respecting bounds
//! - Backtracking unification for union types
//! - Detailed, actionable error messages

use crate::types::{Type, TypeParamDef};
use std::collections::HashMap;

// ============================================================================
// Types and Errors
// ============================================================================

/// A type constraint to be solved
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    Equal(Type, Type),
    /// A type must be assignable to another
    Assignable { from: Type, to: Type },
    /// A type must satisfy a bound
    Bound { ty: Type, bound: Type },
}

/// Error produced during unification
#[derive(Debug, Clone, PartialEq)]
pub enum UnificationError {
    /// Types cannot be unified
    Mismatch { expected: Type, found: Type },
    /// Occurs check failed: type variable would create infinite type
    InfiniteType { var: String, ty: Type },
    /// A bound constraint was violated
    ConstraintViolation {
        ty: Type,
        bound: Type,
        detail: String,
    },
    /// A constraint could not be solved
    Unsolvable { detail: String },
}

impl UnificationError {
    /// Human-readable message for this error
    pub fn message(&self) -> String {
        match self {
            Self::Mismatch { expected, found } => format!(
                "type mismatch: expected {}, found {}",
                expected.display_name(),
                found.display_name()
            ),
            Self::InfiniteType { var, ty } => format!(
                "infinite type: '{}' cannot equal {}",
                var,
                ty.display_name()
            ),
            Self::ConstraintViolation { ty, bound, detail } => format!(
                "'{}' does not satisfy constraint '{}': {}",
                ty.display_name(),
                bound.display_name(),
                detail
            ),
            Self::Unsolvable { detail } => format!("unsolvable constraint: {}", detail),
        }
    }
}

// ============================================================================
// Unification Engine
// ============================================================================

/// Advanced unification engine
///
/// Accumulates type constraints and solves them in batch.
/// Supports backtracking for union types and constraint-aware binding.
pub struct UnificationEngine {
    /// Accumulated constraints to solve
    constraints: Vec<Constraint>,
    /// Current substitutions: type variable name -> concrete type
    substitutions: HashMap<String, Type>,
    /// Bounds for named type parameters
    bounds: HashMap<String, Type>,
    /// Counter for generating fresh type variable IDs
    next_var_id: u32,
}

impl UnificationEngine {
    /// Create a new unification engine
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            substitutions: HashMap::new(),
            bounds: HashMap::new(),
            next_var_id: 0,
        }
    }

    /// Create a fresh type variable (named `?hint<id>`)
    pub fn fresh_var(&mut self, hint: &str) -> Type {
        let id = self.next_var_id;
        self.next_var_id += 1;
        Type::TypeParameter {
            name: format!("?{}{}", hint, id),
        }
    }

    /// Declare a bound for a named type parameter
    pub fn add_bound(&mut self, param: &str, bound: Type) {
        self.bounds.insert(param.to_string(), bound);
    }

    /// Add a constraint: `a` must equal `b`
    pub fn constrain_equal(&mut self, a: Type, b: Type) {
        self.constraints.push(Constraint::Equal(a, b));
    }

    /// Add a constraint: `from` must be assignable to `to`
    pub fn constrain_assignable(&mut self, from: Type, to: Type) {
        self.constraints.push(Constraint::Assignable { from, to });
    }

    /// Add a bound constraint: `ty` must satisfy `bound`
    pub fn constrain_bound(&mut self, ty: Type, bound: Type) {
        self.constraints.push(Constraint::Bound { ty, bound });
    }

    /// Solve all accumulated constraints, returning any errors
    pub fn solve(&mut self) -> Vec<UnificationError> {
        let constraints = std::mem::take(&mut self.constraints);
        let mut errors = Vec::new();
        for constraint in constraints {
            if let Err(e) = self.solve_one(constraint) {
                errors.push(e);
            }
        }
        errors
    }

    /// Simplify constraints by applying current substitutions and removing trivial ones
    pub fn simplify(&mut self) {
        let constraints = std::mem::take(&mut self.constraints);
        self.constraints = constraints
            .into_iter()
            .map(|c| match c {
                Constraint::Equal(a, b) => Constraint::Equal(self.apply(&a), self.apply(&b)),
                Constraint::Bound { ty, bound } => Constraint::Bound {
                    ty: self.apply(&ty),
                    bound: self.apply(&bound),
                },
                Constraint::Assignable { from, to } => Constraint::Assignable {
                    from: self.apply(&from),
                    to: self.apply(&to),
                },
            })
            .filter(|c| !is_trivially_satisfied(c))
            .collect();
    }

    /// Solve a single constraint
    #[allow(clippy::result_large_err)]
    fn solve_one(&mut self, constraint: Constraint) -> Result<(), UnificationError> {
        match constraint {
            Constraint::Equal(a, b) => self.unify(a, b),
            Constraint::Assignable { from, to } => {
                let from_applied = self.apply(&from);
                let to_applied = self.apply(&to);
                if from_applied.is_assignable_to(&to_applied) {
                    Ok(())
                } else {
                    self.unify(from, to)
                }
            }
            Constraint::Bound { ty, bound } => {
                let ty_applied = self.apply(&ty);
                let bound_applied = self.apply(&bound);
                if ty_applied.is_assignable_to(&bound_applied) {
                    Ok(())
                } else {
                    Err(UnificationError::ConstraintViolation {
                        ty: ty_applied.clone(),
                        bound: bound_applied.clone(),
                        detail: format!(
                            "{} does not satisfy {}",
                            ty_applied.display_name(),
                            bound_applied.display_name()
                        ),
                    })
                }
            }
        }
    }

    /// Core unification: make types `a` and `b` equal
    #[allow(clippy::result_large_err)]
    pub fn unify(&mut self, a: Type, b: Type) -> Result<(), UnificationError> {
        let a = self.apply(&a).normalized();
        let b = self.apply(&b).normalized();

        match (&a, &b) {
            // Equal types trivially unify
            _ if a == b => Ok(()),

            // Type variable unifies with anything
            (Type::TypeParameter { name }, _) => {
                let name = name.clone();
                self.bind(name, b)
            }
            (_, Type::TypeParameter { name }) => {
                let name = name.clone();
                self.bind(name, a)
            }

            // Unknown is compatible with everything (error recovery)
            (Type::Unknown, _) | (_, Type::Unknown) => Ok(()),

            // Arrays: unify element types
            (Type::Array(ea), Type::Array(eb)) => {
                let ea = *ea.clone();
                let eb = *eb.clone();
                self.unify(ea, eb)
            }

            // Functions: unify parameter types and return types
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                    ..
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                    ..
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(UnificationError::Mismatch {
                        expected: a.clone(),
                        found: b.clone(),
                    });
                }
                let pairs: Vec<(Type, Type)> = p1
                    .iter()
                    .zip(p2.iter())
                    .map(|(x, y)| (x.clone(), y.clone()))
                    .collect();
                for (pa, pb) in pairs {
                    self.unify(pa, pb)?;
                }
                let ra = *r1.clone();
                let rb = *r2.clone();
                self.unify(ra, rb)
            }

            // Generic types: same name, unify each type argument
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
                if n1 != n2 || args1.len() != args2.len() {
                    return Err(UnificationError::Mismatch {
                        expected: a.clone(),
                        found: b.clone(),
                    });
                }
                let pairs: Vec<(Type, Type)> = args1
                    .iter()
                    .zip(args2.iter())
                    .map(|(x, y)| (x.clone(), y.clone()))
                    .collect();
                for (arg_a, arg_b) in pairs {
                    self.unify(arg_a, arg_b)?;
                }
                Ok(())
            }

            // Structural types: unify matching member types
            (Type::Structural { members: m1 }, Type::Structural { members: m2 }) => {
                let m2_clone = m2.clone();
                for req in &m2_clone {
                    if let Some(found) = m1.iter().find(|m| m.name == req.name) {
                        let found_ty = found.ty.clone();
                        let req_ty = req.ty.clone();
                        self.unify(found_ty, req_ty)?;
                    }
                }
                Ok(())
            }

            // Union type (right side): try to unify with any member via backtracking
            (other, Type::Union(members)) => {
                let other = other.clone();
                let members = members.clone();
                self.unify_with_union(other, members)
            }
            // Union type (left side): try to unify with any member via backtracking
            (Type::Union(members), other) => {
                let other = other.clone();
                let members = members.clone();
                self.unify_with_union(other, members)
            }

            // Alias: unify against the target type
            (Type::Alias { target, .. }, other) => {
                let t = *target.clone();
                let o = other.clone();
                self.unify(t, o)
            }
            (other, Type::Alias { target, .. }) => {
                let o = other.clone();
                let t = *target.clone();
                self.unify(o, t)
            }

            // Incompatible types
            _ => Err(UnificationError::Mismatch {
                expected: a,
                found: b,
            }),
        }
    }

    /// Try to unify a type with any member of a union (backtracking)
    #[allow(clippy::result_large_err)]
    fn unify_with_union(&mut self, ty: Type, members: Vec<Type>) -> Result<(), UnificationError> {
        // First try exact structural match
        for member in &members {
            let mut probe = UnificationEngine::new();
            probe.substitutions = self.substitutions.clone();
            probe.bounds = self.bounds.clone();
            if probe.unify(ty.clone(), member.clone()).is_ok() {
                // Commit this branch's substitutions
                self.substitutions = probe.substitutions;
                return Ok(());
            }
        }
        // No member matched
        Err(UnificationError::Mismatch {
            expected: Type::Union(members),
            found: ty,
        })
    }

    /// Bind a type variable to a concrete type
    #[allow(clippy::result_large_err)]
    fn bind(&mut self, var: String, ty: Type) -> Result<(), UnificationError> {
        // If already bound, unify the existing and new types
        if let Some(existing) = self.substitutions.get(&var).cloned() {
            return self.unify(existing, ty);
        }

        // Occurs check: prevent circular types like T = Option<T>
        if self.occurs_in(&var, &ty) {
            return Err(UnificationError::InfiniteType { var, ty });
        }

        // Check declared bound if any
        if let Some(bound) = self.bounds.get(&var).cloned() {
            let ty_norm = ty.normalized();
            if !ty_norm.is_assignable_to(&bound) {
                return Err(UnificationError::ConstraintViolation {
                    ty: ty_norm,
                    bound,
                    detail: format!("bound not satisfied for '{}'", var),
                });
            }
        }

        self.substitutions.insert(var, ty);
        Ok(())
    }

    /// Apply current substitutions to a type (fully resolved)
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::TypeParameter { name } => {
                if let Some(sub) = self.substitutions.get(name) {
                    self.apply(sub)
                } else {
                    ty.clone()
                }
            }
            Type::Array(elem) => Type::Array(Box::new(self.apply(elem))),
            Type::Function {
                type_params,
                params,
                return_type,
            } => Type::Function {
                type_params: type_params
                    .iter()
                    .map(|tp| TypeParamDef {
                        name: tp.name.clone(),
                        bound: tp.bound.as_ref().map(|b| Box::new(self.apply(b))),
                        trait_bounds: tp.trait_bounds.clone(),
                    })
                    .collect(),
                params: params.iter().map(|p| self.apply(p)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.apply(a)).collect(),
            },
            Type::Alias {
                name,
                type_args,
                target,
            } => Type::Alias {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.apply(a)).collect(),
                target: Box::new(self.apply(target)),
            },
            Type::Structural { members } => Type::Structural {
                members: members
                    .iter()
                    .map(|m| crate::types::StructuralMemberType {
                        name: m.name.clone(),
                        ty: self.apply(&m.ty),
                    })
                    .collect(),
            },
            Type::Union(members) => Type::union(members.iter().map(|m| self.apply(m)).collect()),
            Type::Intersection(members) => {
                Type::intersection(members.iter().map(|m| self.apply(m)).collect())
            }
            _ => ty.clone(),
        }
    }

    /// Occurs check: does variable `var` appear free in `ty`?
    fn occurs_in(&self, var: &str, ty: &Type) -> bool {
        match ty {
            Type::TypeParameter { name } => {
                if name == var {
                    return true;
                }
                // Transitively check through substitutions
                if let Some(sub) = self.substitutions.get(name) {
                    return self.occurs_in(var, sub);
                }
                false
            }
            Type::Array(elem) => self.occurs_in(var, elem),
            Type::Function {
                params,
                return_type,
                ..
            } => params.iter().any(|p| self.occurs_in(var, p)) || self.occurs_in(var, return_type),
            Type::Generic { type_args, .. } => type_args.iter().any(|a| self.occurs_in(var, a)),
            Type::Alias {
                type_args, target, ..
            } => type_args.iter().any(|a| self.occurs_in(var, a)) || self.occurs_in(var, target),
            Type::Structural { members } => members.iter().any(|m| self.occurs_in(var, &m.ty)),
            Type::Union(members) | Type::Intersection(members) => {
                members.iter().any(|m| self.occurs_in(var, m))
            }
            _ => false,
        }
    }

    /// Get the substitution for a named type variable
    pub fn get_substitution(&self, var: &str) -> Option<&Type> {
        self.substitutions.get(var)
    }

    /// Get all current substitutions
    pub fn substitutions(&self) -> &HashMap<String, Type> {
        &self.substitutions
    }

    /// Get the names of type variables that have not been solved
    pub fn unsolved_vars<'a>(&'a self, vars: &'a [String]) -> Vec<&'a str> {
        vars.iter()
            .filter(|v| !self.substitutions.contains_key(*v))
            .map(|v| v.as_str())
            .collect()
    }

    /// Number of pending constraints
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

impl Default for UnificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn is_trivially_satisfied(constraint: &Constraint) -> bool {
    match constraint {
        Constraint::Equal(a, b) => a.normalized() == b.normalized(),
        Constraint::Assignable { from, to } => from.is_assignable_to(to),
        Constraint::Bound { .. } => false,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── Unify type variable with concrete type ──────────────────────────────

    #[test]
    fn test_unify_type_var_with_type() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        assert!(engine.unify(t, Type::Number).is_ok());
        assert_eq!(engine.get_substitution("T"), Some(&Type::Number));
    }

    #[test]
    fn test_unify_concrete_same() {
        let mut engine = UnificationEngine::new();
        assert!(engine.unify(Type::Number, Type::Number).is_ok());
        assert!(engine.unify(Type::String, Type::String).is_ok());
        assert!(engine.unify(Type::Bool, Type::Bool).is_ok());
    }

    #[test]
    fn test_unify_concrete_mismatch() {
        let mut engine = UnificationEngine::new();
        let result = engine.unify(Type::Number, Type::String);
        assert!(matches!(result, Err(UnificationError::Mismatch { .. })));
    }

    #[test]
    fn test_unify_error_message() {
        let err = UnificationError::Mismatch {
            expected: Type::Number,
            found: Type::String,
        };
        let msg = err.message();
        assert!(msg.contains("number"));
        assert!(msg.contains("string"));
    }

    // ── Occurs check ────────────────────────────────────────────────────────

    #[test]
    fn test_occurs_check_prevents_infinite_type() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let option_t = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![t.clone()],
        };
        let result = engine.unify(t, option_t);
        assert!(
            matches!(result, Err(UnificationError::InfiniteType { .. })),
            "expected infinite type error"
        );
    }

    #[test]
    fn test_occurs_check_array() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let arr_t = Type::Array(Box::new(t.clone()));
        let result = engine.unify(t, arr_t);
        assert!(matches!(result, Err(UnificationError::InfiniteType { .. })));
    }

    // ── Structural unification ───────────────────────────────────────────────

    #[test]
    fn test_structural_unification_matching_members() {
        let mut engine = UnificationEngine::new();
        let a = Type::Structural {
            members: vec![crate::types::StructuralMemberType {
                name: "x".to_string(),
                ty: Type::Number,
            }],
        };
        let b = Type::Structural {
            members: vec![crate::types::StructuralMemberType {
                name: "x".to_string(),
                ty: Type::Number,
            }],
        };
        assert!(engine.unify(a, b).is_ok());
    }

    #[test]
    fn test_structural_unification_with_type_var() {
        let mut engine = UnificationEngine::new();
        let a = Type::Structural {
            members: vec![crate::types::StructuralMemberType {
                name: "value".to_string(),
                ty: Type::TypeParameter {
                    name: "T".to_string(),
                },
            }],
        };
        let b = Type::Structural {
            members: vec![crate::types::StructuralMemberType {
                name: "value".to_string(),
                ty: Type::String,
            }],
        };
        assert!(engine.unify(a, b).is_ok());
        assert_eq!(engine.get_substitution("T"), Some(&Type::String));
    }

    // ── Constraint solving ──────────────────────────────────────────────────

    #[test]
    fn test_constraint_equal_solve() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        engine.constrain_equal(t, Type::Number);
        let errors = engine.solve();
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert_eq!(engine.get_substitution("T"), Some(&Type::Number));
    }

    #[test]
    fn test_constraint_assignable_solve() {
        let mut engine = UnificationEngine::new();
        engine.constrain_assignable(Type::Number, Type::Number);
        let errors = engine.solve();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_constraint_assignable_failure() {
        let mut engine = UnificationEngine::new();
        engine.constrain_assignable(Type::String, Type::Number);
        let errors = engine.solve();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_constraint_bound_satisfied() {
        let mut engine = UnificationEngine::new();
        engine.constrain_bound(Type::Number, Type::Number);
        let errors = engine.solve();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_constraint_bound_violated() {
        let mut engine = UnificationEngine::new();
        engine.constrain_bound(Type::String, Type::Number);
        let errors = engine.solve();
        assert!(!errors.is_empty());
        assert!(matches!(
            &errors[0],
            UnificationError::ConstraintViolation { .. }
        ));
    }

    #[test]
    fn test_constraint_violation_message() {
        let err = UnificationError::ConstraintViolation {
            ty: Type::String,
            bound: Type::Number,
            detail: "test detail".to_string(),
        };
        let msg = err.message();
        assert!(msg.contains("string"));
        assert!(msg.contains("number"));
    }

    // ── Simplification ──────────────────────────────────────────────────────

    #[test]
    fn test_simplify_removes_trivial_constraints() {
        let mut engine = UnificationEngine::new();
        engine.constrain_equal(Type::Number, Type::Number);
        engine.constrain_assignable(Type::Bool, Type::Bool);
        engine.simplify();
        assert_eq!(engine.constraint_count(), 0);
    }

    #[test]
    fn test_simplify_keeps_unsolved_constraints() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        engine.constrain_equal(t, Type::Number);
        // Before solving, apply substitutions from other constraints
        engine.simplify();
        // Still has the unsolved constraint (T is not substituted yet)
        assert_eq!(engine.constraint_count(), 1);
    }

    // ── Backtracking unification ─────────────────────────────────────────────

    #[test]
    fn test_backtracking_unify_with_union_success() {
        let mut engine = UnificationEngine::new();
        let union_ty = Type::union(vec![Type::Number, Type::String]);
        assert!(engine.unify(Type::Number, union_ty).is_ok());
    }

    #[test]
    fn test_backtracking_unify_with_union_no_match() {
        let mut engine = UnificationEngine::new();
        let union_ty = Type::union(vec![Type::Number, Type::String]);
        let result = engine.unify(Type::Bool, union_ty);
        assert!(matches!(result, Err(UnificationError::Mismatch { .. })));
    }

    // ── Apply substitutions ──────────────────────────────────────────────────

    #[test]
    fn test_apply_substitution() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        engine.unify(t.clone(), Type::String).unwrap();
        let arr_t = Type::Array(Box::new(t));
        let result = engine.apply(&arr_t);
        assert_eq!(result, Type::Array(Box::new(Type::String)));
    }

    #[test]
    fn test_apply_nested_substitution() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let u = Type::TypeParameter {
            name: "U".to_string(),
        };
        engine.unify(t.clone(), u.clone()).unwrap();
        engine.unify(u, Type::Number).unwrap();
        let result = engine.apply(&t);
        assert_eq!(result, Type::Number);
    }

    // ── Fresh vars ──────────────────────────────────────────────────────────

    #[test]
    fn test_fresh_var_unique() {
        let mut engine = UnificationEngine::new();
        let v0 = engine.fresh_var("a");
        let v1 = engine.fresh_var("a");
        assert_ne!(v0, v1);
    }

    #[test]
    fn test_fresh_var_is_type_parameter() {
        let mut engine = UnificationEngine::new();
        let v = engine.fresh_var("ret");
        assert!(matches!(v, Type::TypeParameter { .. }));
    }

    // ── Bounds ──────────────────────────────────────────────────────────────

    #[test]
    fn test_add_bound_enforced_on_bind() {
        let mut engine = UnificationEngine::new();
        engine.add_bound("T", Type::Number);
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        // Binding to a non-number violates the bound
        let result = engine.unify(t, Type::String);
        assert!(
            matches!(result, Err(UnificationError::ConstraintViolation { .. })),
            "Expected constraint violation"
        );
    }

    #[test]
    fn test_add_bound_satisfied() {
        let mut engine = UnificationEngine::new();
        engine.add_bound("T", Type::Number);
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        assert!(engine.unify(t, Type::Number).is_ok());
    }

    // ── Unsolved vars ────────────────────────────────────────────────────────

    #[test]
    fn test_unsolved_vars() {
        let engine = UnificationEngine::new();
        let vars = vec!["T".to_string(), "U".to_string()];
        let unsolved = engine.unsolved_vars(&vars);
        assert_eq!(unsolved.len(), 2);
    }

    #[test]
    fn test_unsolved_vars_after_solving() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        engine.unify(t, Type::Number).unwrap();
        let vars = vec!["T".to_string(), "U".to_string()];
        let unsolved = engine.unsolved_vars(&vars);
        assert_eq!(unsolved, vec!["U"]);
    }

    // ── Generic type unification ────────────────────────────────────────────

    #[test]
    fn test_unify_generic_types() {
        let mut engine = UnificationEngine::new();
        let opt_num = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::Number],
        };
        let opt_num2 = opt_num.clone();
        assert!(engine.unify(opt_num, opt_num2).is_ok());
    }

    #[test]
    fn test_unify_generic_with_type_var() {
        let mut engine = UnificationEngine::new();
        let t = Type::TypeParameter {
            name: "T".to_string(),
        };
        let opt_t = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![t],
        };
        let opt_num = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::Number],
        };
        assert!(engine.unify(opt_t, opt_num).is_ok());
        assert_eq!(engine.get_substitution("T"), Some(&Type::Number));
    }

    #[test]
    fn test_unify_generic_name_mismatch() {
        let mut engine = UnificationEngine::new();
        let opt = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![Type::Number],
        };
        let result_type = Type::Generic {
            name: "Result".to_string(),
            type_args: vec![Type::Number],
        };
        let result = engine.unify(opt, result_type);
        assert!(matches!(result, Err(UnificationError::Mismatch { .. })));
    }

    // ── Array unification ────────────────────────────────────────────────────

    #[test]
    fn test_unify_arrays() {
        let mut engine = UnificationEngine::new();
        let arr_num = Type::Array(Box::new(Type::Number));
        let arr_num2 = arr_num.clone();
        assert!(engine.unify(arr_num, arr_num2).is_ok());
    }

    #[test]
    fn test_unify_arrays_mismatch() {
        let mut engine = UnificationEngine::new();
        let arr_num = Type::Array(Box::new(Type::Number));
        let arr_str = Type::Array(Box::new(Type::String));
        let result = engine.unify(arr_num, arr_str);
        assert!(matches!(result, Err(UnificationError::Mismatch { .. })));
    }

    // ── Function unification ─────────────────────────────────────────────────

    #[test]
    fn test_unify_function_types() {
        let mut engine = UnificationEngine::new();
        let f1 = Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::String),
        };
        let f2 = f1.clone();
        assert!(engine.unify(f1, f2).is_ok());
    }

    #[test]
    fn test_unify_function_param_count_mismatch() {
        let mut engine = UnificationEngine::new();
        let f1 = Type::Function {
            type_params: vec![],
            params: vec![Type::Number],
            return_type: Box::new(Type::Void),
        };
        let f2 = Type::Function {
            type_params: vec![],
            params: vec![Type::Number, Type::String],
            return_type: Box::new(Type::Void),
        };
        let result = engine.unify(f1, f2);
        assert!(matches!(result, Err(UnificationError::Mismatch { .. })));
    }
}
