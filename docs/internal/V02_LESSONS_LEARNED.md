# Atlas v0.2 Lessons Learned

**Date:** 2026-02-20
**Version:** v0.2
**Purpose:** Capture what worked, what didn't, and insights for future development cycles

> This document is for honest reflection. It guides v0.3 and beyond.

---

## 1. What Worked Well

### Phase-Based Development with ~100-Line Phase Files

**Observation:** Breaking every development task into ~100-line phase specification files was highly effective. Each phase had:
- A clear objective
- Explicit file targets with line estimates
- A minimum test count
- Acceptance criteria

**Why it worked:**
- Forced decomposition of large features into bounded tasks
- Made progress measurable and transparent
- Gave the AI agent clear completion criteria
- Prevented scope creep within a phase
- Made phases reviewable before starting

**Lesson:** Maintain this format for v0.3. The 100-line constraint is about focus, not word count.

---

### Test-First Development

**Observation:** Writing explicit test counts in phase files ("minimum 40 tests") and adopting TDD-style discipline produced a test suite that caught real bugs.

**Evidence:**
- 6,764 tests in atlas-runtime, all passing
- Bugs caught during phase development, not in downstream phases
- Regression suite prevented reintroduction of fixed bugs

**Why it worked:**
- Tests document expected behavior, not just verify it
- Higher confidence when refactoring
- Fuzzing found no panics because unit tests had already exercised edge cases

**Lesson:** Keep minimum test count requirements in phase files. Increase minimums where coverage was shallow.

---

### Dual-Engine Design (Interpreter + VM) From the Start

**Observation:** Building both the tree-walking interpreter and the bytecode VM from phase one, and maintaining parity between them, was architecturally correct.

**Why it worked:**
- Interpreter is simpler to debug; VM is faster in production
- Parity testing caught bugs in both engines
- REPL uses interpreter (fast startup); `atlas run` uses VM (fast execution)
- Neither engine had to be retrofitted as a bolt-on

**Lesson:** Don't abandon the dual-engine design. In v0.3, optimize both engines rather than collapsing to one.

---

### Structured Correctness Phases Before Features

**Observation:** The 12 Correctness phases (fixing security threading, parity issues, bounds safety) came before feature additions. This sequencing was correct.

**Evidence:**
- Phase correctness-01 through 09 fixed structural problems that would have been expensive to fix later
- Security context threading (correctness-01) prevented an entire class of use-after-free bugs
- Parity fixes (correctness-04, 05) prevented divergence that was growing with each new feature

**Lesson:** Begin v0.3 with a correctness audit similar to what correctness-01 through 12 did for v0.2.

---

### Documentation Alongside Code (Not After)

**Observation:** Polish phase 03 (documentation) was done concurrently with the last implementation phases, not after. This avoided the "documentation debt" common in projects.

**Why it worked:**
- Documented behavior while implementation was fresh in context
- Documentation errors found by the 91 verification tests
- Embedding guide written while the embedding API was fresh

**Lesson:** Don't defer documentation. Schedule it alongside feature phases, not after.

---

### Fuzzing Infrastructure Early (Infra Phase 06)

**Observation:** cargo-fuzz infrastructure was established in infra phase 06, before the compiler was complex. This made the fuzzer useful immediately and remained maintained throughout.

**Evidence:**
- Fuzzing found no panics — because early fuzzing forced robust error handling from the start
- 7 fuzz targets by v0.2 completion

**Lesson:** Keep fuzzing integrated into the development workflow. In v0.3, run fuzz campaigns in CI.

---

### STATUS.md as Single Source of Truth

**Observation:** Maintaining STATUS.md as the authoritative progress tracker prevented status inflation and confusion.

**Why it worked:**
- One place to look; always authoritative
- Phase handoffs required updating STATUS.md — made progress explicit
- "Check STATUS.md first" was a reliable answer to "where are we?"

**Lesson:** Keep this practice. Add version-stamping to STATUS.md entries.

---

## 2. What Didn't Work Well

### Stdlib Depth vs. Breadth Imbalance

**Observation:** The stdlib covers 25 categories and 300+ documented functions, but approximately 15-20% of functions have shallow implementations. Breadth was prioritized over depth.

**Why it happened:**
- Phase metrics rewarded function count and documentation lines
- Deep, correct implementations of fewer functions would have been better
- "Completed" phases sometimes masked function stubs

**Consequence:** Programs using edge-case stdlib behavior may encounter bugs.

**Lesson for v0.3:** Set quality criteria per function, not just count. Require integration tests for every stdlib function, not just documentation.

---

### Type Inference Scope Was Underestimated

**Observation:** The type checker was implemented with local inference only. This required function signatures to be fully annotated, which is verbose and frustrating in practice.

**Why it happened:**
- Hindley-Milner inference was not scoped into any v0.2 phase
- Local inference seemed sufficient during foundation phases; the limitation became apparent with complex programs

**Consequence:** Atlas programs are verbose compared to languages with full inference (Rust, Haskell, Swift).

**Lesson for v0.3:** Invest in a proper inference engine early. This is foundational infrastructure, not a feature.

---

### Parser Error Recovery Was Never Prioritized

**Observation:** Parser error recovery (continuing to parse after the first error, to report multiple errors) was never explicitly phased. It remained basic throughout v0.2.

**Why it happened:**
- Parsing architecture made error recovery an afterthought
- Phase files focused on correctness of successful parses, not graceful failure

**Consequence:** Large programs with early syntax errors give unhelpful diagnostic output.

**Lesson for v0.3:** Schedule a dedicated phase for parser error recovery with synchronization points.

---

### Windows Testing Gap

**Observation:** Windows compatibility was never verified. CI only covers Linux x64. macOS ARM is the development platform.

**Why it happened:**
- GitHub Actions Linux runner was the default
- No explicit phase for cross-platform CI

**Consequence:** Windows users (if there were any) would likely find platform-specific bugs.

**Lesson for v0.3:** Add Windows to CI in the first infra phase of the cycle.

---

### Phase Files Accumulated in Active Directories

**Observation:** Some completed phase files were not moved to the archive directory promptly. This created noise in `phases/` listings.

**Why it happened:**
- Phase completion checklist was not strictly enforced
- Archive step was sometimes omitted in handoffs

**Lesson:** Add archive step explicitly to the phase handoff protocol.

---

## 3. Architectural Insights

### The Value Enum Is Both Strength and Weakness

**Observation:** `Value` as a single enum covering all types (Number, String, Bool, Null, Array, Object, Function, Builtin, etc.) is simple to reason about but has downsides.

**Strengths:**
- Single match statement handles all value types
- No boxing needed for small values (numbers, bools, null)
- Arc<Mutex<T>> for shared types is correct and safe

**Weaknesses:**
- Large enum size (~200 bytes) means every value copy is expensive
- No separation between "owned" and "shared" values
- Function values include symbol table references that complicate serialization

**Lesson:** In v0.3, consider boxing large variants. Benchmark first.

---

### Security Context Threading Was Right

**Observation:** The decision to thread `SecurityContext` via `Arc<SecurityContext>` rather than as a raw pointer (or global state) proved correct. It enabled:
- Safe multi-runtime scenarios
- Clean embedding API
- Testable security policies

**Lesson:** Keep this pattern. Extend it to other "ambient" state if needed in v0.3.

---

### Separate Binder and TypeChecker Passes Were Right

**Observation:** The pipeline (parse → bind → typecheck) with separate passes produces cleaner errors and enables better error recovery than a single-pass design.

**Evidence:**
- Binder errors (undefined variable) reported separately from type errors
- TypeChecker can inspect fully-bound AST, enabling better analysis

**Weakness:**
- Coupling between binder symbol table and typechecker makes incremental analysis hard

**Lesson:** Refactor to produce an immutable symbol table from the binder. TypeChecker should not need mutable access.

---

### OnceLock Registry for Builtin Dispatch Was the Right Fix

**Observation:** Correctness phase 02 replaced the dual-match builtin dispatch with a `OnceLock<HashMap>` registry. This eliminated ~500 lines of duplicate match code and made adding new builtins trivial.

**Lesson:** Registries beat match statements for open-ended dispatch. Apply this pattern to any new dispatch surface in v0.3.

---

## 4. Testing Strategy Insights

### Rstest Parameter Matrices Are Effective

**Observation:** `#[rstest] #[case(...)]` parameter matrices dramatically improved test coverage with minimal code duplication.

**Evidence:** Hundreds of test cases written as single `#[rstest]` functions with 10-30 case variants.

**Lesson:** Default to rstest for any test with multiple valid inputs. Avoid writing individual test functions for each case.

---

### Snapshot Tests Are Fragile but Valuable

**Observation:** Insta snapshot tests capture exact diagnostic output. They break on intentional formatting changes but catch unintentional regressions.

**Lesson:** Use snapshots for diagnostic output (error messages, LSP responses). Have a clear "update snapshot" workflow documented.

---

### Determinism Tests Should Be Standard

**Observation:** The stability phase added explicit determinism tests (evaluate twice, compare results). These caught potential (though not actual) non-determinism.

**Lesson:** Include at least a few determinism tests for any stateful component. Add to testing-patterns.md.

---

## 5. AI-First Design Philosophy Insights

### What "AI-First" Means in Practice

**Observation:** The Atlas design principle "What's best for AI?" plays out in specific concrete ways:
- **Predictable grammar:** Consistent syntax patterns that AI can learn to generate correctly
- **Strong typing:** Types help AI verify generated code is correct
- **Clear diagnostics:** Structured error codes (AT-XXXX) that AI can parse and reason about
- **Stable APIs:** No surprise overloading or implicit coercions that confuse AI generation

**What "AI-first" does NOT mean:**
- Making the language permissive (opposite — strict typing helps AI)
- Removing error messages (opposite — detailed errors help AI debug)
- Auto-correcting mistakes (AI needs to learn from exact errors, not masked ones)

**Lesson:** Continue developing the AI-first principles document. The philosophy is sound but needs more concrete language design guidance.

---

### AI Agents as Developers Are Effective for Structured Tasks

**Observation:** AI agents excel at:
- Implementing clearly-specified features from phase documents
- Writing comprehensive test suites
- Generating documentation from existing code patterns
- Refactoring code with explicit before/after criteria

**AI agents struggle with:**
- Architectural decisions without clear constraints
- Debugging subtle semantic bugs without reproduction steps
- Managing cross-cutting concerns across many files simultaneously

**Lesson:** Phase documents must include explicit file targets, test counts, and acceptance criteria. Under-specified phases produce under-specified implementations.

---

## 6. Summary: Top Lessons for v0.3

1. **Type inference first** — Before feature work, implement proper Hindley-Milner inference
2. **Stdlib hardening** — Quality over quantity; require integration tests per function
3. **Parser error recovery** — Dedicated phase early in the cycle
4. **Windows CI** — Add in first infra phase
5. **Incremental analysis** — Foundation for LSP performance and large codebase support
6. **Value enum size** — Benchmark and consider boxing before adding more value types
7. **Phase archive discipline** — Enforce in handoff protocol

---

*This document captures honest reflection on v0.2. Use it to make v0.3 better, not to re-litigate v0.2 decisions.*
