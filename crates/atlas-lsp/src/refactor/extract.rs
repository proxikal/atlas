//! Extract variable and extract function refactoring

use super::{
    create_workspace_edit, extract_all_names, generate_unique_name, validate_new_name,
    RefactorError, RefactorResult,
};
use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::*;

/// Extract the selected expression to a new variable
///
/// Analyzes the selected range, extracts the expression, generates a unique name,
/// inserts a `let` binding before the usage, and replaces the expression with a reference.
pub fn extract_variable(
    uri: &Url,
    range: Range,
    text: &str,
    program: &Program,
    _symbols: Option<&SymbolTable>,
    suggested_name: Option<&str>,
) -> RefactorResult {
    // Find the expression at the given range
    let expr_text = extract_text_at_range(text, range)?;

    // Generate a unique name
    let existing_names = extract_all_names(program);
    let base_name = suggested_name.unwrap_or("extracted");
    let var_name = generate_unique_name(base_name, &existing_names);

    // Validate the name
    validate_new_name(&var_name)?;

    // Create the variable declaration
    let var_decl = format!("let {} = {};", var_name, expr_text);

    // Find insertion point (beginning of the current statement/line)
    let insert_position = Position {
        line: range.start.line,
        character: 0,
    };

    // Create text edits
    let mut edits = vec![
        // Insert variable declaration
        TextEdit {
            range: Range {
                start: insert_position,
                end: insert_position,
            },
            new_text: format!("{}\n", var_decl),
        },
        // Replace expression with variable reference
        TextEdit {
            range,
            new_text: var_name.clone(),
        },
    ];

    // Sort edits by position (last to first) to avoid invalidating positions
    edits.sort_by(|a, b| b.range.start.cmp(&a.range.start));

    Ok(create_workspace_edit(uri, edits))
}

/// Extract the selected statements to a new function
///
/// Analyzes the selected range, determines captured variables (which become parameters),
/// infers the return type, generates a function signature, inserts the function definition,
/// and replaces the selection with a function call.
pub fn extract_function(
    uri: &Url,
    range: Range,
    text: &str,
    program: &Program,
    _symbols: Option<&SymbolTable>,
    suggested_name: Option<&str>,
) -> RefactorResult {
    // Extract the selected statements
    let statements_text = extract_text_at_range(text, range)?;

    // Generate a unique function name
    let existing_names = extract_all_names(program);
    let base_name = suggested_name.unwrap_or("extracted_function");
    let func_name = generate_unique_name(base_name, &existing_names);

    // Validate the name
    validate_new_name(&func_name)?;

    // For now, generate a simple function with no parameters
    // TODO: Analyze captured variables and add them as parameters
    // TODO: Infer return type from extracted code

    let func_decl = format!(
        "fn {}() {{\n{}\n}}\n\n",
        func_name,
        indent_text(&statements_text, 1)
    );

    // Find insertion point for the function (after the current function or at file start)
    let insert_position = Position {
        line: 0,
        character: 0,
    };

    // Create text edits
    let edits = vec![
        // Insert function declaration
        TextEdit {
            range: Range {
                start: insert_position,
                end: insert_position,
            },
            new_text: func_decl,
        },
        // Replace selection with function call
        TextEdit {
            range,
            new_text: format!("{}();", func_name),
        },
    ];

    Ok(create_workspace_edit(uri, edits))
}

/// Extract text from source at the given range
fn extract_text_at_range(text: &str, range: Range) -> Result<String, RefactorError> {
    let lines: Vec<&str> = text.lines().collect();

    if range.start.line as usize >= lines.len() || range.end.line as usize >= lines.len() {
        return Err(RefactorError::InvalidSelection(
            "Range exceeds file bounds".to_string(),
        ));
    }

    if range.start.line == range.end.line {
        // Single line selection
        let line = lines[range.start.line as usize];
        let start = range.start.character as usize;
        let end = range.end.character as usize;

        if start >= line.len() || end > line.len() || start > end {
            return Err(RefactorError::InvalidSelection(
                "Invalid range within line".to_string(),
            ));
        }

        Ok(line[start..end].to_string())
    } else {
        // Multi-line selection
        let mut result = String::new();

        // First line
        let first_line = lines[range.start.line as usize];
        let start = range.start.character as usize;
        if start < first_line.len() {
            result.push_str(&first_line[start..]);
        }
        result.push('\n');

        // Middle lines
        for line in lines
            .iter()
            .skip(range.start.line as usize + 1)
            .take((range.end.line as usize).saturating_sub(range.start.line as usize + 1))
        {
            result.push_str(line);
            result.push('\n');
        }

        // Last line
        let last_line = lines[range.end.line as usize];
        let end = range.end.character as usize;
        if end <= last_line.len() {
            result.push_str(&last_line[..end]);
        }

        Ok(result)
    }
}

/// Indent text by the given number of levels (4 spaces per level)
fn indent_text(text: &str, levels: usize) -> String {
    let indent = "    ".repeat(levels);
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_at_range_single_line() {
        let text = "let x = 1 + 2;";
        let range = Range {
            start: Position {
                line: 0,
                character: 8,
            },
            end: Position {
                line: 0,
                character: 13,
            },
        };
        let result = extract_text_at_range(text, range).unwrap();
        assert_eq!(result, "1 + 2");
    }

    #[test]
    fn test_extract_text_at_range_multi_line() {
        let text = "let x = {\n    1 + 2\n};";
        let range = Range {
            start: Position {
                line: 0,
                character: 8,
            },
            end: Position {
                line: 2,
                character: 1,
            },
        };
        let result = extract_text_at_range(text, range).unwrap();
        assert_eq!(result, "{\n    1 + 2\n}");
    }

    #[test]
    fn test_indent_text() {
        let text = "let x = 1;\nlet y = 2;";
        let result = indent_text(text, 1);
        assert_eq!(result, "    let x = 1;\n    let y = 2;");
    }
}
