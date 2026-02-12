# Phase 05 - LSP Formatting & Code Actions

## Objective
Implement code formatting and basic code actions for Atlas.

## Inputs
- `docs/implementation/16-lsp.md` - LSP implementation guide
- `Atlas-SPEC.md` - Language style guidelines

## Deliverables
- textDocument/formatting - Format entire document
- textDocument/rangeFormatting - Format selection
- textDocument/codeAction - Quick fixes and refactorings

## Steps
- Implement basic formatter (indentation, spacing, line breaks)
- Add format-on-save support
- Implement range formatting for selections
- Add code actions for common fixes:
  - Add missing semicolon
  - Add explicit null check
  - Convert between nullable/non-nullable
- Wire up to editor format commands

## Exit Criteria
- Format document command works (preserves semantics)
- Format on save enabled by default
- Range formatting works for selections
- Code actions appear for fixable diagnostics
- Consistent formatting across all files
