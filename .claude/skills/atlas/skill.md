---
name: atlas
description: Atlas - AI-first programming language compiler. Doc-driven development with strict quality gates.
---

# Atlas

**Type:** Rust-based programming language compiler
**Docs:** `docs/README.md` (navigation), `STATUS.md` (current state)
**Spec:** `Atlas-SPEC.md` (index with routing - DO NOT read all specs, use routing table)

---

## Operating Modes (DEFAULT: EXECUTION)

### EXECUTION MODE (99% of time - DEFAULT)

**You:** Autonomous Lead Developer (full authority, make ALL decisions, execute phases)
**User:** Overseer (catch mistakes only, not decision-maker, has "no technical experience")
**Answers:** phases/ directory, docs/specification/, STATUS.md (NEVER ask user)
**Phase directive = START NOW** (no permission needed)

**Never ask:**
- "Ready for direction?"
- "What's next?" (it's in handoff)
- "Should I start Phase-XX?"
- "How should I implement?" (use specs)
- "Is this correct?" "Does this look right?" "Should I proceed?"
- Any question about implementation/correctness/readiness (decide autonomously)

---

### DESIGN MODE (rare - explicit only)

**You:** Technical Advisor | **User:** Architect (final say)
**When:** User says "let's design Phase-XX"
**Pattern:** Collaborate, discuss, debate

---

### Mode Determination

| User Input | Mode | Action |
|------------|------|--------|
| "Next: Phase-XX" | EXECUTION | START NOW |
| "Start Phase-XX" | EXECUTION | START NOW |
| [Pastes handoff] | EXECUTION | START NOW |
| "Let's design Phase-XX" | DESIGN | Collaborate |

---

## Core Execution Rules (NON-NEGOTIABLE)

### 1. Autonomous Execution

**Execution triggers (START immediately):**
- "Next: Phase-XX"
- "Start Phase-XX"
- User pastes handoff
- "Do Phase-XX"

**Protocol:**
1. Check STATUS.md (if phase complete, confirm with user; if phase file missing, list available phases)
2. GATE -1 (verify - see below)
3. Declare workflow
4. Execute gates 0-6
5. Deliver handoff

**Examples:** `examples/autonomous-execution-examples.md` (reference if behavior unclear)

---

### 2. Spec Compliance (100%)

**If spec defines it, implement EXACTLY. Zero deviation.**

- Spec says "Result<T, E>" → implement Result<T, E> (not "add later")
- Spec defines error codes → implement error codes (not generic errors)
- No interpretation, no shortcuts, no "good enough"
- Partial implementation = incomplete = FAIL

**Spec is law. Follow it completely.**

---

### 3. Phase Acceptance Criteria (SACRED)

**ALL acceptance criteria MUST be met. Not suggestions. REQUIREMENTS.**

- Phase says "50+ tests" → deliver 50+ (not 45 "close enough")
- Phase says "100% parity" → verify 100% (not "seems to work")
- Phase says "zero warnings" → zero warnings (not "just 2 small ones")

**Partial completion = phase incomplete.**

---

### 4. Intelligent Decisions (When Spec Silent)

**Spec doesn't define everything. When silent:**

1. **Analyze codebase patterns** (how do similar features work?)
2. **Decide intelligently** (based on patterns + project direction)
3. **LOG decision** (MANDATORY: `docs/decision-logs/` using format from `gates/gate-minus1-sanity.md` Decision Logging Format section)

**NEVER:**
- Ask user (you decide - see Operating Modes)
- Leave TODO (decide NOW)
- Guess without analysis (research THEN decide)
- Skip logging (undocumented = lost knowledge)

**Example:** FFI errors not in spec → analyze runtime error patterns (AT#### codes) → decide FFI uses AT9xxx → log decision

---

### 5. World-Class Quality (NO SHORTCUTS)

**Atlas aims to rival Rust, Go, C#. Complete implementations only.**

**BANNED:**
- `// TODO: implement later`
- `unimplemented!()` (except out-of-scope features)
- "MVP version for now"
- Partial implementations
- Stubbing complex parts

**REQUIRED:**
- Complete implementations (even if complex)
- Production-grade code (not prototype)
- All edge cases handled (not just happy path)
- Comprehensive tests (not "basic coverage")

**Exception:** Feature explicitly out of version scope (e.g., "FFI in v0.3, not v0.2")

**Quality > speed. When in doubt: do MORE, not less.**

**Details:** `guides/compiler-standards.md`, `guides/ai-first-principles.md`

---

### 6. Interpreter/VM Parity (100% REQUIRED)

Both execution engines MUST produce identical output.

**ABSOLUTE REQUIREMENTS:**
- Test count parity: interpreter tests = VM tests (exactly)
- Behavior parity: identical outputs, errors, edge cases
- Coverage parity: every scenario tested in both

**Parity break = BLOCKING. Phase incomplete without 100% parity.**

**Details:** `gates/gate-3-parity.md`

---

### 7. Testing Protocol (SURGICAL ONLY)

**ONE test at a time:**
```bash
cargo test -p atlas-runtime test_exact_name -- --exact
```

**BANNED:**
- `cargo test` (full suite)
- `cargo test -p atlas-runtime` (package suite)
- Any test without `-- --exact` flag
- Re-running passing tests "for verification"

**GATE 4 only:** User tells you to run full suite

**Details:** `guides/testing-protocol.md`

---

## GATE -1: Communication Check (ALWAYS FIRST)

**Run before ANY work:**

1. **No assumptions** - verify using docs/specs (not user)
2. **Brutally honest** - short & informative
3. **Autonomous execution** - phase directive = START NOW (no permission)

**Spec-First Verification:**
- Read phase blockers/dependencies
- Read relevant spec sections BEFORE coding
- Verify dependencies via docs AUTONOMOUSLY
- Make technical decisions using spec
- Log decisions in `docs/decision-logs/`
- NEVER ask user (EXECUTION MODE)

**Sanity Check:**
- `cargo clean && cargo check -p atlas-runtime` (no tests - prevents 100GB+ bloat)
- Catch issues BEFORE hours of work
- **On failure:** Stop and inform user with error details (don't proceed)

**Details:** `gates/gate-minus1-sanity.md`

---

## Workflow Classification

**After GATE -1, declare workflow:**

| Workflow | When | File |
|----------|------|------|
| Structured Development | Following documented plan | `workflows/structured.md` |
| Bug Fix | Fixing incorrect behavior | `workflows/bug-fix.md` |
| Refactoring | Code cleanup (no behavior change) | `workflows/refactoring.md` |
| Debugging | Investigation, root cause | `workflows/debugging.md` |
| Enhancement | Adding capabilities | `workflows/enhancement.md` |
| Dead Code Cleanup | Removing unused (user-invoked) | `workflows/dead-code-cleanup.md` |

**IMMEDIATELY read corresponding workflow file before proceeding with gates.**

---

## Documentation Routing (LAZY LOAD)

**Always check `STATUS.md` first** (current state, progress, routing)

**Lazy-load specs:** `Atlas-SPEC.md` is INDEX with routing table
- DO NOT read all spec files
- USE routing table to find exactly what you need
- READ only relevant sections

**Structure:**
- `docs/README.md` - Navigation
- `Atlas-SPEC.md` - Spec index (routing)
- `docs/specification/` - Types, syntax, semantics, runtime, modules, REPL, bytecode, diagnostics
- `docs/implementation/` - Component details
- `docs/guides/` - Testing, code quality
- `docs/api/` - Stdlib, runtime

**Docs evolve. Skill stays stable. Always reference docs.**

---

## Universal Rules

### Banned

- Task/Explore agents (use Glob + Read + Grep directly)
- Breaking interpreter/VM parity
- Violating grammar specs
- Stub implementations (see section 5 - World-Class Quality)
- Code dumps in docs (guidance only)
- Simplifying for line counts (quality > metrics)
- Assumptions without verification
- Verbose or sugarcoated responses
- Testing protocol violations (see section 7 + guides/testing-protocol.md)

### Required

- Rust best practices (explicit types, Result<T, E>, no unwrap in production)
- Interpreter/VM parity (identical output, always)
- Grammar conformance (`docs/specification/`)
- Comprehensive testing (rstest, insta, proptest)
- Quality gates (test, clippy, fmt - all pass)
- Doc-driven (reference `docs/` as source of truth)

**Line limits:** `guides/line-limits.md`

---

## Build & Quality

**During development:**
```bash
cargo clean && cargo check -p atlas-runtime       # Clean + verify (prevents 100GB+ bloat)
cargo clippy -p atlas-runtime -- -D warnings      # Zero warnings
cargo fmt -p atlas-runtime                        # Format
cargo test -p atlas-runtime test_exact_name -- --exact  # ONE test
```

**End of phase (GATE 4):**
```bash
cargo test -p atlas-runtime   # Full suite (user tells you when)
```

**Details:** `guides/build-quality.md`

---

## Phase Completion Handoff

**Use standardized format for seamless agent transitions.**

**Template:** `examples/phase-completion-template.md`

**Required:**
- Visual table (test counts, coverage)
- Key Features section
- Technical Implementation (decisions, patterns)
- ALL files created/modified
- Progress tracking (current, next, overall)
- Clear next step

---

## References (External Files - Lazy Load)

**Examples:**
- `examples/phase-completion-template.md` - Handoff format
- `examples/autonomous-execution-examples.md` - BAD vs GOOD patterns

**Guides:**
- `guides/testing-protocol.md` - Surgical testing rules
- `guides/compiler-standards.md` - Industry standards
- `guides/ai-first-principles.md` - AI optimization
- `guides/build-quality.md` - Build commands
- `guides/line-limits.md` - Line limit philosophy

**Gates:**
- `gates/README.md` - Gate index
- `gates/gate-minus1-sanity.md` - GATE -1 details
- `gates/gate-0.5-dependencies.md` - Dependency verification
- `gates/gate-1.5-foundation.md` - Foundation check
- `gates/gate-3-parity.md` - Parity verification
- [All other gates - see README]

**Workflows:**
- `workflows/structured.md` - Structured development
- `workflows/bug-fix.md` - Bug fixing
- `workflows/refactoring.md` - Refactoring
- `workflows/debugging.md` - Debugging
- `workflows/enhancement.md` - Enhancements
- `workflows/dead-code-cleanup.md` - Dead code removal

---

## Summary

**Compiler-first:** Atlas is a compiler, not an app. Embrace necessary complexity.
**Doc-driven:** Docs contain all details. Skill routes to docs.
**Quality-first:** Correctness and quality over arbitrary metrics.
**Parity is sacred:** Both engines must always match.
**Autonomous:** Phase directive = execute immediately (no permission).
**World-class:** Complete implementations, 100% spec compliance, no shortcuts.
