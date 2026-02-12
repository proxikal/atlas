# Phase 01 - LSP Foundation

## Objective
Implement Language Server Protocol (LSP) server foundation for Atlas.

## Inputs
- `docs/implementation/16-lsp.md` - LSP implementation guide
- `docs/implementation/01-project-structure.md` - Crate structure
- LSP Specification: https://microsoft.github.io/language-server-protocol/

## Deliverables
- `crates/atlas-lsp/` crate with LSP server scaffolding
- LSP server initialization and lifecycle management
- Basic communication layer (JSON-RPC 2.0)
- Server capabilities registration

## Steps
- Create `atlas-lsp` crate with tower-lsp dependency
- Implement LSP server lifecycle (initialize, initialized, shutdown, exit)
- Register server capabilities (diagnostics, symbols, completion, hover)
- Add basic document synchronization (textDocument/didOpen, didChange, didClose)
- Wire up to existing atlas-runtime diagnostics

## Exit Criteria
- LSP server starts and responds to initialization
- Documents can be opened and synchronized
- Basic health check passes (connects to VSCode/editor)
- No crashes on invalid requests
