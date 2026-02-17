//! Type narrowing utilities for control flow analysis.

use crate::ast::{BinaryOp, Expr, Literal, UnaryOp};
use crate::typechecker::TypeChecker;
use crate::types::Type;
use std::collections::HashMap;

impl<'a> TypeChecker<'a> {
    pub(super) fn narrow_condition(
        &self,
        expr: &Expr,
    ) -> (HashMap<String, Type>, HashMap<String, Type>) {
        match expr {
            Expr::Binary(binary) => match binary.op {
                BinaryOp::Eq | BinaryOp::Ne => {
                    let is_equal = matches!(binary.op, BinaryOp::Eq);
                    if let Some((name, target)) =
                        self.extract_literal_guard(&binary.left, &binary.right)
                    {
                        return self.narrow_name_by_type(&name, &target, is_equal);
                    }
                    if let Some((name, target)) =
                        self.extract_typeof_guard(&binary.left, &binary.right)
                    {
                        return self.narrow_name_by_type(&name, &target, is_equal);
                    }
                    (HashMap::new(), HashMap::new())
                }
                BinaryOp::And => {
                    let (left_true, left_false) = self.narrow_condition(&binary.left);
                    let (right_true, right_false) = self.narrow_condition(&binary.right);
                    let merged_true = merge_narrowings(left_true, right_true);
                    let merged_false = merge_or_narrowings(left_false, right_false);
                    (merged_true, merged_false)
                }
                BinaryOp::Or => {
                    let (left_true, left_false) = self.narrow_condition(&binary.left);
                    let (right_true, right_false) = self.narrow_condition(&binary.right);
                    let merged_true = merge_or_narrowings(left_true, right_true);
                    let merged_false = merge_narrowings(left_false, right_false);
                    (merged_true, merged_false)
                }
                _ => (HashMap::new(), HashMap::new()),
            },
            Expr::Unary(unary) => {
                if unary.op == UnaryOp::Not {
                    let (true_map, false_map) = self.narrow_condition(&unary.expr);
                    return (false_map, true_map);
                }
                (HashMap::new(), HashMap::new())
            }
            Expr::Call(call) => {
                if let Some((name, target)) = self.extract_type_guard(call) {
                    return self.narrow_name_by_type(&name, &target, true);
                }
                (HashMap::new(), HashMap::new())
            }
            Expr::Group(group) => self.narrow_condition(&group.expr),
            Expr::Try(try_expr) => self.narrow_condition(&try_expr.expr),
            _ => (HashMap::new(), HashMap::new()),
        }
    }

    fn extract_literal_guard(&self, left: &Expr, right: &Expr) -> Option<(String, Type)> {
        if let (Expr::Identifier(id), Expr::Literal(lit, _)) = (left, right) {
            return Some((id.name.clone(), literal_type(lit)));
        }
        if let (Expr::Literal(lit, _), Expr::Identifier(id)) = (left, right) {
            return Some((id.name.clone(), literal_type(lit)));
        }
        None
    }

    fn extract_typeof_guard(&self, left: &Expr, right: &Expr) -> Option<(String, Type)> {
        if let Some(name) = typeof_target(left) {
            if let Expr::Literal(Literal::String(value), _) = right {
                return type_from_typeof_value(value).map(|ty| (name, ty));
            }
        }
        if let Some(name) = typeof_target(right) {
            if let Expr::Literal(Literal::String(value), _) = left {
                return type_from_typeof_value(value).map(|ty| (name, ty));
            }
        }
        None
    }

    fn extract_type_guard(&self, call: &crate::ast::CallExpr) -> Option<(String, Type)> {
        let callee_name = match &*call.callee {
            Expr::Identifier(id) => id.name.as_str(),
            _ => return None,
        };

        if callee_name == "isType" {
            if call.args.len() != 2 {
                return None;
            }
            let arg_name = match &call.args[0] {
                Expr::Identifier(id) => id.name.clone(),
                _ => return None,
            };
            let type_name = match &call.args[1] {
                Expr::Literal(Literal::String(name), _) => name.as_str(),
                _ => return None,
            };
            let target = type_from_typeof_value(type_name)?;
            return Some((arg_name, target));
        }

        if callee_name == "hasField" {
            if call.args.len() != 2 {
                return None;
            }
            let arg_name = match &call.args[0] {
                Expr::Identifier(id) => id.name.clone(),
                _ => return None,
            };
            let field = match &call.args[1] {
                Expr::Literal(Literal::String(name), _) => name.as_str(),
                _ => return None,
            };
            let target = self.guard_target_for_field(field);
            return Some((arg_name, target));
        }

        if callee_name == "hasMethod" {
            if call.args.len() != 2 {
                return None;
            }
            let arg_name = match &call.args[0] {
                Expr::Identifier(id) => id.name.clone(),
                _ => return None,
            };
            let field = match &call.args[1] {
                Expr::Literal(Literal::String(name), _) => name.as_str(),
                _ => return None,
            };
            let target = self.guard_target_for_method(field);
            return Some((arg_name, target));
        }

        if callee_name == "hasTag" {
            if call.args.len() != 2 {
                return None;
            }
            let arg_name = match &call.args[0] {
                Expr::Identifier(id) => id.name.clone(),
                _ => return None,
            };
            let _tag_value = match &call.args[1] {
                Expr::Literal(Literal::String(name), _) => name.as_str(),
                _ => return None,
            };
            let target = self.guard_target_for_tag();
            return Some((arg_name, target));
        }

        let guard = self.type_guards.lookup(callee_name)?;
        let arg = call.args.get(guard.param_index)?;
        let arg_name = match arg {
            Expr::Identifier(id) => id.name.clone(),
            _ => return None,
        };

        Some((arg_name, guard.target.clone()))
    }

    fn narrow_name_by_type(
        &self,
        name: &str,
        target: &Type,
        positive: bool,
    ) -> (HashMap<String, Type>, HashMap<String, Type>) {
        let mut true_map = HashMap::new();
        let mut false_map = HashMap::new();

        let original = match self.symbol_table.lookup(name) {
            Some(symbol) => symbol.ty.clone(),
            None => return (true_map, false_map),
        };

        if positive {
            let narrowed = narrow_to(&original, target);
            true_map.insert(name.to_string(), narrowed);
            let excluded = exclude_from(&original, target);
            false_map.insert(name.to_string(), excluded);
        } else {
            let narrowed = exclude_from(&original, target);
            true_map.insert(name.to_string(), narrowed);
            let included = narrow_to(&original, target);
            false_map.insert(name.to_string(), included);
        }

        (true_map, false_map)
    }
}

fn literal_type(lit: &Literal) -> Type {
    match lit {
        Literal::Number(_) => Type::Number,
        Literal::String(_) => Type::String,
        Literal::Bool(_) => Type::Bool,
        Literal::Null => Type::Null,
    }
}

fn typeof_target(expr: &Expr) -> Option<String> {
    if let Expr::Call(call) = expr {
        let callee_name = match &*call.callee {
            Expr::Identifier(id) => id.name.as_str(),
            _ => return None,
        };
        if callee_name != "typeof" || call.args.len() != 1 {
            return None;
        }
        if let Expr::Identifier(id) = &call.args[0] {
            return Some(id.name.clone());
        }
    }
    None
}

fn type_from_typeof_value(value: &str) -> Option<Type> {
    match value {
        "string" => Some(Type::String),
        "number" => Some(Type::Number),
        "bool" => Some(Type::Bool),
        "null" => Some(Type::Null),
        "array" => Some(Type::Array(Box::new(Type::Unknown))),
        "function" => Some(Type::Function {
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Box::new(Type::Unknown),
        }),
        "json" => Some(Type::JsonValue),
        "object" => Some(Type::JsonValue),
        _ => None,
    }
}

fn narrow_to(original: &Type, target: &Type) -> Type {
    let original = original.normalized();
    let target = target.normalized();

    if let Type::Union(members) = original {
        let filtered: Vec<Type> = members
            .into_iter()
            .filter(|member| member.is_assignable_to(&target))
            .collect();
        return Type::union(filtered);
    }

    if original.is_assignable_to(&target) {
        return target;
    }

    Type::Never
}

fn exclude_from(original: &Type, target: &Type) -> Type {
    let original = original.normalized();
    let target = target.normalized();

    if let Type::Union(members) = original {
        let filtered: Vec<Type> = members
            .into_iter()
            .filter(|member| !member.is_assignable_to(&target))
            .collect();
        return Type::union(filtered);
    }

    if original.is_assignable_to(&target) {
        return Type::Never;
    }

    original
}

fn merge_narrowings(
    mut left: HashMap<String, Type>,
    right: HashMap<String, Type>,
) -> HashMap<String, Type> {
    for (name, ty) in right {
        if let Some(existing) = left.get(&name) {
            let merged = Type::intersection(vec![existing.clone(), ty]);
            left.insert(name, merged);
        } else {
            left.insert(name, ty);
        }
    }
    left
}

fn merge_or_narrowings(
    left: HashMap<String, Type>,
    right: HashMap<String, Type>,
) -> HashMap<String, Type> {
    let mut merged = HashMap::new();
    for (name, left_ty) in left {
        if let Some(right_ty) = right.get(&name) {
            merged.insert(name, Type::union(vec![left_ty, right_ty.clone()]));
        }
    }
    merged
}
