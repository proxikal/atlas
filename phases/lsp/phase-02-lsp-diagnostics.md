# Phase 02 - LSP Diagnostics

## Objective
Wire Atlas diagnostics to LSP textDocument/publishDiagnostics.

## Inputs
- `docs/implementation/16-lsp.md` - LSP implementation guide
- `docs/implementation/08-diagnostics.md` - Diagnostic system
- `crates/atlas-runtime/src/diagnostic.rs` - Existing diagnostic types

## Deliverables
- Real-time diagnostics in editor as you type
- Convert Atlas Diagnostic to LSP Diagnostic format
- Map Atlas severity levels to LSP severity
- Convert Atlas spans to LSP ranges (line/column)

## Steps
- Implement diagnostic conversion layer (Atlas -> LSP format)
- Hook diagnostics to textDocument/didChange events
- Run lexer, parser, and typechecker on document changes
- Publish diagnostics to editor
- Handle multi-file diagnostics (related spans)

## Exit Criteria
- Syntax errors show in editor immediately
- Type errors show with proper severity (error/warning)
- Related information displays for multi-location diagnostics
- Diagnostics clear when errors are fixed
- Performance: diagnostics return within 200ms for typical files
