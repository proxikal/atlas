# Atlas Language Classification & Evolution Path

## Current Classification

**Application Language** (Go-tier, not yet Rust-tier)

| Aspect | Current State |
|--------|---------------|
| Execution | Interpreter + Bytecode VM |
| Memory | Reference counting (`Arc<Mutex<>>`) |
| Output | Requires Atlas runtime |
| Tier | Python/Go level |

## Execution Modes

- **Interpreter**: Tree-walking, for REPL/debugging/dev
- **Bytecode VM**: Faster execution, production runs
- **Native (future)**: LLVM/Cranelift backend

## Evolution Path to Systems-Level

```
Current:                Future (additive):

AST ──► Interpreter     AST ──► Interpreter (dev/REPL)
    │                       │
    └──► Bytecode VM        ├──► Bytecode VM (fast iteration)
                            │
                            └──► LLVM Backend (native binaries)
```

**Key insight**: Frontend (lexer, parser, AST, types, binder) is backend-agnostic. Native codegen is ADDED, not replaced.

## What's Reusable for Systems-Level (100%)

- Lexer/Parser
- AST representation
- Type system
- Binder/Resolver
- Tooling (LSP, CLI, Formatter)
- Test infrastructure
- Bytecode compiler (as intermediate representation)

## Future Additions for Systems-Level

| Component | Purpose | When |
|-----------|---------|------|
| LLVM/Cranelift backend | Native code generation | v1.0+ |
| Stack allocation | Unboxed primitives | With native backend |
| Ownership model | Rust-level memory safety | Design decision TBD |

## Memory Model Options (Future Decision)

1. **Keep `Arc<Mutex<>>`** — Application-level, GC-like, simpler
2. **Add ownership/borrowing** — Rust-level, zero-cost, complex
3. **Hybrid** — Safe mode (current) + unsafe primitives

## For AI Agents

- Atlas is NOT a systems language yet
- Current work builds foundation for systems-level future
- Interpreter/VM remain useful as dev tools even with native backend
- Frontend work is 100% reusable regardless of backend
- Memory model is a future design decision, not a current blocker

## Project Goal

World-class compiler rivaling Rust, Go, C, Python. No MVP — done properly from start. Native codegen is a future phase (v1.0+), not a rewrite.
