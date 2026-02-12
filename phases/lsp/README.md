# LSP & Tooling Phases

## Overview

This section implements the Language Server Protocol (LSP) for Atlas. The LSP provides real-time code intelligence for both AI agents and human developers using editors like VSCode, Neovim, Zed, and others.

## Why LSP for Atlas?

**Atlas is AI-native.** Other languages were built before AI and retrofitted with tooling. Atlas treats AI agents as first-class consumers from the start. The LSP gives AI agents:

- **Structured access** to diagnostics, types, and symbols without repeated compiler invocations
- **Real-time feedback** as code changes
- **Navigation capabilities** (go-to-definition, find references)
- **Context-aware completion** for better code generation

## Timing

The LSP phases come **after CLI** (section 10) and **before Polish** (section 12). By this point:

- ✅ Frontend, typing, and runtime are complete
- ✅ Language is executable (interpreter + VM)
- ✅ CLI tools are working
- ✅ Real Atlas programs can be written

This timing ensures:
- The language design is stable (no constant LSP rewrites)
- We've written enough Atlas code to know what tooling features matter
- The diagnostic and symbol table infrastructure is already built

## Phase Structure

### Phase 01 - LSP Foundation
Set up the LSP server infrastructure using tower-lsp. Implement initialization, lifecycle management, and basic document synchronization.

### Phase 02 - LSP Diagnostics
Wire Atlas diagnostics (syntax errors, type errors) to LSP `publishDiagnostics`. Real-time error feedback as you type.

### Phase 03 - LSP Navigation & Symbols
Implement code navigation: document outline, go-to-definition, find references, and hover information.

### Phase 04 - LSP Completion
Context-aware auto-completion for keywords, variables, functions, and builtins.

### Phase 05 - LSP Formatting & Code Actions
Code formatting (format-on-save) and quick fixes (add null checks, fix common errors).

### Phase 06 - LSP Integration Testing
Comprehensive tests for LSP protocol conformance, editor integration (VSCode, Neovim), and performance benchmarks.

## Implementation Guide

See `docs/implementation/16-lsp.md` for detailed implementation guidance including:
- Architecture and crate structure
- LSP server lifecycle
- Type conversions (Atlas ↔ LSP)
- Feature implementation examples
- Editor integration (VSCode, Neovim)
- Performance targets and optimization strategies

## Crate Structure

The LSP will be implemented as a new crate: `crates/atlas-lsp/`

This maintains the clean separation:
- `atlas-runtime` - Core language logic (library)
- `atlas-cli` - Command-line interface
- `atlas-lsp` - Language server (new)

All three crates depend on `atlas-runtime` but are otherwise independent.

## Dependencies

Primary dependency: **tower-lsp** (Rust LSP framework)

See the implementation guide for full dependency list.

## Exit Criteria

By the end of these phases:
- [ ] LSP server runs and connects to editors
- [ ] Real-time diagnostics work in at least 2 editors (VSCode + one other)
- [ ] Navigation works (go-to-definition, find references, outline)
- [ ] Completion provides relevant suggestions
- [ ] Performance targets met (<200ms diagnostics, <100ms completion)
- [ ] Editor setup documented for VSCode and Neovim
