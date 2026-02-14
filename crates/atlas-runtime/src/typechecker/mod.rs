//! Type checking and inference
//!
//! The type checker enforces Atlas's strict type rules:
//! - No implicit any - all types must be explicit or inferrable
//! - No nullable - null only assigns to null type
//! - No truthy/falsey - conditionals require bool
//! - Strict equality - == requires same-type operands

mod expr;
pub mod generics;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::symbol::{SymbolKind, SymbolTable};
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Type checker state
pub struct TypeChecker<'a> {
    /// Symbol table from binder
    symbol_table: &'a mut SymbolTable,
    /// Collected diagnostics
    pub(super) diagnostics: Vec<Diagnostic>,
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
}

impl<'a> TypeChecker<'a> {
    /// Create a new type checker
    pub fn new(symbol_table: &'a mut SymbolTable) -> Self {
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            current_function_return_type: None,
            current_function_info: None,
            in_loop: false,
            declared_symbols: HashMap::new(),
            used_symbols: HashSet::new(),
        }
    }

    /// Type check a program
    pub fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
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
        }
    }

    /// Check a function declaration
    fn check_function(&mut self, func: &FunctionDecl) {
        let return_type = self.resolve_type_ref(&func.return_type);
        self.current_function_return_type = Some(return_type.clone());
        self.current_function_info = Some((func.name.name.clone(), func.name.span));

        // Clear tracking for this function
        self.declared_symbols.clear();
        self.used_symbols.clear();

        // Track parameters
        for param in &func.params {
            self.declared_symbols.insert(
                param.name.name.clone(),
                (param.name.span, SymbolKind::Parameter),
            );
        }

        self.check_block(&func.body);

        // Check if all paths return (if return type != void/null)
        if return_type != Type::Void
            && return_type != Type::Null
            && !self.block_always_returns(&func.body)
        {
            self.diagnostics.push(
                Diagnostic::error_with_code(
                    "AT3004",
                    "Not all code paths return a value",
                    func.span,
                )
                .with_label("function body"),
            );
        }

        // Emit warnings for unused variables/parameters
        self.emit_unused_warnings();

        self.current_function_return_type = None;
        self.current_function_info = None;
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
                    .with_label("declared here but never used"),
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
                        .with_label("this code will never execute"),
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

                if let Some(type_ref) = &var.type_ref {
                    let declared_type = self.resolve_type_ref(type_ref);
                    if !init_type.is_assignable_to(&declared_type) {
                        self.diagnostics.push(
                            Diagnostic::error_with_code(
                                "AT3001",
                                format!(
                                    "Type mismatch: cannot assign {} to variable of type {}",
                                    init_type.display_name(),
                                    declared_type.display_name()
                                ),
                                var.span,
                            )
                            .with_label("type mismatch"),
                        );
                    }
                } else {
                    // No explicit type annotation - update symbol table with inferred type
                    if init_type != Type::Unknown {
                        if let Some(symbol) = self.symbol_table.lookup_mut(&var.name.name) {
                            symbol.ty = init_type;
                        }
                    }
                }
            }
            Stmt::Assign(assign) => {
                let value_type = self.check_expr(&assign.value);
                let target_type = self.check_assign_target(&assign.target);

                if !value_type.is_assignable_to(&target_type) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Type mismatch in assignment: cannot assign {} to {}",
                                value_type.display_name(),
                                target_type.display_name()
                            ),
                            assign.span,
                        )
                        .with_label("type mismatch"),
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
                            .with_related_location(
                                crate::diagnostic::RelatedLocation {
                                    file: "<input>".to_string(),
                                    line: 1,
                                    column: symbol.span.start + 1,
                                    length: symbol.span.end.saturating_sub(symbol.span.start),
                                    message: format!(
                                        "'{}' declared here as immutable",
                                        symbol.name
                                    ),
                                },
                            );

                            self.diagnostics.push(diag);
                        }
                    }
                }
            }
            Stmt::CompoundAssign(compound) => {
                let value_type = self.check_expr(&compound.value);
                let target_type = self.check_assign_target(&compound.target);

                // Compound assignment requires both sides to be numbers (allow Unknown for error recovery)
                if !matches!(target_type, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Compound assignment requires number type, found {}",
                                target_type.display_name()
                            ),
                            compound.span,
                        )
                        .with_label("type mismatch"),
                    );
                }

                if !matches!(value_type, Type::Number | Type::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!(
                                "Compound assignment requires number value, found {}",
                                value_type.display_name()
                            ),
                            compound.span,
                        )
                        .with_label("type mismatch"),
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

                // Increment requires number type (allow Unknown for error recovery)
                if !matches!(target_type, Type::Number | Type::Unknown) {
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

                // Decrement requires number type (allow Unknown for error recovery)
                if !matches!(target_type, Type::Number | Type::Unknown) {
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
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            if_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }
                self.check_block(&if_stmt.then_block);
                if let Some(else_block) = &if_stmt.else_block {
                    self.check_block(else_block);
                }
            }
            Stmt::While(while_stmt) => {
                let cond_type = self.check_expr(&while_stmt.cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            while_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
                    );
                }
                let old_in_loop = self.in_loop;
                self.in_loop = true;
                self.check_block(&while_stmt.body);
                self.in_loop = old_in_loop;
            }
            Stmt::For(for_stmt) => {
                self.check_statement(&for_stmt.init);
                let cond_type = self.check_expr(&for_stmt.cond);
                if cond_type != Type::Bool && cond_type != Type::Unknown {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT3001",
                            format!("Condition must be bool, found {}", cond_type.display_name()),
                            for_stmt.cond.span(),
                        )
                        .with_label("type mismatch"),
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
                    .with_label("type mismatch");

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
                self.check_expr(&expr_stmt.expr);
            }
        }
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

                // Check that index is a number
                if index_type != Type::Number && index_type != Type::Unknown {
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
                match target_type {
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

    /// Resolve a type reference to a Type
    pub(super) fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Type {
        match type_ref {
            TypeRef::Named(name, _) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                _ => Type::Unknown,
            },
            TypeRef::Array(elem, _) => Type::Array(Box::new(self.resolve_type_ref(elem))),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let param_types = params.iter().map(|p| self.resolve_type_ref(p)).collect();
                let ret_type = Box::new(self.resolve_type_ref(return_type));
                Type::Function {
                    type_params: vec![], // Function types don't have type params (only decls do)
                    params: param_types,
                    return_type: ret_type,
                }
            }
            TypeRef::Generic {
                name,
                type_args,
                span,
            } => {
                // Validate generic type arity
                let expected_arity = self.get_generic_type_arity(name);

                if let Some(arity) = expected_arity {
                    if type_args.len() != arity {
                        self.diagnostics.push(
                            Diagnostic::error(
                                format!(
                                    "Generic type '{}' expects {} type argument(s), found {}",
                                    name,
                                    arity,
                                    type_args.len()
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
                        Diagnostic::error(
                            format!("Unknown generic type '{}'", name),
                            *span,
                        )
                        .with_label("unknown type"),
                    );
                    return Type::Unknown;
                }

                // Resolve type arguments
                let resolved_args = type_args
                    .iter()
                    .map(|arg| self.resolve_type_ref(arg))
                    .collect();

                Type::Generic {
                    name: name.clone(),
                    type_args: resolved_args,
                }
            }
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
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3001");
    }

    #[test]
    fn test_arithmetic_type_error() {
        let diagnostics = typecheck_source(r#"let x = 5 + "hello";"#);
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3002");
    }

    #[test]
    fn test_condition_must_be_bool() {
        let diagnostics = typecheck_source("if (5) { }");
        assert!(diagnostics.len() > 0);
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
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3003");
    }

    #[test]
    fn test_break_outside_loop() {
        let diagnostics = typecheck_source("break;");
        assert!(diagnostics.len() > 0);
        assert_eq!(diagnostics[0].code, "AT3010");
    }

    #[test]
    fn test_return_outside_function() {
        let diagnostics = typecheck_source("return 5;");
        assert!(diagnostics.len() > 0);
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
        assert!(diagnostics.len() > 0);
        assert!(diagnostics[0]
            .message
            .contains("expects 2 type argument(s), found 1"));
    }

    #[test]
    fn test_generic_type_wrong_arity_too_many() {
        // Option expects 1 type argument, got 2
        let diagnostics = typecheck_source("fn test(_x: Option<number, string>) -> void {}");
        assert!(diagnostics.len() > 0);
        assert!(diagnostics[0]
            .message
            .contains("expects 1 type argument(s), found 2"));
    }

    #[test]
    fn test_generic_type_unknown() {
        // Unknown generic type
        let diagnostics = typecheck_source("fn test(_x: UnknownGeneric<number>) -> void {}");
        assert!(diagnostics.len() > 0);
        assert!(diagnostics[0].message.contains("Unknown generic type"));
    }

    #[test]
    fn test_generic_type_nested() {
        // Nested generic types with correct arity
        let diagnostics = typecheck_source(
            "fn test_nested(_x: Option<Result<number, string>>) -> void {}",
        );
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
        assert!(diagnostics.len() > 0, "Expected errors but got none");
        assert!(
            diagnostics[0].message.contains("Type inference failed")
                || diagnostics[0].message.contains("cannot match"),
            "Unexpected error message: {}",
            diagnostics[0].message
        );
    }
}
