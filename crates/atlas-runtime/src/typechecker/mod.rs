//! Type checking and inference
//!
//! The type checker enforces Atlas's strict type rules:
//! - No implicit any - all types must be explicit or inferrable
//! - No nullable - null only assigns to null type
//! - No truthy/falsey - conditionals require bool
//! - Strict equality - == requires same-type operands

mod constraints;
mod expr;
pub mod flow_sensitive;
pub mod generics;
pub mod inference;
mod methods;
mod narrowing;
pub mod suggestions;
mod type_guards;
pub mod unification;

use crate::ast::*;
use crate::diagnostic::error_codes;
use crate::diagnostic::Diagnostic;
use crate::module_loader::ModuleRegistry;
use crate::span::Span;
use crate::symbol::{SymbolKind, SymbolTable};
use crate::types::{Type, TypeParamDef};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AliasKey {
    name: String,
    type_args: Vec<Type>,
}

#[derive(Debug, Clone)]
struct AliasMetadata {
    deprecated: bool,
    since: Option<String>,
}

/// A function parameter with its resolved type and ownership annotation.
/// Populated during `check_function` and queried by Phase 07 call-site checking.
#[derive(Debug, Clone)]
pub struct TypedParam {
    pub name: String,
    pub ty: Type,
    pub ownership: Option<OwnershipAnnotation>,
}

/// Per-function ownership registry key: the function's unique name string.
/// Value: (param annotations in declaration order, return annotation).
pub type FnOwnershipEntry = (
    Vec<Option<OwnershipAnnotation>>,
    Option<OwnershipAnnotation>,
);

/// A registered trait's method signature entry.
#[derive(Debug, Clone)]
pub struct TraitMethodEntry {
    pub name: String,
    pub type_params: Vec<TypeParamDef>,
    pub param_types: Vec<Type>,
    pub return_type: Type,
}

/// Registry of known traits (built-in + user-defined).
#[derive(Debug, Default)]
pub struct TraitRegistry {
    /// Maps trait name -> list of required method signatures
    pub traits: HashMap<String, Vec<TraitMethodEntry>>,
    /// Set of built-in trait names (not user-definable)
    pub built_in: HashSet<String>,
    /// Maps (type_name, trait_name) -> whether type implements the trait.
    /// For built-in types, pre-populated. For user types, populated during impl checking.
    pub implementations: HashMap<(String, String), bool>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_built_ins();
        registry
    }

    fn register_built_ins(&mut self) {
        // Copy — types that can be freely copied (value semantics, marker trait)
        self.register_built_in("Copy", vec![]);

        // Move — types that require explicit ownership transfer (marker trait)
        self.register_built_in("Move", vec![]);

        // Drop — types with custom destructor logic
        self.register_built_in(
            "Drop",
            vec![TraitMethodEntry {
                name: "drop".to_string(),
                type_params: vec![],
                param_types: vec![],
                return_type: Type::Void,
            }],
        );

        // Display — types that can be converted to a string representation
        self.register_built_in(
            "Display",
            vec![TraitMethodEntry {
                name: "display".to_string(),
                type_params: vec![],
                param_types: vec![],
                return_type: Type::String,
            }],
        );

        // Debug — types that can be serialized to a debug string representation
        self.register_built_in(
            "Debug",
            vec![TraitMethodEntry {
                name: "debug_repr".to_string(),
                type_params: vec![],
                param_types: vec![],
                return_type: Type::String,
            }],
        );

        // All primitive types implement Copy (value semantics)
        for primitive in &["number", "string", "bool", "null"] {
            self.mark_implements(primitive, "Copy");
        }
        // Built-in types do NOT implement Display/Debug — they use stdlib str()/debug().
        // User-defined types implement Display/Debug to customize str() output.
    }

    fn register_built_in(&mut self, name: &str, methods: Vec<TraitMethodEntry>) {
        self.traits.insert(name.to_string(), methods);
        self.built_in.insert(name.to_string());
    }

    pub fn register_user_trait(&mut self, name: &str, methods: Vec<TraitMethodEntry>) {
        self.traits.insert(name.to_string(), methods);
    }

    pub fn mark_implements(&mut self, type_name: &str, trait_name: &str) {
        self.implementations
            .insert((type_name.to_string(), trait_name.to_string()), true);
    }

    pub fn implements(&self, type_name: &str, trait_name: &str) -> bool {
        self.implementations
            .get(&(type_name.to_string(), trait_name.to_string()))
            .copied()
            .unwrap_or(false)
    }

    pub fn trait_exists(&self, name: &str) -> bool {
        self.traits.contains_key(name)
    }

    pub fn get_methods(&self, trait_name: &str) -> Option<&Vec<TraitMethodEntry>> {
        self.traits.get(trait_name)
    }

    /// Returns the name of the first trait that declares a method with the given name.
    pub fn find_trait_with_method(&self, method_name: &str) -> Option<&str> {
        self.traits.iter().find_map(|(trait_name, methods)| {
            if methods.iter().any(|m| m.name == method_name) {
                Some(trait_name.as_str())
            } else {
                None
            }
        })
    }

    pub fn is_built_in(&self, name: &str) -> bool {
        self.built_in.contains(name)
    }
}

/// A resolved impl block entry — the methods available for dispatch.
#[derive(Debug, Clone)]
pub struct ImplEntry {
    pub trait_name: String,
    pub type_name: String,
    /// Method name -> impl method definition
    pub methods: HashMap<String, ImplMethod>,
}

/// Registry of all impl blocks: `(type_name, trait_name)` → `ImplEntry`.
/// Used by Phases 12/13/14 for method dispatch.
#[derive(Debug, Default)]
pub struct ImplRegistry {
    pub entries: HashMap<(String, String), ImplEntry>,
}

impl ImplRegistry {
    pub fn register(
        &mut self,
        type_name: &str,
        trait_name: &str,
        methods: HashMap<String, ImplMethod>,
    ) {
        self.entries.insert(
            (type_name.to_string(), trait_name.to_string()),
            ImplEntry {
                trait_name: trait_name.to_string(),
                type_name: type_name.to_string(),
                methods,
            },
        );
    }

    pub fn get_method(
        &self,
        type_name: &str,
        trait_name: &str,
        method_name: &str,
    ) -> Option<&ImplMethod> {
        self.entries
            .get(&(type_name.to_string(), trait_name.to_string()))
            .and_then(|entry| entry.methods.get(method_name))
    }

    pub fn has_impl(&self, type_name: &str, trait_name: &str) -> bool {
        self.entries
            .contains_key(&(type_name.to_string(), trait_name.to_string()))
    }
}

/// Type checker state
pub struct TypeChecker<'a> {
    /// Symbol table from binder
    symbol_table: &'a mut SymbolTable,
    /// Collected diagnostics
    pub(super) diagnostics: Vec<Diagnostic>,
    /// Type of the last expression statement processed
    last_expr_type: Option<Type>,
    /// Current function's return type (for return statement checking)
    current_function_return_type: Option<Type>,
    /// Current function's name and return type span (for related locations)
    current_function_info: Option<(String, Span)>,
    /// Whether we're inside a loop (for break/continue checking)
    in_loop: bool,
    /// Declared symbols in current function (name -> (span, kind))
    pub(super) declared_symbols: HashMap<String, (Span, SymbolKind)>,
    /// Used symbols in current function
    pub(super) used_symbols: HashSet<String>,
    /// Method table for method resolution
    pub(super) method_table: methods::MethodTable,
    /// Type guard registry for predicate-based narrowing
    pub(super) type_guards: type_guards::TypeGuardRegistry,
    /// Type alias declarations available in this module scope
    type_aliases: HashMap<String, TypeAliasDecl>,
    /// Cached alias resolutions (alias name + args -> resolved type)
    alias_cache: HashMap<AliasKey, Type>,
    /// Stack of aliases being resolved (circular detection)
    alias_resolution_stack: Vec<String>,
    /// Ownership annotations per function (name -> (param ownerships, return ownership)).
    /// Populated during `check_function`, queried by call-site checking (Phase 07).
    pub fn_ownership_registry: HashMap<String, FnOwnershipEntry>,
    /// Param ownership annotations for the current function being checked.
    /// Maps param name -> ownership annotation. Used by call-site checks to detect
    /// whether an argument is a `borrow` parameter of the enclosing function.
    pub(super) current_fn_param_ownerships: HashMap<String, Option<OwnershipAnnotation>>,
    /// Registry of all known traits (built-in + user-defined).
    pub trait_registry: TraitRegistry,
    /// Registry of all impl blocks keyed by (type_name, trait_name).
    pub impl_registry: ImplRegistry,
}

/// Convert a `Type` to a string key used for impl registry lookups.
/// Returns `None` for types that don't support trait dispatch in Block 3
/// (e.g. arrays, maps — user-defined struct types are added in v0.4).
/// Convert a `Type` to a string key used for impl registry lookups.
/// Returns `None` for types that don't support trait dispatch in Block 3.
/// User-defined struct types (with named types) are added in v0.4.
pub(crate) fn type_to_impl_key(ty: &Type) -> Option<String> {
    match ty.normalized() {
        Type::Number => Some("number".to_string()),
        Type::String => Some("string".to_string()),
        Type::Bool => Some("bool".to_string()),
        // Generic with no type args represents an opaque user type (e.g. MyType<>)
        Type::Generic { name, type_args } if type_args.is_empty() => Some(name),
        _ => None,
    }
}

impl<'a> TypeChecker<'a> {
    /// Create a new type checker
    pub fn new(symbol_table: &'a mut SymbolTable) -> Self {
        let type_aliases = symbol_table.type_aliases().clone();
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            last_expr_type: None,
            current_function_return_type: None,
            current_function_info: None,
            in_loop: false,
            declared_symbols: HashMap::new(),
            used_symbols: HashSet::new(),
            method_table: methods::MethodTable::new(),
            type_guards: type_guards::TypeGuardRegistry::new(),
            type_aliases,
            alias_cache: HashMap::new(),
            alias_resolution_stack: Vec::new(),
            fn_ownership_registry: HashMap::new(),
            current_fn_param_ownerships: HashMap::new(),
            trait_registry: TraitRegistry::new(),
            impl_registry: ImplRegistry::default(),
        }
    }

    /// Get the most recent expression type processed during checking.
    /// Useful for REPL scenarios where we want to display the type of the
    /// last evaluated expression without re-walking the AST.
    pub fn last_expression_type(&self) -> Option<Type> {
        self.last_expr_type.clone()
    }

    /// Type check a program
    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        self.collect_type_guards(program);
        self.validate_type_aliases(program);
        for item in &program.items {
            self.check_item(item);
        }

        std::mem::take(&mut self.diagnostics)
    }

    /// Type check a program with cross-module support (BLOCKER 04-C)
    ///
    /// Validates cross-module references and export consistency.
    ///
    /// # Arguments
    /// * `program` - The AST to type check
    /// * `_module_path` - Absolute path to this module (for future use)
    /// * `_registry` - Registry of bound modules (for future cross-module validation)
    pub fn check_with_modules(
        &mut self,
        program: &Program,
        _module_path: &Path,
        _registry: &ModuleRegistry,
    ) -> Vec<Diagnostic> {
        // Check for duplicate exports
        let mut exported_names: HashSet<String> = HashSet::new();

        for item in &program.items {
            if let Item::Export(export_decl) = item {
                let name = match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => &func.name.name,
                    crate::ast::ExportItem::Variable(var) => &var.name.name,
                    crate::ast::ExportItem::TypeAlias(alias) => &alias.name.name,
                };

                if exported_names.contains(name) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT5008",
                            format!("Duplicate export: '{}' is exported more than once", name),
                            export_decl.span,
                        )
                        .with_label("duplicate export")
                        .with_help(format!(
                            "remove one of the export statements for '{}'",
                            name
                        )),
                    );
                } else {
                    exported_names.insert(name.clone());
                }
            }
        }

        // Type check all items (imports already validated during binding)
        self.collect_type_guards(program);
        self.validate_type_aliases(program);
        for item in &program.items {
            self.check_item(item);
        }

        std::mem::take(&mut self.diagnostics)
    }

    /// Check a top-level item
    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.check_function(func),
            Item::Statement(stmt) => self.check_statement(stmt),
            Item::Import(_) => {
                // Import type checking handled in BLOCKER 04-C (cross-module types)
                // For now, just skip - imports are syntactically valid but not yet functional
            }
            Item::Export(export_decl) => {
                // Export wraps an item - check the inner item
                match &export_decl.item {
                    crate::ast::ExportItem::Function(func) => self.check_function(func),
                    crate::ast::ExportItem::Variable(var) => {
                        self.check_statement(&crate::ast::Stmt::VarDecl(var.clone()));
                    }
                    crate::ast::ExportItem::TypeAlias(_) => {
                        // Type aliases are validated in a pre-pass
                    }
                }
            }
            Item::Extern(_) => {
                // Extern type checking handled in phase-10b (FFI infrastructure)
                // For now, just skip - full implementation pending
            }
            Item::TypeAlias(_) => {
                // Type aliases are validated in a pre-pass
            }
            Item::Trait(trait_decl) => self.check_trait_decl(trait_decl),
            Item::Impl(impl_block) => self.check_impl_block(impl_block),
        }
    }

    /// Check a function declaration
    /// Hoist a nested function's signature into the current scope
    /// This mirrors what the binder does, ensuring nested function symbols
    /// are available when type-checking calls to them
    fn hoist_nested_function_signature(&mut self, func: &FunctionDecl) {
        // Resolve parameter types, handling type parameters
        let param_types: Vec<Type> = func
            .params
            .iter()
            .map(|p| self.resolve_type_ref_with_params(&p.type_ref, &func.type_params))
            .collect();

        // Resolve return type
        let return_type = self.resolve_type_ref_with_params(&func.return_type, &func.type_params);

        let type_params = func
            .type_params
            .iter()
            .map(|param| TypeParamDef {
                name: param.name.clone(),
                bound: param.bound.as_ref().map(|bound| {
                    Box::new(self.resolve_type_ref_with_params_and_context(
                        bound,
                        &func.type_params,
                        None,
                    ))
                }),
                trait_bounds: param
                    .trait_bounds
                    .iter()
                    .map(|tb| tb.trait_name.clone())
                    .collect(),
            })
            .collect();

        // Create function symbol
        let symbol = crate::symbol::Symbol {
            name: func.name.name.clone(),
            ty: Type::Function {
                type_params,
                params: param_types,
                return_type: Box::new(return_type),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: func.name.span,
            exported: false,
        };

        // Define in current scope (ignore redeclaration errors - binder already checked)
        let _ = self.symbol_table.define(symbol);
    }

    /// Resolve a type reference, treating names in type_params as TypeParameter
    fn resolve_type_ref_with_params(
        &mut self,
        type_ref: &TypeRef,
        type_params: &[crate::ast::TypeParam],
    ) -> Type {
        self.resolve_type_ref_with_params_and_context(type_ref, type_params, None)
    }

    fn resolve_type_ref_with_params_and_context(
        &mut self,
        type_ref: &TypeRef,
        type_params: &[crate::ast::TypeParam],
        expected: Option<&Type>,
    ) -> Type {
        match type_ref {
            TypeRef::Named(name, span) => {
                if type_params.iter().any(|tp| tp.name == *name) {
                    return Type::TypeParameter { name: name.clone() };
                }

                if let Some(alias) = self.type_aliases.get(name).cloned() {
                    if !alias.type_params.is_empty() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type alias '{}' expects {} type argument(s)",
                                    name,
                                    alias.type_params.len()
                                ),
                                *span,
                            )
                            .with_label("missing type arguments")
                            .with_help(format!(
                                "provide {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }
                    return self.resolve_type_alias(&alias, Vec::new(), *span);
                }

                self.resolve_type_ref_with_context(type_ref, expected)
            }
            TypeRef::Array(elem, _) => Type::Array(Box::new(
                self.resolve_type_ref_with_params_and_context(elem, type_params, None),
            )),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params
                    .iter()
                    .map(|p| self.resolve_type_ref_with_params_and_context(p, type_params, None))
                    .collect();
                let ret_type = Box::new(self.resolve_type_ref_with_params_and_context(
                    return_type,
                    type_params,
                    None,
                ));
                Type::Function {
                    type_params: vec![],
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Structural { members, .. } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| crate::types::StructuralMemberType {
                        name: member.name.clone(),
                        ty: self.resolve_type_ref_with_params_and_context(
                            &member.type_ref,
                            type_params,
                            None,
                        ),
                    })
                    .collect(),
            },
            TypeRef::Union { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_params_and_context(m, type_params, None))
                    .collect();
                Type::union(resolved)
            }
            TypeRef::Intersection { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_params_and_context(m, type_params, None))
                    .collect();
                Type::intersection(resolved)
            }
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                let resolved_args = type_args
                    .iter()
                    .map(|arg| {
                        self.resolve_type_ref_with_params_and_context(arg, type_params, None)
                    })
                    .collect::<Vec<_>>();

                if let Some(alias) = self.type_aliases.get(name).cloned() {
                    if alias.type_params.len() != resolved_args.len() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type alias '{}' expects {} type argument(s), found {}",
                                    name,
                                    alias.type_params.len(),
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments")
                            .with_help(format!(
                                "provide exactly {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }
                    return self.resolve_type_alias(&alias, resolved_args, *span);
                }

                // Validate built-in generic types
                let expected_arity = self.get_generic_type_arity(name);
                if let Some(arity) = expected_arity {
                    if resolved_args.len() != arity {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Generic type '{}' expects {} type argument(s), found {}",
                                    name,
                                    arity,
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments"),
                        );
                        return Type::Unknown;
                    }
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Unknown generic type '{}'", name),
                            *span,
                        )
                        .with_label("unknown type"),
                    );
                    return Type::Unknown;
                }

                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
        }
    }

    /// Check a trait declaration: register it in the trait registry, error on duplicates/builtins.
    fn check_trait_decl(&mut self, trait_decl: &TraitDecl) {
        let trait_name = trait_decl.name.name.clone();

        // Error if re-declaring a built-in trait
        if self.trait_registry.is_built_in(&trait_name) {
            self.diagnostics.push(Diagnostic::error_with_code(
                error_codes::TRAIT_REDEFINES_BUILTIN,
                format!("Cannot redefine built-in trait '{}'", trait_name),
                trait_decl.name.span,
            ));
            return;
        }

        // Error if duplicate user trait declaration
        if self.trait_registry.trait_exists(&trait_name) {
            self.diagnostics.push(Diagnostic::error_with_code(
                error_codes::TRAIT_ALREADY_DEFINED,
                format!("Trait '{}' is already defined", trait_name),
                trait_decl.name.span,
            ));
            return;
        }

        // Resolve method signatures and register
        let method_entries: Vec<TraitMethodEntry> = trait_decl
            .methods
            .iter()
            .map(|method_sig| {
                // Exclude `self` param — impl's self type differs from trait's self type
                let param_types: Vec<Type> = method_sig
                    .params
                    .iter()
                    .filter(|p| p.name.name != "self")
                    .map(|p| self.resolve_type_ref(&p.type_ref))
                    .collect();
                let return_type = self.resolve_type_ref(&method_sig.return_type);
                TraitMethodEntry {
                    name: method_sig.name.name.clone(),
                    type_params: method_sig
                        .type_params
                        .iter()
                        .map(|tp| TypeParamDef {
                            name: tp.name.clone(),
                            bound: None,
                            trait_bounds: tp
                                .trait_bounds
                                .iter()
                                .map(|tb| tb.trait_name.clone())
                                .collect(),
                        })
                        .collect(),
                    param_types,
                    return_type,
                }
            })
            .collect();

        self.trait_registry
            .register_user_trait(&trait_name, method_entries);
    }

    /// Check an impl block: verify the trait exists, check method conformance, register.
    fn check_impl_block(&mut self, impl_block: &ImplBlock) {
        let trait_name = impl_block.trait_name.name.clone();
        let type_name = impl_block.type_name.name.clone();

        // 1. Verify trait exists
        if !self.trait_registry.trait_exists(&trait_name) {
            self.diagnostics.push(Diagnostic::error_with_code(
                error_codes::TRAIT_NOT_FOUND,
                format!("Trait '{}' is not defined", trait_name),
                impl_block.trait_name.span,
            ));
            return;
        }

        // 2. Check for duplicate impl
        if self.impl_registry.has_impl(&type_name, &trait_name) {
            self.diagnostics.push(Diagnostic::error_with_code(
                error_codes::IMPL_ALREADY_EXISTS,
                format!("'{}' already implements '{}'", type_name, trait_name),
                impl_block.span,
            ));
            return;
        }

        // 3. Get required methods from trait
        let required_methods: Vec<TraitMethodEntry> = self
            .trait_registry
            .get_methods(&trait_name)
            .cloned()
            .unwrap_or_default();

        // 4. Build a map of provided methods
        let provided: HashMap<String, &ImplMethod> = impl_block
            .methods
            .iter()
            .map(|m| (m.name.name.clone(), m))
            .collect();

        // 5. Check all required methods are provided with matching signatures
        let mut all_ok = true;
        for required in &required_methods {
            match provided.get(&required.name) {
                None => {
                    self.diagnostics.push(Diagnostic::error_with_code(
                        error_codes::IMPL_METHOD_MISSING,
                        format!(
                            "Impl of '{}' for '{}' is missing required method '{}'",
                            trait_name, type_name, required.name
                        ),
                        impl_block.span,
                    ));
                    all_ok = false;
                }
                Some(impl_method) => {
                    // Skip `self` parameter on both sides: trait declares `self: TraitName`
                    // but impl naturally uses the implementing type. Both sides exclude `self`
                    // for comparison so parameter names beyond `self` are checked cleanly.
                    let impl_param_types: Vec<Type> = impl_method
                        .params
                        .iter()
                        .filter(|p| p.name.name != "self")
                        .map(|p| self.resolve_type_ref(&p.type_ref))
                        .collect();

                    if impl_param_types != required.param_types {
                        self.diagnostics.push(Diagnostic::error_with_code(
                            error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
                            format!(
                                "Method '{}' in impl of '{}' for '{}' has wrong parameter types",
                                required.name, trait_name, type_name
                            ),
                            impl_method.span,
                        ));
                        all_ok = false;
                    }

                    let impl_return = self.resolve_type_ref(&impl_method.return_type);
                    if impl_return != required.return_type {
                        self.diagnostics.push(Diagnostic::error_with_code(
                            error_codes::IMPL_METHOD_SIGNATURE_MISMATCH,
                            format!(
                                "Method '{}' in impl of '{}' for '{}' has wrong return type",
                                required.name, trait_name, type_name
                            ),
                            impl_method.span,
                        ));
                        all_ok = false;
                    }
                }
            }
        }

        // 6. Typecheck method bodies
        for impl_method in &impl_block.methods {
            self.check_impl_method_body(impl_method);
        }

        // 7. Register impl if conformance passed
        if all_ok {
            let method_map: HashMap<String, ImplMethod> = impl_block
                .methods
                .iter()
                .map(|m| (m.name.name.clone(), m.clone()))
                .collect();
            self.impl_registry
                .register(&type_name, &trait_name, method_map);
            self.trait_registry.mark_implements(&type_name, &trait_name);
        }
    }

    /// Typecheck an impl method body, using the same pattern as `check_function`.
    fn check_impl_method_body(&mut self, method: &ImplMethod) {
        let prev_return_type = self.current_function_return_type.clone();
        let prev_function_info = self.current_function_info.clone();
        let prev_declared_symbols = std::mem::take(&mut self.declared_symbols);
        let prev_used_symbols = std::mem::take(&mut self.used_symbols);
        let prev_param_ownerships = std::mem::take(&mut self.current_fn_param_ownerships);

        self.current_function_return_type = Some(self.resolve_type_ref(&method.return_type));
        self.current_function_info = Some((method.name.name.clone(), method.span));

        self.enter_scope();

        for param in &method.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            let symbol = crate::symbol::Symbol {
                name: param.name.name.clone(),
                ty,
                mutable: false,
                kind: SymbolKind::Parameter,
                span: param.name.span,
                exported: false,
            };
            let _ = self.symbol_table.define(symbol);
            self.declared_symbols.insert(
                param.name.name.clone(),
                (param.name.span, SymbolKind::Parameter),
            );
        }

        for stmt in &method.body.statements {
            self.check_statement(stmt);
        }

        self.exit_scope();

        self.current_function_return_type = prev_return_type;
        self.current_function_info = prev_function_info;
        self.declared_symbols = prev_declared_symbols;
        self.used_symbols = prev_used_symbols;
        self.current_fn_param_ownerships = prev_param_ownerships;
    }

    fn check_function(&mut self, func: &FunctionDecl) {
        self.validate_type_param_bounds(&func.type_params);
        // Save previous function context (for nested functions)
        let prev_return_type = self.current_function_return_type.clone();
        let prev_function_info = self.current_function_info.clone();
        let prev_declared_symbols = std::mem::take(&mut self.declared_symbols);
        let prev_used_symbols = std::mem::take(&mut self.used_symbols);
        let prev_param_ownerships = std::mem::take(&mut self.current_fn_param_ownerships);

        let return_type = self.resolve_type_ref(&func.return_type);
        self.current_function_return_type = Some(return_type.clone());
        self.current_function_info = Some((func.name.name.clone(), func.name.span));

        // Clear tracking for this function
        self.declared_symbols.clear();
        self.used_symbols.clear();

        // Enter function scope and define parameters
        self.enter_scope();

        for param in &func.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            let symbol = crate::symbol::Symbol {
                name: param.name.name.clone(),
                ty,
                mutable: false,
                kind: SymbolKind::Parameter,
                span: param.name.span,
                exported: false,
            };
            // Define parameter in symbol table for type checking
            let _ = self.symbol_table.define(symbol);

            // Also track for unused warnings
            self.declared_symbols.insert(
                param.name.name.clone(),
                (param.name.span, SymbolKind::Parameter),
            );
        }

        // Validate ownership annotations and populate the ownership registry.
        let mut param_ownerships: Vec<Option<OwnershipAnnotation>> =
            Vec::with_capacity(func.params.len());
        for param in &func.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            if let Some(ann) = &param.ownership {
                match ann {
                    OwnershipAnnotation::Own => {
                        if matches!(ty, Type::Number | Type::Bool | Type::String) {
                            self.diagnostics.push(
                                Diagnostic::warning_with_code(
                                    error_codes::OWN_ON_PRIMITIVE,
                                    format!(
                                        "`own` annotation on parameter `{}` has no effect: \
                                         primitive types are always copied",
                                        param.name.name
                                    ),
                                    param.name.span,
                                )
                                .with_help(
                                    "remove the `own` annotation from this primitive parameter",
                                ),
                            );
                        }
                    }
                    OwnershipAnnotation::Borrow => {
                        if matches!(&ty, Type::Generic { name, .. } if name == "shared") {
                            self.diagnostics.push(
                                Diagnostic::warning_with_code(
                                    error_codes::BORROW_ON_SHARED,
                                    format!(
                                        "`borrow` annotation on parameter `{}` is redundant: \
                                         `shared<T>` already has reference semantics",
                                        param.name.name
                                    ),
                                    param.name.span,
                                )
                                .with_help("remove the `borrow` annotation from this `shared<T>` parameter"),
                            );
                        }
                    }
                    OwnershipAnnotation::Shared => {}
                }
            }
            param_ownerships.push(param.ownership.clone());
        }
        // Populate current_fn_param_ownerships for call-site checking inside this body.
        self.current_fn_param_ownerships = func
            .params
            .iter()
            .map(|p| (p.name.name.clone(), p.ownership.clone()))
            .collect();

        self.fn_ownership_registry.insert(
            func.name.name.clone(),
            (param_ownerships, func.return_ownership.clone()),
        );

        // Hoist nested function signatures (like the binder does)
        // This ensures nested functions are available when type-checking their calls
        for stmt in &func.body.statements {
            if let Stmt::FunctionDecl(nested_func) = stmt {
                self.hoist_nested_function_signature(nested_func);
            }
        }

        // Register nested type guards before checking the function body
        self.register_type_guards_in_block(&func.body);

        self.check_block(&func.body);

        // Check if all paths return (if return type != void/null)
        let return_norm = return_type.normalized();
        if return_norm != Type::Void
            && return_norm != Type::Null
            && !self.block_always_returns(&func.body)
        {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3004",
                    "Not all code paths return a value",
                    func.span,
                )
                .with_label("function body")
                .with_help(format!(
                    "ensure all code paths return a value of type {}",
                    return_type.display_name()
                )),
            );
        }

        // Emit warnings for unused variables/parameters
        self.emit_unused_warnings();

        // Exit function scope
        self.exit_scope();

        // Restore previous function context (for nested functions)
        self.current_function_return_type = prev_return_type;
        self.current_function_info = prev_function_info;
        self.declared_symbols = prev_declared_symbols;
        self.used_symbols = prev_used_symbols;
        self.current_fn_param_ownerships = prev_param_ownerships;
    }

    /// Emit warnings for unused symbols
    fn emit_unused_warnings(&mut self) {
        for (name, (span, kind)) in &self.declared_symbols {
            // Skip if symbol starts with underscore (suppression)
            if name.starts_with('_') {
                continue;
            }

            // Skip if used
            if self.used_symbols.contains(name) {
                continue;
            }

            // Emit warning based on symbol kind
            let message = match kind {
                SymbolKind::Variable => format!("Unused variable '{}'", name),
                SymbolKind::Parameter => format!("Unused parameter '{}'", name),
                _ => continue,
            };

            self.diagnostics.push(
                Diagnostic::warning_with_code("AT2001", &message, *span)
                    .with_label("declared here but never used")
                    .with_help(format!(
                        "remove the {} or prefix with underscore: _{}",
                        match kind {
                            SymbolKind::Variable => "variable",
                            SymbolKind::Parameter => "parameter",
                            _ => "symbol",
                        },
                        name
                    )),
            );
        }
    }

    /// Check a block
    fn check_block(&mut self, block: &Block) {
        let mut found_return = false;
        for stmt in &block.statements {
            if found_return {
                // Code after return is unreachable
                self.diagnostics.push(
                    Diagnostic::warning_with_code("AT2002", "Unreachable code", stmt.span())
                        .with_label("this code will never execute")
                        .with_help("remove this code or restructure your control flow"),
                );
            }

            self.check_statement(stmt);

            // Check if this statement always returns
            if matches!(stmt, Stmt::Return(_)) {
                found_return = true;
            }
        }
    }

    /// Check if a block always returns
    fn block_always_returns(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if self.statement_always_returns(stmt) {
                return true;
            }
        }
        false
    }

    /// Check if a statement always returns
    fn statement_always_returns(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) => true,
            Stmt::If(if_stmt) => {
                if let Some(else_block) = &if_stmt.else_block {
                    self.block_always_returns(&if_stmt.then_block)
                        && self.block_always_returns(else_block)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check a statement
    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                // Track this variable declaration
                self.declared_symbols
                    .insert(var.name.name.clone(), (var.name.span, SymbolKind::Variable));

                let init_type = self.check_expr(&var.init);

                // Determine the final type for this variable
                let final_type = if let Some(type_ref) = &var.type_ref {
                    let declared_type =
                        self.resolve_type_ref_with_context(type_ref, Some(&init_type));
                    if !init_type.is_assignable_to(&declared_type) {
                        let help = suggestions::suggest_type_mismatch(&declared_type, &init_type)
                            .unwrap_or_else(|| {
                                format!(
                                    "expected {}, found {}",
                                    declared_type.display_name(),
                                    init_type.display_name()
                                )
                            });
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type mismatch: expected {}, found {}",
                                    declared_type.display_name(),
                                    init_type.display_name()
                                ),
                                var.span,
                            )
                            .with_label(format!(
                                "expected {}, found {}",
                                declared_type.display_name(),
                                init_type.display_name()
                            ))
                            .with_help(help),
                        );
                    }
                    declared_type
                } else {
                    // No explicit type annotation - use inferred type
                    init_type
                };

                // Update the symbol's type in the symbol table.
                // The binder already defined the symbol but may have set Unknown type
                // if there was no type annotation. We now have the inferred/declared type.
                if let Some(symbol) = self.symbol_table.lookup_mut(&var.name.name) {
                    symbol.ty = final_type;
                } else {
                    // Symbol doesn't exist - this can happen for variables declared in
                    // inner scopes (the binder exited those scopes, removing the symbols).
                    // Define the symbol in the current scope.
                    let symbol = crate::symbol::Symbol {
                        name: var.name.name.clone(),
                        ty: final_type,
                        span: var.name.span,
                        mutable: var.mutable,
                        kind: crate::symbol::SymbolKind::Variable,
                        exported: false,
                    };
                    // Ignore redefinition errors - the binder already validated this
                    let _ = self.symbol_table.define(symbol);
                }
            }
            Stmt::Assign(assign) => {
                let value_type = self.check_expr(&assign.value);
                let target_type = self.check_assign_target(&assign.target);

                if !value_type.is_assignable_to(&target_type) {
                    let help = suggestions::suggest_type_mismatch(&target_type, &value_type)
                        .unwrap_or_else(|| {
                            format!(
                                "expected {}, found {}",
                                target_type.display_name(),
                                value_type.display_name()
                            )
                        });
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Type mismatch in assignment: expected {}, found {}",
                                target_type.display_name(),
                                value_type.display_name()
                            ),
                            assign.span,
                        )
                        .with_label(format!(
                            "expected {}, found {}",
                            target_type.display_name(),
                            value_type.display_name()
                        ))
                        .with_help(help),
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &assign.target {
                    if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                        if !symbol.mutable {
                            let diag = Diagnostic::error_with_code(
                                "AT3003",
                                format!("Cannot assign to immutable variable '{}'", id.name),
                                id.span,
                            )
                            .with_label("immutable variable")
                            .with_related_location(crate::diagnostic::RelatedLocation {
                                file: "<input>".to_string(),
                                line: 1,
                                column: symbol.span.start + 1,
                                length: symbol.span.end.saturating_sub(symbol.span.start),
                                message: format!("'{}' declared here as immutable", symbol.name),
                            })
                            .with_help(suggestions::suggest_mutability_fix(&id.name));

                            self.diagnostics.push(diag);
                        }
                    }
                }
            }
            Stmt::CompoundAssign(compound) => {
                let value_type = self.check_expr(&compound.value);
                let target_type = self.check_assign_target(&compound.target);
                let target_norm = target_type.normalized();
                let value_norm = value_type.normalized();

                // Compound assignment requires both sides to be numbers (allow Unknown for error recovery)
                if !matches!(target_norm, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Compound assignment requires number type, found {}",
                                target_type.display_name()
                            ),
                            compound.span,
                        )
                        .with_label("type mismatch")
                        .with_help(
                            "compound assignment operators (+=, -=, etc.) only work with numbers",
                        ),
                    );
                }

                if !matches!(value_norm, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Compound assignment requires number value, found {}",
                                value_type.display_name()
                            ),
                            compound.span,
                        )
                        .with_label("type mismatch")
                        .with_help("the value must be a number for compound assignment"),
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &compound.target {
                    if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                        if !symbol.mutable {
                            let diag = Diagnostic::error_with_code(
                                "AT3003",
                                format!("Cannot modify immutable variable '{}'", id.name),
                                id.span,
                            )
                            .with_label("immutable variable");
                            self.diagnostics.push(diag);
                        }
                    }
                }
            }
            Stmt::Increment(inc) => {
                let target_type = self.check_assign_target(&inc.target);
                let target_norm = target_type.normalized();

                // Increment requires number type (allow Unknown for error recovery)
                if !matches!(target_norm, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Increment requires number type, found {}",
                                target_type.display_name()
                            ),
                            inc.span,
                        )
                        .with_label("type mismatch"),
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &inc.target {
                    if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                        if !symbol.mutable {
                            let diag = Diagnostic::error_with_code(
                                "AT3003",
                                format!("Cannot modify immutable variable '{}'", id.name),
                                id.span,
                            )
                            .with_label("immutable variable");
                            self.diagnostics.push(diag);
                        }
                    }
                }
            }
            Stmt::Decrement(dec) => {
                let target_type = self.check_assign_target(&dec.target);
                let target_norm = target_type.normalized();

                // Decrement requires number type (allow Unknown for error recovery)
                if !matches!(target_norm, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Decrement requires number type, found {}",
                                target_type.display_name()
                            ),
                            dec.span,
                        )
                        .with_label("type mismatch"),
                    );
                }

                // Check mutability
                if let AssignTarget::Name(id) = &dec.target {
                    if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                        if !symbol.mutable {
                            let diag = Diagnostic::error_with_code(
                                "AT3003",
                                format!("Cannot modify immutable variable '{}'", id.name),
                                id.span,
                            )
                            .with_label("immutable variable");
                            self.diagnostics.push(diag);
                        }
                    }
                }
            }
            Stmt::If(if_stmt) => {
                let cond_type = self.check_expr(&if_stmt.cond);
                let cond_norm = cond_type.normalized();
                if cond_norm != Type::Bool && cond_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            if_stmt.cond.span(),
                        )
                        .with_label(format!("expected bool, found {}", cond_type.display_name()))
                        .with_help(suggestions::suggest_condition_fix(&cond_type)),
                    );
                }
                let (then_narrow, else_narrow) = self.narrow_condition(&if_stmt.cond);
                self.enter_scope();
                self.apply_narrowings(&then_narrow);
                self.check_block(&if_stmt.then_block);
                self.exit_scope();
                if let Some(else_block) = &if_stmt.else_block {
                    self.enter_scope();
                    self.apply_narrowings(&else_narrow);
                    self.check_block(else_block);
                    self.exit_scope();
                }
            }
            Stmt::While(while_stmt) => {
                let cond_type = self.check_expr(&while_stmt.cond);
                let cond_norm = cond_type.normalized();
                if cond_norm != Type::Bool && cond_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            while_stmt.cond.span(),
                        )
                        .with_label(format!("expected bool, found {}", cond_type.display_name()))
                        .with_help(suggestions::suggest_condition_fix(&cond_type)),
                    );
                }
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                let (then_narrow, _) = self.narrow_condition(&while_stmt.cond);
                self.enter_scope();
                self.apply_narrowings(&then_narrow);
                self.check_block(&while_stmt.body);
                self.exit_scope();
                self.in_loop = old_in_loop;
            }
            Stmt::For(for_stmt) => {
                self.check_statement(&for_stmt.init);
                let cond_type = self.check_expr(&for_stmt.cond);
                let cond_norm = cond_type.normalized();
                if cond_norm != Type::Bool && cond_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            for_stmt.cond.span(),
                        )
                        .with_label(format!("expected bool, found {}", cond_type.display_name()))
                        .with_help(suggestions::suggest_condition_fix(&cond_type)),
                    );
                }
                self.check_statement(&for_stmt.step);

                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&for_stmt.body);
                self.in_loop = old_in_loop;
            }
            Stmt::Return(ret) => {
                if self.current_function_return_type.is_none() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3011",
                            "Return statement outside function",
                            ret.span,
                        )
                        .with_label("invalid return"),
                    );
                    return;
                }

                let return_type = if let Some(value) = &ret.value {
                    self.check_expr(value)
                } else {
                    Type::Void
                };

                let expected = self.current_function_return_type.as_ref().unwrap();
                if !return_type.is_assignable_to(expected) {
                    let mut diag = Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Return type mismatch: expected {}, found {}",
                            expected.display_name(),
                            return_type.display_name()
                        ),
                        ret.span,
                    )
                    .with_label(format!(
                        "expected {}, found {}",
                        expected.display_name(),
                        return_type.display_name()
                    ))
                    .with_help(suggestions::suggest_return_fix(expected, &return_type));

                    // Add related location for function declaration
                    if let Some((func_name, func_span)) = &self.current_function_info {
                        diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                            file: "<input>".to_string(),
                            line: 1,
                            column: func_span.start + 1,
                            length: func_span.end.saturating_sub(func_span.start),
                            message: format!("function '{}' declared here", func_name),
                        });
                    }

                    self.diagnostics.push(diag);
                }
            }
            Stmt::Break(span) => {
                if !self.in_loop {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            "Break statement outside loop",
                            *span,
                        )
                        .with_label("invalid break"),
                    );
                }
            }
            Stmt::Continue(span) => {
                if !self.in_loop {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3010",
                            "Continue statement outside loop",
                            *span,
                        )
                        .with_label("invalid continue"),
                    );
                }
            }
            Stmt::Expr(expr_stmt) => {
                let expr_type = self.check_expr(&expr_stmt.expr);
                self.last_expr_type = Some(expr_type);
            }
            Stmt::FunctionDecl(func) => {
                // Nested function declaration - type check it
                // Uses same check_function logic as top-level functions
                self.check_function(func);
            }
            Stmt::ForIn(for_in_stmt) => {
                // Type check the iterable expression
                let iterable_type = self.check_expr(&for_in_stmt.iterable);
                let iterable_norm = iterable_type.normalized();

                // Validate iterable is an array
                // Note: Unknown types are allowed for now (will be inferred)
                match iterable_norm {
                    Type::Array(_) | Type::Unknown => {
                        // Valid - continue
                    }
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "for-in requires an array, found {}",
                                    iterable_type.display_name()
                                ),
                                for_in_stmt.iterable.span(),
                            )
                            .with_label(format!(
                                "expected array, found {}",
                                iterable_type.display_name()
                            ))
                            .with_help(suggestions::suggest_for_in_fix(&iterable_type)),
                        );
                    }
                }

                // Infer loop variable type from array element type
                if let Type::Array(element_type) = &iterable_norm {
                    // Update symbol table with inferred type
                    if let Some(symbol) = self.symbol_table.lookup_mut(&for_in_stmt.variable.name) {
                        symbol.ty = (**element_type).clone();
                    }
                }

                // Type check the loop body
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&for_in_stmt.body);
                self.in_loop = old_in_loop;
            }
        }
    }

    fn apply_narrowings(&mut self, narrowings: &HashMap<String, Type>) {
        for (name, ty) in narrowings {
            let Some(symbol) = self.symbol_table.lookup(name) else {
                continue;
            };
            let shadow = crate::symbol::Symbol {
                name: symbol.name.clone(),
                ty: ty.clone(),
                mutable: symbol.mutable,
                kind: symbol.kind.clone(),
                span: symbol.span,
                exported: symbol.exported,
            };
            let _ = self.symbol_table.define(shadow);
        }
    }

    fn enter_scope(&mut self) {
        self.symbol_table.enter_scope();
        self.type_guards.enter_scope();
    }

    fn exit_scope(&mut self) {
        self.symbol_table.exit_scope();
        self.type_guards.exit_scope();
    }

    /// Check an assignment target and return its type
    fn check_assign_target(&mut self, target: &AssignTarget) -> Type {
        match target {
            AssignTarget::Name(id) => {
                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    symbol.ty.clone()
                } else {
                    Type::Unknown
                }
            }
            AssignTarget::Index { target, index, .. } => {
                let target_type = self.check_expr(target);
                let index_type = self.check_expr(index);
                let target_norm = target_type.normalized();

                // Check that index is a number
                let index_norm = index_type.normalized();
                if index_norm != Type::Number && index_norm != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Array index must be number, found {}",
                                index_type.display_name()
                            ),
                            index.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }

                // Extract element type from array
                match target_norm {
                    Type::Array(elem_type) => *elem_type,
                    Type::Unknown => Type::Unknown,
                    _ => {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Cannot index into non-array type {}",
                                    target_type.display_name()
                                ),
                                target.span(),
                            )
                            .with_label("not an array"),
                        );
                        Type::Unknown
                    }
                }
            }
        }
    }

    /// Get the expected arity for a built-in generic type
    fn get_generic_type_arity(&self, name: &str) -> Option<usize> {
        match name {
            "Option" => Some(1),
            "Result" => Some(2),
            "Array" => Some(1), // Array<T> is sugar for T[]
            "HashMap" => Some(2),
            "HashSet" => Some(1),
            _ => None, // Unknown generic type
        }
    }

    /// Returns `true` if the given type implements `Copy` (value semantics).
    /// Built-in value types are always Copy. User types are Copy only if they have
    /// an explicit `impl Copy for T { }` registered in the trait registry.
    pub fn is_copy_type(&self, ty: &Type) -> bool {
        match ty.normalized() {
            // All built-in value types are Copy
            Type::Number | Type::String | Type::Bool | Type::Null | Type::Void => true,
            Type::Array(_) | Type::JsonValue => true,
            // Function types are Copy (reference-counted internally)
            Type::Function { .. } => true,
            // Generic types: Copy if explicitly registered (e.g. shared<T> is NOT Copy)
            Type::Generic { name, type_args: _ } => self.trait_registry.implements(&name, "Copy"),
            // Type parameters: conservative — not Copy unless registry says so
            Type::TypeParameter { name } => self.trait_registry.implements(&name, "Copy"),
            _ => false,
        }
    }

    /// Returns `true` if the given type requires explicit ownership transfer (Move semantics).
    /// In Block 3: a type is Move if it is not Copy.
    pub fn is_move_type(&self, ty: &Type) -> bool {
        !self.is_copy_type(ty)
    }

    /// Try to resolve a method call through the trait/impl system.
    /// Returns the return type if a matching impl method is found, `None` otherwise.
    /// Only fires after stdlib method dispatch has failed (dispatch priority slot 2).
    /// Like `resolve_trait_method_call_with_info` but discards dispatch info.
    pub(super) fn resolve_trait_method_call_with_info(
        &mut self,
        receiver_type: &Type,
        method_name: &str,
    ) -> Option<(Type, String, String)> {
        let type_name = type_to_impl_key(receiver_type)?;
        // Search all impls for this type — return method + the trait that provides it
        let matching: Option<(ImplMethod, String)> = self
            .impl_registry
            .entries
            .iter()
            .filter(|((impl_type, _), _)| impl_type == &type_name)
            .find_map(|((_, trait_name), entry)| {
                entry
                    .methods
                    .get(method_name)
                    .cloned()
                    .map(|m| (m, trait_name.clone()))
            });

        if let Some((method, trait_name)) = matching {
            let return_type = self.resolve_type_ref(&method.return_type.clone());
            Some((return_type, type_name, trait_name))
        } else {
            None
        }
    }

    /// Resolve a type reference to a Type
    pub(super) fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Type {
        self.resolve_type_ref_with_context(type_ref, None)
    }

    fn resolve_type_ref_with_context(
        &mut self,
        type_ref: &TypeRef,
        expected: Option<&Type>,
    ) -> Type {
        match type_ref {
            TypeRef::Named(name, span) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                "json" => Type::JsonValue,
                "array" => Type::Array(Box::new(Type::Unknown)),
                "Comparable" | "Numeric" => Type::Number,
                "Iterable" => Type::Array(Box::new(Type::Unknown)),
                "Equatable" => {
                    Type::union(vec![Type::Number, Type::String, Type::Bool, Type::Null])
                }
                "Serializable" => Type::union(vec![
                    Type::Number,
                    Type::String,
                    Type::Bool,
                    Type::Null,
                    Type::JsonValue,
                ]),
                _ => {
                    if let Some(alias) = self.type_aliases.get(name).cloned() {
                        if alias.type_params.is_empty() {
                            return self.resolve_type_alias(&alias, Vec::new(), *span);
                        }
                        if let Some(expected_type) = expected {
                            if expected_type.normalized() != Type::Unknown {
                                if let Some(args) =
                                    self.infer_alias_type_args(&alias, expected_type, *span)
                                {
                                    return self.resolve_type_alias(&alias, args, *span);
                                }
                            }
                        }

                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type alias '{}' expects {} type argument(s)",
                                    name,
                                    alias.type_params.len()
                                ),
                                *span,
                            )
                            .with_label("missing type arguments")
                            .with_help(format!(
                                "provide {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }

                    Type::Unknown
                }
            },
            TypeRef::Array(elem, _) => {
                Type::Array(Box::new(self.resolve_type_ref_with_context(elem, None)))
            }
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params
                    .iter()
                    .map(|p| self.resolve_type_ref_with_context(p, None))
                    .collect();
                let ret_type = Box::new(self.resolve_type_ref_with_context(return_type, None));
                Type::Function {
                    type_params: vec![],
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Structural { members, .. } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| crate::types::StructuralMemberType {
                        name: member.name.clone(),
                        ty: self.resolve_type_ref_with_context(&member.type_ref, None),
                    })
                    .collect(),
            },
            TypeRef::Union { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_context(m, None))
                    .collect();
                Type::union(resolved)
            }
            TypeRef::Intersection { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_context(m, None))
                    .collect();
                Type::intersection(resolved)
            }
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                let resolved_args = type_args
                    .iter()
                    .map(|arg| self.resolve_type_ref_with_context(arg, None))
                    .collect::<Vec<_>>();

                if let Some(alias) = self.type_aliases.get(name).cloned() {
                    if alias.type_params.len() != resolved_args.len() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type alias '{}' expects {} type argument(s), found {}",
                                    name,
                                    alias.type_params.len(),
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments")
                            .with_help(format!(
                                "provide exactly {} type argument(s) for '{}'",
                                alias.type_params.len(),
                                name
                            )),
                        );
                        return Type::Unknown;
                    }
                    return self.resolve_type_alias(&alias, resolved_args, *span);
                }

                // Validate built-in generic type arity
                let expected_arity = self.get_generic_type_arity(name);

                if let Some(arity) = expected_arity {
                    if resolved_args.len() != arity {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Generic type '{}' expects {} type argument(s), found {}",
                                    name,
                                    arity,
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments"),
                        );
                        return Type::Unknown;
                    }
                } else {
                    // Unknown generic type
                    self.diagnostics.push(
                        Diagnostic::error(format!("Unknown generic type '{}'", name), *span)
                            .with_label("unknown type"),
                    );
                    return Type::Unknown;
                }

                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
        }
    }

    fn resolve_type_alias(
        &mut self,
        alias: &TypeAliasDecl,
        type_args: Vec<Type>,
        span: Span,
    ) -> Type {
        let alias_name = alias.name.name.clone();
        let key = AliasKey {
            name: alias_name.clone(),
            type_args: type_args.clone(),
        };
        if let Some(cached) = self.alias_cache.get(&key) {
            return cached.clone();
        }

        if let Some(index) = self
            .alias_resolution_stack
            .iter()
            .position(|name| name == &alias_name)
        {
            let mut chain = self.alias_resolution_stack[index..].to_vec();
            chain.push(alias_name.clone());
            let mut diag = Diagnostic::error_with_code(
                "AT3001",
                format!("Circular type alias detected: {}", chain.join(" -> ")),
                span,
            )
            .with_label("circular type alias");
            diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                file: "<input>".to_string(),
                line: 1,
                column: alias.name.span.start + 1,
                length: alias.name.span.end.saturating_sub(alias.name.span.start),
                message: format!("'{}' declared here", alias.name.name),
            });
            self.diagnostics.push(diag);
            return Type::Unknown;
        }

        self.maybe_warn_deprecated_alias(alias, span);

        let substitutions = alias
            .type_params
            .iter()
            .map(|param| param.name.clone())
            .zip(type_args.iter().cloned())
            .collect::<HashMap<_, _>>();

        self.alias_resolution_stack.push(alias_name.clone());
        let resolved_target =
            self.resolve_type_ref_with_substitutions(&alias.type_ref, &substitutions);
        self.alias_resolution_stack.pop();

        let alias_type = Type::Alias {
            name: alias_name,
            type_args,
            target: Box::new(resolved_target),
        };

        self.alias_cache.insert(key, alias_type.clone());
        alias_type
    }

    fn resolve_type_ref_with_substitutions(
        &mut self,
        type_ref: &TypeRef,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match type_ref {
            TypeRef::Named(name, _span) => {
                if let Some(sub) = substitutions.get(name) {
                    return sub.clone();
                }
                self.resolve_type_ref_with_context(type_ref, Some(&Type::Unknown))
            }
            TypeRef::Array(elem, _) => Type::Array(Box::new(
                self.resolve_type_ref_with_substitutions(elem, substitutions),
            )),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params
                    .iter()
                    .map(|p| self.resolve_type_ref_with_substitutions(p, substitutions))
                    .collect();
                let ret_type =
                    Box::new(self.resolve_type_ref_with_substitutions(return_type, substitutions));
                Type::Function {
                    type_params: vec![],
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Structural { members, .. } => Type::Structural {
                members: members
                    .iter()
                    .map(|member| crate::types::StructuralMemberType {
                        name: member.name.clone(),
                        ty: self
                            .resolve_type_ref_with_substitutions(&member.type_ref, substitutions),
                    })
                    .collect(),
            },
            TypeRef::Union { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_substitutions(m, substitutions))
                    .collect();
                Type::union(resolved)
            }
            TypeRef::Intersection { members, .. } => {
                let resolved = members
                    .iter()
                    .map(|m| self.resolve_type_ref_with_substitutions(m, substitutions))
                    .collect();
                Type::intersection(resolved)
            }
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                let resolved_args = type_args
                    .iter()
                    .map(|arg| self.resolve_type_ref_with_substitutions(arg, substitutions))
                    .collect::<Vec<_>>();

                if let Some(alias) = self.type_aliases.get(name).cloned() {
                    if alias.type_params.len() != resolved_args.len() {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type alias '{}' expects {} type argument(s), found {}",
                                    name,
                                    alias.type_params.len(),
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments"),
                        );
                        return Type::Unknown;
                    }
                    return self.resolve_type_alias(&alias, resolved_args, *span);
                }

                let expected_arity = self.get_generic_type_arity(name);
                if let Some(arity) = expected_arity {
                    if resolved_args.len() != arity {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Generic type '{}' expects {} type argument(s), found {}",
                                    name,
                                    arity,
                                    resolved_args.len()
                                ),
                                *span,
                            )
                            .with_label("incorrect number of type arguments"),
                        );
                        return Type::Unknown;
                    }
                } else {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Unknown generic type '{}'", name),
                            *span,
                        )
                        .with_label("unknown type"),
                    );
                    return Type::Unknown;
                }

                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
        }
    }

    fn infer_alias_type_args(
        &mut self,
        alias: &TypeAliasDecl,
        expected: &Type,
        span: Span,
    ) -> Option<Vec<Type>> {
        let substitutions = alias
            .type_params
            .iter()
            .map(|param| param.name.clone())
            .zip(alias.type_params.iter().map(|param| Type::TypeParameter {
                name: param.name.clone(),
            }))
            .collect::<HashMap<_, _>>();

        let target = self.resolve_type_ref_with_substitutions(&alias.type_ref, &substitutions);

        let mut inferer = generics::TypeInferer::new();
        if inferer.unify(&target, expected).is_err() {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3001",
                    format!(
                        "Cannot infer type arguments for alias '{}' from {}",
                        alias.name.name,
                        expected.display_name()
                    ),
                    span,
                )
                .with_label("cannot infer type arguments"),
            );
            return None;
        }

        let mut resolved_args = Vec::new();
        for param in &alias.type_params {
            if let Some(arg) = inferer.get_substitution(&param.name) {
                resolved_args.push(arg.clone());
            } else {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Cannot infer type argument '{}' for alias '{}'",
                            param.name, alias.name.name
                        ),
                        span,
                    )
                    .with_label("cannot infer type argument"),
                );
                return None;
            }
        }

        Some(resolved_args)
    }

    fn parse_alias_metadata(&self, alias: &TypeAliasDecl) -> AliasMetadata {
        let mut metadata = AliasMetadata {
            deprecated: false,
            since: None,
        };
        let Some(doc) = alias.doc_comment.as_ref() else {
            return metadata;
        };

        for line in doc.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();
            if lower.contains("@deprecated") || lower.starts_with("deprecated") {
                metadata.deprecated = true;
            }
            if let Some(rest) = trimmed.strip_prefix("@since") {
                let version = rest.trim();
                if !version.is_empty() {
                    metadata.since = Some(version.to_string());
                }
            }
        }

        metadata
    }

    fn maybe_warn_deprecated_alias(&mut self, alias: &TypeAliasDecl, span: Span) {
        let metadata = self.parse_alias_metadata(alias);
        if metadata.deprecated {
            let mut diag = Diagnostic::warning_with_code(
                "AT2009",
                format!("Type alias '{}' is deprecated", alias.name.name),
                span,
            )
            .with_label("deprecated type alias");

            if let Some(since) = metadata.since {
                diag = diag.with_note(format!("deprecated since {}", since));
            }

            self.diagnostics.push(diag);
        }
    }

    fn validate_type_aliases(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::TypeAlias(alias) => {
                    self.validate_type_alias(alias);
                }
                Item::Export(export_decl) => {
                    if let ExportItem::TypeAlias(alias) = &export_decl.item {
                        self.validate_type_alias(alias);
                    }
                }
                _ => {}
            }
        }
    }

    fn validate_type_alias(&mut self, alias: &TypeAliasDecl) {
        self.validate_type_param_bounds(&alias.type_params);
        let type_args = alias
            .type_params
            .iter()
            .map(|param| Type::TypeParameter {
                name: param.name.clone(),
            })
            .collect::<Vec<_>>();
        let _ = self.resolve_type_alias(alias, type_args, alias.span);
    }

    fn check_constraints(
        &mut self,
        type_params: &[TypeParamDef],
        inferer: &generics::TypeInferer,
        span: Span,
    ) -> bool {
        let mut ok = constraints::check_constraints(
            type_params,
            inferer,
            &self.method_table,
            &mut self.diagnostics,
            span,
        );

        // Also check trait bounds (`T: Copy + UserTrait`) from Phase 05/10
        for param in type_params {
            if param.trait_bounds.is_empty() {
                continue;
            }
            let Some(actual) = inferer.get_substitution(&param.name) else {
                continue;
            };
            for trait_name in &param.trait_bounds {
                if !self.type_satisfies_trait_bound(actual, trait_name) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            error_codes::TRAIT_BOUND_NOT_SATISFIED,
                            format!(
                                "Type '{}' does not implement trait '{}' required by type \
                                 parameter '{}'",
                                actual.display_name(),
                                trait_name,
                                param.name
                            ),
                            span,
                        )
                        .with_label("trait bound not satisfied")
                        .with_help(format!(
                            "implement '{}' for '{}' using `impl {} for {} {{ ... }}`",
                            trait_name,
                            actual.display_name(),
                            trait_name,
                            actual.display_name()
                        )),
                    );
                    ok = false;
                }
            }
        }

        ok
    }

    /// Determine if a resolved type satisfies a trait bound by name.
    fn type_satisfies_trait_bound(&self, ty: &Type, trait_name: &str) -> bool {
        match trait_name {
            "Copy" => self.is_copy_type(ty),
            "Move" => self.is_move_type(ty),
            _ => {
                // Built-in or user-defined trait — check impl registry
                if let Some(type_name) = type_to_impl_key(ty) {
                    self.trait_registry.implements(&type_name, trait_name)
                } else {
                    false
                }
            }
        }
    }

    fn validate_type_param_bounds(&mut self, type_params: &[crate::ast::TypeParam]) {
        for param in type_params {
            let Some(bound) = &param.bound else {
                continue;
            };

            let resolved = self.resolve_type_ref_with_params_and_context(bound, type_params, None);
            if self.contains_type_param(&resolved, &param.name) {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Type parameter '{}' cannot be constrained by itself",
                            param.name
                        ),
                        param.span,
                    )
                    .with_label("circular constraint")
                    .with_help("remove the self-referential constraint"),
                );
            }

            if resolved.normalized() == Type::Unknown {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Unknown constraint type for type parameter '{}'",
                            param.name
                        ),
                        param.span,
                    )
                    .with_label("unknown constraint")
                    .with_help("define the constraint type or use a supported constraint"),
                );
            }

            if resolved.normalized() == Type::Never {
                self.diagnostics.push(
                    Diagnostic::error_with_code(
                        "AT3001",
                        format!(
                            "Constraint for type parameter '{}' is unsatisfiable",
                            param.name
                        ),
                        param.span,
                    )
                    .with_label("conflicting constraint")
                    .with_help("simplify or remove the conflicting constraint"),
                );
            }
        }
    }

    fn contains_type_param(&self, ty: &Type, name: &str) -> bool {
        match ty {
            Type::TypeParameter { name: param_name } => param_name == name,
            Type::Array(elem) => self.contains_type_param(elem, name),
            Type::Function {
                params,
                return_type,
                type_params,
            } => {
                params.iter().any(|p| self.contains_type_param(p, name))
                    || self.contains_type_param(return_type, name)
                    || type_params
                        .iter()
                        .filter_map(|param| param.bound.as_ref())
                        .any(|bound| self.contains_type_param(bound, name))
            }
            Type::Generic { type_args, .. } => type_args
                .iter()
                .any(|arg| self.contains_type_param(arg, name)),
            Type::Alias {
                type_args, target, ..
            } => {
                type_args
                    .iter()
                    .any(|arg| self.contains_type_param(arg, name))
                    || self.contains_type_param(target, name)
            }
            Type::Union(members) | Type::Intersection(members) => members
                .iter()
                .any(|member| self.contains_type_param(member, name)),
            Type::Structural { members } => members
                .iter()
                .any(|member| self.contains_type_param(&member.ty, name)),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binder::Binder;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn typecheck_source(source: &str) -> Vec<Diagnostic> {
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();

        let mut binder = Binder::new();
        let (mut table, mut bind_diagnostics) = binder.bind(&program);

        let mut checker = TypeChecker::new(&mut table);
        let mut check_diagnostics = checker.check(&program);

        // Combine diagnostics from both binding and type checking
        bind_diagnostics.append(&mut check_diagnostics);
        bind_diagnostics
    }

    #[test]
    fn test_valid_variable() {
        let diagnostics = typecheck_source("let _x: number = 42;");
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_type_mismatch() {
        let diagnostics = typecheck_source("let x: number = \"hello\";");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3001");
    }

    #[test]
    fn test_arithmetic_type_error() {
        let diagnostics = typecheck_source(r#"let x = 5 + "hello";"#);
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3002");
    }

    #[test]
    fn test_condition_must_be_bool() {
        let diagnostics = typecheck_source("if (5) { }");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3001");
    }

    #[test]
    fn test_immutable_assignment() {
        let diagnostics = typecheck_source(
            r#"
            let x = 5;
            x = 10;
        "#,
        );
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3003");
    }

    #[test]
    fn test_break_outside_loop() {
        let diagnostics = typecheck_source("break;");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3010");
    }

    #[test]
    fn test_return_outside_function() {
        let diagnostics = typecheck_source("return 5;");
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code, "AT3011");
    }

    #[test]
    fn test_generic_type_valid_arity() {
        // Valid generic types - test arity validation only
        // Use in function parameters to avoid needing valid values
        let diagnostics = typecheck_source(
            r#"
            fn test_option(_x: Option<number>) -> void {}
            fn test_result(_x: Result<number, string>) -> void {}
        "#,
        );
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_generic_type_wrong_arity_too_few() {
        // Result expects 2 type arguments, got 1
        let diagnostics = typecheck_source("fn test(_x: Result<number>) -> void {}");
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0]
            .message
            .contains("expects 2 type argument(s), found 1"));
    }

    #[test]
    fn test_generic_type_wrong_arity_too_many() {
        // Option expects 1 type argument, got 2
        let diagnostics = typecheck_source("fn test(_x: Option<number, string>) -> void {}");
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0]
            .message
            .contains("expects 1 type argument(s), found 2"));
    }

    #[test]
    fn test_generic_type_unknown() {
        // Unknown generic type
        let diagnostics = typecheck_source("fn test(_x: UnknownGeneric<number>) -> void {}");
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("Unknown generic type"));
    }

    #[test]
    fn test_generic_type_nested() {
        // Nested generic types with correct arity
        let diagnostics =
            typecheck_source("fn test_nested(_x: Option<Result<number, string>>) -> void {}");
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    // ============================================================================
    // Type Inference Tests
    // ============================================================================

    #[test]
    fn test_generic_function_inference_simple() {
        // Simple type inference: T=number
        let diagnostics = typecheck_source(
            r#"
            fn identity<T>(x: T) -> T {
                return x;
            }
            let _result = identity(42);
        "#,
        );
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_generic_function_inference_string() {
        // Type inference: T=string
        let diagnostics = typecheck_source(
            r#"
            fn identity<T>(x: T) -> T {
                return x;
            }
            let _result = identity("hello");
        "#,
        );
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_generic_function_multiple_params() {
        // Multiple type parameters: A=number, B=string
        let diagnostics = typecheck_source(
            r#"
            fn pair<A, B>(first: A, _second: B) -> A {
                return first;
            }
            let _result = pair(42, "hello");
        "#,
        );
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_generic_function_inference_array() {
        // Type inference with arrays: T=number
        let diagnostics = typecheck_source(
            r#"
            fn first<T>(arr: T[]) -> T {
                return arr[0];
            }
            let numbers = [1, 2, 3];
            let _result = first(numbers);
        "#,
        );
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_generic_function_type_mismatch() {
        // Type inference should fail when types don't match
        let diagnostics = typecheck_source(
            r#"
            fn both_same<T>(_a: T, _b: T) -> T {
                return _a;
            }
            let _result = both_same(42, "hello");
        "#,
        );
        // Should have error: cannot unify number with string for T
        assert!(!diagnostics.is_empty(), "Expected errors but got none");
        assert!(
            diagnostics[0].message.contains("Type inference failed")
                || diagnostics[0].message.contains("cannot match"),
            "Unexpected error message: {}",
            diagnostics[0].message
        );
    }
}
