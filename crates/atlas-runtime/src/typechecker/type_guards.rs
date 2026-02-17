//! Type guard registry and validation.

use crate::ast::FunctionDecl;
use crate::diagnostic::Diagnostic;
use crate::types::{StructuralMemberType, Type};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeGuardDef {
    pub name: String,
    pub param_index: usize,
    pub target: Type,
}

#[derive(Debug, Clone)]
pub struct TypeGuardRegistry {
    scopes: Vec<HashMap<String, TypeGuardDef>>,
}

impl TypeGuardRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            scopes: vec![HashMap::new()],
        };
        registry.register_builtins();
        registry
    }

    fn register_builtins(&mut self) {
        self.insert_builtin("isString", 0, Type::String);
        self.insert_builtin("isNumber", 0, Type::Number);
        self.insert_builtin("isBool", 0, Type::Bool);
        self.insert_builtin("isNull", 0, Type::Null);
        self.insert_builtin("isArray", 0, Type::Array(Box::new(Type::Unknown)));
        self.insert_builtin(
            "isFunction",
            0,
            Type::Function {
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Box::new(Type::Unknown),
            },
        );
        self.insert_builtin("isObject", 0, Type::JsonValue);
    }

    fn insert_builtin(&mut self, name: &str, param_index: usize, target: Type) {
        let def = TypeGuardDef {
            name: name.to_string(),
            param_index,
            target,
        };
        let scope = self.scopes.first_mut().expect("type guard scope");
        scope.insert(name.to_string(), def);
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, def: TypeGuardDef) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(def.name.clone(), def);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeGuardDef> {
        for scope in self.scopes.iter().rev() {
            if let Some(def) = scope.get(name) {
                return Some(def);
            }
        }
        None
    }
}

impl<'a> crate::typechecker::TypeChecker<'a> {
    pub(super) fn collect_type_guards(&mut self, program: &crate::ast::Program) {
        for item in &program.items {
            match item {
                crate::ast::Item::Function(func) => self.register_type_guard(func),
                crate::ast::Item::Statement(crate::ast::Stmt::FunctionDecl(func)) => {
                    self.register_type_guard(func);
                }
                crate::ast::Item::Export(export_decl) => {
                    if let crate::ast::ExportItem::Function(func) = &export_decl.item {
                        self.register_type_guard(func);
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn register_type_guards_in_block(&mut self, block: &crate::ast::Block) {
        for stmt in &block.statements {
            if let crate::ast::Stmt::FunctionDecl(func) = stmt {
                self.register_type_guard(func);
            }
        }
    }

    fn register_type_guard(&mut self, func: &FunctionDecl) {
        let Some(predicate) = &func.predicate else {
            return;
        };

        let return_type = self.resolve_type_ref_with_params(&func.return_type, &func.type_params);
        let return_norm = return_type.normalized();
        if return_norm != Type::Bool && return_norm != Type::Unknown {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    format!(
                        "Type predicate requires bool return type, found {}",
                        return_type.display_name()
                    ),
                    predicate.span,
                )
                .with_label("type predicate must return bool")
                .with_help("use `-> bool` before the predicate"),
            );
            return;
        }

        let param_index = match func
            .params
            .iter()
            .position(|param| param.name.name == predicate.param.name)
        {
            Some(index) => index,
            None => {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Type predicate refers to unknown parameter '{}'",
                            predicate.param.name
                        ),
                        predicate.param.span,
                    )
                    .with_label("unknown predicate parameter")
                    .with_help("use a parameter name from the function signature"),
                );
                return;
            }
        };

        let param_type = self
            .resolve_type_ref_with_params(&func.params[param_index].type_ref, &func.type_params);
        let target_type = self.resolve_type_ref_with_params(&predicate.target, &func.type_params);

        if !target_type.is_assignable_to(&param_type) {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    format!(
                        "Predicate type {} is not assignable to parameter type {}",
                        target_type.display_name(),
                        param_type.display_name()
                    ),
                    predicate.span,
                )
                .with_label("unsafe type predicate")
                .with_help("ensure the predicate type is a subtype of the parameter type"),
            );
            return;
        }

        let def = TypeGuardDef {
            name: func.name.name.clone(),
            param_index,
            target: target_type,
        };
        self.type_guards.define(def);
    }

    pub(super) fn guard_target_for_field(&self, name: &str) -> Type {
        Type::Structural {
            members: vec![StructuralMemberType {
                name: name.to_string(),
                ty: Type::Unknown,
            }],
        }
    }

    pub(super) fn guard_target_for_method(&self, name: &str) -> Type {
        Type::Structural {
            members: vec![StructuralMemberType {
                name: name.to_string(),
                ty: Type::Unknown,
            }],
        }
    }

    pub(super) fn guard_target_for_tag(&self) -> Type {
        Type::Structural {
            members: vec![StructuralMemberType {
                name: "tag".to_string(),
                ty: Type::String,
            }],
        }
    }
}
