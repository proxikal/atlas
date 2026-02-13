# Atlas Compiler Principles

**Purpose:** Core invariants that define what Atlas IS and how it's built. These rarely change without fundamental redesign.

**For AI Agents:** Use this document during GATE -1 (Sanity Check) to evaluate whether user requests align with Atlas's fundamental nature.

---

## What Atlas IS

### 1. AI-Native Language
**Core identity:** First language designed natively for AI agents while remaining perfectly usable by humans.

**Principles:**
- **Explicit over implicit** - No truthy/falsey, no type coercion, no implicit any
- **Machine-readable errors** - Structured diagnostics with codes, positions, help text
- **Consistent patterns** - AI learns by pattern matching; inconsistency = hallucinations
- **Typed with inference** - Static types required, but inferred where possible

**Reference:** `docs/philosophy/ai-manifesto.md`, `docs/philosophy/why-strict.md`

**Never violate:** Adding implicit behavior, truthy/falsey coercion, dynamic typing

---

### 2. Dual-Engine Architecture
**Core identity:** Every feature runs on BOTH interpreter and VM.

**Principles:**
- **Parity is sacred** - Both engines produce identical output, diagnostics, errors
- **Implementation-driven** - Build feature in both engines, test comprehensively
- **No engine-specific features** - If one engine can't support it, neither gets it

**Reference:** `docs/implementation/10-interpreter.md`, `docs/implementation/12-vm.md`

**Never violate:** Breaking parity, favoring one engine, implementing for only one engine

---

### 3. Production-Grade from Start
**Core identity:** Built for decades, not sprints. Quality over shipping fast.

**Principles:**
- **Comprehensive testing** - Both engines, edge cases, error handling
- **Quality over metrics** - Never simplify code for arbitrary line limits
- **Compiler complexity is necessary** - Not a CRUD app, algorithms can be complex
- **Correctness first** - Get it right, then optimize

**Reference:** `docs/guides/code-quality-standards.md`, `docs/guides/testing-guide.md`

**Never violate:** Oversimplifying for line counts, skipping tests, shipping broken parity

---

### 4. Strict and Explicit
**Core identity:** No implicit behavior, no coercion, no truthy/falsey.

**Principles:**
- **No implicit any** - All types explicit or inferred from explicit source
- **No type coercion** - `"5" + 3` is an error, not `"53"` or `8`
- **Bool only for conditions** - `if (x)` is error, must be `if (x != null)` or `if (x == true)`
- **Same-type comparisons** - `==` and `!=` require matching types

**Reference:** `Atlas-SPEC.md`, `docs/specification/language-semantics.md`

**Never violate:** Adding implicit behavior, loosening type rules, allowing coercion

---

## How We Build Atlas

### 1. Implementation-Driven Development
**Approach:** Real compilers (rustc, Go, TypeScript, Clang) don't use strict TDD for features.

**Process:**
- **Features:** Implement → test alongside/after → iterate (NOT test-first)
- **Bugs:** Test first (strict TDD) → fix → verify GREEN
- **Exploratory work:** Discover edge cases WHILE implementing

**Reference:** `.claude/skills/atlas/gates/gate-2-implement.md`

**Never violate:** Forcing test-first for features (causes v0.1 problem: hours of tests, zero implementation)

---

### 2. Quality Over Metrics
**Approach:** Correctness and clarity over arbitrary limits.

**Principles:**
- **Line limits are soft targets** - 1000 line soft target, justified exceptions
- **Real compiler reality** - VM: 1972 lines, Bytecode: 1421 lines, Lexer: 908 lines
- **Never simplify for counts** - Quality and correctness matter more
- **Complex algorithms needed** - Compilers aren't simple, embrace necessary complexity

**Reference:** `.claude/skills/atlas/gates/README.md` Line Limits section, `.claude/skills/atlas/skill.md`

**Never violate:** Oversimplifying compiler logic to hit arbitrary line counts

---

### 3. Comprehensive Testing
**Approach:** Test heavily, but not test-first for features.

**Principles:**
- **Both engines always** - Every feature tested in interpreter AND VM
- **Parity verification** - Identical output, diagnostics, errors, edge cases
- **Edge case coverage** - Basic functionality, error handling, boundary conditions
- **Test frameworks** - rstest (parameterized), insta (snapshots), proptest (property-based)

**Reference:** `docs/guides/testing-guide.md`, `docs/implementation/15-testing.md`

**Never violate:** Skipping tests, testing only one engine, ignoring parity failures

---

### 4. Phase-Based Development (v0.2)
**Approach:** Structured development via phase files.

**Principles:**
- **Phase dependencies matter** - Prerequisites must be met before starting
- **Version scope is defined** - v0.2 has 68 phases, scope is LOCKED
- **No scope creep** - Features not in v0.2 phases wait for v0.3 or later
- **Complete before moving** - Finish phase fully before next phase

**Reference:** `STATUS.md`, phase files in `phases/*/`

**Never violate:** Starting phase without dependencies, adding out-of-scope features during version

---

## What Atlas Is NOT

### 1. NOT Dynamically Typed
Atlas uses static typing with inference. No runtime type changes, no `any` type.

---

### 2. NOT Loosely Typed
No truthy/falsey, no coercion, no implicit conversions. Explicit always.

---

### 3. NOT Single-Engine
Both interpreter and VM required. Parity is non-negotiable.

---

### 4. NOT A CRUD App
Atlas is a compiler. Complex algorithms are necessary. Parser, typechecker, VM, bytecode = inherently complex.

---

### 5. NOT Feature-Complete Yet
v0.1 is foundation. v0.2 adds depth. Future versions will add:
- Generics beyond arrays (when design is right)
- Async/await (when model is clear)
- Concurrency primitives (when approach is defined)
- JIT compilation (when complexity is justified)

**These are "under research" - not ready for implementation.**

**Reference:** `Atlas-SPEC.md` "Advanced Features Under Research"

---

## Anti-Patterns to Watch For

### 1. Strict TDD for Features
**Problem:** Compilers require exploratory implementation. Writing tests before understanding the algorithm causes the "v0.1 problem" (hours of tests, zero implementation).

**Correct approach:** Implement → test alongside/after → iterate.

**Exception:** Bugs use strict TDD (test first, then fix).

---

### 2. Oversimplifying for Line Counts
**Problem:** Breaking complex compiler algorithms into tiny files to meet arbitrary 250-line limits reduces quality and clarity.

**Correct approach:** Quality over metrics. 1000-line soft target with justified exceptions. Real compilers have large modules.

---

### 3. Breaking Parity
**Problem:** Implementing feature in one engine but not the other, or allowing different output between engines.

**Correct approach:** Both engines always. Identical output, diagnostics, errors. Parity failures are BLOCKING.

---

### 4. Scope Creep During Version
**Problem:** Adding features not in current version's phase plan (e.g., adding goroutines during v0.2 when no v0.2 phase mentions them).

**Correct approach:** Stick to version scope. Out-of-scope features wait for next version.

---

### 5. Skipping Dependencies
**Problem:** Starting a phase without prerequisites (e.g., implementing HashMap before hash function exists).

**Correct approach:** Check dependencies at GATE 0.5. If missing, STOP and report.

---

### 6. Adding Implicit Behavior
**Problem:** Making conditionals accept non-bool, adding truthy/falsey, allowing type coercion "for convenience."

**Correct approach:** Explicit always. Atlas's identity is strictness. Don't compromise for convenience.

---

## Using This Document

**During GATE -1 (Sanity Check):**
1. Read user request
2. Check against principles in this doc
3. If conflict found, open discussion (not block)
4. Present reasoning, alternatives, risks
5. User can override, but risks are clear

**During brainstorming:**
1. Reference principles when evaluating ideas
2. Challenge ideas that conflict with core identity
3. Discuss tradeoffs intelligently
4. Help user understand implications

**During implementation:**
1. Verify approach aligns with principles
2. Question deviations before they become code
3. Maintain quality standards throughout

---

## Evolution

This document can evolve as Atlas matures, but changes should be rare and deliberate:
- **Core identity changes** - Requires fundamental redesign discussion
- **New principles** - Add when pattern emerges from experience
- **Anti-pattern updates** - Add when new mistakes are discovered
- **Reference updates** - Keep links current as docs reorganize

**Don't change on a whim. These are Atlas's foundation.**
