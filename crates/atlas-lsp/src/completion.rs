//! Code completion helpers

use atlas_runtime::ast::*;
use atlas_runtime::symbol::SymbolTable;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, Position,
};

/// Completion items for ownership annotation keywords, shown in parameter position only.
pub fn ownership_annotation_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "own".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Move semantics: caller's binding is invalidated after call.".to_string(),
            )),
            insert_text: Some("own ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "borrow".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Immutable reference: caller retains ownership, no mutation.".to_string(),
            )),
            insert_text: Some("borrow ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "shared".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Ownership annotation".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Shared reference: Arc<T> semantics, requires shared<T> value.".to_string(),
            )),
            insert_text: Some("shared ${1:name}: ${2:Type}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Detect whether the cursor is inside a function parameter list.
///
/// Heuristic: walk backwards from the cursor byte offset. If we encounter `(`
/// before `fn`, we might be in a call. But if `fn` precedes `(`, we are in a
/// parameter definition context. Returns `true` when we are in param-definition
/// position (i.e., after `fn name(`).
pub fn is_in_param_position(text: &str, position: Position) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return false;
    }

    // Build the prefix up to the cursor on the current line
    let line = lines[line_idx];
    let col = (position.character as usize).min(line.len());
    let prefix_on_line = &line[..col];

    // Collect all text up to the cursor (previous lines + prefix)
    let mut prefix = String::new();
    for l in lines.iter().take(line_idx) {
        prefix.push_str(l);
        prefix.push('\n');
    }
    prefix.push_str(prefix_on_line);

    // Walk backwards from the end of the prefix to find context
    let mut paren_depth: i32 = 0;
    let chars: Vec<char> = prefix.chars().collect();
    let mut i = chars.len();

    while i > 0 {
        i -= 1;
        match chars[i] {
            ')' => paren_depth += 1,
            '(' => {
                if paren_depth == 0 {
                    // We found the opening paren — look for `fn` before it
                    // Skip whitespace and the function name
                    let before_paren = &prefix[..chars[..i].iter().collect::<String>().len()];
                    let trimmed = before_paren.trim_end();
                    // The part before the paren should end with an identifier (function name)
                    // and before that should be `fn` keyword
                    let _word_end = trimmed.len();
                    let word_start = trimmed
                        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let before_name = trimmed[..word_start].trim_end();
                    return before_name.ends_with("fn");
                }
                paren_depth -= 1;
            }
            _ => {}
        }
    }

    false
}

/// Format an ownership annotation as a parameter prefix string
fn format_ownership(ownership: &Option<OwnershipAnnotation>) -> &'static str {
    match ownership {
        Some(OwnershipAnnotation::Own) => "own ",
        Some(OwnershipAnnotation::Borrow) => "borrow ",
        Some(OwnershipAnnotation::Shared) => "shared ",
        None => "",
    }
}

/// Generate completion items for keywords
pub fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "let".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Variable declaration".to_string()),
            insert_text: Some("let ${1:name}: ${2:type} = ${3:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "var".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Mutable variable declaration".to_string()),
            insert_text: Some("var ${1:name}: ${2:type} = ${3:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "fn".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Function declaration".to_string()),
            insert_text: Some("fn ${1:name}(${2:params}) -> ${3:type} {\n\t${4}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "if".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("If statement".to_string()),
            insert_text: Some("if (${1:condition}) {\n\t${2}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "while".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("While loop".to_string()),
            insert_text: Some("while (${1:condition}) {\n\t${2}\n}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "for".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("For loop".to_string()),
            insert_text: Some(
                "for (${1:init}; ${2:condition}; ${3:update}) {\n\t${4}\n}".to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "return".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Return statement".to_string()),
            insert_text: Some("return ${1:value};".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "break".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Break statement".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Continue statement".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "true".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean true".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "false".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean false".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "null".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Null value".to_string()),
            ..Default::default()
        },
    ]
}

/// Generate completion items for type keywords
pub fn type_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "number".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Number type".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "string".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("String type".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "bool".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Boolean type".to_string()),
            ..Default::default()
        },
    ]
}

/// Generate completion items for built-in functions
pub fn builtin_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "print".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(value: any) -> null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Print a value to stdout".to_string(),
            )),
            insert_text: Some("print(${1:value})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "len".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[]) -> number".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Get the length of an array".to_string(),
            )),
            insert_text: Some("len(${1:array})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "push".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[], value: T) -> null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Add an element to the end of an array".to_string(),
            )),
            insert_text: Some("push(${1:array}, ${2:value})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
        CompletionItem {
            label: "pop".to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some("fn(array: T[]) -> T | null".to_string()),
            documentation: Some(tower_lsp::lsp_types::Documentation::String(
                "Remove and return the last element of an array".to_string(),
            )),
            insert_text: Some("pop(${1:array})".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        },
    ]
}

/// Generate completion items from symbols in scope
pub fn symbol_completions(program: &Program, _symbols: &SymbolTable) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add functions
    for item in &program.items {
        if let Item::Function(func) = item {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|p| {
                    format!(
                        "{}{}: {:?}",
                        format_ownership(&p.ownership),
                        p.name.name,
                        p.type_ref
                    )
                })
                .collect();

            items.push(CompletionItem {
                label: func.name.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!(
                    "fn({}) -> {:?}",
                    params.join(", "),
                    func.return_type
                )),
                ..Default::default()
            });
        }
    }

    // Add top-level variables
    for item in &program.items {
        if let Item::Statement(Stmt::VarDecl(var)) = item {
            items.push(CompletionItem {
                label: var.name.name.clone(),
                kind: Some(CompletionItemKind::VARIABLE),
                detail: Some(format!("{:?}", var.type_ref)),
                ..Default::default()
            });
        }
    }

    items
}

/// Check if cursor is in the trait name position after `impl `
///
/// Returns true if the text immediately before the cursor (trimmed) ends with `impl`.
pub fn is_after_impl_keyword(text: &str, position: Position) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return false;
    }
    let line = lines[line_idx];
    let col = (position.character as usize).min(line.len());

    let mut prefix = String::new();
    for l in lines.iter().take(line_idx) {
        prefix.push_str(l);
        prefix.push('\n');
    }
    prefix.push_str(&line[..col]);

    let trimmed = prefix.trim_end();
    trimmed.ends_with("impl")
}

/// Check if cursor is inside an impl body after `fn `, and extract the enclosing trait name.
///
/// Looks backward in the text for an `impl TraitName for` pattern before the cursor
/// and checks that the cursor is inside the following `{ ... }` block after `fn `.
pub fn get_impl_trait_for_method_completion(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];
    let col = (position.character as usize).min(line.len());

    // Build prefix up to cursor
    let mut prefix = String::new();
    for l in lines.iter().take(line_idx) {
        prefix.push_str(l);
        prefix.push('\n');
    }
    prefix.push_str(&line[..col]);

    // Check that the cursor is after `fn ` on the current line (inside impl body)
    let line_prefix = &line[..col];
    if !line_prefix.trim_start().starts_with("fn") && !line_prefix.contains(" fn ") {
        return None;
    }

    // Find the most recent `impl <TraitName> for` before the cursor
    // Simple text scan: look for last occurrence of `impl ` in prefix
    let impl_idx = prefix.rfind("impl ")?;
    let after_impl = &prefix[impl_idx + 5..];
    // Extract trait name (first word after `impl `)
    let trait_name: String = after_impl
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if trait_name.is_empty() {
        return None;
    }

    Some(trait_name)
}

/// Completion items for built-in Atlas traits.
pub fn builtin_trait_completions() -> Vec<CompletionItem> {
    let built_ins = [
        ("Copy", "Marker trait for types that can be freely copied (value semantics). All primitive types implement Copy."),
        ("Move", "Marker trait for types that require explicit ownership transfer. Incompatible with Copy."),
        ("Drop", "Trait for types with custom destructor logic. Implement `drop(self: T) -> void` to define cleanup."),
        ("Display", "Trait for types that can be converted to a human-readable string. Implement `display(self: T) -> string`."),
        ("Debug", "Trait for types that can be serialized to a debug string. Implement `debug_repr(self: T) -> string`."),
    ];
    built_ins
        .iter()
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::INTERFACE),
            detail: Some("built-in trait".to_string()),
            documentation: Some(Documentation::String(doc.to_string())),
            ..Default::default()
        })
        .collect()
}

/// Completion items for user-defined trait names found in the AST.
pub fn user_trait_completions(program: &Program) -> Vec<CompletionItem> {
    let built_in_names = ["Copy", "Move", "Drop", "Display", "Debug"];
    program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Trait(trait_decl) = item {
                let name = &trait_decl.name.name;
                if !built_in_names.contains(&name.as_str()) {
                    return Some(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::INTERFACE),
                        detail: Some("trait".to_string()),
                        ..Default::default()
                    });
                }
            }
            None
        })
        .collect()
}

/// Completion items for required method stubs of a trait.
///
/// Given a trait name and the program AST, finds the trait declaration and returns
/// one completion per method with a snippet showing the method signature.
pub fn impl_method_stub_completions(trait_name: &str, program: &Program) -> Vec<CompletionItem> {
    for item in &program.items {
        if let Item::Trait(trait_decl) = item {
            if trait_decl.name.name == trait_name {
                return trait_decl
                    .methods
                    .iter()
                    .map(|method| {
                        let params: Vec<String> = method
                            .params
                            .iter()
                            .enumerate()
                            .map(|(i, p)| {
                                format!(
                                    "${{{}:{}: {}}}",
                                    i + 1,
                                    p.name.name,
                                    format_type_ref_str(&p.type_ref)
                                )
                            })
                            .collect();
                        let snippet = format!(
                            "{}({}) -> {} {{\n\t$0\n}}",
                            method.name.name,
                            params.join(", "),
                            format_type_ref_str(&method.return_type)
                        );
                        CompletionItem {
                            label: method.name.name.clone(),
                            kind: Some(CompletionItemKind::METHOD),
                            detail: Some(format!("required by trait {trait_name}")),
                            insert_text: Some(snippet),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        }
                    })
                    .collect();
            }
        }
    }
    vec![]
}

/// Format a TypeRef as a string (minimal version for snippet generation)
fn format_type_ref_str(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name, _) => name.clone(),
        TypeRef::Array(inner, _) => format!("{}[]", format_type_ref_str(inner)),
        TypeRef::Union { members, .. } => members
            .iter()
            .map(format_type_ref_str)
            .collect::<Vec<_>>()
            .join(" | "),
        TypeRef::Function {
            params,
            return_type,
            ..
        } => {
            let ps: Vec<String> = params.iter().map(format_type_ref_str).collect();
            format!(
                "({}) -> {}",
                ps.join(", "),
                format_type_ref_str(return_type)
            )
        }
        TypeRef::Structural { members, .. } => {
            let fs: Vec<String> = members
                .iter()
                .map(|m| format!("{}: {}", m.name, format_type_ref_str(&m.type_ref)))
                .collect();
            format!("{{ {} }}", fs.join(", "))
        }
        TypeRef::Generic {
            name, type_args, ..
        } => {
            let args: Vec<String> = type_args.iter().map(format_type_ref_str).collect();
            format!("{}<{}>", name, args.join(", "))
        }
        TypeRef::Intersection { members, .. } => members
            .iter()
            .map(format_type_ref_str)
            .collect::<Vec<_>>()
            .join(" & "),
    }
}

/// Generate all completion items for the given cursor position.
///
/// `text` and `position` are used for context detection: ownership annotation
/// completions (`own`, `borrow`, `shared`) are only suggested when the cursor
/// is inside a function parameter list. Other completions are always included.
pub fn generate_completions(
    text: Option<&str>,
    position: Option<Position>,
    program: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Always include keywords and builtins
    items.extend(keyword_completions());
    items.extend(type_completions());
    items.extend(builtin_completions());

    // Ownership annotations only in parameter position
    if let (Some(src), Some(pos)) = (text, position) {
        if is_in_param_position(src, pos) {
            items.extend(ownership_annotation_completions());
        }

        // Trait name completions after `impl `
        if is_after_impl_keyword(src, pos) {
            items.extend(builtin_trait_completions());
            if let Some(prog) = program {
                items.extend(user_trait_completions(prog));
            }
        }

        // Method stub completions inside impl body after `fn `
        if let Some(prog) = program {
            if let Some(trait_name) = get_impl_trait_for_method_completion(src, pos) {
                items.extend(impl_method_stub_completions(&trait_name, prog));
            }
        }
    }

    // Add symbols from current document if available
    if let (Some(prog), Some(syms)) = (program, symbols) {
        items.extend(symbol_completions(prog, syms));
    }

    items
}
