# Atlas Implementation Guide

This directory contains concrete architectural decisions and implementation details split by component. Each file is focused and self-contained.

## Quick Reference

**When working on a phase, read only the relevant implementation file:**

| Phase Section | Read These Files |
|--------------|------------------|
| **Foundation** | `01-project-structure.md`, `02-core-types.md` |
| **Frontend (Lexer)** | `02-core-types.md`, `03-lexer.md` |
| **Frontend (Parser)** | `04-parser.md`, `05-ast.md` |
| **Typing (Binder)** | `06-symbol-table.md`, `02-core-types.md` |
| **Typing (Typechecker)** | `07-typechecker.md`, `02-core-types.md` |
| **Diagnostics** | `08-diagnostics.md` |
| **Interpreter** | `09-value-model.md`, `10-interpreter.md` |
| **Bytecode Compiler** | `11-bytecode.md`, `09-value-model.md` |
| **VM** | `12-vm.md`, `11-bytecode.md`, `09-value-model.md` |
| **Standard Library** | `13-stdlib.md`, `09-value-model.md` |
| **REPL** | `14-repl.md` |
| **LSP & Tooling** | `16-lsp.md` |
| **Testing** | `15-testing.md` |

## File Descriptions

- **01-project-structure.md** - Cargo workspace layout, dependencies, module organization
- **02-core-types.md** - Span, Symbol, Type enums (used everywhere)
- **03-lexer.md** - Token definition, lexer state machine, character handling
- **04-parser.md** - Pratt parsing strategy, recursive descent, precedence
- **05-ast.md** - Complete Rust AST struct definitions
- **06-symbol-table.md** - Scope stack, binding algorithm, function hoisting
- **07-typechecker.md** - Type checking strategy, inference, assignability
- **08-diagnostics.md** - Diagnostic builder pattern, formatting
- **09-value-model.md** - Runtime Value enum, memory model (Rc/RefCell)
- **10-interpreter.md** - AST evaluation, environment model
- **11-bytecode.md** - Complete opcode set (30 instructions)
- **12-vm.md** - Stack-based VM, call frames, dispatch loop
- **13-stdlib.md** - print, len, str implementation strategy
- **14-repl.md** - REPL core architecture, state management
- **15-testing.md** - Golden test patterns, integration testing
- **16-lsp.md** - Language Server Protocol implementation, editor integration

## Key Architectural Decisions

| Component | Decision |
|-----------|----------|
| Lexer | Hand-written state machine |
| Parser | Pratt parsing (expressions) + recursive descent (statements) |
| Symbol Table | `Vec<HashMap<String, Symbol>>` (scope stack) |
| Type System | `Type` enum with `Unknown` for error recovery |
| Bytecode | Stack-based VM, 30 opcodes, little-endian |
| Values | `Rc` for strings/arrays, `RefCell` for mutability |
| Errors | Collect during compilation, fail-fast at runtime |

## How to Use This Guide

1. **Find your phase** in BUILD-ORDER.md
2. **Look up the relevant files** in the table above
3. **Read only what you need** - don't load everything
4. **Follow the concrete patterns** - no guessing needed

Each implementation file is self-contained with code examples and decision rationale.
