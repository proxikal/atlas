//! Navigation helpers for LSP features

use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

/// Find the identifier at a given position in the source
pub fn find_identifier_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col >= line.len() {
        return None;
    }

    // Find word boundaries around the cursor
    let start = line[..col]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    let end = line[col..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| col + i)
        .unwrap_or(line.len());

    if start >= end {
        return None;
    }

    Some(line[start..end].to_string())
}

/// Extract document symbols from an AST
pub fn extract_document_symbols(program: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for item in &program.items {
        match item {
            Item::Function(func) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: func.name.name.clone(),
                    detail: Some(format_function_signature(func)),
                    kind: SymbolKind::FUNCTION,
                    range: Range::default(), // Would need span info
                    selection_range: Range::default(),
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Item::Statement(Stmt::VarDecl(var_decl)) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: var_decl.name.name.clone(),
                    detail: Some(format!("{:?}", var_decl.type_ref)),
                    kind: SymbolKind::VARIABLE,
                    range: Range::default(),
                    selection_range: Range::default(),
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            Item::TypeAlias(alias) => {
                #[allow(deprecated)]
                symbols.push(DocumentSymbol {
                    name: alias.name.name.clone(),
                    detail: Some(format!("{:?}", alias.type_ref)),
                    kind: SymbolKind::TYPE_PARAMETER,
                    range: Range::default(),
                    selection_range: Range::default(),
                    children: None,
                    tags: None,
                    deprecated: None,
                });
            }
            _ => {}
        }
    }

    symbols
}

/// Format a function signature for display
fn format_function_signature(func: &FunctionDecl) -> String {
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("{}: {:?}", p.name.name, p.type_ref))
        .collect();

    format!(
        "fn {}({}) -> {:?}",
        func.name.name,
        params.join(", "),
        func.return_type
    )
}

/// Find the definition location of a symbol
pub fn find_definition(
    _program: &Program,
    _symbols: &SymbolTable,
    _identifier: &str,
) -> Option<Range> {
    // TODO: Implement once we have position info in symbol table
    None
}

/// Find all references to a symbol
///
/// NOTE: This is a basic implementation that finds all identifier usages by name.
/// Phase 05 will enhance this with proper span-based tracking and cross-file support.
pub fn find_references(program: &Program, _symbols: &SymbolTable, identifier: &str) -> Vec<Range> {
    let mut references = Vec::new();

    // Walk the AST and collect all identifier usages
    for item in &program.items {
        find_references_in_item(item, identifier, &mut references);
    }

    references
}

/// Find references in a program item
fn find_references_in_item(item: &Item, identifier: &str, references: &mut Vec<Range>) {
    match item {
        Item::Function(func) => {
            // Check function name
            if func.name.name == identifier {
                references.push(Range::default()); // Would use func.name.span when available
            }

            // Check parameters
            for param in &func.params {
                if param.name.name == identifier {
                    references.push(Range::default());
                }
            }

            // Check body
            find_references_in_block(&func.body, identifier, references);
        }
        Item::Statement(stmt) => {
            find_references_in_stmt(stmt, identifier, references);
        }
        Item::TypeAlias(alias) => {
            if alias.name.name == identifier {
                references.push(Range::default());
            }
        }
        Item::Import(_) | Item::Export(_) | Item::Extern(_) => {}
        Item::Trait(_) | Item::Impl(_) => {}
    }
}

/// Find references in a block
fn find_references_in_block(block: &Block, identifier: &str, references: &mut Vec<Range>) {
    for stmt in &block.statements {
        find_references_in_stmt(stmt, identifier, references);
    }
}

/// Find references in a statement
fn find_references_in_stmt(stmt: &Stmt, identifier: &str, references: &mut Vec<Range>) {
    match stmt {
        Stmt::VarDecl(var_decl) => {
            if var_decl.name.name == identifier {
                references.push(Range::default());
            }
            find_references_in_expr(&var_decl.init, identifier, references);
        }
        Stmt::FunctionDecl(func) => {
            if func.name.name == identifier {
                references.push(Range::default());
            }
            for param in &func.params {
                if param.name.name == identifier {
                    references.push(Range::default());
                }
            }
            find_references_in_block(&func.body, identifier, references);
        }
        Stmt::If(if_stmt) => {
            find_references_in_expr(&if_stmt.cond, identifier, references);
            find_references_in_block(&if_stmt.then_block, identifier, references);
            if let Some(else_block) = &if_stmt.else_block {
                find_references_in_block(else_block, identifier, references);
            }
        }
        Stmt::While(while_stmt) => {
            find_references_in_expr(&while_stmt.cond, identifier, references);
            find_references_in_block(&while_stmt.body, identifier, references);
        }
        Stmt::For(for_stmt) => {
            find_references_in_stmt(&for_stmt.init, identifier, references);
            find_references_in_expr(&for_stmt.cond, identifier, references);
            find_references_in_stmt(&for_stmt.step, identifier, references);
            find_references_in_block(&for_stmt.body, identifier, references);
        }
        Stmt::ForIn(for_in_stmt) => {
            if for_in_stmt.variable.name == identifier {
                references.push(Range::default());
            }
            find_references_in_expr(&for_in_stmt.iterable, identifier, references);
            find_references_in_block(&for_in_stmt.body, identifier, references);
        }
        Stmt::Return(ret_stmt) => {
            if let Some(expr) = &ret_stmt.value {
                find_references_in_expr(expr, identifier, references);
            }
        }
        Stmt::Expr(expr_stmt) => {
            find_references_in_expr(&expr_stmt.expr, identifier, references);
        }
        Stmt::Assign(assign) => {
            find_references_in_assign_target(&assign.target, identifier, references);
            find_references_in_expr(&assign.value, identifier, references);
        }
        Stmt::CompoundAssign(assign) => {
            find_references_in_assign_target(&assign.target, identifier, references);
            find_references_in_expr(&assign.value, identifier, references);
        }
        Stmt::Increment(inc) => {
            find_references_in_assign_target(&inc.target, identifier, references);
        }
        Stmt::Decrement(dec) => {
            find_references_in_assign_target(&dec.target, identifier, references);
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Find references in an assignment target
fn find_references_in_assign_target(
    target: &AssignTarget,
    identifier: &str,
    references: &mut Vec<Range>,
) {
    match target {
        AssignTarget::Name(name) => {
            if name.name == identifier {
                references.push(Range::default());
            }
        }
        AssignTarget::Index {
            target: expr,
            index,
            ..
        } => {
            find_references_in_expr(expr, identifier, references);
            find_references_in_expr(index, identifier, references);
        }
    }
}

/// Find references in an expression
fn find_references_in_expr(expr: &Expr, identifier: &str, references: &mut Vec<Range>) {
    match expr {
        Expr::Identifier(ident) => {
            if ident.name == identifier {
                references.push(Range::default());
            }
        }
        Expr::Unary(unary) => {
            find_references_in_expr(&unary.expr, identifier, references);
        }
        Expr::Binary(binary) => {
            find_references_in_expr(&binary.left, identifier, references);
            find_references_in_expr(&binary.right, identifier, references);
        }
        Expr::Call(call) => {
            find_references_in_expr(&call.callee, identifier, references);
            for arg in &call.args {
                find_references_in_expr(arg, identifier, references);
            }
        }
        Expr::Index(index) => {
            find_references_in_expr(&index.target, identifier, references);
            find_references_in_expr(&index.index, identifier, references);
        }
        Expr::Member(member) => {
            find_references_in_expr(&member.target, identifier, references);
            // Don't check member.member as it's a field name, not an identifier reference
        }
        Expr::ArrayLiteral(array) => {
            for elem in &array.elements {
                find_references_in_expr(elem, identifier, references);
            }
        }
        Expr::Group(group) => {
            find_references_in_expr(&group.expr, identifier, references);
        }
        Expr::Match(match_expr) => {
            find_references_in_expr(&match_expr.scrutinee, identifier, references);
            for arm in &match_expr.arms {
                find_references_in_expr(&arm.body, identifier, references);
            }
        }
        Expr::Try(try_expr) => {
            find_references_in_expr(&try_expr.expr, identifier, references);
        }
        Expr::Literal(_, _) => {}
    }
}

/// Generate hover information for a symbol
pub fn generate_hover_info(
    program: &Program,
    _symbols: &SymbolTable,
    identifier: &str,
) -> Option<String> {
    // Find function declarations
    for item in &program.items {
        if let Item::Function(func) = item {
            if func.name.name == identifier {
                return Some(format_function_signature(func));
            }
        }
    }

    None
}
