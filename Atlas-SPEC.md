# Atlas Language Specification (Index)

**Version:** v0.2 (Draft)
**Last Updated:** 2026-02-13

---

## ⚠️ CRITICAL: AI Agent Routing Guide

**DO NOT read all specification files.** This index routes you to exactly what you need.

### When to Read Which Spec

| Your Task | Read This File |
|-----------|----------------|
| **Implementing types, generics, patterns** | `docs/specification/types.md` |
| **Parser, lexer, grammar work** | `docs/specification/syntax.md` |
| **Type checking, evaluation rules** | `docs/specification/language-semantics.md` |
| **Runtime, memory model, execution** | `docs/specification/runtime.md` |
| **Module system, imports, exports** | `docs/specification/modules.md` |
| **REPL behavior, interactive mode** | `docs/specification/repl.md` |
| **Bytecode, VM, compilation** | `docs/specification/bytecode.md` |
| **Error codes, diagnostics** | `docs/specification/diagnostic-system.md` |
| **Stdlib functions, API** | `docs/api/stdlib.md` |

**Example:** "I'm implementing pattern matching" → Read `docs/specification/types.md` (Pattern Matching section)

**Example:** "I'm fixing a parser bug" → Read `docs/specification/syntax.md` (Grammar section)

---

## Project Goals

- **Typed:** Strict type system, no implicit `any`
- **REPL-first:** Interactive development with fast feedback
- **Compiled:** Bytecode VM for performance
- **Cross-platform:** macOS, Windows, Linux
- **AI-friendly:** Clear semantics, explicit behavior
- **Embeddable:** Runtime as a library (future)

---

## Quick Reference

### File Format
- Extension: `.atl`
- Encoding: UTF-8
- Bytecode: `.atb`

### Types
- **Primitives:** `number`, `string`, `bool`, `null`, `void`
- **Composite:** `T[]` (arrays), `(T) -> R` (functions)
- **Special:** `json` (isolated dynamic type)
- **Generics:** `Option<T>`, `Result<T, E>`, `Array<T>`

**Details:** `docs/specification/types.md`

### Keywords
```
let var fn if else while for return break continue
true false null match import export from as
```

**Details:** `docs/specification/syntax.md`

### Built-in Functions (Prelude)
```atlas
print(value: string | number | bool | null) -> void
len(value: string | T[]) -> number
str(value: number | bool | null) -> string
```

**Full API:** `docs/api/stdlib.md`

---

## Language Features by Version

### v0.1 (Released)
✅ Core syntax, types, control flow
✅ Functions, arrays, strings
✅ REPL and bytecode VM
✅ Basic stdlib (print, len, str)
✅ Error diagnostics

### v0.2 (In Progress)
🚧 First-class functions
🚧 Generic types (Option, Result)
🚧 Pattern matching
🚧 Module system
🚧 JSON type
🚧 Expanded stdlib (100+ functions)

### v0.3+ (Future)
📅 Closures and anonymous functions
📅 User-defined structs
📅 Union types
📅 Async/await
📅 JIT compilation

**Roadmap:** `STATUS.md`

---

## Specification Files

### Core Language Specs

**docs/specification/types.md** (~467 lines, ~12kb)
- Type system comprehensive reference
- Primitive, function, generic, JSON, pattern types
- Type rules and assignability
- Module type exports

**docs/specification/syntax.md** (~492 lines, ~13kb)
- Lexical structure, keywords, literals
- Expression and statement syntax
- Complete EBNF grammar
- Operator precedence

**docs/specification/language-semantics.md** (~345 lines, ~9kb)
- Execution semantics
- String/array/numeric edge cases
- Operator type rules
- Evaluation order

**docs/specification/runtime.md** (~326 lines, ~9kb)
- Value representation
- Memory model (Rc, RefCell)
- Execution model
- Scoping and function calls

### Module System

**docs/specification/modules.md** (~376 lines, ~10kb)
- Import/export syntax
- Module resolution
- Circular dependency handling
- Build system integration

### Interactive & Execution

**docs/specification/repl.md** (~410 lines, ~11kb)
- REPL vs file mode differences
- Expression evaluation
- State persistence
- Error handling in REPL

**docs/specification/bytecode.md** (~377 lines, ~10kb)
- Bytecode instruction set
- Stack-based VM architecture
- Compilation strategy
- Debug information

### Diagnostics & API

**docs/specification/diagnostic-system.md** (~300 lines, ~12kb)
- Error code reference
- Diagnostic formats (human + JSON)
- Warning system
- Stack traces

**docs/api/stdlib.md** (~200 lines, ~8kb)
- Standard library functions
- Function signatures
- Usage examples
- Error behaviors

---

## Design Documents

Additional design docs (read when implementing specific features):

- `docs/design/generics.md` - Generic type system design
- `docs/design/pattern-matching.md` - Pattern matching design
- `docs/design/modules.md` - Module system architecture
- `docs/decision-logs/` - Architecture decisions (categorized by component)

---

## For AI Agents: Navigation Strategy

### ✅ DO
- Read index (this file) first
- Use routing table to find relevant spec
- Read ONLY the spec files needed for your task
- Check `STATUS.md` for current implementation state

### ❌ DON'T
- Read all spec files at once
- Guess which spec to read
- Skip the routing table
- Assume specs haven't changed

### Example Workflow

**Task:** Implement array slice function

1. Read Atlas-SPEC.md (this file) - routing
2. See "stdlib API" → Read `docs/api/stdlib.md`
3. See arrays → Read `docs/specification/types.md` (Array section)
4. See evaluation → Read `docs/specification/language-semantics.md` (Array semantics)

**Token cost:** ~30kb instead of 150kb (80% savings)

---

## Version History

- **v0.1.0** (2024) - Core language complete
- **v0.2.0** (In Progress) - Production foundation
  - First-class functions (complete)
  - JSON type (BLOCKER 01 complete)
  - Generic types (in progress)
  - Module system (planned)
  - Pattern matching (planned)

---

## Compliance

**Grammar:** `docs/specification/grammar-conformance.md` - Parser conformance requirements

**Parity:** Interpreter and VM must produce identical output for all programs

**Testing:** See `docs/guides/testing-guide.md` for test requirements

---

## Advanced Features (Research Phase)

These features require careful design before implementation:

- Advanced type features (unions, intersections)
- Async/await execution model
- JIT compilation / native codegen
- Concurrency primitives

**Status:** Design phase - no implementation timeline

---

## Contributing

When adding new language features:

1. Update relevant spec file in `docs/specification/`
2. Update this index if adding new spec file
3. Update routing table
4. Run `docs/specification/verify-specs.sh` (if exists)
5. Update `STATUS.md`

---

## File Size Targets

Keep spec files focused and under 15kb:

- ✅ All current specs under 13kb
- ✅ Index under 5kb
- ⚠️ If spec grows beyond 15kb, consider splitting

**Rationale:** Token cost optimization for AI agents

---

## Notes

- Spec is living document - evolves with language
- Breaking changes marked with version numbers
- Cross-references use relative paths
- All examples are valid Atlas code (tested)
