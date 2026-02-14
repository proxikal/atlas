//! Name binding and scope resolution
//!
//! The binder performs two-pass analysis:
//! 1. Collect all top-level function declarations (hoisting)
//! 2. Bind all items and resolve identifiers

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::symbol::{Symbol, SymbolKind, SymbolTable};
use crate::types::Type;
use std::collections::HashMap;

/// Binder for name resolution and scope management
pub struct Binder {
    /// Symbol table
    symbol_table: SymbolTable,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Type parameter scopes (stack of scopes, each scope maps param name -> TypeParam)
    type_param_scopes: Vec<HashMap<String, TypeParam>>,
}

impl Binder {
    /// Create a new binder
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
            type_param_scopes: Vec::new(),
        }
    }

    /// Create a binder with an existing symbol table (for REPL state persistence)
    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        Self {
            symbol_table,
            diagnostics: Vec::new(),
            type_param_scopes: Vec::new(),
        }
    }

    /// Bind a program (two-pass: hoist functions, then bind everything)
    pub fn bind(&mut self, program: &Program) -> (SymbolTable, Vec<Diagnostic>) {
        // Phase 1: Collect all top-level function declarations (hoisting)
        for item in &program.items {
            if let Item::Function(func) = item {
                self.hoist_function(func);
            }
        }

        // Phase 2: Bind all items
        for item in &program.items {
            self.bind_item(item);
        }

        (
            std::mem::take(&mut self.symbol_table),
            std::mem::take(&mut self.diagnostics),
        )
    }

    /// Hoist a top-level function declaration
    fn hoist_function(&mut self, func: &FunctionDecl) {
        // Check for global shadowing of prelude builtins
        if self.symbol_table.is_prelude_builtin(&func.name.name) {
            let diag = Diagnostic::error_with_code(
                "AT1012",
                format!(
                    "Cannot shadow prelude builtin '{}' in global scope",
                    func.name.name
                ),
                func.name.span,
            )
            .with_label("shadows prelude builtin")
            .with_help("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope.".to_string());

            self.diagnostics.push(diag);
            return;
        }

        // Enter type parameter scope to resolve generic types
        self.enter_type_param_scope();
        for type_param in &func.type_params {
            self.register_type_parameter(type_param);
        }

        let param_types: Vec<Type> = func
            .params
            .iter()
            .map(|p| self.resolve_type_ref(&p.type_ref))
            .collect();

        let return_type = self.resolve_type_ref(&func.return_type);

        // Exit type parameter scope
        self.exit_type_param_scope();

        let symbol = Symbol {
            name: func.name.name.clone(),
            ty: Type::Function {
                type_params: func.type_params.iter().map(|tp| tp.name.clone()).collect(),
                params: param_types,
                return_type: Box::new(return_type),
            },
            mutable: false,
            kind: SymbolKind::Function,
            span: func.name.span,
        };

        if let Err((msg, existing)) = self.symbol_table.define_function(symbol) {
            let mut diag = Diagnostic::error_with_code("AT2003", &msg, func.name.span)
                .with_label("redeclaration");

            // Add related location if we have the existing symbol
            if let Some(existing_symbol) = existing {
                diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                    file: "<input>".to_string(),
                    line: 1,
                    column: existing_symbol.span.start + 1,
                    length: existing_symbol
                        .span
                        .end
                        .saturating_sub(existing_symbol.span.start),
                    message: format!("'{}' first defined here", existing_symbol.name),
                });
            }

            self.diagnostics.push(diag);
        }
    }

    /// Bind a top-level item
    fn bind_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.bind_function(func),
            Item::Statement(stmt) => self.bind_statement(stmt),
        }
    }

    /// Bind a function declaration
    fn bind_function(&mut self, func: &FunctionDecl) {
        // Enter function scope
        self.symbol_table.enter_scope();

        // Note: Type parameter scope was already handled in hoist_function
        // when we resolved the function signature types

        // Bind parameters
        for param in &func.params {
            let ty = self.resolve_type_ref(&param.type_ref);
            let symbol = Symbol {
                name: param.name.name.clone(),
                ty,
                mutable: false,
                kind: SymbolKind::Parameter,
                span: param.name.span,
            };

            if let Err((msg, existing)) = self.symbol_table.define(symbol) {
                let mut diag = Diagnostic::error_with_code("AT2003", &msg, param.name.span)
                    .with_label("parameter redeclaration");

                // Add related location if we have the existing symbol
                if let Some(existing_symbol) = existing {
                    diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                        file: "<input>".to_string(),
                        line: 1,
                        column: existing_symbol.span.start + 1,
                        length: existing_symbol
                            .span
                            .end
                            .saturating_sub(existing_symbol.span.start),
                        message: format!("'{}' first defined here", existing_symbol.name),
                    });
                }

                self.diagnostics.push(diag);
            }
        }

        // Bind function body
        self.bind_block(&func.body);

        // Exit function scope
        self.symbol_table.exit_scope();
    }

    /// Bind a block
    fn bind_block(&mut self, block: &Block) {
        // Blocks create their own scope
        self.symbol_table.enter_scope();

        for stmt in &block.statements {
            self.bind_statement(stmt);
        }

        self.symbol_table.exit_scope();
    }

    /// Bind a statement
    fn bind_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(var) => {
                // Check for global shadowing of prelude builtins
                if self.symbol_table.is_global_scope()
                    && self.symbol_table.is_prelude_builtin(&var.name.name)
                {
                    let diag = Diagnostic::error_with_code(
                        "AT1012",
                        format!(
                            "Cannot shadow prelude builtin '{}' in global scope",
                            var.name.name
                        ),
                        var.name.span,
                    )
                    .with_label("shadows prelude builtin")
                    .with_help("Prelude builtins cannot be redefined at the top level. Use a different name or shadow in a nested scope.".to_string());

                    self.diagnostics.push(diag);
                    return;
                }

                // First bind the initializer (can't reference the variable being declared)
                self.bind_expr(&var.init);

                // Then define the variable
                let ty = if let Some(type_ref) = &var.type_ref {
                    self.resolve_type_ref(type_ref)
                } else {
                    Type::Unknown // Will be inferred by typechecker
                };

                let symbol = Symbol {
                    name: var.name.name.clone(),
                    ty,
                    mutable: var.mutable,
                    kind: SymbolKind::Variable,
                    span: var.name.span,
                };

                if let Err((msg, existing)) = self.symbol_table.define(symbol) {
                    let mut diag = Diagnostic::error_with_code("AT2003", &msg, var.name.span)
                        .with_label("variable redeclaration");

                    // Add related location if we have the existing symbol
                    if let Some(existing_symbol) = existing {
                        diag = diag.with_related_location(crate::diagnostic::RelatedLocation {
                            file: "<input>".to_string(),
                            line: 1,
                            column: existing_symbol.span.start + 1,
                            length: existing_symbol
                                .span
                                .end
                                .saturating_sub(existing_symbol.span.start),
                            message: format!("'{}' first defined here", existing_symbol.name),
                        });
                    }

                    self.diagnostics.push(diag);
                }
            }
            Stmt::Assign(assign) => {
                // Bind assignment target and value
                self.bind_assign_target(&assign.target);
                self.bind_expr(&assign.value);
            }
            Stmt::CompoundAssign(compound) => {
                // Bind compound assignment target and value
                self.bind_assign_target(&compound.target);
                self.bind_expr(&compound.value);
            }
            Stmt::Increment(inc) => {
                // Bind increment target
                self.bind_assign_target(&inc.target);
            }
            Stmt::Decrement(dec) => {
                // Bind decrement target
                self.bind_assign_target(&dec.target);
            }
            Stmt::If(if_stmt) => {
                self.bind_expr(&if_stmt.cond);
                self.bind_block(&if_stmt.then_block);
                if let Some(else_block) = &if_stmt.else_block {
                    self.bind_block(else_block);
                }
            }
            Stmt::While(while_stmt) => {
                self.bind_expr(&while_stmt.cond);
                self.bind_block(&while_stmt.body);
            }
            Stmt::For(for_stmt) => {
                // For loops create their own scope for the initializer
                self.symbol_table.enter_scope();

                self.bind_statement(&for_stmt.init);
                self.bind_expr(&for_stmt.cond);
                self.bind_statement(&for_stmt.step);
                self.bind_block(&for_stmt.body);

                self.symbol_table.exit_scope();
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.value {
                    self.bind_expr(expr);
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {
                // No binding needed
            }
            Stmt::Expr(expr_stmt) => {
                self.bind_expr(&expr_stmt.expr);
            }
        }
    }

    /// Bind an assignment target
    fn bind_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::Name(id) => {
                // Check if the identifier exists
                if self.symbol_table.lookup(&id.name).is_none() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT2002",
                            format!("Unknown symbol '{}'", id.name),
                            id.span,
                        )
                        .with_label("undefined variable"),
                    );
                }
            }
            AssignTarget::Index { target, index, .. } => {
                self.bind_expr(target);
                self.bind_expr(index);
            }
        }
    }

    /// Bind an expression
    fn bind_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(_, _) => {
                // Literals don't need binding
            }
            Expr::Identifier(id) => {
                // Check if identifier is defined
                if self.symbol_table.lookup(&id.name).is_none() {
                    self.diagnostics.push(
                        Diagnostic::error_with_code(
                            "AT2002",
                            format!("Unknown symbol '{}'", id.name),
                            id.span,
                        )
                        .with_label("undefined variable"),
                    );
                }
            }
            Expr::Binary(binary) => {
                self.bind_expr(&binary.left);
                self.bind_expr(&binary.right);
            }
            Expr::Unary(unary) => {
                self.bind_expr(&unary.expr);
            }
            Expr::Call(call) => {
                self.bind_expr(&call.callee);
                for arg in &call.args {
                    self.bind_expr(arg);
                }
            }
            Expr::Index(index) => {
                self.bind_expr(&index.target);
                self.bind_expr(&index.index);
            }
            Expr::ArrayLiteral(arr) => {
                for elem in &arr.elements {
                    self.bind_expr(elem);
                }
            }
            Expr::Group(group) => {
                self.bind_expr(&group.expr);
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
    fn resolve_type_ref(&mut self, type_ref: &TypeRef) -> Type {
        match type_ref {
            TypeRef::Named(name, _) => match name.as_str() {
                "number" => Type::Number,
                "string" => Type::String,
                "bool" => Type::Bool,
                "void" => Type::Void,
                "null" => Type::Null,
                "json" => Type::JsonValue,
                _ => {
                    // Check if it's a type parameter
                    if let Some(_type_param) = self.lookup_type_parameter(name) {
                        return Type::TypeParameter { name: name.clone() };
                    }
                    // Unknown type - will be caught by typechecker
                    Type::Unknown
                }
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
                // BLOCKER 02-B: Validate generic type arity
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
                        Diagnostic::error(format!("Unknown generic type '{}'", name), *span)
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

    // === Type Parameter Scope Management ===

    /// Enter a new type parameter scope
    fn enter_type_param_scope(&mut self) {
        self.type_param_scopes.push(HashMap::new());
    }

    /// Exit the current type parameter scope
    fn exit_type_param_scope(&mut self) {
        self.type_param_scopes.pop();
    }

    /// Register a type parameter in the current scope
    fn register_type_parameter(&mut self, type_param: &TypeParam) {
        if let Some(current_scope) = self.type_param_scopes.last_mut() {
            // Check for duplicate type parameter in this scope
            if current_scope.contains_key(&type_param.name) {
                self.diagnostics.push(
                    Diagnostic::error(
                        format!("Duplicate type parameter '{}'", type_param.name),
                        type_param.span,
                    )
                    .with_label("duplicate type parameter"),
                );
                return;
            }

            current_scope.insert(type_param.name.clone(), type_param.clone());
        }
    }

    /// Look up a type parameter in the scope stack
    fn lookup_type_parameter(&self, name: &str) -> Option<&TypeParam> {
        // Search from innermost to outermost scope
        for scope in self.type_param_scopes.iter().rev() {
            if let Some(type_param) = scope.get(name) {
                return Some(type_param);
            }
        }
        None
    }
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn bind_source(source: &str) -> (SymbolTable, Vec<Diagnostic>) {
        let mut lexer = Lexer::new(source);
        let (tokens, _) = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (program, _) = parser.parse();

        let mut binder = Binder::new();
        binder.bind(&program)
    }

    #[test]
    fn test_bind_simple_variable() {
        let (table, diagnostics) = bind_source("let x = 42;");

        assert_eq!(diagnostics.len(), 0);
        assert!(table.lookup("x").is_some());
    }

    #[test]
    fn test_bind_function() {
        let (table, diagnostics) = bind_source("fn foo() {}");

        // Debug: print diagnostics if any
        for diag in &diagnostics {
            eprintln!("Diagnostic: {} - {}", diag.code, diag.message);
        }

        assert_eq!(
            diagnostics.len(),
            0,
            "Expected no diagnostics, got: {:?}",
            diagnostics
        );
        assert!(
            table.lookup("foo").is_some(),
            "Function 'foo' not found in symbol table"
        );
        assert_eq!(table.lookup("foo").unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_function_hoisting() {
        // Function should be accessible even before its declaration
        let (table, diagnostics) = bind_source(
            r#"
            let x = foo();
            fn foo() -> number { return 42; }
        "#,
        );

        assert_eq!(diagnostics.len(), 0);
        assert!(table.lookup("foo").is_some());
    }

    #[test]
    fn test_unknown_symbol() {
        let (_table, diagnostics) = bind_source("let x = y;");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT2002");
        assert!(diagnostics[0].message.contains("Unknown symbol 'y'"));
    }

    #[test]
    fn test_redeclaration_error() {
        let (_table, diagnostics) = bind_source(
            r#"
            let x = 1;
            let x = 2;
        "#,
        );

        assert!(diagnostics.len() >= 1);
        assert!(diagnostics.iter().any(|d| d.code == "AT2003"));
    }

    #[test]
    fn test_scope_shadowing() {
        // Shadowing in inner scope should be allowed
        let (table, diagnostics) = bind_source(
            r#"
            let x = 1;
            {
                let x = 2;
            }
        "#,
        );

        // Should have no errors (shadowing is allowed)
        assert_eq!(diagnostics.len(), 0);
        // Outer x should still be in table
        assert!(table.lookup("x").is_some());
    }

    #[test]
    fn test_builtin_functions() {
        let (table, diagnostics) = bind_source(
            r#"
            print("hello");
            let l = len("world");
            let s = str(42);
        "#,
        );

        assert_eq!(diagnostics.len(), 0);
        assert!(table.lookup("print").is_some());
        assert!(table.lookup("len").is_some());
        assert!(table.lookup("str").is_some());
    }

    #[test]
    fn test_global_prelude_shadowing_function() {
        // Shadowing prelude function at global scope should produce AT1012
        let (_table, diagnostics) = bind_source(
            r#"
            fn print() -> void {}
        "#,
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1012");
        assert!(diagnostics[0]
            .message
            .contains("Cannot shadow prelude builtin 'print'"));
    }

    #[test]
    fn test_global_prelude_shadowing_variable() {
        // Shadowing prelude function with variable at global scope should produce AT1012
        let (_table, diagnostics) = bind_source(
            r#"
            let len = 42;
        "#,
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "AT1012");
        assert!(diagnostics[0]
            .message
            .contains("Cannot shadow prelude builtin 'len'"));
    }

    #[test]
    fn test_nested_prelude_shadowing_allowed() {
        // Shadowing prelude in nested scope should be allowed
        let (table, diagnostics) = bind_source(
            r#"
            fn foo() -> void {
                let print = 42;
                let len = "hello";
            }
        "#,
        );

        assert_eq!(diagnostics.len(), 0);
        // Original builtins should still be accessible
        assert!(table.lookup("print").is_some());
        assert_eq!(table.lookup("print").unwrap().kind, SymbolKind::Builtin);
    }

    #[test]
    fn test_all_prelude_builtins_shadowing() {
        // Test shadowing all three prelude builtins
        let (_table, diagnostics) = bind_source(
            r#"
            fn print() -> void {}
            let len = 42;
            var str = "test";
        "#,
        );

        // Should have 3 errors (one for each prelude builtin)
        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().all(|d| d.code == "AT1012"));
    }

    #[test]
    fn test_type_parameter_binding() {
        // Test that type parameters are correctly registered and resolved
        let (table, diagnostics) = bind_source(
            r#"
            fn identity<T>(x: T) -> T {
                return x;
            }
        "#,
        );

        // Should have no errors
        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);

        // Function should be bound
        let identity_symbol = table.lookup("identity");
        assert!(identity_symbol.is_some());

        // Check function type has type parameters
        if let Type::Function { type_params, .. } = &identity_symbol.unwrap().ty {
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0], "T");
        } else {
            panic!("Expected Function type");
        }
    }

    #[test]
    fn test_multiple_type_parameters() {
        // Test multiple type parameters
        let (table, diagnostics) = bind_source(
            r#"
            fn pair<A, B>(first: A, second: B) -> number {
                return 42;
            }
        "#,
        );

        assert_eq!(diagnostics.len(), 0, "Diagnostics: {:?}", diagnostics);

        let pair_symbol = table.lookup("pair");
        assert!(pair_symbol.is_some());

        if let Type::Function { type_params, .. } = &pair_symbol.unwrap().ty {
            assert_eq!(type_params.len(), 2);
            assert_eq!(type_params[0], "A");
            assert_eq!(type_params[1], "B");
        } else {
            panic!("Expected Function type");
        }
    }

    #[test]
    fn test_duplicate_type_parameter() {
        // Test that duplicate type parameters are rejected
        let (_table, diagnostics) = bind_source(
            r#"
            fn bad<T, T>(x: T) -> T {
                return x;
            }
        "#,
        );

        // Should have 1 error for duplicate type parameter
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0]
            .message
            .contains("Duplicate type parameter 'T'"));
    }
}
