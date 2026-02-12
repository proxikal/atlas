# Phase 04 - LSP Completion

## Objective
Implement context-aware auto-completion for Atlas.

## Inputs
- `docs/implementation/16-lsp.md` - LSP implementation guide
- `docs/implementation/06-symbol-table.md` - Symbol table for scope lookup
- `Atlas-SPEC.md` - Language keywords and builtins

## Deliverables
- textDocument/completion - Auto-completion suggestions
- completionItem/resolve - Detailed completion information
- Context-aware completions (keywords, variables, functions)

## Steps
- Implement keyword completion (let, if, while, func, return, etc.)
- Implement variable/function completion from current scope
- Add builtin function completion (print, len, etc.)
- Add type-aware completion where applicable
- Implement completion item resolve for detailed docs

## Exit Criteria
- Keywords complete with proper snippets (e.g., "if" -> "if condition { }")
- Variables in scope appear in completion list
- Functions show signature in completion detail
- Completions respect scope (no out-of-scope suggestions)
- Completion triggers on typing (no manual invocation needed)
