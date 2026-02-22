//! Type system representation

use crate::ffi::ExternType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic type parameter with optional constraint bound
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeParamDef {
    pub name: String,
    pub bound: Option<Box<Type>>,
    /// Trait bounds from `:` syntax — e.g. `T: Copy + Display` → `["Copy", "Display"]`.
    /// Populated from AST `TypeParam.trait_bounds` during type checking.
    #[serde(default)]
    pub trait_bounds: Vec<String>,
}

/// Structural type member (field or method signature)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StructuralMemberType {
    pub name: String,
    pub ty: Type,
}

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Never type (empty set of values)
    Never,
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
        type_params: Vec<TypeParamDef>,
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
    /// Union type (A | B)
    Union(Vec<Type>),
    /// Intersection type (A & B)
    Intersection(Vec<Type>),
    /// Structural type { field: type, method: (params) -> return }
    Structural { members: Vec<StructuralMemberType> },
}

impl Type {
    /// Construct a normalized union type from members.
    pub fn union(mut members: Vec<Type>) -> Type {
        let mut flat = Vec::new();
        for member in members.drain(..) {
            match member {
                Type::Union(inner) => {
                    flat.extend(inner);
                }
                Type::Never => {}
                other => flat.push(other),
            }
        }

        let mut normalized = Vec::new();
        for member in flat {
            let norm = member.normalized();
            if !normalized
                .iter()
                .any(|existing: &Type| existing.normalized() == norm)
            {
                normalized.push(norm);
            }
        }

        if normalized.is_empty() {
            return Type::Never;
        }
        if normalized.len() == 1 {
            return normalized.remove(0);
        }

        Type::Union(normalized)
    }

    /// Construct a normalized intersection type from members.
    pub fn intersection(mut members: Vec<Type>) -> Type {
        let mut flat = Vec::new();
        for member in members.drain(..) {
            match member {
                Type::Intersection(inner) => flat.extend(inner),
                Type::Never => return Type::Never,
                other => flat.push(other),
            }
        }

        let mut normalized = Vec::new();
        for member in flat {
            let norm = member.normalized();
            if !normalized
                .iter()
                .any(|existing: &Type| existing.normalized() == norm)
            {
                normalized.push(norm);
            }
        }

        if normalized.is_empty() {
            return Type::Never;
        }
        if normalized.len() == 1 {
            return normalized.remove(0);
        }

        if let Some((index, union_members)) =
            normalized
                .iter()
                .enumerate()
                .find_map(|(idx, member)| match member {
                    Type::Union(members) => Some((idx, members.clone())),
                    _ => None,
                })
        {
            let mut others = normalized;
            others.remove(index);
            let mut distributed = Vec::new();
            for member in union_members {
                let mut group = others.clone();
                group.push(member);
                distributed.push(Type::intersection(group));
            }
            return Type::union(distributed);
        }

        if Self::has_incompatible_primitives(&normalized) {
            return Type::Never;
        }

        Type::Intersection(normalized)
    }

    /// Check if this type is compatible with another type
    pub fn is_assignable_to(&self, other: &Type) -> bool {
        let self_norm = self.normalized();
        let other_norm = other.normalized();

        // Unknown type is assignable to anything (error recovery)
        if matches!(self_norm, Type::Unknown) || matches!(other_norm, Type::Unknown) {
            return true;
        }

        match (&self_norm, &other_norm) {
            (Type::Never, _) => true,
            (_, Type::Never) => matches!(self_norm, Type::Never),
            // Same type is always assignable
            (a, b) if a == b => true,
            (Type::Union(members), target) => {
                members.iter().all(|member| member.is_assignable_to(target))
            }
            (source, Type::Union(members)) => {
                members.iter().any(|member| source.is_assignable_to(member))
            }
            (Type::Intersection(members), target) => {
                members.iter().any(|member| member.is_assignable_to(target))
            }
            (source, Type::Intersection(members)) => {
                members.iter().all(|member| source.is_assignable_to(member))
            }

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
                // Treat () -> unknown as a wildcard function type for guard checks
                if tp2.is_empty() && p2.is_empty() && matches!(r2.normalized(), Type::Unknown) {
                    return true;
                }

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

                if tp1.len() != tp2.len() {
                    return false;
                }
                for (a, b) in tp1.iter().zip(tp2.iter()) {
                    let a_bound = a.bound.as_ref().map(|bound| bound.normalized());
                    let b_bound = b.bound.as_ref().map(|bound| bound.normalized());
                    if a_bound != b_bound {
                        return false;
                    }
                }

                p1.iter().zip(p2.iter()).all(|(a, b)| a.is_assignable_to(b))
                    && r1.is_assignable_to(r2)
            }

            // CRITICAL: JsonValue is isolated - only json to json
            // Cannot assign json to non-json types (requires explicit extraction)
            (Type::JsonValue, Type::JsonValue) => true,

            // Extern types are assignable if they match
            (Type::Extern(a), Type::Extern(b)) => a == b,

            (Type::Structural { members: a }, Type::Structural { members: b }) => {
                for member in b {
                    let Some(actual) = a.iter().find(|m| m.name == member.name) else {
                        return false;
                    };
                    if !actual.ty.is_assignable_to(&member.ty) {
                        return false;
                    }
                }
                true
            }

            // No other types are assignable
            _ => false,
        }
    }

    /// Get a human-readable name for this type
    pub fn display_name(&self) -> String {
        match self {
            Type::Never => "never".to_string(),
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
                    let params = type_params
                        .iter()
                        .map(|param| {
                            if let Some(bound) = &param.bound {
                                format!("{} extends {}", param.name, bound.display_name())
                            } else {
                                param.name.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    result.push_str(&params);
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
            Type::Union(members) => members
                .iter()
                .map(|t| t.display_name())
                .collect::<Vec<_>>()
                .join(" | "),
            Type::Intersection(members) => members
                .iter()
                .map(|t| t.display_name())
                .collect::<Vec<_>>()
                .join(" & "),
            Type::Structural { members } => {
                let parts = members
                    .iter()
                    .map(|member| format!("{}: {}", member.name, member.ty.display_name()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", parts)
            }
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
                type_params: type_params
                    .iter()
                    .map(|param| TypeParamDef {
                        name: param.name.clone(),
                        bound: param.bound.as_ref().map(|b| Box::new(b.normalized())),
                        trait_bounds: param.trait_bounds.clone(),
                    })
                    .collect(),
                params: params.iter().map(|p| p.normalized()).collect(),
                return_type: Box::new(return_type.normalized()),
            },
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|t| t.normalized()).collect(),
            },
            Type::Union(members) => Type::union(members.clone()),
            Type::Intersection(members) => Type::intersection(members.clone()),
            Type::Structural { members } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| StructuralMemberType {
                        name: member.name.clone(),
                        ty: member.ty.normalized(),
                    })
                    .collect(),
            },
            other => other.clone(),
        }
    }

    fn has_incompatible_primitives(members: &[Type]) -> bool {
        let mut primitive = None;
        for member in members {
            let is_primitive = matches!(
                member,
                Type::Number | Type::String | Type::Bool | Type::Null | Type::Void
            );
            if !is_primitive {
                continue;
            }
            if let Some(ref existing) = primitive {
                if existing != member {
                    return true;
                }
            } else {
                primitive = Some(member.clone());
            }
        }
        false
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
        (Type::Union(a_members), Type::Union(b_members)) => {
            if a_members.len() != b_members.len() {
                return false;
            }
            a_members
                .iter()
                .zip(b_members.iter())
                .all(|(a, b)| match_type_params(a, b, substitutions))
        }
        (Type::Intersection(a_members), Type::Intersection(b_members)) => {
            if a_members.len() != b_members.len() {
                return false;
            }
            a_members
                .iter()
                .zip(b_members.iter())
                .all(|(a, b)| match_type_params(a, b, substitutions))
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
        assert_eq!(Type::Never.display_name(), "never");
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

    #[test]
    fn test_union_display() {
        let ty = Type::union(vec![Type::Number, Type::String]);
        assert_eq!(ty.display_name(), "number | string");
    }

    #[test]
    fn test_intersection_display() {
        let ty = Type::intersection(vec![Type::Number, Type::Number]);
        assert_eq!(ty.display_name(), "number");
    }

    #[test]
    fn test_union_assignability() {
        let union = Type::union(vec![Type::Number, Type::String]);
        assert!(Type::Number.is_assignable_to(&union));
        assert!(Type::String.is_assignable_to(&union));
        assert!(!Type::Bool.is_assignable_to(&union));
        assert!(!union.is_assignable_to(&Type::Number));
    }

    #[test]
    fn test_intersection_assignability() {
        let intersection = Type::intersection(vec![Type::Number, Type::Number]);
        assert!(intersection.is_assignable_to(&Type::Number));
        assert!(Type::Number.is_assignable_to(&intersection));
        let bad = Type::intersection(vec![Type::Number, Type::String]);
        assert_eq!(bad, Type::Never);
    }
}
