# atlas-lsp/src/

LSP server for Atlas. Provides IDE features via tower-lsp.

## File Map

| File | What it does |
|------|-------------|
| `server.rs` | `AtlasLspServer` struct + `LanguageServer` trait impl |
| `document.rs` | `DocumentState` — per-file parse/typecheck state |
| `index.rs` | `SymbolIndex` — workspace symbol table |
| `hover.rs` | Hover provider — `find_parameter_hover`: ownership-aware param hover; `format_function_signature`: includes ownership prefix |
| `completion.rs` | Completion provider — `ownership_annotation_completions()`: own/borrow/shared; `is_in_param_position()`: context detection; `generate_completions(text, pos, ...)` |
| `semantic_tokens.rs` | Syntax highlighting token classification |
| `inlay_hints.rs` | Inlay hint rendering + `InlayHintConfig` |
| `navigation.rs` | Go-to-definition, go-to-declaration |
| `references.rs` | Find all references |
| `symbols.rs` | Document + workspace symbols, `WorkspaceIndex` |
| `call_hierarchy.rs` | Call hierarchy (incoming/outgoing) |
| `folding.rs` | Code folding ranges |
| `formatting.rs` | Document formatting (delegates to atlas-formatter) |
| `convert.rs` | LSP type conversions (Position ↔ offset, etc.) |
| `actions.rs` | Code actions |
| `refactor/` | Refactoring operations |
| `handlers/` | IPC handler stubs (if present) |

## Tests

**Location:** `crates/atlas-lsp/tests/`
**Pattern: INLINE SERVER CREATION. NO helper functions. Ever.**

Every test creates its own server inline:
```rust
#[tokio::test]
async fn test_something() {
    let (service, socket) = LspService::new(|client| AtlasLspServer::new(client));
    // ... test body inline, no extracted helpers
}
```

Adding a `create_test_server()` helper or any shared setup function is BANNED.
This is a deliberate architectural decision — LSP tests are self-contained.

Existing test files (add to these, do NOT create new ones unless clearly a new domain):

| File | Domain |
|------|--------|
| `tests/completion_tests.rs` | Completion |
| `tests/lsp_hover_tests.rs` | Hover |
| `tests/lsp_tokens_tests.rs` | Semantic tokens |
| `tests/navigation_tests.rs` | Navigation |
| `tests/references_tests.rs` | References |
| `tests/lsp_symbols_tests.rs` | Symbols |
| `tests/diagnostics_tests.rs` | Diagnostics |
| `tests/lsp_inlay_tests.rs` | Inlay hints |
| `tests/lsp_integration_tests.rs` | End-to-end LSP |

## Key Types

- `AtlasLspServer` — main server struct, holds `documents`, `workspace_index`, `symbol_index`
- `DocumentState` — per-file: source text, parsed AST, typecheck result
- `SymbolIndex` — queryable symbol table built from AST

## Critical Rules

**No helper functions in tests.** Each test is standalone. This prevents shared state bugs
that are nearly impossible to debug in async LSP tests.

**Typecheck before hover/completion.** Features that need type information must go through
`DocumentState`'s typecheck result — don't re-parse or re-typecheck inline.

**tower-lsp pattern.** Server implements `LanguageServer` trait. All handlers are async.
Use `self.documents.lock().await` for document access.
