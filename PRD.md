# Atlas PRD (Product Requirements Document)

**Purpose:** Define Atlas's vision, principles, and requirements.
**Audience:** AI agents and contributors understanding project goals.

---

## Summary

Atlas is a strict, typed, REPL-first programming language with a bytecode VM and a single cross-platform binary. It is designed to be readable, predictable, and AI-friendly. Atlas borrows proven ideas from TypeScript, Go, Rust, and Python while remaining cohesive and small.

---

## Vision

Create a language that feels natural for humans and AI agents, combining strict typing with a fast iteration loop (REPL) and a clear compilation path. Atlas must remain lightweight, deterministic, and easy to embed in tooling or applications.

---

## Non-Negotiable Principles

1. **Strict typing:** No implicit any, no implicit nullable
2. **Clear diagnostics:** Precise error locations, helpful messages, JSON output
3. **Cohesion over sprawl:** Only add features when truly needed and well-designed
4. **Single binary:** No runtime dependencies
5. **Cross-platform:** macOS, Windows, Linux
6. **Small surface area:** Keep syntax and stdlib focused

---

## Primary Users

- **Developers** who want a strict, readable scripting language
- **AI agents** that need consistent syntax and high-quality diagnostics

---

## Goals

1. REPL with type checking and safe evaluation
2. Bytecode compiler + VM for performance
3. Clear and deterministic CLI workflow
4. Strong error handling (human + JSON diagnostics)
5. Embeddable runtime for tooling integration

---

## Functional Requirements

### Language
- Parse and type-check `.atl` files
- Evaluate scripts and REPL inputs
- Emit bytecode for performance
- Module system with explicit imports/exports

### Standard Library
- Core: `print`, `len`, `str`
- Collections: arrays, maps, sets, queues, stacks
- I/O: filesystem, HTTP, process
- Utilities: regex, datetime, JSON, compression

### Diagnostics
- Errors include file, line, column, length, code, hints
- JSON diagnostic output for tooling/AI
- Warning system for suspicious patterns

### Tooling
- Formatter with comment preservation
- Debugger with breakpoints and stepping
- Profiler for performance analysis
- LSP for IDE integration

---

## Success Criteria

- Atlas runs programs with correct typing and clear diagnostics
- REPL handles errors gracefully without crashing
- Bytecode VM produces identical results to interpreter (parity)
- Type system catches bugs that would be runtime errors in dynamic languages
- Error messages are precise and actionable for humans and AI

**Quality is measured by correctness and usability, not feature count.**

---

## Design Constraints

- Language implemented in Rust
- Runtime structured for library exposure
- Minimal dependencies
- Thread-safe value types (`Arc<Mutex<>>`)

---

## Quality Bar

- Every feature must include tests
- No ambiguous syntax
- Diagnostics must be actionable and consistent
- Interpreter/VM parity is mandatory

---

## Documentation

### Specifications
- `docs/specification/` — Language specs (syntax, types, runtime)
- `Atlas-SPEC.md` — Spec index and routing

### Implementation
- `docs/` — Feature documentation
- `memory/` — AI knowledge base (patterns, decisions)

### Tracking
- `STATUS.md` — Current progress
- `ROADMAP.md` — Future direction

---

## Risks

| Risk | Mitigation |
|------|------------|
| Scope creep | Phased development, clear scope per phase |
| Inconsistent diagnostics | Defined schema, spec-first workflow |
| Over-engineering | Quality gates before new features |
| Breaking changes | Spec versioning, backward compatibility |

---

## Principles for Growth

1. **Spec-first:** Define behavior before implementing
2. **Test-driven:** Tests define correctness
3. **Quality gates:** Previous work solid before new features
4. **Honest assessment:** Progress measured by reality, not hopes
5. **AI-optimized:** Documentation serves both humans and AI

---

## Notes

- Features ship when ready, not on schedule
- Breaking changes require version bumps
- Backward compatibility valued after 1.0
