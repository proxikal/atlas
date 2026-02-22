# Atlas Roadmap

**Purpose:** Strategic direction for Atlas development. Reference for AI agents during phase scaffolding.
**Philosophy:** Quality gates, not deadlines. Features ship when ready, not on schedule.
**Last verified:** 2026-02-21 (post-v0.2 architectural decision session)

---

## Long-Term Goal: Systems Language

Atlas's end goal is **systems-level programming** — a language capable of replacing C/Rust/Zig
for performance-critical, low-level work, while being the most AI-generation-friendly language
ever built. These goals are not in conflict. They reinforce each other.

**The trajectory:**
- v0.1–v0.2: Bootstrap (scripting semantics, Arc<Mutex<>> model, establish correctness)
- v0.3: Foundation (value semantics, ownership model, traits — the permanent base layer)
- v0.4–v0.6: Type system maturity (compile-time ownership verification, advanced generics)
- v0.7–v0.9: Systems features (AOT native compilation, embedded targets, OS-level primitives)
- v1.0: Stability commitment — first stable release

**Why this trajectory is right:**
- Go chose GC early and permanently closed the systems-programming door. Atlas will not repeat this.
- Rust got memory safety right but the borrow checker is the hardest thing for LLMs to generate.
  Atlas achieves the same safety with explicit ownership syntax — visible in signatures, generatable by AI.
- Swift started high-level and is retroactively adding ownership (Swift 5.9/6 — `consuming`, `borrowing`).
  Atlas bakes ownership syntax in from the foundation, avoiding the retrofit problem.

**The memory model decision is final.** See `docs/specification/memory-model.md`.

---

## Memory Model Strategy (LOCKED)

**No GC. Ever.**

The permanent Atlas memory model:
1. **Value types by default** — arrays, maps, and objects are copy-on-write (CoW)
2. **Explicit ownership** — `own`, `borrow`, `shared` parameter annotations
3. **`shared<T>` for opt-in reference semantics** — explicit Arc, not implicit
4. **No implicit borrow checker** — ownership expressed in syntax, verified by compiler

This is implemented in **v0.3** (value semantics + ownership syntax + runtime verification).
Compile-time ownership proof is **v0.4** (requires v0.3 foundation to be stable first).

Full specification: `docs/specification/memory-model.md`

---

## Version Strategy

Atlas uses semantic versioning with deliberate pacing:

- **0.x versions:** Language evolution, breaking changes allowed
- **1.0:** Production-ready, stability commitment begins
- **Post-1.0:** Semantic versioning strictly followed

**Pacing principle:** Don't rush versions. Each version should represent meaningful, complete work.

---

## Implementation Status (Verified — 2026-02-21)

### Already Implemented

| Feature | Location | Notes |
|---------|----------|-------|
| **atlas.toml manifest** | `atlas-config/`, `atlas-package/` | Full manifest parsing |
| **Package management** | `atlas-package/` | Resolver, lockfile, registry, caching |
| **Pattern matching** | `ast.rs`, `parser/`, `compiler/` | All variants including guards/OR (v0.3) |
| **Bytecode optimizer** | `optimizer/` | Constant folding, dead code, peephole |
| **Async runtime** | `async_runtime/` | Tokio integration, futures, channels, tasks |
| **Debugger** | `debugger/` | Breakpoints, stepping, source mapping |
| **Formatter** | `atlas-formatter/` | Comment preservation, configurable |
| **REPL** | `cli/commands/repl.rs` | History, TUI mode |
| **LSP** | `atlas-lsp/` | 16 features: navigation, completion, symbols |
| **Compression** | `stdlib/compression/` | gzip, tar, zip |
| **HTTP client** | `stdlib/http.rs` | Full HTTP, security-context enforced |
| **Regex** | `stdlib/regex.rs` | Pattern matching |
| **DateTime** | `stdlib/datetime.rs` | Chrono integration |
| **Collections** | `stdlib/collections/` | HashMap, HashSet, Queue, Stack |
| **FFI** | `ffi/` | Callbacks, marshaling, safety |
| **Security** | `security/` | Permissions, sandbox, audit |
| **JIT foundation** | `atlas-jit/` | Cranelift backend, arithmetic opcodes, hotspot profiler |
| **VM upvalue capture** | `vm/mod.rs` | MakeClosure, GetUpvalue, SetUpvalue opcodes |
| **for-in loops** | `compiler/stmt.rs` | VM parity complete (v0.2 close) |
| **Source maps** | `sourcemap/` | v3 spec, inline generation |

### Missing — Targeted in v0.3

| Feature | Block | Notes |
|---------|-------|-------|
| **Value semantics (CoW)** | Block 1 | Replace Arc<Mutex<Value>> — foundational |
| **Ownership syntax** | Block 2 | `own`, `borrow`, `shared` annotations |
| **Trait system** | Block 3 | `trait`, `impl`, `Copy`/`Move`/`Drop` built-ins |
| **Anonymous functions** | Block 4 | `fn(x) { }` and `(x) => x` syntax |
| **Closure value capture** | Block 4 | CoW semantics for captured values |
| **Type inference** | Block 5 | Local variable and return type inference |
| **`?` operator** | Block 6 | Result/Option error propagation |
| **JIT control flow** | Block 7 | Wire atlas-jit to VM, Jump/Call opcodes |
| **Async/await syntax** | Block 8 | `async fn`, `await expr` (runtime exists) |
| **String interpolation** | Block 9 | `"Hello, ${name}!"` |
| **Implicit returns** | Block 9 | Single-expression functions |

---

## Current: v0.3 — The Foundation Version

**Status:** Ready for phase scaffolding
**Phase plan:** `docs/internal/V03_PLAN.md` — READ THIS BEFORE SCAFFOLDING
**Phase target:** ~130–150 phases across 9 blocks
**Theme:** Memory model, ownership, traits — the permanent architectural foundation

### The 9 Blocks (strict dependency order)

| Block | Theme | Phases | Depends On |
|-------|-------|--------|-----------|
| 1 | Memory Model (CoW value types) | 25–35 | Nothing |
| 2 | Ownership Syntax | 15–20 | Block 1 |
| 3 | Trait System | 20–25 | Block 2 |
| 4 | Closures + Anonymous Functions | 15–20 | Block 3 |
| 5 | Type Inference | 10–15 | Block 3 |
| 6 | Error Handling (`?` operator) | 10–15 | Block 3 |
| 7 | JIT Integration | 10–15 | Block 1 only |
| 8 | Async/Await Syntax | 10–15 | Block 6 |
| 9 | Quick Wins | 5–10 | Block 1 |

**CRITICAL:** Blocks are strictly sequential. Block N cannot begin until Block N-1 acceptance
criteria are ALL met. This is the lesson from v0.2 dependency hell. Do not reorder.

### v0.3 Exit Criteria
- [ ] No `Arc<Mutex<Value>>` in production code
- [ ] Value semantics: mutation does not affect aliased copies
- [ ] Ownership annotations runtime-verified in both engines
- [ ] Trait system: Copy, Move, Drop + user-defined traits
- [ ] Closures with correct capture semantics
- [ ] Type inference for locals and return types
- [ ] `?` operator propagates Result/Option
- [ ] JIT hot functions compile to native (10x+ speedup target)
- [ ] Async/await syntax in both engines
- [ ] ≥ 9,000 tests, 0 failures

---

## v0.4: Compile-Time Verification

**Theme:** Prove ownership at compile time. Type system maturity.

### Why v0.4 and not v0.3
Compile-time ownership verification requires the v0.3 syntax foundation to be stable.
You cannot write a verifier until you know what syntax you're verifying. Sequencing is correct.

### Planned work
| Feature | Notes |
|---------|-------|
| **Compile-time ownership verification** | Static analysis pass over typed AST |
| **Trait object dispatch (vtable)** | Dynamic dispatch for trait objects |
| **User-defined generics** | Generic types, not just functions |
| **Advanced type inference** | Cross-function inference, improved H-M |
| **Lifetime annotations (if needed)** | Only if runtime verification proves insufficient |

---

## v0.5: Native Compilation (AOT)

**Theme:** Compile Atlas programs to native binaries. No runtime required.

### Planned work
| Feature | Notes |
|---------|-------|
| **Cranelift AOT backend** | atlas-jit already has Cranelift — extend to full AOT |
| **LLVM IR backend** | Alternative backend for maximum optimization |
| **Static binary output** | `atlas build --release` produces native executable |
| **Embedded target** | No-std mode for microcontrollers |

---

## v0.6: Developer Experience

**Theme:** World-class tooling for the world-class language.

| Feature | Notes |
|---------|-------|
| **REPL tab completion** | Keywords, locals, stdlib, types |
| **REPL syntax highlighting** | Colorized output |
| **`.type expr` inspection** | Show inferred type in REPL |
| **LSP incremental analysis** | Re-analyze only changed functions |
| **LSP semantic tokens** | Full syntax highlighting in editors |

---

## v0.7: Package Ecosystem

**Theme:** Atlas packages are a first-class ecosystem.

| Feature | Notes |
|---------|-------|
| **`atlas publish`** | Publish to registry |
| **Private registry auth** | Token-based authentication |
| **Workspace support** | Monorepo with multiple Atlas packages |
| **Content-addressed storage** | Reproducible builds, hash-pinned deps |

---

## v0.8: Performance & Profiling

**Theme:** Make Atlas fast enough to replace C for numeric workloads.

| Feature | Notes |
|---------|-------|
| **Inline caching** | Method dispatch without hash map lookup |
| **Benchmark regression CI** | Block performance regressions at PR time |
| **Memory profiling** | Track allocation patterns, identify waste |
| **Profile-guided optimization** | Use profiler data to drive JIT thresholds |

---

## v0.9: Stabilization

**Theme:** Prepare for 1.0. Fix everything before the stability commitment.

| Area | Work |
|------|------|
| **Fuzz testing** | Expand corpus to 50+ targets, fix all crashes |
| **Error messages** | Audit all 200+ diagnostic codes |
| **Documentation** | Complete language reference, stdlib reference |
| **Breaking changes** | Final list of changes before 1.0 lock |
| **Security audit** | External review of security model |

---

## v1.0: Production Ready

**What v1.0 means:**
- Language syntax is stable — no breaking changes without major version bump
- Stdlib is stable — semver followed strictly
- Behavior is predictable — no known correctness bugs
- Security is verified — audit complete
- AI generation is proven — measurable reliability metrics

---

## Phase Scaffolding Reference

### File structure
```
phases/
  v0.3/
    block-01-memory-model/
    block-02-ownership-syntax/
    block-03-trait-system/
    block-04-closures/
    block-05-type-inference/
    block-06-error-handling/
    block-07-jit-integration/
    block-08-async-await/
    block-09-quick-wins/
```

### Phase file template
```markdown
# Phase XX: {Title}

**Block:** {N} ({Block Name})
**Depends on:** {Block N-1 complete | None}
**Complexity:** {low/medium/high}
**Files to modify:** {list}

## Summary
{1-2 sentences}

## Current State
{Verified against codebase — not assumed}

## Requirements
{Numbered, specific, testable}

## Acceptance Criteria
{Checkboxes — all must pass}

## Tests Required
{What tests prove this works}
```

### Quality rules
1. ~100 lines per phase file
2. Verify current state against codebase before writing
3. Every requirement must be testable
4. Every phase must pass: build + tests (100%) + clippy + fmt
5. No phase is done until parity is verified (both engines identical output)

### Final phase of every block (spec update + AC check)
The last phase of each block MUST include:
1. Spec update (`docs/specification/` — whatever changed this block)
2. STATUS.md block row: ⬜ → ✅
3. Auto-memory update (GATE 7) — `decisions/{domain}.md`
4. **Crate CLAUDE.md audit** — update `crates/*/src/CLAUDE.md` for any structural changes
   introduced this block (new files, new invariants, new test domains). Takes 2 minutes.
   Skipping this causes stale context in future blocks.

---

## References

| Document | Purpose |
|----------|---------|
| `PRD.md` | Product requirements, principles |
| `STATUS.md` | Current progress tracking |
| `docs/specification/memory-model.md` | **Memory model decision — READ FIRST** |
| `docs/specification/` | Language specifications |
| `docs/internal/V03_PLAN.md` | v0.3 block plan and scaffolding guide |
| `memory/` | AI agent knowledge base (auto-memory) |
| `phases/` | Active work queue |

---

*Last updated: 2026-02-21*
*Memory model decision locked. v0.3 plan locked. Ready for phase scaffolding.*
