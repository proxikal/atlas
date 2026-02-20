# GATE 0: Environment Prep + Read Docs

**Condition:** Starting any task

---

## Step 1: Clean Build Artifacts (MANDATORY)

**Prevent disk bloat:** Cargo accumulates GB of build artifacts rapidly (51GB in ~5 hours).

```bash
cargo clean
```

**Why:** Fresh build environment prevents accumulation. Clean slate for each task.

**ONE TIME ONLY:** Run once at task start, not during implementation.

---

## Step 2: Read Docs (Selective Reading)

1. **ALWAYS:** Read `STATUS.md` (current state, progress, doc map with routing)
2. **IF structured development:** Read complete development plan (phase file)
3. **SELECTIVE:** Read ONLY the spec files your task needs (see routing below)
4. **CHECK EXISTING CODE:** Before writing tests, read existing test files in the target crate

### Specification Routing (DO NOT read all specs)

**Available specs in `docs/specification/`:**
- Implementing types/generics? → Read `docs/specification/types.md`
- Parser/grammar work? → Read `docs/specification/syntax.md`
- Type checking? → Read `docs/specification/language-semantics.md`
- Runtime/execution? → Read `docs/specification/runtime.md`
- Module system? → Read `docs/specification/modules.md`
- REPL behavior? → Read `docs/specification/repl.md`
- Bytecode/VM? → Read `docs/specification/bytecode.md`
- Error codes? → Read `docs/specification/diagnostics.md`
- Stdlib API? → Read `docs/specification/stdlib.md`

### Implementation Patterns (As Needed)

- Codebase patterns: auto-memory `patterns.md`
- Architectural decisions: auto-memory `decisions/*.md`

---

## ⚠️ CRITICAL: Lazy Loading Rules

**DO:**
- Read Atlas-SPEC.md as index/routing ONLY
- Use routing table to find exact file needed
- Read ONLY relevant spec files for task

**DON'T:**
- Read all spec files at once
- Skip the routing table
- Guess which spec to read

**Token savings:** 80-95% (read 5-15kb instead of 150kb)

---

**BLOCKING:** Cannot proceed without understanding current state and requirements.

---

## Step 3: Check Dependencies (formerly GATE 0.5)

**For EACH dependency in phase file:**
1. Does it exist in codebase? (grep for implementation)
2. Does it match spec? (compare to `docs/specification/`)
3. Is it complete? (check STATUS.md, run tests)

**Before implementing anything:** Search for similar existing code. Follow established patterns. Check auto-memory `decisions/*.md` for constraints.

**Status per dependency:**
- ✅ Exists, complete, spec-compliant → Proceed
- ⚠️ Exists but incomplete → Flag, may need to finish first
- 🚫 Doesn't exist → BLOCKING, report to user

---

## Step 4: Domain Pattern Verification (CRITICAL)

**Purpose:** Prevent hallucinated syntax by verifying actual codebase patterns before writing code.

**Registry:** auto-memory `domain-prereqs.md` (Claude auto-memory)

### Process

1. **Identify domains** touched by phase (AST, stdlib, VM, type system, etc.)
2. **For EACH domain**, consult auto-memory `domain-prereqs.md`
3. **Run verification queries** listed for that domain
4. **Note 3-5 patterns** you will use (mentally or in scratch)
5. **If uncertain**, read more — NEVER guess structure

### Common Domains (see registry for queries)

| Domain | Trigger Keywords | Key Verification |
|--------|-----------------|------------------|
| AST | parser, expression, statement, node | `ast.rs` enum variants |
| Value | Value enum, runtime type | `value.rs` Value variants |
| Stdlib | builtin, stdlib function | `stdlib/mod.rs` signatures |
| Interpreter | eval, tree-walk | `interpreter/mod.rs` methods |
| VM | bytecode, opcode, compile | `vm/` opcodes and execution |
| Type System | type check, infer, annotation | `type_checker/` types |
| Errors | RuntimeError, diagnostic | `errors.rs` variants |

### Example

Phase says "Add new AST node for X":
```bash
# Before writing ANY code, verify actual AST structure:
Grep pattern="^pub enum Expr" path="crates/atlas-runtime/src/ast.rs" -A=30
```

**Output:** You now know the exact variant format. Use it.

### Anti-Pattern (BANNED)

```rust
// WRONG: Guessing AST structure without verification
Expr::Function { name, params, body }  // Did you verify this exists?

// RIGHT: Verified from grep output
Expr::FunctionDef { name, params, body, return_type }  // Matches actual code
```

---

**BLOCKING:** Cannot proceed to implementation without pattern verification for touched domains.

---

**Next:** GATE 1
