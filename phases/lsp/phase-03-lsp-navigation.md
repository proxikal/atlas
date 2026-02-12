# Phase 03 - LSP Navigation & Symbols

## Objective
Implement code navigation features using Atlas symbol table.

## Inputs
- `docs/implementation/16-lsp.md` - LSP implementation guide
- `docs/implementation/06-symbol-table.md` - Symbol table structure
- `crates/atlas-runtime/src/symbol.rs` - Existing symbol implementation

## Deliverables
- textDocument/documentSymbol - Outline view
- textDocument/definition - Go to definition
- textDocument/references - Find all references
- textDocument/hover - Symbol information on hover

## Steps
- Extract document symbols from AST and symbol table
- Implement go-to-definition using symbol bindings
- Implement find-references by traversing AST for identifier usage
- Generate hover information (type, documentation, signature)
- Handle cross-file navigation (once modules exist)

## Exit Criteria
- Outline view shows all functions and variables
- Cmd/Ctrl+Click jumps to definition
- Find all references works for local and function symbols
- Hover shows type information and function signatures
- Navigation works within single file (multi-file deferred to module system)
