# Atlas Documentation

Documentation for the Atlas programming language.

---

## Structure

### [`specification/`](specification/) — Language Specification
Core language definition:
- `syntax.md` — Grammar, keywords, operators, EBNF
- `types.md` — Type system, generics, patterns
- `language-semantics.md` — Evaluation rules, edge cases
- `runtime.md` — Execution model, memory, scoping
- `bytecode.md` — VM, compilation, instructions
- `modules.md` — Import/export, resolution
- `diagnostic-system.md` — Error codes, warnings, diagnostic format
- `grammar-conformance.md` — Parser conformance requirements
- `json-formats.md` — AST/typecheck dump formats
- `repl.md` — Interactive mode behavior
- `stdlib.md` — Standard library API reference

### Feature Documentation (root level)
Phase-generated docs for implemented features:
- `build-system.md` — Build system architecture
- `configuration.md` — Configuration system
- `dependency-resolution.md` — Package resolver internals
- `embedding-guide.md` — Runtime embedding API
- `frontend-status.md` — Frontend phase completion report
- `jit.md` — JIT compilation foundation
- `package-manifest.md` — Package manifest format
- `reflection.md` — Reflection API
- `repl.md` — REPL architecture
- `security-model.md` — Security and permissions
- `source-maps.md` — Source Map v3 generation
- `vm-architecture.md` — VM design and optimization

---

## For AI Agents

- **Start with:** `STATUS.md` (project root) for current phase
- **Language spec:** `specification/` directory
- **Memory system:** `/memory/` (patterns, decisions, gates)
- **Atlas skill:** `/.claude/skills/atlas/` (gates, workflows)
- **Phase files:** `phases/` directory
