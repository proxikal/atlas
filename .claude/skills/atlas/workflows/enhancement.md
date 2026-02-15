# Enhancement Workflow

**When to use:** Adding capabilities, improvements, new features

**Approach:** Implementation-driven (NOT strict TDD)

**Reference:** See `gates/README.md` for detailed gate definitions

---

## Workflow Gates

Enhancements use **GATE 0, 0.5, 1, 1.5, 2-5** from central gate workflow.

| Gate | Action | Reference |
|------|--------|-----------|
| 0 | **Read Docs** | See gates/gate-0-read-docs.md |
| 0.5 | **Check Dependencies** | See gates/gate-0.5-dependencies.md |
| 1 | **Size Estimation** | See gates/gate-1-sizing.md |
| 1.5 | **Foundation Check** | See gates/gate-1.5-foundation.md (CRITICAL) |
| 2 | **Implement + Test** | See gates/gate-2-implement.md (implementation-driven) |
| 3 | **Verify Parity** | See gates/gate-3-parity.md |
| 4 | **Quality Gates** | See gates/gate-4-quality.md |
| 5 | **Doc Update** | See gates/gate-5-docs.md |

---

## Enhancement Specifics

### Before GATE 0: Define Enhancement

**Answer these questions:**
- **What:** Exact feature description
- **Why:** Motivation, use case
- **Scope:** What's included, what's excluded
- **Impact:** What components affected?

**Check specs:**
- `Atlas-SPEC.md` (index) - Use routing to find relevant spec for language design check
- `docs/specification/` - Specification implications?

---

### GATE 0.5 Dependencies

**What must exist first?**
- Required components
- Language features
- Related documentation

**BLOCKING:** If dependencies missing, STOP and report.

---

### GATE 1 Planning

**Identify affected components:**
- Lexer, parser, AST, typechecker?
- Compiler, VM, interpreter?
- Stdlib, LSP?

**Reference:** `docs/implementation/` for component patterns

---

### GATE 2: Implementation-Driven

**NOT strict TDD** - Mirrors real compilers (rustc, Go, TypeScript, Clang)

**Approach:**
1. Implement enhancement (exploratory)
2. Write tests alongside or after
3. Iterate: implement → test → refine

**Tests required:** Comprehensive, but NOT before implementation.

---

### GATE 3-5

Follow standard gates from `gates/README.md`:
- **GATE 3:** Verify parity (interpreter/VM identical)
- **GATE 4:** Quality gates (test, clippy, fmt)
- **GATE 5:** Update docs (specification/api/implementation)

---

## Notes

- **Flexible but rigorous** - No strict plan required, but same quality standards
- **Implementation-driven:** Build first, test alongside/after (compiler approach)
- **NOT strict TDD:** Tests can come after implementation
- **Parity required:** Both engines must work (GATE 3)
- **Quality gates:** Must all pass (GATE 4)
- **Doc-driven:** Reference specs and guides
- **Reference gates/README.md** for detailed gate definitions
