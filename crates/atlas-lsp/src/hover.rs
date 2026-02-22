//! Hover information provider for LSP
//!
//! Provides rich hover information including:
//! - Type signatures for variables and functions
//! - Documentation from doc comments
//! - Type information for expressions

use atlas_runtime::ast::*;
use atlas_runtime::symbol::{SymbolKind, SymbolTable};

/// Format an ownership annotation as a parameter prefix
fn ownership_prefix(ownership: &Option<OwnershipAnnotation>) -> &'static str {
    match ownership {
        Some(OwnershipAnnotation::Own) => "own ",
        Some(OwnershipAnnotation::Borrow) => "borrow ",
        Some(OwnershipAnnotation::Shared) => "shared ",
        None => "",
    }
}

/// Format an ownership annotation as a hover label prefix (with parentheses)
fn ownership_label(ownership: &Option<OwnershipAnnotation>) -> &'static str {
    match ownership {
        Some(OwnershipAnnotation::Own) => "(own parameter) ",
        Some(OwnershipAnnotation::Borrow) => "(borrow parameter) ",
        Some(OwnershipAnnotation::Shared) => "(shared parameter) ",
        None => "(parameter) ",
    }
}
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

/// Generate hover information for a position in the document
pub fn generate_hover(
    text: &str,
    position: Position,
    ast: Option<&Program>,
    symbols: Option<&SymbolTable>,
) -> Option<Hover> {
    // Find the identifier at the cursor position
    let identifier = find_identifier_at_position(text, position)?;
    let identifier_range = find_identifier_range(text, position)?;

    // Try to find information about this identifier
    let mut contents = Vec::new();

    // Check AST for function declarations
    if let Some(program) = ast {
        if let Some(hover_info) = find_function_hover(program, &identifier, text) {
            contents.push(hover_info);
        } else if let Some(hover_info) = find_variable_hover(program, &identifier) {
            contents.push(hover_info);
        } else if let Some(hover_info) = find_type_alias_hover(program, &identifier) {
            contents.push(hover_info);
        } else if let Some(hover_info) = find_parameter_hover(program, &identifier) {
            contents.push(hover_info);
        } else if let Some(hover_info) = find_trait_hover(program, &identifier) {
            contents.push(hover_info);
        } else if let Some(hover_info) = find_impl_hover(program, &identifier) {
            contents.push(hover_info);
        }
    }

    // Check symbol table for additional type information
    if let Some(symbol_table) = symbols {
        if let Some(hover_info) = find_symbol_hover(symbol_table, &identifier) {
            // Only add if we don't already have info
            if contents.is_empty() {
                contents.push(hover_info);
            }
        }
    }

    // Check if it's a builtin function
    if contents.is_empty() {
        if let Some(hover_info) = find_builtin_hover(&identifier) {
            contents.push(hover_info);
        }
    }

    // Check if it's a keyword
    if contents.is_empty() {
        if let Some(hover_info) = find_keyword_hover(&identifier) {
            contents.push(hover_info);
        }
    }

    if contents.is_empty() {
        return None;
    }

    let combined = contents.join("\n\n---\n\n");

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: combined,
        }),
        range: Some(identifier_range),
    })
}

/// Find the identifier at a given position in the source
pub fn find_identifier_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col > line.len() {
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

/// Find the range of the identifier at a given position
fn find_identifier_range(text: &str, position: Position) -> Option<Range> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let col = position.character as usize;

    if col > line.len() {
        return None;
    }

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

    Some(Range {
        start: Position {
            line: position.line,
            character: start as u32,
        },
        end: Position {
            line: position.line,
            character: end as u32,
        },
    })
}

/// Find hover information for a function declaration
fn find_function_hover(program: &Program, identifier: &str, source: &str) -> Option<String> {
    for item in &program.items {
        if let Item::Function(func) = item {
            if func.name.name == identifier {
                let mut hover = String::new();

                // Extract doc comments
                if let Some(doc) = extract_doc_comment(source, func.name.span.start) {
                    hover.push_str(&doc);
                    hover.push_str("\n\n");
                }

                // Format signature
                hover.push_str("```atlas\n");
                hover.push_str(&format_function_signature(func));
                hover.push_str("\n```");

                return Some(hover);
            }
        }
    }
    None
}

/// Find hover information for a variable declaration
fn find_variable_hover(program: &Program, identifier: &str) -> Option<String> {
    for item in &program.items {
        if let Item::Statement(Stmt::VarDecl(var_decl)) = item {
            if var_decl.name.name == identifier {
                let mut hover = String::new();

                // Show mutability and type
                let mutability = if var_decl.mutable { "var" } else { "let" };
                hover.push_str("```atlas\n");
                hover.push_str(&format!("{} {}", mutability, var_decl.name.name));

                if let Some(ref type_ref) = var_decl.type_ref {
                    hover.push_str(&format!(": {}", format_type_ref(type_ref)));
                }

                hover.push_str("\n```");

                return Some(hover);
            }
        }
    }

    // Also check inside function bodies
    for item in &program.items {
        if let Item::Function(func) = item {
            if let Some(hover) = find_variable_in_block(&func.body.statements, identifier) {
                return Some(hover);
            }
        }
    }

    None
}

/// Find variable declaration in a block
fn find_variable_in_block(stmts: &[Stmt], identifier: &str) -> Option<String> {
    for stmt in stmts {
        if let Stmt::VarDecl(var_decl) = stmt {
            if var_decl.name.name == identifier {
                let mutability = if var_decl.mutable { "var" } else { "let" };
                let mut hover = String::new();
                hover.push_str("```atlas\n");
                hover.push_str(&format!("{} {}", mutability, var_decl.name.name));

                if let Some(ref type_ref) = var_decl.type_ref {
                    hover.push_str(&format!(": {}", format_type_ref(type_ref)));
                }

                hover.push_str("\n```");
                return Some(hover);
            }
        }
    }
    None
}

/// Find hover information for a type alias
fn find_type_alias_hover(program: &Program, identifier: &str) -> Option<String> {
    for item in &program.items {
        if let Item::TypeAlias(alias) = item {
            if alias.name.name == identifier {
                let mut hover = String::new();
                hover.push_str("```atlas\n");
                hover.push_str(&format!(
                    "type {} = {}",
                    alias.name.name,
                    format_type_ref(&alias.type_ref)
                ));
                hover.push_str("\n```");
                return Some(hover);
            }
        }
    }
    None
}

/// Find hover information for a parameter (with ownership annotation)
fn find_parameter_hover(program: &Program, identifier: &str) -> Option<String> {
    for item in &program.items {
        if let Item::Function(func) = item {
            for param in &func.params {
                if param.name.name == identifier {
                    let label = ownership_label(&param.ownership);
                    let mut hover = String::new();
                    hover.push_str("```atlas\n");
                    hover.push_str(&format!(
                        "{}{}: {}",
                        label,
                        param.name.name,
                        format_type_ref(&param.type_ref)
                    ));
                    hover.push_str("\n```");
                    return Some(hover);
                }
            }
        }
    }
    None
}

/// Find hover information for a trait declaration
fn find_trait_hover(program: &Program, identifier: &str) -> Option<String> {
    for item in &program.items {
        if let Item::Trait(trait_decl) = item {
            if trait_decl.name.name == identifier {
                let mut hover = String::new();
                hover.push_str(&format!("**(trait)** `{}`\n\n", trait_decl.name.name));
                if !trait_decl.methods.is_empty() {
                    hover.push_str("```atlas\n");
                    for method in &trait_decl.methods {
                        hover.push_str(&format_trait_method_sig(method));
                        hover.push('\n');
                    }
                    hover.push_str("```");
                }
                return Some(hover);
            }
        }
    }
    None
}

/// Find hover information for an impl block (hovering on trait name or type name)
fn find_impl_hover(program: &Program, identifier: &str) -> Option<String> {
    for item in &program.items {
        if let Item::Impl(impl_block) = item {
            let is_trait_name = impl_block.trait_name.name == identifier;
            let is_type_name = impl_block.type_name.name == identifier;
            if is_trait_name || is_type_name {
                let mut hover = String::new();
                hover.push_str(&format!(
                    "**(impl)** `{}` implements `{}`\n\n",
                    impl_block.type_name.name, impl_block.trait_name.name
                ));
                if !impl_block.methods.is_empty() {
                    hover.push_str("```atlas\n");
                    for method in &impl_block.methods {
                        hover.push_str(&format_impl_method_sig(method));
                        hover.push('\n');
                    }
                    hover.push_str("```");
                }
                return Some(hover);
            }
        }
    }
    None
}

/// Format a trait method signature (no body)
fn format_trait_method_sig(method: &TraitMethodSig) -> String {
    let params: Vec<String> = method
        .params
        .iter()
        .map(|p| {
            format!(
                "{}{}: {}",
                ownership_prefix(&p.ownership),
                p.name.name,
                format_type_ref(&p.type_ref)
            )
        })
        .collect();
    format!(
        "fn {}({}) -> {};",
        method.name.name,
        params.join(", "),
        format_type_ref(&method.return_type)
    )
}

/// Format an impl method signature (with body indicator)
fn format_impl_method_sig(method: &ImplMethod) -> String {
    let params: Vec<String> = method
        .params
        .iter()
        .map(|p| {
            format!(
                "{}{}: {}",
                ownership_prefix(&p.ownership),
                p.name.name,
                format_type_ref(&p.type_ref)
            )
        })
        .collect();
    format!(
        "fn {}({}) -> {} {{ ... }}",
        method.name.name,
        params.join(", "),
        format_type_ref(&method.return_type)
    )
}

/// Find hover information from symbol table
fn find_symbol_hover(symbols: &SymbolTable, identifier: &str) -> Option<String> {
    let symbol = symbols.lookup(identifier)?;

    let mut hover = String::new();
    hover.push_str("```atlas\n");

    match symbol.kind {
        SymbolKind::Variable => {
            let mutability = if symbol.mutable { "var" } else { "let" };
            hover.push_str(&format!("{} {}: {:?}", mutability, symbol.name, symbol.ty));
        }
        SymbolKind::Function => {
            hover.push_str(&format!("fn {}", symbol.name));
            hover.push_str(&format!(": {:?}", symbol.ty));
        }
        SymbolKind::Parameter => {
            hover.push_str(&format!("(parameter) {}: {:?}", symbol.name, symbol.ty));
        }
        SymbolKind::Builtin => {
            hover.push_str(&format!("(builtin) {}", symbol.name));
        }
    }

    hover.push_str("\n```");
    Some(hover)
}

/// Find hover information for builtin functions
fn find_builtin_hover(identifier: &str) -> Option<String> {
    let (signature, description) = match identifier {
        // I/O
        "print" => ("fn print(...args: any) -> null", "Prints values to stdout"),
        "println" => (
            "fn println(...args: any) -> null",
            "Prints values to stdout with newline",
        ),
        "input" => (
            "fn input(prompt?: string) -> string",
            "Reads a line from stdin",
        ),

        // Type conversion
        "string" => (
            "fn string(value: any) -> string",
            "Converts a value to string",
        ),
        "number" => (
            "fn number(value: any) -> number",
            "Converts a value to number",
        ),
        "bool" => ("fn bool(value: any) -> bool", "Converts a value to boolean"),
        "int" => (
            "fn int(value: number) -> number",
            "Truncates a number to integer",
        ),

        // Type checking
        "typeof" => (
            "fn typeof(value: any) -> string",
            "Returns the type of a value as a string",
        ),
        "is_number" => (
            "fn is_number(value: any) -> bool",
            "Checks if value is a number",
        ),
        "is_string" => (
            "fn is_string(value: any) -> bool",
            "Checks if value is a string",
        ),
        "is_bool" => (
            "fn is_bool(value: any) -> bool",
            "Checks if value is a boolean",
        ),
        "is_null" => ("fn is_null(value: any) -> bool", "Checks if value is null"),
        "is_array" => (
            "fn is_array(value: any) -> bool",
            "Checks if value is an array",
        ),
        "is_function" => (
            "fn is_function(value: any) -> bool",
            "Checks if value is a function",
        ),

        // Array operations
        "len" => (
            "fn len(collection: array | string | HashMap) -> number",
            "Returns the length of a collection",
        ),
        "push" => (
            "fn push(array: array, value: any) -> array",
            "Adds an element to the end of an array",
        ),
        "pop" => (
            "fn pop(array: array) -> any",
            "Removes and returns the last element",
        ),
        "shift" => (
            "fn shift(array: array) -> any",
            "Removes and returns the first element",
        ),
        "unshift" => (
            "fn unshift(array: array, value: any) -> array",
            "Adds an element to the beginning",
        ),
        "slice" => (
            "fn slice(array: array, start: number, end?: number) -> array",
            "Returns a portion of an array",
        ),
        "concat" => (
            "fn concat(array: array, other: array) -> array",
            "Concatenates two arrays",
        ),
        "reverse" => (
            "fn reverse(array: array) -> array",
            "Reverses an array in place",
        ),
        "sort" => (
            "fn sort(array: array, comparator?: fn) -> array",
            "Sorts an array",
        ),
        "map" => (
            "fn map(array: array, fn: (item: any) -> any) -> array",
            "Maps a function over an array",
        ),
        "filter" => (
            "fn filter(array: array, fn: (item: any) -> bool) -> array",
            "Filters an array by predicate",
        ),
        "reduce" => (
            "fn reduce(array: array, fn: (acc: any, item: any) -> any, initial: any) -> any",
            "Reduces an array to a single value",
        ),
        "find" => (
            "fn find(array: array, fn: (item: any) -> bool) -> any",
            "Finds first element matching predicate",
        ),
        "every" => (
            "fn every(array: array, fn: (item: any) -> bool) -> bool",
            "Tests if all elements match predicate",
        ),
        "some" => (
            "fn some(array: array, fn: (item: any) -> bool) -> bool",
            "Tests if any element matches predicate",
        ),
        "includes" => (
            "fn includes(array: array, value: any) -> bool",
            "Checks if array contains a value",
        ),
        "index_of" => (
            "fn index_of(array: array, value: any) -> number",
            "Returns index of value or -1",
        ),
        "join" => (
            "fn join(array: array, separator?: string) -> string",
            "Joins array elements into a string",
        ),
        "flat" => (
            "fn flat(array: array, depth?: number) -> array",
            "Flattens nested arrays",
        ),

        // String operations
        "split" => (
            "fn split(str: string, separator: string) -> array",
            "Splits a string into an array",
        ),
        "trim" => (
            "fn trim(str: string) -> string",
            "Removes whitespace from both ends",
        ),
        "to_upper" => (
            "fn to_upper(str: string) -> string",
            "Converts string to uppercase",
        ),
        "to_lower" => (
            "fn to_lower(str: string) -> string",
            "Converts string to lowercase",
        ),
        "starts_with" => (
            "fn starts_with(str: string, prefix: string) -> bool",
            "Checks if string starts with prefix",
        ),
        "ends_with" => (
            "fn ends_with(str: string, suffix: string) -> bool",
            "Checks if string ends with suffix",
        ),
        "contains" => (
            "fn contains(str: string, substr: string) -> bool",
            "Checks if string contains substring",
        ),
        "replace" => (
            "fn replace(str: string, from: string, to: string) -> string",
            "Replaces occurrences in string",
        ),
        "char_at" => (
            "fn char_at(str: string, index: number) -> string",
            "Returns character at index",
        ),
        "substring" => (
            "fn substring(str: string, start: number, end?: number) -> string",
            "Returns a substring",
        ),
        "pad_start" => (
            "fn pad_start(str: string, length: number, pad?: string) -> string",
            "Pads string at start",
        ),
        "pad_end" => (
            "fn pad_end(str: string, length: number, pad?: string) -> string",
            "Pads string at end",
        ),
        "repeat" => (
            "fn repeat(str: string, count: number) -> string",
            "Repeats a string",
        ),

        // Math
        "abs" => ("fn abs(x: number) -> number", "Returns absolute value"),
        "floor" => (
            "fn floor(x: number) -> number",
            "Rounds down to nearest integer",
        ),
        "ceil" => (
            "fn ceil(x: number) -> number",
            "Rounds up to nearest integer",
        ),
        "round" => ("fn round(x: number) -> number", "Rounds to nearest integer"),
        "sqrt" => ("fn sqrt(x: number) -> number", "Returns square root"),
        "pow" => (
            "fn pow(base: number, exp: number) -> number",
            "Returns base raised to exp",
        ),
        "min" => (
            "fn min(...values: number) -> number",
            "Returns minimum value",
        ),
        "max" => (
            "fn max(...values: number) -> number",
            "Returns maximum value",
        ),
        "sin" => ("fn sin(x: number) -> number", "Returns sine of x (radians)"),
        "cos" => (
            "fn cos(x: number) -> number",
            "Returns cosine of x (radians)",
        ),
        "tan" => (
            "fn tan(x: number) -> number",
            "Returns tangent of x (radians)",
        ),
        "log" => ("fn log(x: number) -> number", "Returns natural logarithm"),
        "log10" => ("fn log10(x: number) -> number", "Returns base-10 logarithm"),
        "exp" => ("fn exp(x: number) -> number", "Returns e raised to x"),
        "random" => (
            "fn random() -> number",
            "Returns random number between 0 and 1",
        ),
        "random_range" => (
            "fn random_range(min: number, max: number) -> number",
            "Returns random number in range",
        ),

        // HashMap operations
        "keys" => (
            "fn keys(map: HashMap) -> array",
            "Returns array of map keys",
        ),
        "values" => (
            "fn values(map: HashMap) -> array",
            "Returns array of map values",
        ),
        "entries" => (
            "fn entries(map: HashMap) -> array",
            "Returns array of [key, value] pairs",
        ),
        "has_key" => (
            "fn has_key(map: HashMap, key: any) -> bool",
            "Checks if map contains key",
        ),
        "remove" => (
            "fn remove(map: HashMap, key: any) -> any",
            "Removes and returns value for key",
        ),

        // Assertions
        "assert" => (
            "fn assert(condition: bool, message?: string) -> null",
            "Throws if condition is false",
        ),
        "assert_eq" => (
            "fn assert_eq(actual: any, expected: any, message?: string) -> null",
            "Throws if values are not equal",
        ),
        "assert_ne" => (
            "fn assert_ne(actual: any, expected: any, message?: string) -> null",
            "Throws if values are equal",
        ),

        // Time
        "time" => (
            "fn time() -> number",
            "Returns current Unix timestamp in seconds",
        ),
        "sleep" => (
            "fn sleep(ms: number) -> null",
            "Pauses execution for milliseconds",
        ),

        // Error handling
        "error" => (
            "fn error(message: string) -> never",
            "Throws a runtime error",
        ),
        "try_catch" => (
            "fn try_catch(fn: () -> any, handler: (err: string) -> any) -> any",
            "Catches errors from a function",
        ),

        _ => return None,
    };

    let mut hover = String::new();
    hover.push_str(&format!("```atlas\n{}\n```\n\n", signature));
    hover.push_str(description);
    hover.push_str("\n\n*builtin function*");

    Some(hover)
}

/// Find hover information for keywords
fn find_keyword_hover(identifier: &str) -> Option<String> {
    let description = match identifier {
        "let" => "Declares an immutable variable binding.",
        "var" => "Declares a mutable variable binding.",
        "fn" => "Declares a function.",
        "if" => "Conditional branching statement.",
        "else" => "Alternative branch in conditional statement.",
        "while" => "Loop that continues while condition is true.",
        "for" => "Loop that iterates over a range or collection.",
        "in" => "Used in for-in loops to iterate over collections.",
        "return" => "Returns a value from a function.",
        "break" => "Exits the current loop.",
        "continue" => "Skips to the next iteration of a loop.",
        "match" => "Pattern matching expression.",
        "type" => "Declares a type alias.",
        "import" => "Imports symbols from another module.",
        "export" => "Exports symbols from the current module.",
        "from" => "Specifies the source module in an import statement.",
        "true" => "Boolean literal representing true.",
        "false" => "Boolean literal representing false.",
        "null" => "Represents the absence of a value.",
        "extern" => "Declares an external (FFI) function.",
        "as" => "Used for type casts and import aliases.",
        "extends" => "Used in generic constraints.",
        "is" => "Type predicate operator.",
        "own" => "Ownership annotation: parameter takes exclusive ownership of the value.",
        "borrow" => "Ownership annotation: parameter borrows the value without taking ownership.",
        "shared" => {
            "Ownership annotation: parameter receives a shared (reference-counted) reference."
        }
        "trait" => {
            return Some(
                "**trait** — Declares a trait — a named set of method signatures that types can implement.\n\n\
                 ```atlas\n\
                 trait Display {\n    fn display(self: Display) -> string;\n}\n\
                 ```"
                .to_string(),
            );
        }
        "impl" => {
            return Some(
                "**impl** — Implements a trait for a type. All trait methods must be provided with matching signatures.\n\n\
                 ```atlas\n\
                 impl Display for number {\n    fn display(self: number) -> string { return str(self); }\n}\n\
                 ```"
                .to_string(),
            );
        }
        _ => return None,
    };

    Some(format!("**{}** — {}", identifier, description))
}

/// Format a function signature for display
fn format_function_signature(func: &FunctionDecl) -> String {
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| {
            format!(
                "{}{}: {}",
                ownership_prefix(&p.ownership),
                p.name.name,
                format_type_ref(&p.type_ref)
            )
        })
        .collect();

    let return_type = format!(" -> {}", format_type_ref(&func.return_type));

    format!(
        "fn {}({}){}",
        func.name.name,
        params.join(", "),
        return_type
    )
}

/// Format a type reference for display
fn format_type_ref(type_ref: &TypeRef) -> String {
    match type_ref {
        TypeRef::Named(name, _) => name.clone(),
        TypeRef::Array(inner, _) => format!("{}[]", format_type_ref(inner)),
        TypeRef::Union { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type_ref).collect();
            formatted.join(" | ")
        }
        TypeRef::Function {
            params,
            return_type,
            ..
        } => {
            let param_strs: Vec<String> = params.iter().map(format_type_ref).collect();
            format!(
                "({}) -> {}",
                param_strs.join(", "),
                format_type_ref(return_type)
            )
        }
        TypeRef::Structural { members, .. } => {
            let field_strs: Vec<String> = members
                .iter()
                .map(|m| format!("{}: {}", m.name, format_type_ref(&m.type_ref)))
                .collect();
            format!("{{ {} }}", field_strs.join(", "))
        }
        TypeRef::Generic {
            name, type_args, ..
        } => {
            let arg_strs: Vec<String> = type_args.iter().map(format_type_ref).collect();
            format!("{}<{}>", name, arg_strs.join(", "))
        }
        TypeRef::Intersection { members, .. } => {
            let formatted: Vec<String> = members.iter().map(format_type_ref).collect();
            formatted.join(" & ")
        }
    }
}

/// Extract doc comment preceding a position
fn extract_doc_comment(source: &str, position: usize) -> Option<String> {
    // Simple approach: look for /// comments in the lines preceding the position
    let lines: Vec<&str> = source.lines().collect();

    // Find which line the position is on
    let mut current_pos = 0;
    let mut target_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_end = current_pos + line.len() + 1; // +1 for newline
        if position <= line_end {
            target_line = i;
            break;
        }
        current_pos = line_end;
    }

    // Collect doc comments from preceding lines
    let mut doc_comments = Vec::new();

    // Look backwards for doc comments starting from line before target
    for line_idx in (0..target_line).rev() {
        let line = lines.get(line_idx)?;
        let trimmed = line.trim();

        if trimmed.starts_with("///") {
            let content = trimmed.trim_start_matches('/').trim();
            doc_comments.insert(0, content.to_string());
        } else if !trimmed.is_empty() {
            // Stop at non-comment, non-empty line
            break;
        }
    }

    if doc_comments.is_empty() {
        return None;
    }

    Some(doc_comments.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_identifier_at_position() {
        let text = "let foo = 42;";
        let pos = Position {
            line: 0,
            character: 5,
        };
        assert_eq!(
            find_identifier_at_position(text, pos),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_find_identifier_at_start() {
        let text = "foo bar";
        let pos = Position {
            line: 0,
            character: 0,
        };
        assert_eq!(
            find_identifier_at_position(text, pos),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_find_identifier_at_end() {
        let text = "foo bar";
        let pos = Position {
            line: 0,
            character: 6,
        };
        assert_eq!(
            find_identifier_at_position(text, pos),
            Some("bar".to_string())
        );
    }

    #[test]
    fn test_find_identifier_multiline() {
        let text = "let x = 1;\nlet y = 2;";
        let pos = Position {
            line: 1,
            character: 4,
        };
        assert_eq!(
            find_identifier_at_position(text, pos),
            Some("y".to_string())
        );
    }

    #[test]
    fn test_find_identifier_on_operator() {
        let text = "x + y";
        let pos = Position {
            line: 0,
            character: 2,
        };
        assert_eq!(find_identifier_at_position(text, pos), None);
    }

    #[test]
    fn test_builtin_hover_print() {
        let hover = find_builtin_hover("print").unwrap();
        assert!(hover.contains("fn print"));
        assert!(hover.contains("builtin function"));
    }

    #[test]
    fn test_builtin_hover_len() {
        let hover = find_builtin_hover("len").unwrap();
        assert!(hover.contains("fn len"));
        assert!(hover.contains("length"));
    }

    #[test]
    fn test_keyword_hover_let() {
        let hover = find_keyword_hover("let").unwrap();
        assert!(hover.contains("immutable"));
    }

    #[test]
    fn test_keyword_hover_fn() {
        let hover = find_keyword_hover("fn").unwrap();
        assert!(hover.contains("function"));
    }

    #[test]
    fn test_format_type_ref_simple() {
        let type_ref = TypeRef::Named("number".to_string(), atlas_runtime::span::Span::dummy());
        assert_eq!(format_type_ref(&type_ref), "number");
    }

    #[test]
    fn test_format_type_ref_array() {
        let inner = TypeRef::Named("string".to_string(), atlas_runtime::span::Span::dummy());
        let type_ref = TypeRef::Array(Box::new(inner), atlas_runtime::span::Span::dummy());
        assert_eq!(format_type_ref(&type_ref), "string[]");
    }

    #[test]
    fn test_format_type_ref_union() {
        let t1 = TypeRef::Named("number".to_string(), atlas_runtime::span::Span::dummy());
        let t2 = TypeRef::Named("string".to_string(), atlas_runtime::span::Span::dummy());
        let type_ref = TypeRef::Union {
            members: vec![t1, t2],
            span: atlas_runtime::span::Span::dummy(),
        };
        assert_eq!(format_type_ref(&type_ref), "number | string");
    }
}
