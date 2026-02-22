# Atlas v0.2 Known Issues and Technical Debt

**Date:** 2026-02-20
**Version:** v0.2
**Purpose:** Internal documentation of known limitations and technical debt

> Honesty is mandatory. No issue swept under the rug. Every known problem documented with severity and workaround (where applicable).

---

## Issue Severity Scale

| Level | Description |
|-------|-------------|
| 🔴 **Critical** | Blocks correct behavior; must fix before v0.3 |
| 🟠 **High** | Significant limitation; prioritize in v0.3 |
| 🟡 **Medium** | Notable gap; should address in v0.3 |
| 🟢 **Low** | Minor or cosmetic; backlog |

---

## Language / Semantics

### 🟡 No Tail Call Optimization
**Description:** Recursive functions without tail call optimization (TCO) will eventually stack-overflow on deep recursion (>~500 levels depending on system stack size).

**Impact:** Programs relying on deep recursion (e.g., functional-style tree traversal) will crash rather than running indefinitely.

**Workaround:** Use iterative patterns with while loops for deep recursion.

**v0.3 plan:** Investigate TCO for the VM (trampoline-style or direct jump).

---

### 🟡 Integer/Float Conflation
**Description:** Atlas has a single `number` type (f64). There are no distinct integer and float types. This matches JavaScript semantics but diverges from systems-language expectations.

**Impact:** Integer arithmetic can lose precision for values > 2^53. Bitwise operations are absent. No integer overflow detection.

**Workaround:** For integer-precise arithmetic, keep values < 2^53.

**v0.3 plan:** Consider `int` type for integer-specific operations. Research separate int/float type system.

---

### 🟡 No First-Class Error Handling
**Description:** Atlas does not have a `Result<T, E>` type or `try/catch` semantics. Runtime errors terminate the program with diagnostics.

**Impact:** Programs cannot recover from runtime errors. This is suitable for scripts but not for robust applications.

**Workaround:** Validate inputs before operations that might error (e.g., check array bounds before indexing).

**v0.3 plan:** Research error handling approaches (Result type, try/catch, error unions).

---

### 🟡 No Closures Over Mutable Variables
**Description:** Closures capture the environment at definition time. Capturing and mutating `var` variables inside closures may not work as expected.

**Impact:** Certain functional patterns (accumulators, state machines via closures) may not work.

**Workaround:** Return mutated values rather than relying on closure mutation.

**v0.3 plan:** Clarify closure semantics in spec; implement properly.

---

### 🟢 String Escape Sequences Incomplete
**Description:** Only `\n`, `\t`, `\\`, `\"` escape sequences are supported. Unicode escapes (`\u{XXXX}`) and raw string literals are not implemented.

**Impact:** Programs cannot embed arbitrary unicode via escape sequences.

**Workaround:** Paste unicode characters directly into string literals.

---

## Compiler / Type System

### 🟠 Type Inference Limited to Local Context
**Description:** Type inference works for local variable declarations but does not propagate across function boundaries or infer function return types.

**Impact:** All function parameter types and return types must be explicitly annotated.

**Workaround:** Always annotate function signatures (this is also good practice).

**v0.3 plan:** Research Hindley-Milner inference for Atlas.

---

### 🟡 Generic Constraints Not Enforced on Complex Types
**Description:** The generic type system supports basic `T` parameters but does not support trait/interface bounds (e.g., `T: Comparable`).

**Impact:** Generic functions cannot express type constraints. Type errors may occur at call sites rather than definition sites.

**v0.3 plan:** Design constraint system for generics.

---

### 🟡 Parser Error Recovery Is Basic
**Description:** On the first syntax error, the parser enters recovery mode but often cannot accurately parse subsequent statements. Multiple errors in a file may not all be reported.

**Impact:** Large files with early syntax errors may produce unhelpful cascading diagnostics.

**Workaround:** Fix errors one at a time, starting from the top of the file.

**v0.3 plan:** Implement panic-mode error recovery with statement synchronization.

---

### 🟢 No Unused Variable Warnings
**Description:** The compiler does not warn on unused variables, only enforces `let` vs `var` semantics for mutation.

**Impact:** Dead code not flagged; typos in variable names may not be caught.

**v0.3 plan:** Add unused variable lints.

---

## Standard Library

### 🟡 HTTP Client Is Synchronous Only
**Description:** The HTTP stdlib module provides synchronous request/response. No async HTTP is available.

**Impact:** Programs making multiple HTTP requests must make them sequentially.

**Workaround:** Use sequential requests; they are correct, just slower.

**v0.3 plan:** Research async runtime integration with stdlib.

---

### 🟡 File System Operations Lack Advanced Features
**Description:** File system functions cover read, write, list, delete, and move. Advanced operations (symlinks, permissions, extended attributes) are not implemented.

**Impact:** Programs needing fine-grained file system control are limited.

**Workaround:** Use FFI for advanced file system operations.

---

### 🟡 Regex Engine Limited Feature Set
**Description:** The regex stdlib wraps Rust's `regex` crate but exposes only basic operations (match, find, replace). Named capture groups, lookaheads, and non-greedy matching API is limited.

**Impact:** Complex text processing may require workarounds.

---

### 🟢 No DateTime Arithmetic
**Description:** The datetime module provides parsing and formatting but not arithmetic (e.g., adding days, computing differences).

**v0.3 plan:** Add duration arithmetic to datetime module.

---

## VM / Performance

### 🟠 No Incremental Compilation
**Description:** Every evaluation requires a full lex/parse/bind/typecheck/compile cycle. There is no caching of compiled bytecode.

**Impact:** Large programs are slow to start; repeated evaluation of the same program re-compiles from scratch.

**v0.3 plan:** Research bytecode serialization cache.

---

### 🟡 Large Value Enum (Memory)
**Description:** The `Value` enum is large (~200 bytes per value on 64-bit systems) because it contains all value variants inline.

**Impact:** Memory usage may be higher than necessary for programs with many values.

**v0.3 plan:** Box large variants (Array, Object, Function) to reduce enum size.

---

### 🟡 No JIT Compilation
**Description:** The VM executes bytecode via a dispatch loop. No just-in-time compilation to native code.

**Impact:** Compute-intensive programs (e.g., numerical simulations) are 10-100x slower than equivalent Rust/C.

**v0.3 plan:** Long-term research topic. Not feasible without significant investment.

---

### 🟢 Optimizer Not Applied to All Patterns
**Description:** The three-pass bytecode optimizer handles constant folding, dead code elimination, and peephole patterns. More sophisticated patterns (loop invariant hoisting, inlining) are not implemented.

---

## LSP / Tooling

### 🟡 LSP Re-parses on Every Change
**Description:** The LSP server re-parses the entire document on every keystroke change. No incremental parsing or change delta processing.

**Impact:** Large files may have noticeable LSP response latency (>100ms) on slow hardware.

**v0.3 plan:** Implement incremental parsing with change ranges.

---

### 🟡 LSP Workspace Indexing Is Single-Threaded
**Description:** Workspace symbol indexing runs synchronously in the background. Very large workspaces may take several seconds to index initially.

**v0.3 plan:** Parallelize workspace indexing.

---

### 🟢 Debugger Session Persistence
**Description:** The VM debugger state is not persisted between sessions. Breakpoints must be set each session.

---

### 🟢 REPL History Not Persisted
**Description:** REPL command history is not saved between sessions.

---

## Infrastructure

### 🟡 Windows CI Not Running
**Description:** GitHub Actions CI only runs on Linux x64. Windows is not in the CI matrix.

**Impact:** Windows-specific bugs will not be caught automatically.

**v0.3 plan:** Add Windows CI job.

---

### 🟡 No Code Coverage Measurement
**Description:** No `cargo-tarpaulin` or similar coverage tool is integrated. Test coverage cannot be quantified.

**Impact:** Coverage gaps are unknown. High test count does not guarantee high coverage.

**v0.3 plan:** Add coverage reporting to CI.

---

### 🟢 `cargo audit` Not in CI
**Description:** Dependency security auditing (`cargo audit`) is not automated in CI.

**v0.3 plan:** Add `cargo audit` as a CI step.

---

## Technical Debt (Code Level)

### 🟠 Some Stdlib Functions Have Shallow Implementations
**Description:** Approximately 15-20% of documented stdlib functions have basic implementations that may not handle all edge cases. Documented as working for the common case.

**Impact:** Programs using edge-case stdlib behavior may encounter unexpected errors or incorrect results.

**Prioritization:** Address in v0.3 as usage patterns reveal which functions need hardening.

---

### 🟡 Binder and TypeChecker Coupling
**Description:** The binder and type checker share mutable symbol table state in ways that make them difficult to run independently. This makes incremental analysis challenging.

**v0.3 plan:** Refactor to produce immutable symbol table from binder; type checker operates on it.

---

### 🟡 Test Infrastructure: Snapshot Drift Risk
**Description:** Insta snapshot tests capture exact output format. If error message formatting changes, many snapshots may need updating.

**Mitigation:** `cargo insta review` workflow makes updates manageable.

---

### 🟢 Phase Files Not Archived Consistently
**Description:** Some completed phase files remain in active directories rather than `archive/v0.2/`. Cleanup is cosmetic.

---

## Summary

**Critical issues:** 0 (no blockers to continued development)

**High priority for v0.3:**
- No incremental compilation (performance)
- Type inference limited to local context
- Parser error recovery basic
- Windows CI missing
- Shallow stdlib implementations

**The foundation is solid.** Known issues are characteristic of a language at this maturity level, not fundamental design flaws.
