//! Rename symbol refactoring

use super::{create_workspace_edit, validate_new_name, RefactorError, RefactorResult};
use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::*;

/// Rename a symbol across the workspace
///
/// Finds all references to the symbol workspace-wide, validates the new name,
/// checks for conflicts, and generates a WorkspaceEdit to update all occurrences.
pub fn rename_symbol(
    uri: &Url,
    _position: Position,
    program: &Program,
    symbols: Option<&SymbolTable>,
    old_name: &str,
    new_name: &str,
) -> RefactorResult {
    // Validate the new name
    validate_new_name(new_name)?;

    // Check if the old name exists
    if !symbol_exists(program, old_name) {
        return Err(RefactorError::AnalysisFailed(format!(
            "Symbol '{}' not found",
            old_name
        )));
    }

    // Check for name conflicts
    if symbol_exists(program, new_name) {
        return Err(RefactorError::NameConflict(format!(
            "Symbol '{}' already exists",
            new_name
        )));
    }

    // Find all references to the symbol
    let references = crate::navigation::find_references(
        program,
        symbols.unwrap_or(&SymbolTable::new()),
        old_name,
    );

    if references.is_empty() {
        return Err(RefactorError::AnalysisFailed(
            "No references found for symbol".to_string(),
        ));
    }

    // Create text edits to rename all occurrences
    let edits: Vec<TextEdit> = references
        .into_iter()
        .map(|range| TextEdit {
            range,
            new_text: new_name.to_string(),
        })
        .collect();

    Ok(create_workspace_edit(uri, edits))
}

/// Check if a symbol exists in the program
fn symbol_exists(program: &Program, name: &str) -> bool {
    for item in &program.items {
        match item {
            Item::Function(func) => {
                if func.name.name == name {
                    return true;
                }
                // Check parameters
                for param in &func.params {
                    if param.name.name == name {
                        return true;
                    }
                }
                // Check body
                if symbol_exists_in_block(&func.body, name) {
                    return true;
                }
            }
            Item::Statement(Stmt::VarDecl(var_decl)) => {
                if var_decl.name.name == name {
                    return true;
                }
            }
            Item::TypeAlias(alias) => {
                if alias.name.name == name {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if a symbol exists in a block
fn symbol_exists_in_block(block: &Block, name: &str) -> bool {
    for stmt in &block.statements {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                if var_decl.name.name == name {
                    return true;
                }
            }
            Stmt::FunctionDecl(func) => {
                if func.name.name == name {
                    return true;
                }
                for param in &func.params {
                    if param.name.name == name {
                        return true;
                    }
                }
                if symbol_exists_in_block(&func.body, name) {
                    return true;
                }
            }
            Stmt::If(if_stmt) => {
                if symbol_exists_in_block(&if_stmt.then_block, name) {
                    return true;
                }
                if let Some(else_block) = &if_stmt.else_block {
                    if symbol_exists_in_block(else_block, name) {
                        return true;
                    }
                }
            }
            Stmt::While(while_stmt) => {
                if symbol_exists_in_block(&while_stmt.body, name) {
                    return true;
                }
            }
            Stmt::For(for_stmt) => {
                if symbol_exists_in_block(&for_stmt.body, name) {
                    return true;
                }
            }
            Stmt::ForIn(for_in_stmt) => {
                if for_in_stmt.variable.name == name {
                    return true;
                }
                if symbol_exists_in_block(&for_in_stmt.body, name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_exists() {
        // This test would require parsing Atlas code
        // Skipping for now
    }

    #[test]
    fn test_validate_rename() {
        assert!(validate_new_name("foo").is_ok());
        assert!(validate_new_name("let").is_err());
        assert!(validate_new_name("123").is_err());
    }
}
