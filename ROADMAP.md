# Atlas Roadmap

**Purpose:** Strategic direction for Atlas development. Reference for AI agents during phase scaffolding.
**Philosophy:** Quality gates, not deadlines. Features ship when ready, not on schedule.
**Last verified:** 2026-02-19 (codebase audit performed)

---

## Version Strategy

Atlas uses semantic versioning with deliberate pacing:

- **0.x versions:** Language evolution, breaking changes allowed
- **1.0:** Production-ready, stability commitment begins
- **Post-1.0:** Semantic versioning strictly followed

**Pacing principle:** Don't rush versions. Each version should represent meaningful, complete work.

---

## Implementation Status (Verified)

### Already Implemented

These features exist in the codebase and should NOT be re-planned:

| Feature | Location | Notes |
|---------|----------|-------|
| **atlas.toml manifest** | `atlas-config/`, `atlas-package/` | Full manifest parsing, demos exist |
| **Package management** | `atlas-package/` | Resolver, lockfile, registry, caching, conflict resolution |
| **Pattern matching** | `ast.rs`, `parser/`, `compiler/` | `MatchExpr`, `Pattern` enum, all variants |
| **Bytecode optimizer** | `optimizer/` | Constant folding, dead code elimination, peephole |
| **Async runtime** | `async_runtime/` | Tokio integration, futures, channels, tasks |
| **Debugger** | `debugger/` | Breakpoints, stepping, inspection, source mapping |
| **Formatter** | `atlas-formatter/` | Comment preservation, configurable |
| **REPL history** | `cli/commands/repl.rs` | File persistence, TUI mode |
| **LSP foundation** | `atlas-lsp/` | Navigation, completion, document symbols |
| **Compression** | `stdlib/compression/` | gzip, tar, zip |
| **HTTP client** | `stdlib/http.rs` | Full HTTP support |
| **Regex** | `stdlib/regex.rs` | Pattern matching |
| **DateTime** | `stdlib/datetime.rs` | Chrono integration |
| **Collections** | `stdlib/collections/` | HashMap, HashSet, Queue, Stack |
| **FFI** | `ffi/` | Callbacks, marshaling, safety |
| **Security** | `security/` | Permissions, sandbox, audit |

### Still Missing (Verified)

| Feature | Spec Reference | Blocker? |
|---------|---------------|----------|
| **Closures** | types.md: "No closure capture... planned for v0.3+" | High impact |
| **Anonymous functions** | types.md: "No anonymous function syntax" | High impact |
| **async/await syntax** | Runtime exists, no language syntax | Medium |
| **User-defined generics** | Only built-in (Option, Result, Array) | Medium |
| **Guard clauses** | `pattern if condition` not in parser | Low |
| **OR patterns** | `0 \| 1 \| 2` not in Pattern enum | Low |
| **Rest patterns** | `[first, ...rest]` not implemented | Low |
| **Package CLI** | `atlas add`, `atlas remove` commands | Low |

---

## Current: v0.2 (In Progress)

**Theme:** First Usable Release

**Status:** 110/131 phases (84%) — see STATUS.md

### Remaining Work

| Category | Phases | Focus |
|----------|--------|-------|
| Interpreter | 2 | Debugger integration, REPL polish |
| CLI | 6 | Test runner, watch mode, package CLI |
| LSP | 5 | Hover, refactoring, find references |
| Polish | 5 | Testing, docs, stability |

### v0.2 Exit Criteria

- [ ] All 131 phases complete
- [ ] Interpreter/VM parity verified
- [ ] CLI tools documented
- [ ] LSP provides usable IDE experience

---

## v0.3: Closures & Lambdas

**Theme:** Complete Functional Programming Support

### Planned Features

| Feature | Current State | Work Required |
|---------|---------------|---------------|
| **Closures** | Functions can't capture outer scope | Environment capture, lifetime tracking |
| **Anonymous functions** | No `fn(x) { x + 1 }` syntax | Parser, typechecker, compiler changes |
| **Nested function capture** | Nested fns exist but can't close over | Same as closures |

### Why This Matters

First-class functions without closures feels incomplete. This is the single most impactful gap for users expecting modern language ergonomics.

### v0.3 Exit Criteria

- [ ] `let add = fn(x) { x + y }` works (captures `y`)
- [ ] Closures work in interpreter AND VM (parity)
- [ ] No memory leaks from captured references
- [ ] All existing tests pass

---

## v0.4: Async Language Syntax

**Theme:** async/await as First-Class Citizens

### Current State

The async **runtime** exists (`async_runtime/`):
- Tokio integration
- `AtlasFuture` type
- Channels, tasks, primitives
- `spawn_task`, `join_all`, `timeout`

What's missing is **language syntax**:
- `async fn` declarations
- `await` expressions
- Integration with type system

### Planned Features

| Feature | Notes |
|---------|-------|
| `async fn` | Function modifier |
| `await expr` | Suspend until future completes |
| `Future<T>` type | First-class in type system |

### v0.4 Exit Criteria

- [ ] `async fn fetch() -> string { ... }` parses and typechecks
- [ ] `await fetch()` suspends execution correctly
- [ ] Existing stdlib async functions use new syntax
- [ ] Interpreter/VM parity for async

---

## v0.5: Developer Experience

**Theme:** REPL and Tooling Polish

### Current State

- REPL history: ✅ Implemented
- TUI mode: ✅ Implemented
- Tab completion: ❌ Missing
- Syntax highlighting: ❌ Missing
- Inspection commands: ❌ Missing

### Planned Features

| Feature | Notes |
|---------|-------|
| **Tab completion** | Keywords, locals, stdlib |
| **Syntax highlighting** | Colorized output |
| **`.type expr`** | Show inferred type |
| **`.ast expr`** | Show parsed AST |
| **Session save/load** | Export REPL session |

### v0.5 Exit Criteria

- [ ] Tab completion works for all identifiers
- [ ] REPL output is colorized
- [ ] Inspection commands functional

---

## v0.6: Advanced Types

**Theme:** Type System Expansion

### Candidates

| Feature | Complexity | Value |
|---------|------------|-------|
| **User-defined generics** | High | High - enables libraries |
| **Type aliases** | Low | Medium - ergonomics |
| **Union types** | Medium | Medium - flexibility |
| **Guard clauses** | Low | Low - pattern matching |
| **OR patterns** | Low | Low - pattern matching |

### v0.6 Exit Criteria

- [ ] At least user-defined generics OR union types
- [ ] Type aliases if low-hanging fruit
- [ ] Pattern matching enhancements if time permits

---

## v0.7: Package Ecosystem CLI

**Theme:** Package Management Commands

### Current State

Package **infrastructure** exists:
- `atlas-package/` crate with full resolver
- `atlas.toml` manifest parsing
- Lockfile generation
- Registry support
- Conflict resolution

What's missing is **CLI integration**:
- `atlas add <pkg>` command
- `atlas remove <pkg>` command
- `atlas update` command
- `atlas publish` command

### v0.7 Exit Criteria

- [ ] `atlas add foo` adds dependency to atlas.toml
- [ ] `atlas remove foo` removes it
- [ ] Lockfile updated automatically
- [ ] Works with local registry

---

## v0.8: Performance & Profiling

**Theme:** Production Readiness

### Current State

- Optimizer: ✅ All 3 passes implemented
- Profiler: ✅ Exists in `profiler/`
- Benchmarks: ✅ Criterion suite exists

### Planned Focus

- Inline caching for method dispatch
- Benchmark regression testing
- Memory profiling
- GC evaluation (Arc vs tracing)

### v0.8 Exit Criteria

- [ ] Benchmark suite tracks regressions
- [ ] No performance cliffs identified
- [ ] Memory model finalized

---

## v0.9: Stabilization

**Theme:** Production Preparation

### Focus Areas

| Area | Work |
|------|------|
| **Fuzz testing** | Expand corpus, fix edge cases |
| **Error messages** | Audit all diagnostics |
| **Documentation** | Complete language reference |
| **Breaking changes** | Finalize before 1.0 lock |

### v0.9 Exit Criteria

- [ ] No known crashes
- [ ] All error messages reviewed
- [ ] Documentation complete
- [ ] 6+ months of v0.8 stability

---

## v1.0: Production Ready

**Theme:** Stability Commitment

### What v1.0 Means

- **Language stability:** Syntax won't break
- **API stability:** Stdlib follows semver
- **Semantic stability:** Behavior predictable
- **Support commitment:** Security fixes guaranteed

### v1.0 Requirements

- [ ] All major features shipped or explicitly deferred
- [ ] Real-world usage feedback incorporated
- [ ] Security audit completed
- [ ] Performance acceptable

---

## Post-1.0: Future Considerations

Not committed, may be explored:

| Feature | Notes |
|---------|-------|
| **JIT compilation** | Major effort, ~10-100x speedup potential |
| **Native codegen** | LLVM or Cranelift backend |
| **WASM target** | Browser execution |
| **Embeddability** | Library mode for other apps |

---

## Phase Scaffolding Guidelines

When creating phases for a new version:

### Phase File Template

```markdown
# Phase XX: {Title}

**Category:** {category}
**Version:** {target version}
**Complexity:** {low/medium/high}

## Summary
{1-2 sentences}

## Current State
{What exists today - VERIFY against codebase}

## Requirements
{Numbered list of specific, testable requirements}

## Acceptance Criteria
{Checkboxes that must all pass}

## Files to Modify
{List expected files - helps AI agents}

## References
- {Spec files}
- {Related decisions}
```

### Quality Rules

1. **~100 lines per phase** - Keep focused
2. **Verify current state** - Check codebase before planning
3. **Testable criteria** - Every requirement verifiable
4. **Dependencies explicit** - List blocking phases

---

## References

| Document | Purpose |
|----------|---------|
| `PRD.md` | Product requirements, principles |
| `STATUS.md` | Current progress tracking |
| `docs/specification/` | Language specifications |
| `memory/` | AI knowledge base |
| `phases/` | Active work queue |

---

*Last updated: 2026-02-19*
*Based on: Codebase verification (not just spec documents)*
