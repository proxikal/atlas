//! Code refactoring engine for LSP
//!
//! Provides safe, semantic-preserving refactoring operations:
//! - Extract variable: Extract expression to named variable
//! - Extract function: Extract statements to new function
//! - Inline variable: Replace variable usages with its value
//! - Inline function: Expand function calls inline
//! - Rename symbol: Rename across workspace

use atlas_runtime::ast::*;
use tower_lsp::lsp_types::*;

pub mod extract;
pub mod inline;
pub mod rename;

pub use extract::{extract_function, extract_variable};
pub use inline::{inline_function, inline_variable};
pub use rename::rename_symbol;

/// Refactoring action result
pub type RefactorResult = Result<WorkspaceEdit, RefactorError>;

/// Refactoring errors
#[derive(Debug, Clone, PartialEq)]
pub enum RefactorError {
    /// Invalid selection or range
    InvalidSelection(String),
    /// Name conflict detected
    NameConflict(String),
    /// Type safety violation
    TypeSafetyViolation(String),
    /// Cannot preserve semantics
    SemanticsViolation(String),
    /// AST analysis failed
    AnalysisFailed(String),
    /// Feature not yet implemented
    NotImplemented(String),
}

impl std::fmt::Display for RefactorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefactorError::InvalidSelection(msg) => write!(f, "Invalid selection: {}", msg),
            RefactorError::NameConflict(msg) => write!(f, "Name conflict: {}", msg),
            RefactorError::TypeSafetyViolation(msg) => write!(f, "Type safety: {}", msg),
            RefactorError::SemanticsViolation(msg) => write!(f, "Semantics: {}", msg),
            RefactorError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
            RefactorError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
        }
    }
}

impl std::error::Error for RefactorError {}

/// Generate a unique variable name that doesn't conflict with existing names
pub fn generate_unique_name(base: &str, existing_names: &[String]) -> String {
    if !existing_names.contains(&base.to_string()) {
        return base.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", base, counter);
        if !existing_names.contains(&candidate) {
            return candidate;
        }
        counter += 1;
    }
}

/// Extract all identifier names from a program for conflict checking
pub fn extract_all_names(program: &Program) -> Vec<String> {
    let mut names = Vec::new();

    for item in &program.items {
        match item {
            Item::Function(func) => {
                names.push(func.name.name.clone());
                // Add parameter names
                for param in &func.params {
                    names.push(param.name.name.clone());
                }
                // Extract names from function body
                extract_names_from_block(&func.body, &mut names);
            }
            Item::Statement(stmt) => {
                extract_names_from_stmt(stmt, &mut names);
            }
            Item::TypeAlias(alias) => {
                names.push(alias.name.name.clone());
            }
            Item::Import(_) | Item::Export(_) | Item::Extern(_) => {
                // Skip for now
            }
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl refactoring handled in Block 3
            }
        }
    }

    names
}

/// Extract names from a block
fn extract_names_from_block(block: &Block, names: &mut Vec<String>) {
    for stmt in &block.statements {
        extract_names_from_stmt(stmt, names);
    }
}

/// Extract names from a statement
fn extract_names_from_stmt(stmt: &Stmt, names: &mut Vec<String>) {
    match stmt {
        Stmt::VarDecl(var_decl) => {
            names.push(var_decl.name.name.clone());
            extract_names_from_expr(&var_decl.init, names);
        }
        Stmt::FunctionDecl(func) => {
            names.push(func.name.name.clone());
            for param in &func.params {
                names.push(param.name.name.clone());
            }
            extract_names_from_block(&func.body, names);
        }
        Stmt::If(if_stmt) => {
            extract_names_from_expr(&if_stmt.cond, names);
            extract_names_from_block(&if_stmt.then_block, names);
            if let Some(else_block) = &if_stmt.else_block {
                extract_names_from_block(else_block, names);
            }
        }
        Stmt::While(while_stmt) => {
            extract_names_from_expr(&while_stmt.cond, names);
            extract_names_from_block(&while_stmt.body, names);
        }
        Stmt::For(for_stmt) => {
            extract_names_from_stmt(&for_stmt.init, names);
            extract_names_from_expr(&for_stmt.cond, names);
            extract_names_from_stmt(&for_stmt.step, names);
            extract_names_from_block(&for_stmt.body, names);
        }
        Stmt::ForIn(for_in_stmt) => {
            names.push(for_in_stmt.variable.name.clone());
            extract_names_from_expr(&for_in_stmt.iterable, names);
            extract_names_from_block(&for_in_stmt.body, names);
        }
        Stmt::Return(ret_stmt) => {
            if let Some(expr) = &ret_stmt.value {
                extract_names_from_expr(expr, names);
            }
        }
        Stmt::Expr(expr_stmt) => {
            extract_names_from_expr(&expr_stmt.expr, names);
        }
        Stmt::Assign(assign) => {
            extract_names_from_assign_target(&assign.target, names);
            extract_names_from_expr(&assign.value, names);
        }
        Stmt::CompoundAssign(assign) => {
            extract_names_from_assign_target(&assign.target, names);
            extract_names_from_expr(&assign.value, names);
        }
        Stmt::Increment(inc) => {
            extract_names_from_assign_target(&inc.target, names);
        }
        Stmt::Decrement(dec) => {
            extract_names_from_assign_target(&dec.target, names);
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

/// Extract names from an assignment target
fn extract_names_from_assign_target(target: &AssignTarget, names: &mut Vec<String>) {
    match target {
        AssignTarget::Name(name) => {
            names.push(name.name.clone());
        }
        AssignTarget::Index {
            target: expr,
            index,
            ..
        } => {
            extract_names_from_expr(expr, names);
            extract_names_from_expr(index, names);
        }
    }
}

/// Extract names from an expression (for nested function declarations, etc.)
fn extract_names_from_expr(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::Identifier(ident) => {
            names.push(ident.name.clone());
        }
        Expr::Unary(unary) => {
            extract_names_from_expr(&unary.expr, names);
        }
        Expr::Binary(binary) => {
            extract_names_from_expr(&binary.left, names);
            extract_names_from_expr(&binary.right, names);
        }
        Expr::Call(call) => {
            extract_names_from_expr(&call.callee, names);
            for arg in &call.args {
                extract_names_from_expr(arg, names);
            }
        }
        Expr::Index(index) => {
            extract_names_from_expr(&index.target, names);
            extract_names_from_expr(&index.index, names);
        }
        Expr::Member(member) => {
            extract_names_from_expr(&member.target, names);
        }
        Expr::ArrayLiteral(array) => {
            for elem in &array.elements {
                extract_names_from_expr(elem, names);
            }
        }
        Expr::Group(group) => {
            extract_names_from_expr(&group.expr, names);
        }
        Expr::Match(match_expr) => {
            extract_names_from_expr(&match_expr.scrutinee, names);
            for arm in &match_expr.arms {
                extract_names_from_expr(&arm.body, names);
            }
        }
        Expr::Try(try_expr) => {
            extract_names_from_expr(&try_expr.expr, names);
        }
        Expr::Literal(_, _) => {}
    }
}

/// Create a WorkspaceEdit from a single file text edit
pub fn create_workspace_edit(uri: &Url, edits: Vec<TextEdit>) -> WorkspaceEdit {
    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), edits);

    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

/// Validate that a name is a valid identifier
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Check first character is letter or underscore
    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Check remaining characters are alphanumeric or underscore
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if a name is a reserved keyword
pub fn is_reserved_keyword(name: &str) -> bool {
    matches!(
        name,
        "let"
            | "var"
            | "fn"
            | "if"
            | "else"
            | "while"
            | "for"
            | "in"
            | "return"
            | "break"
            | "continue"
            | "match"
            | "type"
            | "import"
            | "export"
            | "try"
            | "catch"
            | "async"
            | "await"
            | "yield"
            | "true"
            | "false"
            | "null"
    )
}

/// Validate a new name for refactoring
pub fn validate_new_name(name: &str) -> Result<(), RefactorError> {
    if !is_valid_identifier(name) {
        return Err(RefactorError::NameConflict(format!(
            "'{}' is not a valid identifier",
            name
        )));
    }

    if is_reserved_keyword(name) {
        return Err(RefactorError::NameConflict(format!(
            "'{}' is a reserved keyword",
            name
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_name() {
        let existing = vec!["foo".to_string(), "foo_1".to_string()];
        assert_eq!(generate_unique_name("bar", &existing), "bar");
        assert_eq!(generate_unique_name("foo", &existing), "foo_2");
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("foo-bar"));
    }

    #[test]
    fn test_is_reserved_keyword() {
        assert!(is_reserved_keyword("let"));
        assert!(is_reserved_keyword("fn"));
        assert!(!is_reserved_keyword("foo"));
    }

    #[test]
    fn test_validate_new_name() {
        assert!(validate_new_name("foo").is_ok());
        assert!(validate_new_name("let").is_err());
        assert!(validate_new_name("123").is_err());
    }
}
