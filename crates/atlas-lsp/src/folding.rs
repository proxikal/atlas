//! Folding range provider
//!
//! Provides code folding for:
//! - Function bodies
//! - Block statements (if, while, for, match)
//! - Multi-line comments
//! - Array literals
//! - Import blocks

use atlas_runtime::ast::*;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};

use crate::symbols::span_to_range;

/// Generate folding ranges for a document
pub fn generate_folding_ranges(text: &str, ast: Option<&Program>) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();

    // Extract comment folding ranges from text
    extract_comment_folds(text, &mut ranges);

    // Extract import folding ranges from text
    extract_import_folds(text, &mut ranges);

    // Extract AST-based folding ranges
    if let Some(program) = ast {
        for item in &program.items {
            extract_item_folds(text, item, &mut ranges);
        }
    }

    // Sort by start line and deduplicate
    ranges.sort_by(|a, b| a.start_line.cmp(&b.start_line));
    ranges.dedup_by(|a, b| a.start_line == b.start_line && a.end_line == b.end_line);

    ranges
}

/// Extract folding ranges from an AST item
fn extract_item_folds(text: &str, item: &Item, ranges: &mut Vec<FoldingRange>) {
    match item {
        Item::Function(func) => {
            extract_function_folds(text, func, ranges);
        }
        Item::Statement(stmt) => {
            extract_statement_folds(text, stmt, ranges);
        }
        Item::TypeAlias(alias) => {
            // Type aliases with structural types can be folded
            if let TypeRef::Structural { .. } = &alias.type_ref {
                let range = span_to_range(text, alias.span);
                if range.end.line > range.start.line {
                    ranges.push(FoldingRange {
                        start_line: range.start.line,
                        start_character: Some(range.start.character),
                        end_line: range.end.line,
                        end_character: Some(range.end.character),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
            }
        }
        Item::Import(import) => {
            // Multi-line imports
            let range = span_to_range(text, import.span);
            if range.end.line > range.start.line {
                ranges.push(FoldingRange {
                    start_line: range.start.line,
                    start_character: Some(range.start.character),
                    end_line: range.end.line,
                    end_character: Some(range.end.character),
                    kind: Some(FoldingRangeKind::Imports),
                    collapsed_text: None,
                });
            }
        }
        Item::Extern(ext) => {
            let range = span_to_range(text, ext.span);
            if range.end.line > range.start.line {
                ranges.push(FoldingRange {
                    start_line: range.start.line,
                    start_character: Some(range.start.character),
                    end_line: range.end.line,
                    end_character: Some(range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }
        Item::Export(export) => {
            if let ExportItem::Function(func) = &export.item {
                extract_function_folds(text, func, ranges);
            }
        }
        Item::Trait(_) | Item::Impl(_) => {
            // Trait/impl folding handled in Block 3
        }
    }
}

/// Extract folding ranges from a function
fn extract_function_folds(text: &str, func: &FunctionDecl, ranges: &mut Vec<FoldingRange>) {
    // Function body is the primary fold
    let body_range = span_to_range(text, func.body.span);
    if body_range.end.line > body_range.start.line {
        ranges.push(FoldingRange {
            start_line: body_range.start.line,
            start_character: Some(body_range.start.character),
            end_line: body_range.end.line,
            end_character: Some(body_range.end.character),
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        });
    }

    // Fold nested structures in the body
    extract_block_folds(text, &func.body, ranges);
}

/// Extract folding ranges from a block
fn extract_block_folds(text: &str, block: &Block, ranges: &mut Vec<FoldingRange>) {
    for stmt in &block.statements {
        extract_statement_folds(text, stmt, ranges);
    }
}

/// Extract folding ranges from a statement
fn extract_statement_folds(text: &str, stmt: &Stmt, ranges: &mut Vec<FoldingRange>) {
    match stmt {
        Stmt::FunctionDecl(func) => {
            extract_function_folds(text, func, ranges);
        }
        Stmt::If(if_stmt) => {
            // Then block
            let then_range = span_to_range(text, if_stmt.then_block.span);
            if then_range.end.line > then_range.start.line {
                ranges.push(FoldingRange {
                    start_line: then_range.start.line,
                    start_character: Some(then_range.start.character),
                    end_line: then_range.end.line,
                    end_character: Some(then_range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            extract_block_folds(text, &if_stmt.then_block, ranges);

            // Else block
            if let Some(else_block) = &if_stmt.else_block {
                let else_range = span_to_range(text, else_block.span);
                if else_range.end.line > else_range.start.line {
                    ranges.push(FoldingRange {
                        start_line: else_range.start.line,
                        start_character: Some(else_range.start.character),
                        end_line: else_range.end.line,
                        end_character: Some(else_range.end.character),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
                extract_block_folds(text, else_block, ranges);
            }
        }
        Stmt::While(while_stmt) => {
            let body_range = span_to_range(text, while_stmt.body.span);
            if body_range.end.line > body_range.start.line {
                ranges.push(FoldingRange {
                    start_line: body_range.start.line,
                    start_character: Some(body_range.start.character),
                    end_line: body_range.end.line,
                    end_character: Some(body_range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            extract_block_folds(text, &while_stmt.body, ranges);
        }
        Stmt::For(for_stmt) => {
            let body_range = span_to_range(text, for_stmt.body.span);
            if body_range.end.line > body_range.start.line {
                ranges.push(FoldingRange {
                    start_line: body_range.start.line,
                    start_character: Some(body_range.start.character),
                    end_line: body_range.end.line,
                    end_character: Some(body_range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            extract_block_folds(text, &for_stmt.body, ranges);
        }
        Stmt::ForIn(for_in) => {
            let body_range = span_to_range(text, for_in.body.span);
            if body_range.end.line > body_range.start.line {
                ranges.push(FoldingRange {
                    start_line: body_range.start.line,
                    start_character: Some(body_range.start.character),
                    end_line: body_range.end.line,
                    end_character: Some(body_range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
            extract_block_folds(text, &for_in.body, ranges);
        }
        Stmt::VarDecl(var) => {
            // Large array literals can be folded
            extract_expression_folds(text, &var.init, ranges);
        }
        Stmt::Expr(expr_stmt) => {
            // Check for match expressions
            if let Expr::Match(match_expr) = &expr_stmt.expr {
                let match_range = span_to_range(text, match_expr.span);
                if match_range.end.line > match_range.start.line {
                    ranges.push(FoldingRange {
                        start_line: match_range.start.line,
                        start_character: Some(match_range.start.character),
                        end_line: match_range.end.line,
                        end_character: Some(match_range.end.character),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
            }
            extract_expression_folds(text, &expr_stmt.expr, ranges);
        }
        // Other statements don't need folding
        _ => {}
    }
}

/// Extract folding ranges from expressions (arrays, match)
fn extract_expression_folds(text: &str, expr: &Expr, ranges: &mut Vec<FoldingRange>) {
    match expr {
        Expr::ArrayLiteral(arr) => {
            let range = span_to_range(text, arr.span);
            if range.end.line > range.start.line {
                ranges.push(FoldingRange {
                    start_line: range.start.line,
                    start_character: Some(range.start.character),
                    end_line: range.end.line,
                    end_character: Some(range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some(format!("[{} items]", arr.elements.len())),
                });
            }
            // Recursively check elements
            for elem in &arr.elements {
                extract_expression_folds(text, elem, ranges);
            }
        }
        Expr::Match(match_expr) => {
            let range = span_to_range(text, match_expr.span);
            if range.end.line > range.start.line {
                ranges.push(FoldingRange {
                    start_line: range.start.line,
                    start_character: Some(range.start.character),
                    end_line: range.end.line,
                    end_character: Some(range.end.character),
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        }
        Expr::Binary(bin) => {
            extract_expression_folds(text, &bin.left, ranges);
            extract_expression_folds(text, &bin.right, ranges);
        }
        Expr::Unary(unary) => {
            extract_expression_folds(text, &unary.expr, ranges);
        }
        Expr::Call(call) => {
            extract_expression_folds(text, &call.callee, ranges);
            for arg in &call.args {
                extract_expression_folds(text, arg, ranges);
            }
        }
        Expr::Index(index) => {
            extract_expression_folds(text, &index.target, ranges);
            extract_expression_folds(text, &index.index, ranges);
        }
        Expr::Member(member) => {
            extract_expression_folds(text, &member.target, ranges);
            if let Some(args) = &member.args {
                for arg in args {
                    extract_expression_folds(text, arg, ranges);
                }
            }
        }
        Expr::Group(group) => {
            extract_expression_folds(text, &group.expr, ranges);
        }
        _ => {}
    }
}

/// Extract comment folding ranges from source text
fn extract_comment_folds(text: &str, ranges: &mut Vec<FoldingRange>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Multi-line comment /* ... */
        if line.starts_with("/*") {
            let start_line = i;
            // Find the end
            while i < lines.len() && !lines[i].contains("*/") {
                i += 1;
            }
            let end_line = i;

            if end_line > start_line {
                ranges.push(FoldingRange {
                    start_line: start_line as u32,
                    start_character: None,
                    end_line: end_line as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: None,
                });
            }
        }
        // Consecutive single-line comments
        else if line.starts_with("//") {
            let start_line = i;
            while i + 1 < lines.len() && lines[i + 1].trim().starts_with("//") {
                i += 1;
            }
            let end_line = i;

            // Only fold if there are 2+ consecutive comment lines
            if end_line > start_line {
                ranges.push(FoldingRange {
                    start_line: start_line as u32,
                    start_character: None,
                    end_line: end_line as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Comment),
                    collapsed_text: None,
                });
            }
        }

        i += 1;
    }
}

/// Extract import block folding ranges
fn extract_import_folds(text: &str, ranges: &mut Vec<FoldingRange>) {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Find consecutive import lines
        if line.starts_with("import ") {
            let start_line = i;
            while i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                if next.starts_with("import ") || next.is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }
            // Skip trailing empty lines
            while i > start_line && lines[i].trim().is_empty() {
                i -= 1;
            }
            let end_line = i;

            // Only fold if there are 2+ import lines
            if end_line > start_line {
                ranges.push(FoldingRange {
                    start_line: start_line as u32,
                    start_character: None,
                    end_line: end_line as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Imports),
                    collapsed_text: None,
                });
            }
        }

        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_comment_folds_multiline() {
        let text = "/*\n * This is a\n * multi-line comment\n */\nlet x = 1;";
        let mut ranges = Vec::new();
        extract_comment_folds(text, &mut ranges);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 3);
        assert_eq!(ranges[0].kind, Some(FoldingRangeKind::Comment));
    }

    #[test]
    fn test_extract_comment_folds_consecutive_single() {
        let text = "// Line 1\n// Line 2\n// Line 3\nlet x = 1;";
        let mut ranges = Vec::new();
        extract_comment_folds(text, &mut ranges);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_extract_comment_folds_single_line_no_fold() {
        let text = "// Just one comment\nlet x = 1;";
        let mut ranges = Vec::new();
        extract_comment_folds(text, &mut ranges);

        assert!(ranges.is_empty());
    }

    #[test]
    fn test_extract_import_folds() {
        let text = "import { foo } from \"mod1\";\nimport { bar } from \"mod2\";\nimport { baz } from \"mod3\";\n\nlet x = 1;";
        let mut ranges = Vec::new();
        extract_import_folds(text, &mut ranges);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
        assert_eq!(ranges[0].kind, Some(FoldingRangeKind::Imports));
    }

    #[test]
    fn test_extract_import_folds_single_no_fold() {
        let text = "import { foo } from \"mod\";\n\nlet x = 1;";
        let mut ranges = Vec::new();
        extract_import_folds(text, &mut ranges);

        assert!(ranges.is_empty());
    }

    #[test]
    fn test_generate_folding_ranges_no_ast() {
        let text = "// Comment 1\n// Comment 2\nlet x = 1;";
        let ranges = generate_folding_ranges(text, None);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].kind, Some(FoldingRangeKind::Comment));
    }
}
