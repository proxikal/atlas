# CRITICAL: Missing Design Documents

**Status:** 🚨 BLOCKERS ARE BLOCKED - Features not designed yet

**Issue:** Blockers 02, 03, 04 implement features that don't exist in Atlas-SPEC.md and have no design documents.

---

## What's Missing

### BLOCKER 02: Generics
**Problem:** Atlas-SPEC.md says generics are "under research"
**Missing:**
- Syntax design (Type<T> vs other approaches?)
- Semantics (monomorphization vs erasure?)
- Type parameter constraints
- Built-in generic types (Result, Option)
- Spec update

**Current spec:** Only `T[]` arrays, nothing else

### BLOCKER 03: Pattern Matching
**Problem:** `match` is "reserved for future use (not in v0.1 grammar)"
**Missing:**
- Syntax design (Rust-style vs other?)
- Pattern types (what patterns are supported?)
- Exhaustiveness checking algorithm
- Spec update

**Current spec:** Keyword reserved, no grammar defined

### BLOCKER 04: Module System
**Problem:** `import` is "reserved for future use", spec references non-existent `docs/modules.md`
**Missing:**
- Syntax design (ES modules vs CommonJS vs Rust?)
- Resolution algorithm (relative paths? absolute?)
- Module format (single file? directory?)
- Circular dependency handling
- Spec update

**Current spec:** Keyword reserved, no design exists

### BLOCKER 06: Security Model
**Status:** ✅ Actually documented in `docs/reference/io-security-model.md`

---

## Assumptions Being Made

### BLOCKER 02 Assumes:
- Generic syntax: `Type<T1, T2>` (angle brackets)
- Monomorphization (like Rust, not type erasure)
- Built-in Result<T,E> and Option<T>
- Hindley-Milner type inference
- **NO SPEC APPROVAL**

### BLOCKER 03 Assumes:
- Rust-style match syntax
- Exhaustiveness checking required
- Pattern types: literal, wildcard, variable, constructor
- Match is an expression (has a type)
- **NO SPEC APPROVAL**

### BLOCKER 04 Assumes:
- ES-module style: `import { x } from "./mod"`
- Relative paths: `./file`, `../sibling`
- Absolute paths: `/project/src/file`
- Module caching by path
- Circular detection required
- **NO SPEC APPROVAL**

---

## What Needs to Happen FIRST

### Before ANY blocker implementation:

1. **Design Phase for Each Feature**
   - Research options (Rust vs TypeScript vs Go vs ...)
   - Choose syntax and semantics
   - Document rationale in decision log
   - Update Atlas-SPEC.md with grammar
   - Get user/architect approval

2. **Create Design Documents**
   - `docs/design/generics.md` - Full generic type system design
   - `docs/design/pattern-matching.md` - Match expression design
   - `docs/design/modules.md` - Module system design (referenced but missing)
   - Each doc: syntax, semantics, examples, edge cases

3. **Update Atlas-SPEC.md**
   - Add generic syntax to grammar
   - Add match expressions to grammar
   - Add import/export to grammar
   - Define type rules
   - Define execution semantics

4. **THEN Implementation**
   - Only after designs approved
   - Only after spec updated
   - Follow spec exactly

---

## Risk of Proceeding Without Design

**If we implement blockers now:**
- Making up syntax/semantics as we go
- May conflict with Atlas principles
- May need to redo everything if design changes
- No spec to verify against
- Agents have no source of truth

**Example:**
- We choose `Type<T>` syntax
- Later decide `Type[T]` is better for Atlas
- Must rewrite all blocker work

---

## Recommendation

**DO NOT proceed with blockers until:**
1. User/architect designs features
2. Atlas-SPEC.md updated
3. Design docs created
4. Approach approved

**Estimated time for design work:** 2-4 weeks
- Research existing approaches
- Make design decisions
- Document in spec
- Get approval

**This is NOT wasted time** - designing correctly prevents 6-9 weeks of wrong implementation.

---

**The user was RIGHT to verify. We almost implemented 14-20 weeks of work based on assumptions.**
