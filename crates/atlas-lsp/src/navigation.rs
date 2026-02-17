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
pub fn find_references(
    _program: &Program,
    _symbols: &SymbolTable,
    _identifier: &str,
) -> Vec<Range> {
    // TODO: Implement once we have position info in AST
    Vec::new()
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
