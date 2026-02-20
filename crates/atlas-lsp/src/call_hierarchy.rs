//! Call hierarchy navigation for functions
//!
//! Implements LSP call hierarchy requests:
//! - prepare: Find function at cursor position
//! - incoming calls: Find all callers of this function
//! - outgoing calls: Find all functions called by this function

use crate::index::{span_to_range, SymbolIndex};
use crate::navigation::find_identifier_at_position;
use atlas_runtime::ast::*;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, Position, Range, SymbolKind, Url,
};

/// Prepare call hierarchy by finding the function at the cursor position
///
/// Returns the function definition as a CallHierarchyItem if found
pub fn prepare_call_hierarchy(
    uri: &Url,
    text: &str,
    position: Position,
    ast: Option<&Program>,
) -> Option<Vec<CallHierarchyItem>> {
    let ast = ast?;

    // Find the identifier at the cursor position
    let identifier = find_identifier_at_position(text, position)?;

    // Find the function definition with this name
    for item in &ast.items {
        if let Item::Function(func) = item {
            if func.name.name == identifier {
                // Convert function to CallHierarchyItem
                let range = span_to_range(&func.span, text);
                let selection_range = span_to_range(&func.name.span, text);

                let item = CallHierarchyItem {
                    name: func.name.name.clone(),
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    detail: Some(format_function_signature(func)),
                    uri: uri.clone(),
                    range,
                    selection_range,
                    data: None,
                };

                return Some(vec![item]);
            }
        }
    }

    None
}

/// Find all incoming calls to the function
///
/// Returns a list of functions that call this function and the call sites
pub fn find_incoming_calls(
    params: CallHierarchyIncomingCallsParams,
    index: &SymbolIndex,
    documents: &HashMap<Url, (String, Option<Program>)>,
) -> Option<Vec<CallHierarchyIncomingCall>> {
    let item = &params.item;
    let function_name = &item.name;

    // Find all references to this function
    let references = index.find_references(function_name, false);

    if references.is_empty() {
        return None;
    }

    // Group references by containing function
    let mut callers: HashMap<String, Vec<Range>> = HashMap::new();

    for location in references {
        // Find the containing function for this call site
        if let Some((text, Some(ast))) = documents.get(&location.uri) {
            if let Some(caller_name) = find_containing_function(ast, text, location.range.start) {
                callers.entry(caller_name).or_default().push(location.range);
            }
        }
    }

    // Convert to CallHierarchyIncomingCall
    let mut incoming_calls = Vec::new();

    for (caller_name, call_ranges) in callers {
        // Find the caller function definition
        for (uri, (text, ast)) in documents {
            if let Some(ast) = ast {
                if let Some(caller_func) = find_function_by_name(ast, &caller_name) {
                    let from_range = span_to_range(&caller_func.span, text);
                    let from_selection_range = span_to_range(&caller_func.name.span, text);

                    let from = CallHierarchyItem {
                        name: caller_name.clone(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some(format_function_signature(caller_func)),
                        uri: uri.clone(),
                        range: from_range,
                        selection_range: from_selection_range,
                        data: None,
                    };

                    incoming_calls.push(CallHierarchyIncomingCall {
                        from,
                        from_ranges: call_ranges.clone(),
                    });

                    break;
                }
            }
        }
    }

    if incoming_calls.is_empty() {
        None
    } else {
        Some(incoming_calls)
    }
}

/// Find all outgoing calls from the function
///
/// Returns a list of functions called by this function and the call sites
pub fn find_outgoing_calls(
    params: CallHierarchyOutgoingCallsParams,
    index: &SymbolIndex,
    documents: &HashMap<Url, (String, Option<Program>)>,
) -> Option<Vec<CallHierarchyOutgoingCall>> {
    let item = &params.item;

    // Find the function definition
    let (text, ast) = documents.get(&item.uri)?;
    let ast = ast.as_ref()?;
    let func = find_function_by_name(ast, &item.name)?;

    // Find all function calls in the body
    let mut calls = Vec::new();
    collect_function_calls(&func.body, &mut calls);

    if calls.is_empty() {
        return None;
    }

    // Group calls by target function name
    let mut callees: HashMap<String, Vec<Range>> = HashMap::new();

    for call_expr in calls {
        if let Some(callee_name) = extract_function_name(&call_expr.callee) {
            let range = span_to_range(&call_expr.span, text);
            callees.entry(callee_name).or_default().push(range);
        }
    }

    // Convert to CallHierarchyOutgoingCall
    let mut outgoing_calls = Vec::new();

    for (callee_name, call_ranges) in callees {
        // Find the callee function definition
        let definitions = index.find_definitions(&callee_name);
        let definition = definitions.first();

        if let Some(def) = definition {
            if let Some((callee_text, Some(callee_ast))) = documents.get(&def.location.uri) {
                if let Some(callee_func) = find_function_by_name(callee_ast, &callee_name) {
                    let to_range = span_to_range(&callee_func.span, callee_text);
                    let to_selection_range = span_to_range(&callee_func.name.span, callee_text);

                    let to = CallHierarchyItem {
                        name: callee_name.clone(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        detail: Some(format_function_signature(callee_func)),
                        uri: def.location.uri.clone(),
                        range: to_range,
                        selection_range: to_selection_range,
                        data: None,
                    };

                    outgoing_calls.push(CallHierarchyOutgoingCall {
                        to,
                        from_ranges: call_ranges.clone(),
                    });
                }
            }
        }
    }

    if outgoing_calls.is_empty() {
        None
    } else {
        Some(outgoing_calls)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format function signature for display
fn format_function_signature(func: &FunctionDecl) -> String {
    let mut sig = String::from("fn ");
    sig.push_str(&func.name.name);

    // Type parameters
    if !func.type_params.is_empty() {
        sig.push('<');
        let params: Vec<String> = func.type_params.iter().map(|p| p.name.clone()).collect();
        sig.push_str(&params.join(", "));
        sig.push('>');
    }

    // Parameters
    sig.push('(');
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.name, format_type(&p.type_ref)))
        .collect();
    sig.push_str(&params.join(", "));
    sig.push(')');

    // Return type
    if !matches!(func.return_type, TypeRef::Named(ref name, _) if name == "void") {
        sig.push_str(" -> ");
        sig.push_str(&format_type(&func.return_type));
    }

    sig
}

/// Format a type reference for display
fn format_type(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name, _) => name.clone(),
        TypeRef::Array(elem, _) => format!("{}[]", format_type(elem)),
        TypeRef::Union { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type).collect();
            formatted.join(" | ")
        }
        TypeRef::Intersection { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type).collect();
            formatted.join(" & ")
        }
        TypeRef::Function {
            params,
            return_type,
            ..
        } => {
            let param_types: Vec<String> = params.iter().map(format_type).collect();
            format!(
                "({}) => {}",
                param_types.join(", "),
                format_type(return_type)
            )
        }
        TypeRef::Generic {
            name, type_args, ..
        } => {
            let arg_types: Vec<String> = type_args.iter().map(format_type).collect();
            format!("{}<{}>", name, arg_types.join(", "))
        }
        TypeRef::Structural { .. } => "{ ... }".to_string(),
    }
}

/// Find a function by name in the AST
fn find_function_by_name<'a>(ast: &'a Program, name: &str) -> Option<&'a FunctionDecl> {
    for item in &ast.items {
        if let Item::Function(func) = item {
            if func.name.name == name {
                return Some(func);
            }
        }
    }
    None
}

/// Find the containing function for a position
fn find_containing_function(ast: &Program, text: &str, position: Position) -> Option<String> {
    for item in &ast.items {
        if let Item::Function(func) = item {
            let func_range = span_to_range(&func.span, text);
            if position_in_range(position, func_range) {
                return Some(func.name.name.clone());
            }
        }
    }
    None
}

/// Check if a position is within a range
fn position_in_range(position: Position, range: Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    true
}

/// Collect all function call expressions from a block
fn collect_function_calls(block: &Block, calls: &mut Vec<CallExpr>) {
    for stmt in &block.statements {
        collect_calls_from_stmt(stmt, calls);
    }
}

/// Collect function calls from a statement
fn collect_calls_from_stmt(stmt: &Stmt, calls: &mut Vec<CallExpr>) {
    match stmt {
        Stmt::Expr(expr_stmt) => collect_calls_from_expr(&expr_stmt.expr, calls),
        Stmt::Return(return_stmt) => {
            if let Some(value) = &return_stmt.value {
                collect_calls_from_expr(value, calls);
            }
        }
        Stmt::VarDecl(decl) => {
            collect_calls_from_expr(&decl.init, calls);
        }
        Stmt::If(if_stmt) => {
            collect_calls_from_expr(&if_stmt.cond, calls);
            collect_function_calls(&if_stmt.then_block, calls);
            if let Some(else_block) = &if_stmt.else_block {
                collect_function_calls(else_block, calls);
            }
        }
        Stmt::While(while_stmt) => {
            collect_calls_from_expr(&while_stmt.cond, calls);
            collect_function_calls(&while_stmt.body, calls);
        }
        Stmt::For(for_stmt) => {
            collect_calls_from_expr(&for_stmt.cond, calls);
            collect_calls_from_stmt(&for_stmt.init, calls);
            collect_calls_from_stmt(&for_stmt.step, calls);
            collect_function_calls(&for_stmt.body, calls);
        }
        Stmt::ForIn(for_in_stmt) => {
            collect_calls_from_expr(&for_in_stmt.iterable, calls);
            collect_function_calls(&for_in_stmt.body, calls);
        }
        _ => {}
    }
}

/// Collect function calls from an expression
fn collect_calls_from_expr(expr: &Expr, calls: &mut Vec<CallExpr>) {
    match expr {
        Expr::Call(call_expr) => {
            // Add this call
            calls.push(call_expr.clone());
            // Recursively check arguments
            for arg in &call_expr.args {
                collect_calls_from_expr(arg, calls);
            }
            // Check callee (in case of nested calls like foo()())
            collect_calls_from_expr(&call_expr.callee, calls);
        }
        Expr::Unary(unary) => {
            collect_calls_from_expr(&unary.expr, calls);
        }
        Expr::Binary(binary) => {
            collect_calls_from_expr(&binary.left, calls);
            collect_calls_from_expr(&binary.right, calls);
        }
        Expr::Index(index) => {
            collect_calls_from_expr(&index.target, calls);
            collect_calls_from_expr(&index.index, calls);
        }
        Expr::Member(member) => {
            collect_calls_from_expr(&member.target, calls);
        }
        Expr::ArrayLiteral(array) => {
            for elem in &array.elements {
                collect_calls_from_expr(elem, calls);
            }
        }
        Expr::Group(group) => {
            collect_calls_from_expr(&group.expr, calls);
        }
        Expr::Match(match_expr) => {
            collect_calls_from_expr(&match_expr.scrutinee, calls);
            for arm in &match_expr.arms {
                // Note: Atlas match arms don't have guards in the current AST
                collect_calls_from_expr(&arm.body, calls);
            }
        }
        Expr::Try(try_expr) => {
            collect_calls_from_expr(&try_expr.expr, calls);
        }
        _ => {}
    }
}

/// Extract function name from a callee expression
fn extract_function_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier(ident) => Some(ident.name.clone()),
        Expr::Member(member) => {
            // For method calls like obj.method(), extract the member name
            Some(member.member.name.clone())
        }
        _ => None,
    }
}
