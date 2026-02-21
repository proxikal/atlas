# Atlas v0.3 Exploration Plan

**Date:** 2026-02-20
**Status:** Research phase — NOT a roadmap
**Purpose:** Identify areas to investigate before v0.3 planning begins

> This is an exploration document, not a commitment. Every item here requires research before deciding whether to implement it. We are asking "what should we investigate?" not "what will we build?"

---

## Philosophy

v0.3 exploration should be guided by one question: **"What would make Atlas the best AI-first language it can be?"**

Not:
- "What features do other languages have?" (feature parity is not the goal)
- "What would users want?" (we don't have users yet)
- "What's easy to implement?" (ease is not a tiebreaker)

The tiebreaker remains: **"What's best for AI?"**

---

## Priority Research Areas

### Area 1: Type System Improvements

#### 1a. Hindley-Milner Type Inference
**Research question:** Can Atlas adopt full H-M inference without breaking the AI-friendly explicit-annotation design?

**Hypothesis:** Full inference would make AI-generated code less verbose but might make AI-generated code harder to debug (inferred types are invisible).

**Research tasks:**
- Study how Rust balances inference and annotation requirements
- Prototype inference for local variables beyond declarations
- Measure: does inference reduce or increase AI generation errors?
- Evaluate: when should inference be mandatory vs. optional?

**Key tension:** More inference = less code for AI to generate. But explicit annotations = AI verifies its own types. Both serve AI-first differently.

---

#### 1b. Interface / Trait System
**Research question:** Should Atlas have structural typing, nominal typing, or traits for polymorphism?

**Currently:** Atlas has generics `T` but no constraints. This limits expressiveness.

**Options to research:**
- **Structural typing** (like TypeScript interfaces): types compatible if shapes match
- **Nominal traits** (like Rust traits): explicit implementation required
- **Protocol-based** (like Swift): implicit conformance to named protocols

**Key question for AI-first:** Which approach makes AI-generated code most likely to be correct? Structural typing is more permissive; nominal types provide better error messages.

---

#### 1c. Result<T, E> and Error Handling
**Research question:** What error handling model best serves Atlas programs?

**Currently:** Runtime errors terminate with diagnostics. No recoverable errors.

**Options to research:**
- `Result<T, E>` with `?` operator (Rust style)
- `try/catch` exception handling (JS style)
- Effect system for errors (Koka style)
- Error union types (Zig style)

**Key question for AI-first:** Which model produces the most predictable AI-generated error handling code?

---

### Area 2: Module and Package System Improvements

#### 2a. Module Resolution Research
**Research question:** What module system best serves AI code generation?

**Currently:** Basic module imports via `import` keyword. `ModuleExecutor` handles resolution.

**Questions:**
- Should modules be content-addressed (hash-based, like nix)?
- Should imports be explicit re-exports or wildcard?
- Should circular imports be allowed or forbidden?
- What's the right import granularity (file, directory, package)?

**AI-first consideration:** AI agents need deterministic import semantics. Ambiguous imports (multiple files match the same import path) should not exist.

---

#### 2b. Package Registry Design
**Research question:** What does a package registry that serves AI code generation look like?

**Currently:** Package manager CLI exists but no actual registry.

**Research areas:**
- Content-addressed storage for reproducible builds
- Machine-readable package metadata format
- Versioning strategy (semver vs. date-based vs. hash-pinned)
- Dependency resolution algorithm (pure functional preferred)

---

### Area 3: Performance Improvements

#### 3a. JIT Compilation Research
**Research question:** Is JIT compilation feasible for Atlas within a reasonable scope?

**Currently:** VM executes interpreted bytecode (no native code generation).

**Research areas:**
- Cranelift as a backend (Rust-native, used by Wasmtime)
- LLVM IR generation from Atlas bytecode
- Copy-and-patch JIT (Python 3.13 approach — simpler)
- Trade-off: JIT complexity vs. performance gain for typical Atlas programs

**Honest assessment:** Full JIT is a multi-year project. Research first to calibrate scope.

---

#### 3b. Incremental Compilation
**Research question:** How to add incremental compilation without a full architecture rewrite?

**Currently:** Full reparse/recompile on every change.

**Research areas:**
- Demand-driven compilation (only recompile changed modules)
- Bytecode caching with content-hash invalidation
- Incremental type checking (re-typecheck only changed functions)
- What does Rust's incremental compilation look like?

**Feasibility estimate:** Medium difficulty. Doesn't require architecture overhaul.

---

#### 3c. Memory Allocator Tuning
**Research question:** Would a custom allocator improve Atlas performance?

**Currently:** System allocator (malloc/free on Linux, HeapAlloc on Windows).

**Research areas:**
- `jemalloc` for allocation-heavy workloads
- Arena allocators for AST/compilation phases (free the arena, not individual nodes)
- Value pool for common small values (null, true, false, small integers)

**Feasibility estimate:** Low effort, potentially meaningful gains.

---

### Area 4: Concurrency and Async

#### 4a. Async/Await Language Semantics
**Research question:** Should Atlas have native async/await? If so, how?

**Currently:** Basic async support in stdlib via Tokio (Rust side). No async/await syntax in Atlas source language.

**Research areas:**
- Is async/await syntax desirable for an AI-first language?
- What does async semantics look like in Atlas's type system?
- Single-threaded event loop (JS model) vs. multi-threaded (Go model)?
- How does the embedding API handle async programs?

**Key question for AI-first:** AI agents are often orchestrating concurrent operations. Native async support would make Atlas a natural choice for AI workflows.

---

#### 4b. Actor Model Exploration
**Research question:** Would an actor model (like Erlang) suit Atlas better than shared-memory concurrency?

**Research areas:**
- Message-passing concurrency fits well with AI agent coordination
- Atlas programs as lightweight processes with mailboxes
- Supervision trees for robust multi-agent systems

---

### Area 5: Tooling Improvements

#### 5a. Incremental LSP Analysis
**Research question:** How to implement incremental document analysis in the LSP server?

**Currently:** Full reparse on every keystroke.

**Research areas:**
- Tree-sitter for incremental parsing (separate from Atlas parser)
- Edit deltas from LSP `textDocument/didChange` with partial re-parsing
- Lazy analysis: only typecheck the function containing the cursor

**Feasibility estimate:** High value, medium difficulty.

---

#### 5b. Better REPL Experience
**Research question:** What would make the Atlas REPL excellent for exploration?

**Currently:** Basic REPL with session state. History not persisted.

**Research areas:**
- Multiline input with bracket matching
- Type display for every expression
- Symbol completion in REPL
- History persistence across sessions
- REPL-specific syntax shortcuts

---

#### 5c. AI Integration APIs
**Research question:** What APIs would make Atlas the ideal substrate for AI agent workflows?

**Currently:** Embedding API exists. Security context threading enabled.

**Research areas:**
- Structured output extraction (JSON paths from Atlas values)
- Program synthesis API (generate Atlas from natural language spec)
- Explanation API (explain why a program type-checked or failed)
- Verification API (prove properties about Atlas programs)

**Key question:** How can Atlas become not just AI-generated, but AI-analyzable?

---

### Area 6: Language Features to Research

#### 6a. Pattern Matching
**Research question:** Should Atlas have pattern matching (match expressions)?

**Currently:** Only if/else for branching.

**Research areas:**
- Pattern matching syntax that AI can generate reliably
- Exhaustiveness checking (every variant matched)
- Destructuring in let bindings
- Guard clauses

**AI-first case for pattern matching:** Patterns make control flow explicit and checkable. AI-generated pattern matches can be verified exhaustive by the type system.

---

#### 6b. Closures and First-Class Functions
**Research question:** What gaps exist in the current closure/function system?

**Currently:** Functions can be passed as values. Closures capture environment.

**Known gaps:**
- Closures over mutable variables may not work correctly
- No anonymous function syntax (only `fn name(...) { }`)
- No implicit return for single-expression functions

**Research tasks:**
- Define precise closure semantics in spec
- Consider arrow function syntax `(x) => x + 1`
- Research how other AI-friendly languages handle closures

---

#### 6c. String Interpolation
**Research question:** Should Atlas support string interpolation?

**Currently:** String concatenation only (`"hello " + name`).

**Research:** What syntax is most AI-generation-friendly? (Template literals, `${}`, `#{}`, `\()`?)

---

## Not in Scope for v0.3 Research

These are important but too large for v0.3 research scope:

- **Garbage collector**: Current reference counting is fine for now
- **Self-hosting**: Compiling Atlas in Atlas is a v1.0+ goal
- **Type classes / Typeclasses**: Explore simple traits first
- **Macros**: Avoid metaprogramming complexity at this stage
- **Native compilation**: LLVM/Cranelift backend is v0.5+ territory
- **Package registry infrastructure**: Website, auth, CDN — not yet

---

## Research Process

For each research area above:

1. **Define the question clearly** — What exactly are we trying to learn?
2. **Literature review** — How do existing languages solve this?
3. **Prototype** — Small implementation to test feasibility
4. **Evaluate against AI-first criteria** — Does this make Atlas better for AI?
5. **Spec draft** — If promising, draft language specification changes
6. **Phase planning** — If approved, write phase files for implementation

**Time estimate:** v0.3 research phase should take 2-3 months before implementation phases begin.

---

## Prioritized Research Queue

**Investigate first (highest leverage):**
1. Type inference (H-M) — removes verbosity, helps AI generation
2. Result<T, E> error handling — needed for robust programs
3. Incremental LSP analysis — quality-of-life for developers
4. Incremental compilation — performance and LSP quality

**Investigate second:**
5. Interface/trait system — needed for polymorphism
6. Arrow function syntax — ergonomics for AI generation
7. Pattern matching — expressiveness and verifiability
8. String interpolation — developer ergonomics

**Investigate third (longer-horizon):**
9. Async/await semantics
10. Actor model exploration
11. JIT research
12. Package registry design

---

## Open Questions Before v0.3 Begins

1. What is the target "user" of Atlas in v0.3? (Embedding host? Script runner? Full application framework?)
2. Should v0.3 prioritize language features or tooling quality?
3. What is the right test count target for v0.3? (6,764 → ?)
4. Should v0.3 have a public preview? (Even for AI agents, not humans?)
5. How do we measure "AI-first" success? What metrics prove the design is working?

---

*This document is a starting point for v0.3 discussion, not a commitment. Every item needs validation before implementation. Good exploration produces clarity about what NOT to do, not just what to do.*
