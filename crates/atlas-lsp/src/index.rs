//! Workspace-wide symbol indexing for LSP navigation features
//!
//! Maintains an index of all symbol definitions and references across the workspace,
//! enabling fast lookups for find-all-references, call hierarchy, and workspace symbols.

use atlas_runtime::ast::*;
use atlas_runtime::span::Span;
use std::collections::HashMap;
use tower_lsp::lsp_types::{Location, Position, Range, Url};

/// A symbol definition in the workspace
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    /// Symbol name
    pub name: String,
    /// Location of the definition
    pub location: Location,
    /// Kind of symbol (function, variable, type, etc.)
    pub kind: SymbolKind,
    /// Scope information (e.g., function name for local variables)
    pub scope: Option<String>,
}

/// A symbol reference in the workspace
#[derive(Debug, Clone)]
pub struct SymbolReference {
    /// Symbol name
    pub name: String,
    /// Location of the reference
    pub location: Location,
    /// Whether this is a write (assignment) or read
    pub is_write: bool,
}

/// Kind of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Variable,
    Parameter,
    Type,
}

/// Workspace-wide symbol index
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// Map from symbol name to all its definitions
    definitions: HashMap<String, Vec<SymbolDefinition>>,
    /// Map from symbol name to all its references
    references: HashMap<String, Vec<SymbolReference>>,
    /// Map from file URI to all symbols defined in that file
    file_definitions: HashMap<Url, Vec<String>>,
    /// Map from file URI to all symbols referenced in that file
    file_references: HashMap<Url, Vec<String>>,
}

impl SymbolIndex {
    /// Create a new empty symbol index
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a document (parse and extract all symbols)
    pub fn index_document(&mut self, uri: &Url, text: &str, ast: Option<&Program>) {
        // Remove existing entries for this file
        self.remove_document(uri);

        if let Some(program) = ast {
            // Track definitions and references in this file
            let mut ctx = IndexContext {
                uri: uri.clone(),
                text,
                current_scope: None,
            };

            // Index all items in the program
            for item in &program.items {
                self.index_item(item, &mut ctx);
            }
        }
    }

    /// Remove all symbols from a document
    pub fn remove_document(&mut self, uri: &Url) {
        // Remove all definitions from this file
        if let Some(symbol_names) = self.file_definitions.remove(uri) {
            for name in symbol_names {
                if let Some(defs) = self.definitions.get_mut(&name) {
                    defs.retain(|def| def.location.uri != *uri);
                    if defs.is_empty() {
                        self.definitions.remove(&name);
                    }
                }
            }
        }

        // Remove all references from this file
        if let Some(symbol_names) = self.file_references.remove(uri) {
            for name in symbol_names {
                if let Some(refs) = self.references.get_mut(&name) {
                    refs.retain(|ref_| ref_.location.uri != *uri);
                    if refs.is_empty() {
                        self.references.remove(&name);
                    }
                }
            }
        }
    }

    /// Find all definitions of a symbol
    pub fn find_definitions(&self, name: &str) -> Vec<SymbolDefinition> {
        self.definitions.get(name).cloned().unwrap_or_default()
    }

    /// Find all references to a symbol
    pub fn find_references(&self, name: &str, include_definition: bool) -> Vec<Location> {
        let mut locations = Vec::new();

        // Add all references
        if let Some(refs) = self.references.get(name) {
            locations.extend(refs.iter().map(|r| r.location.clone()));
        }

        // Optionally include the definition
        if include_definition {
            if let Some(defs) = self.definitions.get(name) {
                locations.extend(defs.iter().map(|d| d.location.clone()));
            }
        }

        locations
    }

    /// Find the definition at a specific location
    pub fn find_definition_at(&self, _uri: &Url, _position: Position) -> Option<SymbolDefinition> {
        // TODO: Implement position-based lookup
        // This requires converting Position to byte offset and checking Spans
        None
    }

    /// Get all symbols in the workspace (for workspace symbol search)
    pub fn all_symbols(&self) -> Vec<SymbolDefinition> {
        self.definitions
            .values()
            .flat_map(|defs| defs.iter().cloned())
            .collect()
    }

    // Private indexing methods

    fn index_item(&mut self, item: &Item, ctx: &mut IndexContext) {
        match item {
            Item::Function(func) => {
                self.add_definition(
                    &func.name.name,
                    &func.name.span,
                    SymbolKind::Function,
                    None,
                    ctx,
                );

                // Save current scope and set new scope to this function
                let prev_scope = ctx.current_scope.clone();
                ctx.current_scope = Some(func.name.name.clone());

                // Index parameters
                for param in &func.params {
                    self.add_definition(
                        &param.name.name,
                        &param.name.span,
                        SymbolKind::Parameter,
                        ctx.current_scope.clone(),
                        ctx,
                    );
                }

                // Index function body
                self.index_block(&func.body, ctx);

                // Restore previous scope
                ctx.current_scope = prev_scope;
            }
            Item::Statement(stmt) => {
                self.index_stmt(stmt, ctx);
            }
            Item::TypeAlias(alias) => {
                self.add_definition(
                    &alias.name.name,
                    &alias.name.span,
                    SymbolKind::Type,
                    None,
                    ctx,
                );
            }
            Item::Import(_) | Item::Export(_) | Item::Extern(_) => {
                // TODO: Handle imports/exports for cross-file indexing
            }
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl indexing handled in Block 3
            }
        }
    }

    fn index_block(&mut self, block: &Block, ctx: &mut IndexContext) {
        for stmt in &block.statements {
            self.index_stmt(stmt, ctx);
        }
    }

    fn index_stmt(&mut self, stmt: &Stmt, ctx: &mut IndexContext) {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                self.add_definition(
                    &var_decl.name.name,
                    &var_decl.name.span,
                    SymbolKind::Variable,
                    ctx.current_scope.clone(),
                    ctx,
                );
                self.index_expr(&var_decl.init, ctx, false);
            }
            Stmt::FunctionDecl(func) => {
                self.add_definition(
                    &func.name.name,
                    &func.name.span,
                    SymbolKind::Function,
                    ctx.current_scope.clone(),
                    ctx,
                );

                // Save current scope and set new scope to this function
                let prev_scope = ctx.current_scope.clone();
                ctx.current_scope = Some(func.name.name.clone());

                // Index parameters
                for param in &func.params {
                    self.add_definition(
                        &param.name.name,
                        &param.name.span,
                        SymbolKind::Parameter,
                        ctx.current_scope.clone(),
                        ctx,
                    );
                }

                // Index function body
                self.index_block(&func.body, ctx);

                // Restore previous scope
                ctx.current_scope = prev_scope;
            }
            Stmt::If(if_stmt) => {
                self.index_expr(&if_stmt.cond, ctx, false);
                self.index_block(&if_stmt.then_block, ctx);
                if let Some(else_block) = &if_stmt.else_block {
                    self.index_block(else_block, ctx);
                }
            }
            Stmt::While(while_stmt) => {
                self.index_expr(&while_stmt.cond, ctx, false);
                self.index_block(&while_stmt.body, ctx);
            }
            Stmt::For(for_stmt) => {
                self.index_stmt(&for_stmt.init, ctx);
                self.index_expr(&for_stmt.cond, ctx, false);
                self.index_stmt(&for_stmt.step, ctx);
                self.index_block(&for_stmt.body, ctx);
            }
            Stmt::ForIn(for_in_stmt) => {
                self.add_definition(
                    &for_in_stmt.variable.name,
                    &for_in_stmt.variable.span,
                    SymbolKind::Variable,
                    ctx.current_scope.clone(),
                    ctx,
                );
                self.index_expr(&for_in_stmt.iterable, ctx, false);
                self.index_block(&for_in_stmt.body, ctx);
            }
            Stmt::Return(ret_stmt) => {
                if let Some(expr) = &ret_stmt.value {
                    self.index_expr(expr, ctx, false);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.index_expr(&expr_stmt.expr, ctx, false);
            }
            Stmt::Assign(assign) => {
                self.index_assign_target(&assign.target, ctx, true);
                self.index_expr(&assign.value, ctx, false);
            }
            Stmt::CompoundAssign(assign) => {
                self.index_assign_target(&assign.target, ctx, true);
                self.index_expr(&assign.value, ctx, false);
            }
            Stmt::Increment(inc) => {
                self.index_assign_target(&inc.target, ctx, true);
            }
            Stmt::Decrement(dec) => {
                self.index_assign_target(&dec.target, ctx, true);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn index_assign_target(
        &mut self,
        target: &AssignTarget,
        ctx: &mut IndexContext,
        is_write: bool,
    ) {
        match target {
            AssignTarget::Name(name) => {
                self.add_reference(&name.name, &name.span, is_write, ctx);
            }
            AssignTarget::Index { target, index, .. } => {
                self.index_expr(target, ctx, false);
                self.index_expr(index, ctx, false);
            }
        }
    }

    fn index_expr(&mut self, expr: &Expr, ctx: &mut IndexContext, is_write: bool) {
        match expr {
            Expr::Identifier(ident) => {
                self.add_reference(&ident.name, &ident.span, is_write, ctx);
            }
            Expr::Unary(unary) => {
                self.index_expr(&unary.expr, ctx, false);
            }
            Expr::Binary(binary) => {
                self.index_expr(&binary.left, ctx, false);
                self.index_expr(&binary.right, ctx, false);
            }
            Expr::Call(call) => {
                self.index_expr(&call.callee, ctx, false);
                for arg in &call.args {
                    self.index_expr(arg, ctx, false);
                }
            }
            Expr::Index(index) => {
                self.index_expr(&index.target, ctx, false);
                self.index_expr(&index.index, ctx, false);
            }
            Expr::Member(member) => {
                self.index_expr(&member.target, ctx, false);
                // Don't index member.member - it's a field name, not a variable reference
            }
            Expr::ArrayLiteral(array) => {
                for elem in &array.elements {
                    self.index_expr(elem, ctx, false);
                }
            }
            Expr::Group(group) => {
                self.index_expr(&group.expr, ctx, false);
            }
            Expr::Match(match_expr) => {
                self.index_expr(&match_expr.scrutinee, ctx, false);
                for arm in &match_expr.arms {
                    self.index_expr(&arm.body, ctx, false);
                }
            }
            Expr::Try(try_expr) => {
                self.index_expr(&try_expr.expr, ctx, false);
            }
            Expr::Literal(_, _) => {}
        }
    }

    fn add_definition(
        &mut self,
        name: &str,
        span: &Span,
        kind: SymbolKind,
        scope: Option<String>,
        ctx: &IndexContext,
    ) {
        let def = SymbolDefinition {
            name: name.to_string(),
            location: Location {
                uri: ctx.uri.clone(),
                range: span_to_range(span, ctx.text),
            },
            kind,
            scope,
        };

        self.definitions
            .entry(name.to_string())
            .or_default()
            .push(def);

        self.file_definitions
            .entry(ctx.uri.clone())
            .or_default()
            .push(name.to_string());
    }

    fn add_reference(&mut self, name: &str, span: &Span, is_write: bool, ctx: &IndexContext) {
        let ref_ = SymbolReference {
            name: name.to_string(),
            location: Location {
                uri: ctx.uri.clone(),
                range: span_to_range(span, ctx.text),
            },
            is_write,
        };

        self.references
            .entry(name.to_string())
            .or_default()
            .push(ref_);

        self.file_references
            .entry(ctx.uri.clone())
            .or_default()
            .push(name.to_string());
    }
}

/// Context for indexing a single document
struct IndexContext<'a> {
    uri: Url,
    text: &'a str,
    current_scope: Option<String>,
}

/// Convert a Span (byte offsets) to an LSP Range (line/column positions)
pub fn span_to_range(span: &Span, text: &str) -> Range {
    Range {
        start: offset_to_position(span.start, text),
        end: offset_to_position(span.end, text),
    }
}

/// Convert a byte offset to an LSP Position (line/column)
pub fn offset_to_position(offset: usize, text: &str) -> Position {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for ch in text.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        current_offset += ch.len_utf8();
    }

    Position {
        line: line as u32,
        character: col as u32,
    }
}

/// Convert an LSP Position (line/column) to a byte offset
pub fn position_to_offset(position: Position, text: &str) -> usize {
    let mut current_line = 0;
    let mut current_col = 0;
    let mut offset = 0;

    for ch in text.chars() {
        if current_line == position.line as usize && current_col == position.character as usize {
            return offset;
        }

        if ch == '\n' {
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }

        offset += ch.len_utf8();
    }

    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_position() {
        let text = "fn add(a: number) -> number {\n    return a + 1;\n}";

        // "fn" starts at offset 0, line 0
        assert_eq!(
            offset_to_position(0, text),
            Position {
                line: 0,
                character: 0
            }
        );

        // "add" starts at offset 3, line 0
        assert_eq!(
            offset_to_position(3, text),
            Position {
                line: 0,
                character: 3
            }
        );

        // "return" starts at offset 34 (after newline), line 1
        assert_eq!(
            offset_to_position(34, text),
            Position {
                line: 1,
                character: 4
            }
        );
    }

    #[test]
    fn test_position_to_offset() {
        let text = "fn add(a: number) -> number {\n    return a + 1;\n}";

        // Line 0, character 0 = offset 0
        assert_eq!(
            position_to_offset(
                Position {
                    line: 0,
                    character: 0
                },
                text
            ),
            0
        );

        // Line 0, character 3 = offset 3
        assert_eq!(
            position_to_offset(
                Position {
                    line: 0,
                    character: 3
                },
                text
            ),
            3
        );

        // Line 1, character 4 = offset 34
        assert_eq!(
            position_to_offset(
                Position {
                    line: 1,
                    character: 4
                },
                text
            ),
            34
        );
    }

    #[test]
    fn test_span_to_range() {
        let text = "fn add(a: number) -> number {\n    return a + 1;\n}";
        let span = Span::new(3, 6); // "add"

        let range = span_to_range(&span, text);
        assert_eq!(
            range.start,
            Position {
                line: 0,
                character: 3
            }
        );
        assert_eq!(
            range.end,
            Position {
                line: 0,
                character: 6
            }
        );
    }
}
