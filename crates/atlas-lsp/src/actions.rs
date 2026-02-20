//! Code actions provider for LSP
//!
//! Provides:
//! - Quick fixes for diagnostics (5+ types)
//! - Refactoring actions (3+ types)
//! - Source actions (organize imports, etc.)

use atlas_runtime::ast::*;
use atlas_runtime::diagnostic::error_codes;
use atlas_runtime::symbol::SymbolTable;
use atlas_runtime::Diagnostic;
use tower_lsp::lsp_types::*;

/// Code action kind constants
pub mod action_kinds {
    use tower_lsp::lsp_types::CodeActionKind;

    pub fn quick_fix() -> CodeActionKind {
        CodeActionKind::QUICKFIX
    }

    pub fn refactor() -> CodeActionKind {
        CodeActionKind::REFACTOR
    }

    pub fn refactor_extract() -> CodeActionKind {
        CodeActionKind::REFACTOR_EXTRACT
    }

    pub fn refactor_inline() -> CodeActionKind {
        CodeActionKind::REFACTOR_INLINE
    }

    pub fn refactor_rewrite() -> CodeActionKind {
        CodeActionKind::REFACTOR_REWRITE
    }

    pub fn source() -> CodeActionKind {
        CodeActionKind::SOURCE
    }

    pub fn source_organize_imports() -> CodeActionKind {
        CodeActionKind::SOURCE_ORGANIZE_IMPORTS
    }
}

/// Generate code actions for the given context
pub fn generate_code_actions(
    uri: &Url,
    range: Range,
    context: &CodeActionContext,
    text: &str,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
    diagnostics: &[Diagnostic],
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Generate quick fixes for diagnostics in the range
    for diag in &context.diagnostics {
        if let Some(quick_fixes) = generate_quick_fixes(uri, text, diag, diagnostics) {
            actions.extend(quick_fixes);
        }
    }

    // Generate refactoring actions if we have AST
    if let Some(program) = ast {
        actions.extend(generate_refactoring_actions(uri, range, text, program, symbols));
    }

    // Generate source actions
    actions.extend(generate_source_actions(uri, text, ast));

    actions
}

/// Generate quick fixes for a diagnostic
fn generate_quick_fixes(
    uri: &Url,
    text: &str,
    lsp_diag: &tower_lsp::lsp_types::Diagnostic,
    _atlas_diagnostics: &[Diagnostic],
) -> Option<Vec<CodeActionOrCommand>> {
    let code = match &lsp_diag.code {
        Some(NumberOrString::String(s)) => s.as_str(),
        _ => return None,
    };

    let mut actions = Vec::new();

    match code {
        // AT0002: Undefined symbol - suggest declaration
        c if c == error_codes::UNDEFINED_SYMBOL => {
            if let Some(action) = fix_undefined_symbol(uri, text, lsp_diag) {
                actions.push(action);
            }
        }

        // AT1002: Unterminated string - add closing quote
        c if c == error_codes::UNTERMINATED_STRING => {
            if let Some(action) = fix_unterminated_string(uri, lsp_diag) {
                actions.push(action);
            }
        }

        // AT3003: Immutable assignment - suggest var instead of let
        c if c == error_codes::IMMUTABLE_ASSIGNMENT => {
            if let Some(action) = fix_immutable_assignment(uri, text, lsp_diag) {
                actions.push(action);
            }
        }

        // AT3005: Arity mismatch - remove or add arguments
        c if c == error_codes::ARITY_MISMATCH => {
            // Suggest checking function signature
            actions.push(create_diagnostic_action(
                "Check function signature",
                uri,
                lsp_diag,
            ));
        }

        // AT2001: Unused variable - prefix with underscore or remove
        c if c == error_codes::UNUSED_VARIABLE => {
            if let Some(action) = fix_unused_variable(uri, text, lsp_diag) {
                actions.push(action);
            }
            if let Some(action) = remove_unused_variable(uri, text, lsp_diag) {
                actions.push(action);
            }
        }

        // AT2008: Unused import - remove it
        c if c == error_codes::UNUSED_IMPORT => {
            if let Some(action) = remove_unused_import(uri, text, lsp_diag) {
                actions.push(action);
            }
        }

        // AT5002: Module not found - suggest creating it
        c if c == error_codes::MODULE_NOT_FOUND => {
            actions.push(create_diagnostic_action(
                "Create missing module",
                uri,
                lsp_diag,
            ));
        }

        // AT3027: Non-exhaustive match - add wildcard arm
        c if c == error_codes::NON_EXHAUSTIVE_MATCH => {
            if let Some(action) = add_wildcard_arm(uri, text, lsp_diag) {
                actions.push(action);
            }
        }

        _ => {}
    }

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

/// Fix undefined symbol by declaring a variable
fn fix_undefined_symbol(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    // Extract the undefined identifier from the diagnostic message
    let identifier = extract_identifier_from_message(&diag.message)?;

    // Find the line where the error occurs
    let lines: Vec<&str> = text.lines().collect();
    let line_num = diag.range.start.line as usize;

    if line_num >= lines.len() {
        return None;
    }

    // Create a variable declaration before this line
    let indent = get_line_indent(lines[line_num]);
    let declaration = format!("{}let {} = null; // TODO: initialize\n", indent, identifier);

    let insert_pos = Position {
        line: diag.range.start.line,
        character: 0,
    };

    let edit = TextEdit {
        range: Range {
            start: insert_pos,
            end: insert_pos,
        },
        new_text: declaration,
    };

    Some(create_code_action(
        format!("Declare variable '{}'", identifier),
        uri.clone(),
        vec![edit],
        action_kinds::quick_fix(),
        Some(diag.clone()),
    ))
}

/// Fix unterminated string by adding closing quote
fn fix_unterminated_string(
    uri: &Url,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    let end_pos = diag.range.end;

    let edit = TextEdit {
        range: Range {
            start: end_pos,
            end: end_pos,
        },
        new_text: "\"".to_string(),
    };

    Some(create_code_action(
        "Add closing quote".to_string(),
        uri.clone(),
        vec![edit],
        action_kinds::quick_fix(),
        Some(diag.clone()),
    ))
}

/// Fix immutable assignment by changing let to var
fn fix_immutable_assignment(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    // Find the variable declaration that needs to be changed
    let identifier = extract_identifier_from_message(&diag.message)?;

    // Search for the declaration in the text
    let lines: Vec<&str> = text.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Look for "let <identifier>"
        if let Some(pos) = line.find(&format!("let {}", identifier)) {
            let start = Position {
                line: line_num as u32,
                character: pos as u32,
            };
            let end = Position {
                line: line_num as u32,
                character: (pos + 3) as u32, // "let" is 3 chars
            };

            let edit = TextEdit {
                range: Range { start, end },
                new_text: "var".to_string(),
            };

            return Some(create_code_action(
                format!("Change '{}' to mutable (var)", identifier),
                uri.clone(),
                vec![edit],
                action_kinds::quick_fix(),
                Some(diag.clone()),
            ));
        }
    }

    None
}

/// Fix unused variable by prefixing with underscore
fn fix_unused_variable(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    let identifier = extract_identifier_from_message(&diag.message)?;

    // Already prefixed with underscore?
    if identifier.starts_with('_') {
        return None;
    }

    // Find the declaration
    let lines: Vec<&str> = text.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Look for variable declaration patterns
        for pattern in &[format!("let {}", identifier), format!("var {}", identifier)] {
            if let Some(let_pos) = line.find(pattern.as_str()) {
                // Both "let " and "var " are 4 chars
                let var_start = let_pos + 4;

                let start = Position {
                    line: line_num as u32,
                    character: var_start as u32,
                };
                let end = Position {
                    line: line_num as u32,
                    character: (var_start + identifier.len()) as u32,
                };

                let edit = TextEdit {
                    range: Range { start, end },
                    new_text: format!("_{}", identifier),
                };

                return Some(create_code_action(
                    format!("Prefix '{}' with underscore", identifier),
                    uri.clone(),
                    vec![edit],
                    action_kinds::quick_fix(),
                    Some(diag.clone()),
                ));
            }
        }
    }

    None
}

/// Remove unused variable declaration
fn remove_unused_variable(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    let identifier = extract_identifier_from_message(&diag.message)?;

    let lines: Vec<&str> = text.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Check if this line is just a variable declaration containing the identifier
        let trimmed = line.trim();
        if (trimmed.starts_with("let ") || trimmed.starts_with("var "))
            && trimmed.contains(&identifier)
        {
            // Remove the entire line
            let start = Position {
                line: line_num as u32,
                character: 0,
            };
            let end = Position {
                line: line_num as u32 + 1,
                character: 0,
            };

            let edit = TextEdit {
                range: Range { start, end },
                new_text: String::new(),
            };

            return Some(create_code_action(
                format!("Remove unused variable '{}'", identifier),
                uri.clone(),
                vec![edit],
                action_kinds::quick_fix(),
                Some(diag.clone()),
            ));
        }
    }

    None
}

/// Remove unused import statement
fn remove_unused_import(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    let line_num = diag.range.start.line as usize;
    let lines: Vec<&str> = text.lines().collect();

    if line_num >= lines.len() {
        return None;
    }

    let line = lines[line_num];
    if !line.trim().starts_with("import") {
        return None;
    }

    // Remove the entire import line
    let start = Position {
        line: line_num as u32,
        character: 0,
    };
    let end = Position {
        line: line_num as u32 + 1,
        character: 0,
    };

    let edit = TextEdit {
        range: Range { start, end },
        new_text: String::new(),
    };

    Some(create_code_action(
        "Remove unused import".to_string(),
        uri.clone(),
        vec![edit],
        action_kinds::quick_fix(),
        Some(diag.clone()),
    ))
}

/// Add wildcard arm to non-exhaustive match
fn add_wildcard_arm(
    uri: &Url,
    text: &str,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> Option<CodeActionOrCommand> {
    // Find the closing brace of the match expression
    let lines: Vec<&str> = text.lines().collect();
    let start_line = diag.range.start.line as usize;

    // Look for the closing brace
    let mut brace_count = 0;
    let mut found_match = false;

    for (line_num, line) in lines.iter().enumerate().skip(start_line) {
        for c in line.chars() {
            if c == '{' {
                brace_count += 1;
                found_match = true;
            } else if c == '}' {
                brace_count -= 1;
                if found_match && brace_count == 0 {
                    // Found the closing brace, insert wildcard arm before it
                    let indent = get_line_indent(line);
                    let wildcard = format!("{}    _ => null, // TODO: handle remaining cases\n", indent);

                    let insert_pos = Position {
                        line: line_num as u32,
                        character: 0,
                    };

                    let edit = TextEdit {
                        range: Range {
                            start: insert_pos,
                            end: insert_pos,
                        },
                        new_text: wildcard,
                    };

                    return Some(create_code_action(
                        "Add wildcard arm".to_string(),
                        uri.clone(),
                        vec![edit],
                        action_kinds::quick_fix(),
                        Some(diag.clone()),
                    ));
                }
            }
        }
    }

    None
}

/// Generate refactoring actions based on selection
fn generate_refactoring_actions(
    uri: &Url,
    range: Range,
    text: &str,
    _program: &Program,
    _symbols: Option<&SymbolTable>,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Only suggest refactorings for non-empty selections
    if range.start == range.end {
        return actions;
    }

    // Extract the selected text
    let selected_text = extract_text_in_range(text, range);

    if selected_text.trim().is_empty() {
        return actions;
    }

    // Extract variable refactoring
    if is_valid_expression(&selected_text) {
        actions.push(create_extract_variable_action(uri, range, &selected_text));
    }

    // Extract function refactoring (for statement blocks)
    if contains_statements(&selected_text) {
        actions.push(create_extract_function_action(uri, range, &selected_text));
    }

    // Inline variable (if selection is a single identifier that's a simple assignment)
    if is_simple_identifier(&selected_text) {
        actions.push(create_inline_variable_action(uri, &selected_text));
    }

    // Convert to template string (if selection contains string concatenation)
    if contains_string_concat(&selected_text) {
        if let Some(action) = create_convert_to_template_action(uri, range, &selected_text) {
            actions.push(action);
        }
    }

    actions
}

/// Generate source actions
fn generate_source_actions(
    uri: &Url,
    text: &str,
    _ast: Option<&Program>,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    // Organize imports action
    if text.contains("import") {
        actions.push(create_organize_imports_action(uri, text));
    }

    // Add missing imports (placeholder)
    // This would require more sophisticated analysis

    actions
}

/// Create extract variable refactoring action
fn create_extract_variable_action(
    uri: &Url,
    range: Range,
    _selected_text: &str,
) -> CodeActionOrCommand {
    // Create the refactoring action
    // In a real implementation, this would prompt for variable name

    let var_name = "extracted";
    let _lines_before = range.start.line;

    // This is a placeholder - real implementation would compute proper edits
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Extract to variable '{}'", var_name),
        kind: Some(action_kinds::refactor_extract()),
        diagnostics: None,
        edit: None, // Would be computed with actual variable name
        command: Some(Command {
            title: "Extract Variable".to_string(),
            command: "atlas.extractVariable".to_string(),
            arguments: Some(vec![
                serde_json::to_value(uri.to_string()).unwrap(),
                serde_json::to_value(range).unwrap(),
            ]),
        }),
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

/// Create extract function refactoring action
fn create_extract_function_action(
    uri: &Url,
    range: Range,
    _selected_text: &str,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Extract to function".to_string(),
        kind: Some(action_kinds::refactor_extract()),
        diagnostics: None,
        edit: None,
        command: Some(Command {
            title: "Extract Function".to_string(),
            command: "atlas.extractFunction".to_string(),
            arguments: Some(vec![
                serde_json::to_value(uri.to_string()).unwrap(),
                serde_json::to_value(range).unwrap(),
            ]),
        }),
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

/// Create inline variable refactoring action
fn create_inline_variable_action(uri: &Url, identifier: &str) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Inline variable '{}'", identifier),
        kind: Some(action_kinds::refactor_inline()),
        diagnostics: None,
        edit: None,
        command: Some(Command {
            title: "Inline Variable".to_string(),
            command: "atlas.inlineVariable".to_string(),
            arguments: Some(vec![
                serde_json::to_value(uri.to_string()).unwrap(),
                serde_json::to_value(identifier).unwrap(),
            ]),
        }),
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

/// Create convert to template string action
fn create_convert_to_template_action(
    uri: &Url,
    range: Range,
    text: &str,
) -> Option<CodeActionOrCommand> {
    // Parse the concatenation and convert to template
    let template = convert_concat_to_template(text)?;

    let edit = TextEdit {
        range,
        new_text: template,
    };

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: "Convert to template string".to_string(),
        kind: Some(action_kinds::refactor_rewrite()),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    }))
}

/// Create organize imports action
fn create_organize_imports_action(uri: &Url, text: &str) -> CodeActionOrCommand {
    // Collect and sort imports
    let organized = organize_imports(text);

    // Find the range of existing imports
    let lines: Vec<&str> = text.lines().collect();
    let mut import_start = None;
    let mut import_end = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("import") {
            if import_start.is_none() {
                import_start = Some(i);
            }
            import_end = i + 1;
        }
    }

    let edit = if let Some(start) = import_start {
        TextEdit {
            range: Range {
                start: Position {
                    line: start as u32,
                    character: 0,
                },
                end: Position {
                    line: import_end as u32,
                    character: 0,
                },
            },
            new_text: organized,
        }
    } else {
        // No imports to organize
        TextEdit {
            range: Range::default(),
            new_text: String::new(),
        }
    };

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    CodeActionOrCommand::CodeAction(CodeAction {
        title: "Organize imports".to_string(),
        kind: Some(action_kinds::source_organize_imports()),
        diagnostics: None,
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

// === Helper functions ===

/// Create a simple diagnostic-related action
fn create_diagnostic_action(
    title: &str,
    _uri: &Url,
    diag: &tower_lsp::lsp_types::Diagnostic,
) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(action_kinds::quick_fix()),
        diagnostics: Some(vec![diag.clone()]),
        edit: None,
        command: None,
        is_preferred: Some(false),
        disabled: Some(CodeActionDisabled {
            reason: "Manual action required".to_string(),
        }),
        data: None,
    })
}

/// Create a code action with text edits
fn create_code_action(
    title: String,
    uri: Url,
    edits: Vec<TextEdit>,
    kind: CodeActionKind,
    diagnostic: Option<tower_lsp::lsp_types::Diagnostic>,
) -> CodeActionOrCommand {
    let mut changes = std::collections::HashMap::new();
    changes.insert(uri, edits);

    CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(kind),
        diagnostics: diagnostic.map(|d| vec![d]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

/// Extract identifier from diagnostic message
fn extract_identifier_from_message(message: &str) -> Option<String> {
    // Look for patterns like "undefined variable 'foo'" or "'foo' is not defined"
    // or "variable 'foo'" etc.

    // Try to find quoted identifier
    if let Some(start) = message.find('\'') {
        if let Some(end) = message[start + 1..].find('\'') {
            return Some(message[start + 1..start + 1 + end].to_string());
        }
    }

    // Try to find backtick-quoted identifier
    if let Some(start) = message.find('`') {
        if let Some(end) = message[start + 1..].find('`') {
            return Some(message[start + 1..start + 1 + end].to_string());
        }
    }

    None
}

/// Get indentation of a line
fn get_line_indent(line: &str) -> String {
    let indent_len = line.len() - line.trim_start().len();
    line[..indent_len].to_string()
}

/// Extract text within a range
fn extract_text_in_range(text: &str, range: Range) -> String {
    let lines: Vec<&str> = text.lines().collect();

    if range.start.line == range.end.line {
        let line_num = range.start.line as usize;
        if line_num < lines.len() {
            let line = lines[line_num];
            let start = (range.start.character as usize).min(line.len());
            let end = (range.end.character as usize).min(line.len());
            return line[start..end].to_string();
        }
    } else {
        let mut result = String::new();
        for line_num in range.start.line..=range.end.line {
            let idx = line_num as usize;
            if idx < lines.len() {
                let line = lines[idx];
                if line_num == range.start.line {
                    let start = (range.start.character as usize).min(line.len());
                    result.push_str(&line[start..]);
                } else if line_num == range.end.line {
                    let end = (range.end.character as usize).min(line.len());
                    result.push_str(&line[..end]);
                } else {
                    result.push_str(line);
                }
                result.push('\n');
            }
        }
        return result;
    }

    String::new()
}

/// Check if text looks like a valid expression
fn is_valid_expression(text: &str) -> bool {
    let trimmed = text.trim();

    // Must not be empty
    if trimmed.is_empty() {
        return false;
    }

    // Must not contain statement keywords at the start
    let statement_starters = ["let ", "var ", "if ", "while ", "for ", "return ", "fn "];
    for starter in &statement_starters {
        if trimmed.starts_with(starter) {
            return false;
        }
    }

    // Should not end with semicolon (that's a statement)
    if trimmed.ends_with(';') {
        return false;
    }

    true
}

/// Check if text contains multiple statements
fn contains_statements(text: &str) -> bool {
    // Simple heuristic: multiple lines or semicolons
    text.contains(';') || text.lines().count() > 1
}

/// Check if text is a simple identifier
fn is_simple_identifier(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        && trimmed.chars().next().is_some_and(|c| !c.is_ascii_digit())
}

/// Check if text contains string concatenation
fn contains_string_concat(text: &str) -> bool {
    // Look for patterns like: "..." + variable or variable + "..."
    text.contains("\" + ") || text.contains(" + \"")
}

/// Convert string concatenation to template string
fn convert_concat_to_template(text: &str) -> Option<String> {
    // Simple implementation - could be more sophisticated
    // "Hello, " + name + "!" -> `Hello, ${name}!`

    let mut result = String::from("`");
    let mut remaining = text.trim();

    while !remaining.is_empty() {
        if remaining.starts_with('"') {
            // String literal
            let end = remaining[1..].find('"')?;
            result.push_str(&remaining[1..end + 1]);
            remaining = remaining[end + 2..].trim();

            // Skip the + operator
            if remaining.starts_with('+') {
                remaining = remaining[1..].trim();
            }
        } else {
            // Variable or expression
            let end = remaining
                .find(['+', '"'])
                .unwrap_or(remaining.len());

            let expr = remaining[..end].trim();
            if !expr.is_empty() {
                result.push_str("${");
                result.push_str(expr);
                result.push('}');
            }

            remaining = remaining[end..].trim();

            // Skip the + operator
            if remaining.starts_with('+') {
                remaining = remaining[1..].trim();
            }
        }
    }

    result.push('`');
    Some(result)
}

/// Organize imports by sorting them
fn organize_imports(text: &str) -> String {
    let mut imports: Vec<&str> = text
        .lines()
        .filter(|line| line.trim().starts_with("import"))
        .collect();

    imports.sort();

    let mut result = String::new();
    for import in imports {
        result.push_str(import);
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identifier_single_quote() {
        let msg = "undefined variable 'foo'";
        assert_eq!(extract_identifier_from_message(msg), Some("foo".to_string()));
    }

    #[test]
    fn test_extract_identifier_backtick() {
        let msg = "variable `bar` is not defined";
        assert_eq!(extract_identifier_from_message(msg), Some("bar".to_string()));
    }

    #[test]
    fn test_get_line_indent_spaces() {
        assert_eq!(get_line_indent("    let x = 1;"), "    ");
    }

    #[test]
    fn test_get_line_indent_tabs() {
        assert_eq!(get_line_indent("\t\tlet x = 1;"), "\t\t");
    }

    #[test]
    fn test_is_valid_expression_simple() {
        assert!(is_valid_expression("x + y"));
        assert!(is_valid_expression("foo()"));
        assert!(is_valid_expression("42"));
    }

    #[test]
    fn test_is_valid_expression_not() {
        assert!(!is_valid_expression("let x = 1"));
        assert!(!is_valid_expression("return 42;"));
        assert!(!is_valid_expression(""));
    }

    #[test]
    fn test_is_simple_identifier() {
        assert!(is_simple_identifier("foo"));
        assert!(is_simple_identifier("_bar"));
        assert!(is_simple_identifier("baz_123"));
    }

    #[test]
    fn test_is_simple_identifier_not() {
        assert!(!is_simple_identifier("foo.bar"));
        assert!(!is_simple_identifier("123abc"));
        assert!(!is_simple_identifier(""));
    }

    #[test]
    fn test_contains_string_concat() {
        assert!(contains_string_concat("\"Hello, \" + name"));
        assert!(contains_string_concat("name + \"!\""));
        assert!(!contains_string_concat("x + y"));
    }

    #[test]
    fn test_convert_concat_to_template() {
        let input = "\"Hello, \" + name + \"!\"";
        let expected = "`Hello, ${name}!`";
        assert_eq!(convert_concat_to_template(input), Some(expected.to_string()));
    }

    #[test]
    fn test_organize_imports() {
        let text = "import { z } from \"z_mod\";\nimport { a } from \"a_mod\";";
        let organized = organize_imports(text);
        assert!(organized.starts_with("import { a }"));
    }

    #[test]
    fn test_contains_statements() {
        assert!(contains_statements("let x = 1; let y = 2;"));
        assert!(contains_statements("foo()\nbar()"));
        assert!(!contains_statements("x + y"));
    }

    #[test]
    fn test_extract_text_single_line() {
        let text = "let foo = 42;";
        let range = Range {
            start: Position { line: 0, character: 4 },
            end: Position { line: 0, character: 7 },
        };
        assert_eq!(extract_text_in_range(text, range), "foo");
    }

    #[test]
    fn test_action_kinds() {
        assert_eq!(action_kinds::quick_fix(), CodeActionKind::QUICKFIX);
        assert_eq!(action_kinds::refactor(), CodeActionKind::REFACTOR);
    }
}
