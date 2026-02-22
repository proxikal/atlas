# Atlas Language Specification (Index)

**Purpose:** Route AI agents to the correct specification file.
**Status:** Living document — reflects current implementation.

---

## AI Agent Routing Guide

**DO NOT read all specification files.** This index routes you to exactly what you need.

### Quick Routing Table

| Your Task | Read This File |
|-----------|----------------|
| **Types, generics, patterns** | `docs/specification/types.md` |
| **Parser, lexer, grammar** | `docs/specification/syntax.md` |
| **Type checking, evaluation** | `docs/specification/language-semantics.md` |
| **Runtime, memory, execution** | `docs/specification/runtime.md` |
| **Modules, imports, exports** | `docs/specification/modules.md` |
| **REPL, interactive mode** | `docs/specification/repl.md` |
| **Bytecode, VM, compilation** | `docs/specification/bytecode.md` |
| **Error codes, diagnostics** | `docs/specification/diagnostic-system.md` |
| **Stdlib functions** | `docs/specification/stdlib.md` |
| **Build system** | `docs/build-system.md` |
| **Package management** | `docs/dependency-resolution.md` |
| **Security model** | `docs/security-model.md` |
| **Reflection API** | `docs/reflection.md` |

**Example:** "I'm implementing pattern matching" → `docs/specification/types.md` (Pattern Matching section)

---

## Project Goals

- **Typed:** Strict type system, no implicit `any`
- **REPL-first:** Interactive development with fast feedback
- **Compiled:** Bytecode VM with optimization passes
- **Cross-platform:** macOS, Windows, Linux
- **AI-friendly:** Clear semantics, explicit behavior
- **Embeddable:** Runtime usable as library

---

## Quick Reference

### File Format
- Source: `.atl`
- Bytecode: `.atb`
- Manifest: `atlas.toml`
- Encoding: UTF-8

### Types
- **Primitives:** `number`, `string`, `bool`, `null`, `void`
- **Composite:** `T[]` (arrays), `(T) -> R` (functions)
- **Special:** `json` (isolated dynamic type)
- **Generics:** `Option<T>`, `Result<T, E>`, `Array<T>`

### Keywords
```
let var fn if else while for return break continue
true false null match import export from as
```

### Prelude (Always Available)
```atlas
print(value: any) -> void
len(value: string | T[]) -> number
str(value: any) -> string
```

---

## Specification Files

### Core Language

| File | Content |
|------|---------|
| `syntax.md` | Grammar, keywords, operators, EBNF |
| `types.md` | Type system, generics, pattern matching |
| `language-semantics.md` | Evaluation rules, edge cases |
| `runtime.md` | Execution model, memory, scoping |
| `bytecode.md` | VM architecture, instructions, optimization |
| `modules.md` | Import/export, resolution, cycles |
| `repl.md` | Interactive mode behavior |
| `diagnostic-system.md` | Error codes, diagnostic format |
| `stdlib.md` | Standard library reference |

### Implementation Guides

| File | Content |
|------|---------|
| `build-system.md` | Build architecture |
| `dependency-resolution.md` | Package resolution |
| `security-model.md` | Permissions, sandbox |
| `reflection.md` | Reflection API |
| `vm-architecture.md` | VM design details |

---

## Current Capabilities

Atlas currently supports:

- **Core:** Variables, functions, control flow, arrays
- **Types:** Strict typing, generics (Option, Result), pattern matching
- **Modules:** Import/export, namespace imports, cycle detection
- **Execution:** Interpreter and bytecode VM (with parity)
- **Optimization:** Constant folding, dead code elimination, peephole
- **Stdlib:** 100+ functions (collections, HTTP, filesystem, regex, datetime)
- **Tooling:** Formatter, debugger, profiler, LSP

### Current Limitations

See `ROADMAP.md` for planned enhancements:

- Closures (functions cannot capture outer scope) — v0.3
- Anonymous function syntax — v0.3
- `async/await` language syntax (runtime exists) — v0.4
- User-defined generic types — v0.6

---

## For AI Agents

### Navigation Strategy

1. **Check STATUS.md** — Current phase and progress
2. **Read this index** — Find relevant spec file
3. **Read ONE spec** — Only what you need
4. **Check memory/** — Patterns and decisions

### Token Efficiency

| Approach | Tokens |
|----------|--------|
| Read all specs | ~150k |
| Read index + 1 spec | ~20k |
| **Savings** | **~87%** |

### Example Workflow

**Task:** Fix array bounds checking

1. Read `Atlas-SPEC.md` → route to `types.md`
2. Read `docs/specification/types.md` (array section)
3. Read `docs/specification/runtime.md` (error handling)
4. Check `memory/patterns.md` for conventions

---

## Compliance

- **Parity:** Interpreter and VM produce identical output
- **Grammar:** `docs/specification/grammar-conformance.md`
- **Testing:** All features require tests

---

## Versioning

- **Current state:** See `STATUS.md`
- **Future plans:** See `ROADMAP.md`
- **Decisions:** See `memory/decisions.md`

Specs describe current implementation. Version history tracked in STATUS.md.

---

## Contributing

When modifying language features:

1. Update relevant spec in `docs/specification/`
2. Update this index if adding new spec
3. Update `STATUS.md` with completion
4. Update `memory/` if adding patterns

---

## Notes

- Specs are living documents
- Cross-references use relative paths
- All code examples are tested
- Keep spec files under 15kb (token efficiency)
