# Foundation Blockers

**Purpose:** These phases address critical foundation gaps that block multiple v0.2 phases.

**Status:** 6 blockers broken into 19 sub-phases - covers 100% of foundation requirements.

**For AI Agents:** Complete these BEFORE v0.2 phases. See STATUS.md for progress tracker.

---

## Blocker Overview

| # | Blocker | Sub-Phases | Effort | Complexity | Blocks |
|---|---------|------------|--------|------------|--------|
| 01 | JSON Value Type | 1 | 1-2 weeks | Medium | 10+ phases (JSON API, HTTP, config) |
| 02 | Generic Type Parameters | **4** (A, B, C, D) | 4-6 weeks | Very High | 15+ phases (Result, HashMap, advanced types) |
| 03 | Pattern Matching | **2** (A, B) | 2-3 weeks | High | Error handling, Option/Result usage |
| 04 | Module System | **4** (A, B, C, D) | 3-4 weeks | Very High | 15+ phases (imports, packages, code org) |
| 05 | Configuration System | 1 | 1-2 weeks | Medium | 5+ phases (config, manifest, CLI) |
| 06 | Security Model | **3** (A, B, C) | 2-3 weeks | High | All I/O phases (file, network, process) |

**Total:** 19 sub-phases, 14-20 weeks estimated effort (3.5-5 months)

**Each sub-phase:** 1 week max, focused scope, must pass all tests before proceeding

---

## Sub-Phase Breakdown

### BLOCKER 01: JSON Value Type (1 phase)
- `blocker-01-json-value-type.md` - Complete implementation

### BLOCKER 02: Generic Type Parameters (4 phases)
1. `blocker-02-a-type-system-foundation.md` - Syntax, AST, parser (Week 1)
2. `blocker-02-b-type-checker-inference.md` - Type checking & inference (Weeks 2-3)
3. `blocker-02-c-runtime-implementation.md` - Monomorphization & execution (Weeks 4-5)
4. `blocker-02-d-builtin-types.md` - Option<T>, Result<T,E> (Week 6)

### BLOCKER 03: Pattern Matching (2 phases)
1. `blocker-03-a-pattern-syntax-typechecking.md` - Syntax, AST, type checking (Week 1)
2. `blocker-03-b-runtime-execution.md` - Interpreter & VM execution (Weeks 2-3)

### BLOCKER 04: Module System (4 phases)
1. `blocker-04-a-syntax-resolution.md` - Import/export syntax & resolution (Week 1)
2. `blocker-04-b-loading-caching.md` - Module loading & caching (Week 2)
3. `blocker-04-c-type-system-integration.md` - Cross-module type checking (Week 3)
4. `blocker-04-d-runtime-implementation.md` - Interpreter & VM execution (Week 4)

### BLOCKER 05: Configuration System (1 phase)
- `blocker-05-configuration-system.md` - Complete atlas-config crate

### BLOCKER 06: Security Model (3 phases)
1. `blocker-06-a-permission-system.md` - Permission types & policies (Week 1)
2. `blocker-06-b-runtime-enforcement.md` - Runtime integration (Week 2)
3. `blocker-06-c-audit-configuration.md` - Audit logging & prompts (Week 3)

**Each sub-phase MUST be completed in order. Tests must pass before moving to next.**

---

## Dependency Graph

```
BLOCKER 01: JSON Value Type
├─ Blocks: Stdlib Phase 4 (JSON API)
├─ Blocks: Stdlib Phase 10 (Network HTTP)
└─ Blocks: Foundation Phase 9 (may benefit from JSON)

BLOCKER 02: Generic Type Parameters
├─ Requires: Stable type system from v0.1 ✅
├─ Blocks: BLOCKER 03 (Pattern Matching needs generics)
├─ Blocks: Foundation Phase 9 (Result<T,E>)
├─ Blocks: Stdlib Phase 7 (HashMap<K,V>, HashSet<T>)
├─ Blocks: Typing Phase 4 (Union Types)
├─ Blocks: Typing Phase 5 (Generic Constraints)
└─ Blocks: Typing Phase 6-7 (Advanced types)

BLOCKER 03: Pattern Matching
├─ Requires: BLOCKER 02 (Generic Type Parameters)
├─ Blocks: Foundation Phase 9 (Result error handling)
├─ Blocks: Option<T> usage patterns
└─ Blocks: Any phase using Result/Option types

BLOCKER 04: Module System
├─ Requires: v0.1 complete and stable ✅
├─ Blocks: Foundation Phase 6 (Module System Core)
├─ Blocks: Foundation Phase 7 (Package Manifest)
├─ Blocks: Foundation Phase 8 (Package Manager)
└─ Blocks: All multi-file programs

BLOCKER 05: Configuration System
├─ Requires: Stable crate structure ✅
├─ Blocks: Foundation Phase 4 (Configuration)
├─ Blocks: Foundation Phase 7 (Package Manifest)
├─ Blocks: BLOCKER 06 (Security needs config)
└─ Blocks: CLI phases needing config

BLOCKER 06: Security Model Implementation
├─ Requires: BLOCKER 05 (Configuration System)
├─ Requires: docs/reference/io-security-model.md ✅
├─ Blocks: Stdlib Phase 5 (File I/O API)
├─ Blocks: Stdlib Phase 10 (Network HTTP)
└─ Blocks: All I/O operations
```

---

## Critical Path Analysis

**Shortest path to usable stdlib:**
1. BLOCKER 01: JSON Value Type (1-2 weeks)
2. BLOCKER 05: Configuration System (1-2 weeks)
3. BLOCKER 06: Security Model (2-3 weeks)
4. Then: Stdlib Phases 4, 5, 10 can proceed

**Total: 4-7 weeks for JSON + File I/O + HTTP**

---

**Shortest path to advanced type system:**
1. BLOCKER 02: Generic Type Parameters (4-6 weeks)
2. BLOCKER 03: Pattern Matching (2-3 weeks)
3. Then: Result<T,E>, HashMap<K,V>, advanced typing features

**Total: 6-9 weeks for generics + pattern matching**

---

**Shortest path to package management:**
1. BLOCKER 04: Module System (3-4 weeks)
2. BLOCKER 05: Configuration System (1-2 weeks) - can run parallel
3. Then: Foundation Phases 7, 8 (Package Manifest + Manager)

**Total: 3-4 weeks (if parallelized) or 4-6 weeks (if sequential)**

---

## Implementation Order Recommendation

### Parallel Track A: Type System (Long Poles)
1. **BLOCKER 02: Generic Type Parameters** (4-6 weeks)
   - Longest blocker, most complex
   - Start ASAP
2. **BLOCKER 03: Pattern Matching** (2-3 weeks)
   - Depends on BLOCKER 02
   - Start immediately after

**Track A Total: 6-9 weeks**

---

### Parallel Track B: Infrastructure (Quick Wins)
1. **BLOCKER 01: JSON Value Type** (1-2 weeks)
   - Independent, can start immediately
2. **BLOCKER 05: Configuration System** (1-2 weeks)
   - Independent, can start immediately
3. **BLOCKER 06: Security Model** (2-3 weeks)
   - Depends on BLOCKER 05
   - Start after config complete

**Track B Total: 4-7 weeks**

---

### Parallel Track C: Module System (Major Feature)
1. **BLOCKER 04: Module System** (3-4 weeks)
   - Independent, can start anytime
   - Consider starting after Track B to benefit from config system

**Track C Total: 3-4 weeks**

---

**With full parallelization:** 6-9 weeks (limited by Track A)
**With 2-track parallelization:** 9-13 weeks
**Fully sequential:** 14-20 weeks

---

## Blocker Details

### BLOCKER 01: JSON Value Type
**File:** `blocker-01-json-value-type.md`

**What:** Add `Value::JsonValue` variant for dynamic JSON handling
**Why:** JSON critical for APIs, config, data interchange
**Impact:** Unblocks JSON API, HTTP, config parsing

**Key deliverables:**
- JsonValue enum (6 variants)
- Safe indexing (null for missing keys)
- Type extraction methods
- Strict type isolation

---

### BLOCKER 02: Generic Type Parameters
**File:** `blocker-02-generic-type-parameters.md`

**What:** Extend type system for `Type<T>` parameterized types
**Why:** Foundation for Result<T,E>, HashMap<K,V>, Option<T>
**Impact:** Unblocks 15+ phases needing generic types

**Key deliverables:**
- Type::Generic and Type::TypeParameter
- Monomorphization (code generation per type)
- Type inference for generic functions
- Built-in Option<T> and Result<T,E>

---

### BLOCKER 03: Pattern Matching
**File:** `blocker-03-pattern-matching.md`

**What:** Add `match` expressions with exhaustiveness checking
**Why:** Ergonomic Result<T,E> and Option<T> handling
**Impact:** Unblocks error handling patterns, safer code

**Key deliverables:**
- Match expression syntax
- Pattern types (literal, wildcard, variable, constructor)
- Exhaustiveness checking
- Result/Option integration

---

### BLOCKER 04: Module System
**File:** `blocker-04-module-system.md`

**What:** Add import/export for multi-file programs
**Why:** Code organization, package management foundation
**Impact:** Unblocks package manager, multi-file programs

**Key deliverables:**
- Import/export syntax
- Module resolution algorithm
- Module loader with caching
- Cross-module type checking

---

### BLOCKER 05: Configuration System
**File:** `blocker-05-configuration-system.md`

**What:** Create `atlas-config` crate for settings management
**Why:** Project config, package manifest, CLI behavior
**Impact:** Unblocks package manifest, CLI features, security config

**Key deliverables:**
- ProjectConfig (atlas.toml)
- GlobalConfig (~/.atlas/config.toml)
- Package Manifest parsing
- Config validation and merging

---

### BLOCKER 06: Security Model Implementation
**File:** `blocker-06-security-model-implementation.md`

**What:** Implement security model from docs/reference/io-security-model.md
**Why:** Safe execution of untrusted Atlas code
**Impact:** Unblocks all I/O operations (file, network, process)

**Key deliverables:**
- Permission system (filesystem, network, process, env)
- SecurityContext with runtime checks
- Sandbox modes (standard, strict, maximum)
- Audit logging

---

## Verification

**Before starting any blocker:**
```bash
# Check prerequisites
cargo build --release
cargo test --all --no-fail-fast
cargo clippy -- -D warnings
```

**After completing blocker:**
```bash
# All quality gates must pass
cargo test --all
cargo clippy -- -D warnings
cargo fmt -- --check
```

---

## Notes

**Not in scope (deferred to v0.3+):**
- Closures with captured variables
- Anonymous functions/lambdas
- User-defined generic types (structs/enums with type params)
- Trait bounds for generics
- Higher-kinded types
- OS-level sandboxing (containers, chroot)

**These blockers provide 100% foundation for v0.2 phases.**

**Philosophy:** Do it right, not fast. Atlas built for decades, not sprints.

---

**Last updated:** 2026-02-13
**Status:** All blocker phases documented and ready for implementation
