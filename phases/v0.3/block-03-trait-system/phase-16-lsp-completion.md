# Phase 16 — LSP: Completion for Traits

**Block:** 3 (Trait System)
**Depends on:** Phase 15 complete
**Estimated tests added:** 10–14

---

## Objective

Add completion support for trait-related positions:
1. Trait name completion in `impl` blocks (suggest known traits)
2. Type name completion after `for` in `impl Trait for _`
3. Method name completion inside impl bodies (suggest method names from the trait)

---

## Current State (verified after Phase 15)

`crates/atlas-lsp/src/completion.rs`:
- `generate_completions()` produces completions based on cursor context
- Block 2 added `ownership_annotation_completions()` for `own`/`borrow`/`shared`
- `is_in_param_position()` detects parameter context

---

## Context Detection

Three new completion contexts to detect:

**Context 1: After `impl ` (trait name position)**
```atlas
impl |         ← cursor after "impl "
```
→ Suggest all known trait names (built-in + user-defined)

**Context 2: After `impl TraitName for ` (type name position)**
```atlas
impl Display for |    ← cursor after "for "
```
→ Suggest known type names (primitive types + any user types in scope)

**Context 3: Inside `impl Trait for Type { fn ` (method stub)**
```atlas
impl Display for number {
    fn |    ← cursor inside impl, after "fn "
}
```
→ Suggest method names from the `Display` trait

---

## Detection Implementation

```rust
/// Check if cursor is in the trait name position after `impl`
fn is_after_impl_keyword(text: &str, position: usize) -> bool {
    // Look backward from position for `impl` followed by whitespace
    let before = &text[..position];
    let trimmed = before.trim_end();
    // Simple heuristic: last non-whitespace sequence before cursor starts with `impl`
    // (More robust: check token stream)
    trimmed.ends_with("impl") || {
        // Or: `impl SomeName for` pattern
        false
    }
}

/// Get the trait being implemented (for method stub completion)
fn get_impl_trait_at_position(text: &str, position: usize) -> Option<String> {
    // Parse backward from position to find enclosing `impl TraitName for TypeName {`
    // Extract the TraitName
    // ...
    None // implement with regex or simple parsing
}
```

Simple approach: use line-aware text scanning (look at the current line and context lines).
More robust: use the AST from `DocumentState` to find which `ImplBlock` contains the cursor.

**Recommendation:** Use AST-based detection. The `DocumentState` has the parsed AST.
Find the `ImplBlock` (if any) that spans the cursor position. This gives the trait name
exactly without text scanning.

---

## Completions

### Trait name completions

```rust
fn trait_name_completions(trait_registry: &TraitRegistry) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Built-in traits
    for trait_name in &["Copy", "Move", "Drop", "Display", "Debug"] {
        items.push(CompletionItem {
            label: trait_name.to_string(),
            kind: Some(CompletionItemKind::INTERFACE),
            detail: Some(format!("built-in trait")),
            documentation: Some(Documentation::String(
                builtin_trait_doc(trait_name)
            )),
            ..Default::default()
        });
    }

    // User-defined traits
    for (name, _) in &trait_registry.traits {
        if !trait_registry.is_built_in(name) {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::INTERFACE),
                detail: Some("trait".to_string()),
                ..Default::default()
            });
        }
    }

    items
}
```

### Method stub completions

When inside `impl TraitName for Type { fn |`, suggest the trait's required methods:

```rust
fn impl_method_stub_completions(trait_name: &str, trait_registry: &TraitRegistry) -> Vec<CompletionItem> {
    let methods = trait_registry.get_methods(trait_name)?;
    methods.iter().map(|method| {
        // Generate a method stub snippet
        let snippet = format!(
            "{}({}) -> {} {{\n    $0\n}}",
            method.name,
            // params as snippet placeholders
            method.param_types.iter().enumerate()
                .map(|(i, _)| format!("${{{}:param{}}}", i + 1, i + 1))
                .collect::<Vec<_>>().join(", "),
            type_to_string(&method.return_type)
        );
        CompletionItem {
            label: method.name.clone(),
            kind: Some(CompletionItemKind::METHOD),
            detail: Some(format!("required by trait {}", trait_name)),
            insert_text: Some(snippet),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }
    }).collect()
}
```

---

## Tests

Add to `crates/atlas-lsp/tests/completion_tests.rs`:

```rust
#[tokio::test]
async fn test_completion_trait_names_after_impl() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Setup: open file with `impl ` (cursor after "impl ")
    // Request completions at cursor
    // Expect: "Copy", "Move", "Drop", "Display", "Debug" in results
}

#[tokio::test]
async fn test_completion_user_trait_after_impl() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Setup: file with `trait MyTrait { }` then `impl ` (cursor after impl)
    // Expect: "MyTrait" in completions
}

#[tokio::test]
async fn test_completion_method_stubs_in_impl_body() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // Source:
    //   trait Display { fn display(self: Display) -> string; }
    //   impl Display for number { fn | }  ← cursor
    // Expect: "display" in completions with correct snippet
}

#[tokio::test]
async fn test_completion_builtin_traits_have_docs() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // After `impl `, "Copy" completion should have documentation string
}

#[tokio::test]
async fn test_completion_trait_names_classified_as_interface() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // All trait completions should have kind = INTERFACE
}
```

---

## Built-in Trait Documentation Strings

```rust
fn builtin_trait_doc(name: &str) -> String {
    match name {
        "Copy" => "Marker trait for types that can be freely copied (value semantics). \
                   All primitive types implement Copy.".to_string(),
        "Move" => "Marker trait for types that require explicit ownership transfer. \
                   Incompatible with Copy.".to_string(),
        "Drop" => "Trait for types with custom destructor logic. \
                   Implement `drop(self: T) -> void` to define cleanup.".to_string(),
        "Display" => "Trait for types that can be converted to a human-readable string. \
                     Implement `display(self: T) -> string`.".to_string(),
        "Debug" => "Trait for types that can be serialized to a debug string. \
                   Implement `debug_repr(self: T) -> string`.".to_string(),
        _ => String::new(),
    }
}
```

---

## Acceptance Criteria

- [ ] Typing `impl ` triggers trait name completions
- [ ] Built-in traits (`Copy`, `Move`, `Drop`, `Display`, `Debug`) appear in completions
- [ ] User-defined traits from the current file appear in completions
- [ ] Trait completions use `CompletionItemKind::INTERFACE`
- [ ] Inside impl body after `fn `, required method names from the trait are suggested
- [ ] Method stub completions include correct parameter structure
- [ ] All existing completion tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- **No helper functions in LSP tests.** Each test is completely self-contained (CLAUDE.md rule).
- The trait registry must be accessible from the LSP server. The LSP server wraps an
  `Atlas` instance or has access to the typechecker state. Check how the LSP accesses
  type information for existing completions, and follow the same pattern.
- Completion for `for TypeName` (type position) is lower priority — if it's complex,
  skip it and document as a known limitation. Trait name and method stub completion
  are the high-value items.
