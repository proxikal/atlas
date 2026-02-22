# Atlas v0.3 — The Foundation Version

**Status:** IN PROGRESS — Blocks 1–3 complete, Blocks 4–9 in execution
**Replaces:** V03_EXPLORATION_PLAN.md (research phase — complete)
**Research basis:** 2026-02-21 architectural decision session
**Block plan:** 9 blocks defined (current milestone). More blocks will follow.
**Phase target:** Unknown total — v0.3 is complete when exit criteria are met, not when a block count is reached.

---

## Vision

v0.3 is not a feature version. It is **The Foundation Version** — the architectural base that
Atlas builds on for all future versions. Every decision here is permanent.

The central insight: Atlas is one week old. Fixing the memory model now costs 130 phases.
Fixing it after v0.4, v0.5, v0.6 are built on top of it costs 400+ phases and introduces
regressions across thousands of tests. We do it now. We do it right. We never do it again.

**v0.3 exit state:** Atlas is a language with value semantics, explicit ownership, a trait
system, closures, type inference, proper error handling, live JIT compilation, and async/await
syntax. That is a world-class language foundation.

---

## How v0.3 Grows

v0.3 is not closed at 9 blocks. The 9 blocks are the **first milestone** — the core
language features. When Block 9 is complete, the following happens:

1. **Milestone commit** — push to remote, tag the milestone (not the version)
2. **Architectural session** — user and AI review what was built, identify gaps,
   define the next set of blocks together
3. **New blocks are planned** and added to this document
4. **Execution continues** until all exit criteria (all 3 groups) are verified

**v0.3 is done when the language works, not when a list of blocks is exhausted.**
New blocks discovered during architecture sessions are expected and normal.
The exit criteria are the authority — not the block count.

---

## Critical Constraints for Phase Scaffolding

### The Dependency Problem (v0.2 lesson)
v0.2 started with ~100 phases, ended at ~133 due to untracked dependencies and cascading
rewrites. The root cause: phases were scaffolded without a strict dependency layer order.

**v0.3 rule: phases within a block can run in parallel. Blocks are strictly sequential.**
Block 2 cannot begin until Block 1 is 100% complete. Block 3 cannot begin until Block 2
is 100% complete. This is non-negotiable.

### Parity is Sacred
Every change to value representation must maintain interpreter/VM parity throughout.
No phase is complete unless both engines produce identical output.

### No Partial Blocks
A block is not "mostly done." It is done or it is not started. Partial blocks create
the dependency hell we are explicitly avoiding.

---

## Block Dependency Graph

```
Block 1: Memory Model (Foundation)
    │
    ├──> Block 2: Ownership Syntax
    │        │
    │        └──> Block 3: Trait System
    │                  │
    │                  ├──> Block 4: Closures + Lambdas
    │                  │
    │                  ├──> Block 5: Type Inference
    │                  │
    │                  └──> Block 6: Error Handling (? operator)
    │
    └──> Block 7: JIT Integration (depends on Block 1 only)

Block 8: Async/Await (depends on Block 6)

Block 9: Quick Wins (mostly independent, after Block 1)
```

**Rule for scaffolding agent:** When writing phase files, every phase must declare its
block. Phases within the same block have no cross-block dependencies. If a phase needs
something from a different block, it belongs in a later block — not the current one.

---

## Block 1: Memory Model

**Depends on:** Nothing (this is the foundation)
**Estimated phases:** 25–35
**Key output:** Arc<Mutex<Value>> is gone. Value types work. shared<T> exists.

### What gets done
1. Define `ValueArray` (CoW array), `ValueMap` (CoW map) internal types
2. Replace `Arc<Mutex<Vec<Value>>>` with `ValueArray` throughout
3. Replace `Arc<Mutex<HashMap<...>>>` with `ValueMap` throughout
4. Implement CoW semantics: clone-on-write triggered by mutation of non-exclusively-owned value
5. Introduce `Shared<T>` wrapper for explicit reference semantics
6. Update all 300+ stdlib functions to work with value types
7. Update interpreter evaluation to use value semantics
8. Update VM execution to use value semantics
9. Update bytecode compiler for value type operations
10. Fix all tests broken by semantics change (array aliasing tests expected old behavior)
11. Full test suite passes at 100%

### Files changed (primary)
- `src/value.rs` — Value enum, ValueArray, ValueMap types
- `src/interpreter/` — eval functions, all collection operations
- `src/vm/mod.rs` — execution, stack semantics
- `src/stdlib/` — all 25 stdlib modules
- `src/compiler/` — value type handling in codegen

### Acceptance criteria (ALL required)
- [ ] No `Arc<Mutex<Vec<Value>>>` in production code
- [ ] No `Arc<Mutex<HashMap<...>>>` in production code
- [ ] Array mutation does not affect aliased copies
- [ ] Map mutation does not affect aliased copies
- [ ] `shared<T>` wrapper exists and works
- [ ] All existing tests pass (no regressions)
- [ ] Both engines produce identical output for all value operations
- [ ] No deadlock-class bugs possible (Arc::ptr_eq hacks gone)

---

## Block 2: Ownership Syntax

**Depends on:** Block 1 complete
**Estimated phases:** 15–20
**Key output:** `own`, `borrow`, `shared` as parameter annotations that parse, typecheck, and
are runtime-verified.

### What gets done
1. Parser: `own`, `borrow`, `shared` as parameter modifiers
2. AST: `OwnershipAnnotation` enum on function parameters and return types
3. Type checker: validate ownership annotations are consistent
4. Compiler: emit ownership transfer/borrow opcodes
5. VM: enforce ownership at runtime (debug mode assertions)
6. Interpreter: enforce ownership at runtime (debug mode assertions)
7. LSP: ownership annotations in hover, completion, diagnostics
8. Spec: `docs/specification/memory-model.md` updated to reflect implementation

### Syntax locked
```atlas
fn process(own data: Buffer) -> Result<string, string>
fn read(borrow data: Buffer) -> number
fn transform(shared data: Buffer) -> Buffer
fn allocate(size: number) -> own Buffer
```

### Acceptance criteria
- [x] All three annotations parse correctly (Phase 03–05)
- [x] Type checker rejects mismatched ownership (passing `borrow` where `own` required) (Phase 06–07)
- [x] Runtime assertion fires when ownership is violated (debug mode) (Phases 08–09, 11–12)
- [x] Both engines enforce ownership consistently (Phase 13 — 22 parity tests)
- [x] LSP shows ownership annotations in hover info (Phase 14–15)

**Completed:** 2026-02-22 — 16 phases, 9,236 tests passing

### Planned vs. Actual

- **Phases:** Estimated 15–20, delivered exactly 16.
- **Bug found:** `compiler/mod.rs::updated_ref` had `param_names: vec![]` (script artifact from scaffolding) — caught and fixed in Phase 13 parity check.
- **Insta snapshot gap:** 2 `ast_dump_tests` snapshots weren't updated when `return_ownership` field was added to `FunctionDecl`. Fixed in Phase 14.
- **LSP semantic tokens:** `own`/`borrow`/`shared` were already classified as KEYWORD by existing wildcard match — Phase 14 was primarily verification + hover.
- **No scope changes:** Block 2 executed exactly as planned. No phases merged, split, or added.

---

## Block 3: Trait System

**Depends on:** Block 2 complete
**Estimated phases:** 20–25
**Key output:** `trait` declarations, `impl` blocks, trait bounds on generics.

### What gets done
1. Parser: `trait Foo { fn method(...) }` declarations
2. Parser: `impl Foo for TypeName { ... }` implementations
3. AST: `TraitDecl`, `ImplBlock` nodes
4. Type checker: trait conformance, method resolution
5. Built-in traits: `Copy`, `Move`, `Drop`, `Display`, `Debug`
6. Trait bounds on generic parameters: `fn foo<T: Copy>(x: T)`
7. Compiler: trait method dispatch (static dispatch first, vtable in v0.4)
8. VM: trait dispatch execution
9. Ownership traits: `Copy` types can be freely copied, non-`Copy` types require `own`/`borrow`

### Why traits before closures
Closures capture values — those values need `Copy`/`Move` semantics to know whether they
are copied into the closure or moved. The trait system defines these semantics.

### Acceptance criteria
- [x] `trait` and `impl` declarations parse and typecheck
- [x] Built-in `Copy`, `Move`, `Drop` traits work
- [x] Trait bounds on generics compile and enforce correctly
- [x] Ownership traits integrate with Block 2 annotations
- [x] Both engines dispatch trait methods identically

### Planned vs. Actual (Block 3 — completed 2026-02-22)

- **Phases:** Estimated 20–25, delivered exactly 18.
- **Mangled names:** Impl methods compile to `__impl__Type__Trait__Method` named globals.
  Static dispatch via existing `GetGlobal` + `Call` opcodes — no new opcodes needed.
- **`trait_dispatch` annotation:** Added `RefCell<Option<(String, String)>>` to `MemberExpr`
  AST node. Typechecker annotates it; compiler and interpreter read it for dispatch.
- **Drop:** Defined as a built-in trait; explicit invocation only in Block 3.
  Automatic scope-exit drop deferred to v0.4 (requires scope tracking in both engines).
- **Display integration:** `Display` trait and `str()` stdlib are independent.
  `str()` does not dispatch through `Display` in Block 3 — types must call `.display()`
  explicitly. Automatic `str()` integration via Display is v0.4.
- **`Colon` token:** Already existed in the lexer. `TypeParamBound` parsing added.
- **Binder bug:** Binder was constructing `TypeParamDef` with `trait_bounds: vec![]`,
  never reading from AST. Fixed in Phase 10 by propagating `param.trait_bounds`.
- **LSP partial AST:** `document.rs` was discarding AST on parse errors. Fixed to store
  partial AST (items parsed before error) enabling hover/completion for in-progress code.
- **Test count added:** ~230+ tests across phases 01–18.

---

## Block 4: Closures + Anonymous Functions

**Depends on:** Block 3 complete (needs Copy/Move for capture semantics)
**Estimated phases:** 15–20
**Key output:** Anonymous functions, arrow syntax, closures over value types and owned values.

### What gets done
1. Parser: anonymous function syntax `fn(x: number) -> number { x + 1 }`
2. Parser: arrow syntax `(x) => x + 1` (sugar for single-expression anonymous fn)
3. Closure capture: values captured by copy (if `Copy`) or by move (if non-`Copy`)
4. Compiler: `MakeClosure` opcode with correct CoW capture semantics
5. VM: closure execution with value-semantic upvalues
6. Interpreter: closure execution with value-semantic upvalues
7. Type checker: infer closure types from context
8. Higher-order function type signatures: `fn(number) -> number` as a type

### Note on existing VM upvalue support
VM already has `MakeClosure`, `GetUpvalue`, `SetUpvalue` opcodes from v0.2. Block 4 updates
these to use value semantics (Block 1) and ownership-aware capture (Blocks 2+3).

### Acceptance criteria
- [ ] Anonymous functions parse, compile, and execute
- [ ] Arrow functions parse as sugar for anonymous functions
- [ ] Closures capture `Copy` values by copy
- [ ] Closures capture non-`Copy` values by move (caller loses ownership)
- [ ] Both engines handle closures identically
- [ ] Existing closure tests continue to pass

---

## Block 5: Type Inference

**Depends on:** Block 3 complete (inference needs trait constraints)
**Estimated phases:** 10–15
**Key output:** Local variable type inference, return type inference.

### What gets done
1. Local variable inference: `let x = 42` infers `number` without annotation
2. Return type inference: single-expression functions infer return type
3. Generic type inference: `identity(42)` infers `T = number`
4. Trait-constrained inference: inference respects `Copy`/`Move` bounds
5. Error messages: inference failure shows what was inferred vs. expected

### Scope boundary
Full Hindley-Milner inference is NOT in scope. Local inference only — function signatures
remain explicit. This is intentional: explicit signatures are better for AI generation
and codebase readability.

### Acceptance criteria
- [ ] Local `let` bindings do not require type annotation
- [ ] Return type can be omitted for single-expression functions
- [ ] Generic type arguments can be omitted when inferable
- [ ] Inference errors report clearly what was inferred vs. what was needed
- [ ] Both engines handle inferred types identically

---

## Block 6: Error Handling (`?` operator)

**Depends on:** Block 3 complete (Result<T,E> needs proper trait support)
**Estimated phases:** 10–15
**Key output:** `?` propagation operator, `try` blocks, proper Result/Option integration.

### What gets done
1. `?` operator: `let x = operation()?` — propagates Err/None to caller
2. Parser: `?` as postfix operator on expressions
3. Type checker: `?` only valid in functions returning `Result<T, E>` or `Option<T>`
4. Type checker: verify inner/outer error types are compatible
5. Compiler: emit early return on Err/None
6. VM + Interpreter: execute `?` propagation
7. `try` block (optional): `try { ... }` as expression that catches `?` propagation
8. Stdlib audit: update stdlib functions to return `Result<T, E>` where currently panicking

### Acceptance criteria
- [ ] `?` operator propagates Err to caller
- [ ] `?` operator propagates None to caller
- [ ] Type checker rejects `?` in non-Result/Option-returning functions
- [ ] Both engines propagate errors identically
- [ ] At least 20 stdlib functions updated to use Result<T, E>

---

## Block 7: JIT Integration

**Depends on:** Block 1 complete (value semantics changes bytecode format)
**Can run parallel to:** Blocks 2–6
**Estimated phases:** 10–15
**Key output:** `atlas-jit` wired to VM hotspot profiler. Hot numeric loops run native code.

### What gets done
1. Implement `Jump`, `JumpIfFalse`, `Loop` opcodes in Cranelift codegen
2. Implement `Call` opcode in Cranelift codegen (indirect dispatch)
3. Implement `GetGlobal`/`SetGlobal` with VM's global value array as pointer
4. Implement `And`/`Or` short-circuit logic with Cranelift conditional blocks
5. Wire `hotspot.rs` threshold to VM execution loop (threshold: 1000 invocations)
6. Replace interpreter loop for hot functions with JIT-compiled native function pointer
7. JIT cache invalidation when bytecode changes (REPL support)
8. Benchmark: JIT vs. interpreted for numeric loops (target: 10x+ speedup)

### Note
`atlas-jit` already has arithmetic opcodes implemented with Cranelift. Block 7 adds
control flow — the missing piece for real-world function compilation.

### Acceptance criteria
- [ ] Hot functions (>1000 calls) compile to native code automatically
- [ ] JIT and interpreter produce identical output for all supported opcodes
- [ ] Unsupported opcodes gracefully fall back to interpreter
- [ ] 10x+ speedup on arithmetic-heavy benchmarks vs. interpreted mode
- [ ] JIT can be disabled with `--no-jit` flag

---

## Block 8: Async/Await Syntax

**Depends on:** Block 6 complete (async fns return Result<Future<T>, E>)
**Estimated phases:** 10–15
**Key output:** `async fn`, `await expr`, `Future<T>` as first-class type.

### What gets done
1. Parser: `async fn` function modifier
2. Parser: `await expr` as expression
3. AST: `AsyncFn`, `AwaitExpr` nodes
4. Type checker: `async fn` return type is implicitly `Future<T>`
5. Type checker: `await` only valid inside `async fn`
6. Compiler: emit async/await bytecode (Tokio integration exists in stdlib)
7. VM: execute async functions using existing async_runtime infrastructure
8. Interpreter: execute async functions
9. Update stdlib async functions to use new syntax

### Note
The async runtime (Tokio integration, AtlasFuture, channels, tasks) already exists from
v0.2. Block 8 adds only the **language syntax** on top of this infrastructure.

### Acceptance criteria
- [ ] `async fn fetch() -> string` parses and type-checks
- [ ] `await fetch()` suspends and resumes execution
- [ ] `Future<T>` is a first-class type in the type system
- [ ] Both engines execute async code identically
- [ ] Existing stdlib async functions adopt new syntax

---

## Block 9: Quick Wins

**Depends on:** Block 1 (minimum), most are independent
**Estimated phases:** 5–10
**Key output:** String interpolation, guard clauses, OR patterns, implicit returns.

### What gets done
1. **String interpolation:** `"Hello, ${name}!"` syntax
2. **Implicit returns:** single-expression function body returns without `return` keyword
3. **Guard clauses in match:** `Ok(x) if x > 0 => ...`
4. **OR patterns in match:** `0 | 1 | 2 => "small number"`
5. **Rest patterns:** `[first, ...rest]` in destructuring

### Acceptance criteria
- [ ] String interpolation evaluates embedded expressions correctly
- [ ] Implicit returns work for single-expression functions
- [ ] Guard clauses filter match arms correctly
- [ ] OR patterns match any of the listed values
- [ ] All new syntax works in both engines identically

---

## Phase Scaffolding Guidelines

### For the scaffolding agent

**Read this before writing any phase file.**

1. **Assign every phase to a block.** No phase is "misc" or "general." Every phase belongs
   to exactly one block.

2. **Phases within a block are independent.** They touch different files or different aspects
   of the same feature. If two phases in the same block need to modify the same core file
   in incompatible ways, split them into sequential sub-phases within the block.

3. **State the block dependency explicitly.** Every phase file header must include:
   ```
   Block: 1 (Memory Model)
   Depends on: Block 1 complete  ← or "None" for Block 1 phases
   ```

4. **~100 lines per phase file.** Focus. One thing done completely.

5. **Test count is not a metric.** Quality and correctness are the metrics. Don't pad phases
   with trivial tests.

6. **Verify current state first.** Every phase file must include a "Current State" section
   that was verified against the actual codebase — not assumed.

7. **The most dangerous phase is always the first one in Block 1.** Changing `Value` in
   `value.rs` breaks everything. Phase 1 of Block 1 must be: define the new types,
   make them compile alongside the old types, run tests. Only then rip out the old types.

### Phase naming convention
```
phases/v0.3/
  block-01-memory-model/
    phase-01-cow-value-types.md
    phase-02-valuearray-implementation.md
    phase-03-valuemap-implementation.md
    ...
  block-02-ownership-syntax/
    phase-01-parser-own-borrow-shared.md
    ...
  block-03-trait-system/
    ...
```

### The v0.2 mistake to never repeat
In v0.2, Phase 60 needed something from Phase 90. This happened because phases were
scaffolded by feature area, not by dependency layer. The dependency graph above is the
correct scaffold order. Do not reorder blocks. Do not start a block before the previous
block's acceptance criteria are all checked.

---

## v0.3 Exit Criteria (ALL required)

**Completing all 9 blocks satisfies the first group only. All three groups must be ✅.**

### Group 1: Block completion (necessary, not sufficient)
- [ ] Block 1–9 complete (all AC phases committed, all ACs verified)
- [ ] No `Arc<Mutex<Value>>` in production code
- [ ] Value semantics: mutation does not affect aliases
- [ ] Ownership annotations parse, execute, and are runtime-verified
- [ ] Trait system: `Copy`, `Move`, `Drop` + user-defined traits
- [ ] Closures and anonymous functions work in both engines
- [ ] Type inference: locals and return types can omit annotations
- [ ] `?` operator propagates Result/Option errors
- [ ] JIT: hot functions compile to native code (10x+ benchmark improvement)
- [ ] Async/await: syntax exists, both engines execute async code

### Group 2: Integration and quality (post-block hardening)
- [ ] **Integration hardening:** closures × traits × ownership interaction — dedicated test
  suite covering capture of `Copy`/non-`Copy` types, trait methods in closures, `?` in
  closures. Both engines pass. No interactions discovered after block completion.
- [ ] **Stdlib ownership audit:** every stdlib function reviewed against value semantics and
  ownership model. Functions that should return `Result<T,E>` do. No silent panics remain.
  Target: all 25 stdlib modules audited, changes committed.
- [ ] **Error message quality:** every AT-coded error has a clear, actionable message.
  Verified by attempting to trigger each error class and reading the output.
- [ ] **REPL stability:** REPL works correctly with all v0.3 features. Tested manually
  against closures, traits, ownership, `?` operator.

### Group 3: Proof of completeness
- [ ] **Example program:** at least one non-trivial Atlas program exists in `examples/`
  that uses ownership annotations, a user-defined trait, a closure, and `?` propagation.
  It compiles and runs correctly in both interpreter and VM mode.
- [ ] **Performance baseline:** benchmark established for a numeric loop. JIT vs interpreted
  result documented in `docs/internal/benchmarks.md`. 10x+ improvement confirmed.
- [ ] **Test count:** ≥ 11,000 (accounts for integration + stdlib audit + example tests)
- [ ] Zero test failures
- [ ] Clippy clean
- [ ] Fmt clean
- [ ] **Spec complete:** `memory-model.md`, `types.md`, `syntax.md` all updated and
  internally consistent. No spec sections marked TODO or "v0.4."

### Instructions for AI — milestone vs. version

**When Block 9 AC check is committed:**
1. Run GATE V — verify Groups 1, 2, 3 against actual codebase
2. Groups 2 and 3 will almost certainly be incomplete. That is expected.
3. Log all blockers in STATUS.md under "v0.3 Version Gate Blockers"
4. Tag the milestone: `git tag v0.3-milestone-blocks-1-9` and push
5. Trigger an architectural session: surface the blocker list and gaps to the user
   so new blocks can be planned together. This is NOT asking for permission — it is
   the defined handoff point between the execution phase and the architecture phase.

**When new blocks are defined (after architectural session):**
- Add them to this document under a new `## Block N: Theme` section
- They follow the same scaffolding and execution protocol as the original 9
- Update the block dependency graph if dependencies exist

**When ALL three groups are ✅:**
- Run GATE V — advance to v0.3.0, tag, update STATUS.md and ROADMAP.md
- v0.3 is closed. Begin v0.4 planning.

---

## What v0.4 Gets (Not v0.3)

- **Compile-time ownership verification** — static analysis pass that proves ownership
  annotations correct without running the program. Requires v0.3 syntax to be stable first.
- **Full trait object dispatch (vtable)** — Block 3 uses static dispatch. v0.4 adds dynamic.
- **Advanced type inference** — cross-function inference, full H-M if validated as AI-friendly.
- **LLVM/AOT native compilation** — compile Atlas to a native binary. Cranelift exists, wire it.
- **Self-hosting bootstrap** — first steps toward Atlas compiling itself.

---

*This plan is locked. Changes require explicit justification and architectural review.*
*Last updated: 2026-02-21*
