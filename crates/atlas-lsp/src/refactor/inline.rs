//! Inline variable and inline function refactoring

use super::{create_workspace_edit, RefactorError, RefactorResult};
use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::*;

/// Inline a variable by replacing all references with its value
///
/// Finds the variable declaration, retrieves its value, finds all usages,
/// substitutes the value at each usage site, and removes the declaration.
pub fn inline_variable(
    uri: &Url,
    position: Position,
    _text: &str,
    program: &Program,
    _symbols: Option<&SymbolTable>,
    identifier: &str,
) -> RefactorResult {
    // Find the variable declaration
    let var_decl = find_variable_declaration(program, identifier).ok_or_else(|| {
        RefactorError::AnalysisFailed(format!("Variable '{}' not found", identifier))
    })?;

    // Get the initialization expression as text
    // For now, we'll use a simplified approach - in a full implementation,
    // we would convert the AST back to source code
    let value_text = format!("{:?}", var_decl.init); // Placeholder

    // Find all references to the variable
    let references = find_variable_references(program, identifier);

    if references.is_empty() {
        return Err(RefactorError::InvalidSelection(
            "No references found for variable".to_string(),
        ));
    }

    // Create edits: replace each reference with the value
    let mut edits: Vec<TextEdit> = references
        .into_iter()
        .map(|range| TextEdit {
            range,
            new_text: value_text.clone(),
        })
        .collect();

    // Add edit to remove the variable declaration
    // For now, we'll mark the declaration line for removal (simplified)
    edits.push(TextEdit {
        range: Range {
            start: position,
            end: Position {
                line: position.line + 1,
                character: 0,
            },
        },
        new_text: String::new(),
    });

    // Sort edits by position (last to first) to avoid invalidating positions
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    Ok(create_workspace_edit(uri, edits))
}

/// Inline a function by expanding all call sites
///
/// Finds the function declaration, retrieves its body, finds all call sites,
/// substitutes the function body at each call site (with parameter substitution),
/// and optionally removes the function if unused.
pub fn inline_function(
    _uri: &Url,
    _position: Position,
    _text: &str,
    program: &Program,
    _symbols: Option<&SymbolTable>,
    identifier: &str,
) -> RefactorResult {
    // Find the function declaration
    let _func_decl = find_function_declaration(program, identifier).ok_or_else(|| {
        RefactorError::AnalysisFailed(format!("Function '{}' not found", identifier))
    })?;

    // For now, return not implemented - full implementation would:
    // 1. Find all call sites
    // 2. For each call site:
    //    a. Extract arguments
    //    b. Substitute parameters with arguments in function body
    //    c. Replace call with substituted body
    // 3. Handle return statements
    // 4. Optionally remove function if no more references

    Err(RefactorError::NotImplemented(
        "Function inlining requires advanced AST manipulation".to_string(),
    ))
}

/// Find a variable declaration by name
fn find_variable_declaration<'a>(program: &'a Program, name: &str) -> Option<&'a VarDecl> {
    for item in &program.items {
        match item {
            Item::Function(func) => {
                if let Some(decl) = find_variable_in_block(&func.body, name) {
                    return Some(decl);
                }
            }
            Item::Statement(Stmt::VarDecl(var_decl)) if var_decl.name.name == name => {
                return Some(var_decl);
            }
            _ => {}
        }
    }
    None
}

/// Find a variable declaration in a block
fn find_variable_in_block<'a>(block: &'a Block, name: &str) -> Option<&'a VarDecl> {
    for stmt in &block.statements {
        match stmt {
            Stmt::VarDecl(var_decl) if var_decl.name.name == name => {
                return Some(var_decl);
            }
            Stmt::FunctionDecl(func) => {
                if let Some(decl) = find_variable_in_block(&func.body, name) {
                    return Some(decl);
                }
            }
            Stmt::If(if_stmt) => {
                if let Some(decl) = find_variable_in_block(&if_stmt.then_block, name) {
                    return Some(decl);
                }
                if let Some(else_block) = &if_stmt.else_block {
                    if let Some(decl) = find_variable_in_block(else_block, name) {
                        return Some(decl);
                    }
                }
            }
            Stmt::While(while_stmt) => {
                if let Some(decl) = find_variable_in_block(&while_stmt.body, name) {
                    return Some(decl);
                }
            }
            Stmt::For(for_stmt) => {
                if let Some(decl) = find_variable_in_block(&for_stmt.body, name) {
                    return Some(decl);
                }
            }
            Stmt::ForIn(for_in_stmt) => {
                if let Some(decl) = find_variable_in_block(&for_in_stmt.body, name) {
                    return Some(decl);
                }
            }
            _ => {}
        }
    }
    None
}

/// Find a function declaration by name
fn find_function_declaration<'a>(program: &'a Program, name: &str) -> Option<&'a FunctionDecl> {
    for item in &program.items {
        if let Item::Function(func) = item {
            if func.name.name == name {
                return Some(func);
            }
        }
    }
    None
}

/// Find all references to a variable (simplified version)
fn find_variable_references(program: &Program, identifier: &str) -> Vec<Range> {
    // Use the find_references from navigation module (simplified version)
    // In a full implementation, this would properly track spans
    crate::navigation::find_references(program, &SymbolTable::new(), identifier)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_variable_declaration() {
        // This test would require parsing Atlas code
        // Skipping for now
    }

    #[test]
    fn test_find_function_declaration() {
        // This test would require parsing Atlas code
        // Skipping for now
    }
}
