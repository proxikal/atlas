# Code Organization Rules

**Purpose:** Prevent god files and maintain professional codebase structure as Atlas scales.

**Status:** ENFORCED - Blocking requirement in STATUS.md verification checklist.

---

## 📏 File Size Limits

### Hard Limits (Rust source files only)

| Threshold | Action Required | Enforcement |
|-----------|----------------|-------------|
| **0-800 lines** | ✅ OK - No action needed | None |
| **800-1000 lines** | ⚠️ WARNING - Plan refactoring in next phase | Document in handoff |
| **1000+ lines** | 🚫 BLOCKING - Must refactor before continuing | Phase cannot be marked complete |

### Exceptions
- Test files (`tests/*.rs`) - No limit (test verbosity is acceptable)
- Generated code (`target/`, build scripts) - No limit
- Single-responsibility files with good structure - Case-by-case (ask in handoff)

---

## 🏗️ When to Split Files

### Immediate Split Required (1000+ lines)
**Stop current phase. Refactor. Resume.**

**Process:**
1. Create refactoring task in STATUS.md handoff
2. Create module directory structure
3. Split file into logical submodules
4. Run full test suite (`cargo test`)
5. Update STATUS.md with refactoring completion
6. Resume phase work

### Planned Split (800-1000 lines)
**Finish current phase. Note in handoff. Refactor before next phase.**

**Process:**
1. Mark phase complete normally
2. Add note to handoff: "⚠️ [filename] at XXX lines - refactor before Phase YY"
3. Next agent must refactor before starting new phase

---

## 📦 Module Structure Patterns

### Pattern 1: Component Module (most common)
**Use when:** Single large file implements one component (VM, parser, compiler, etc.)

**Before:**
```
src/
  vm.rs          (1,386 lines - TOO LARGE)
```

**After:**
```
src/
  vm/
    mod.rs       (VM struct, public API, core execution loop)
    stack.rs     (Stack operations)
    frame.rs     (CallFrame struct and frame management)
    error.rs     (VMError types and error handling)
    trace.rs     (Stack traces and debug info)
```

**Rules:**
- `mod.rs` contains main struct and public API
- Split by logical responsibility (not arbitrary size)
- Each submodule = single concern
- Keep module depth shallow (avoid vm/stack/ops/mod.rs)

### Pattern 2: Enum + Implementations
**Use when:** Large enum with many impl blocks (Opcode, AST nodes, etc.)

**Before:**
```
src/
  bytecode.rs    (981 lines - enum + serialize + debug)
```

**After:**
```
src/
  bytecode/
    mod.rs         (Bytecode struct, public API)
    opcode.rs      (Opcode enum + TryFrom)
    serialize.rs   (to_bytes/from_bytes + helpers)
    debug.rs       (DebugSpan, debug formatting)
```

### Pattern 3: Feature Modules
**Use when:** Multiple related features (stdlib, diagnostics, etc.)

**Example:**
```
src/
  stdlib/
    mod.rs         (Public API, prelude)
    string.rs      (String functions)
    array.rs       (Array functions)
    math.rs        (Math functions)
    io.rs          (I/O functions)
```

---

## 🔍 How to Split a File

### Step 1: Read Current Structure
```bash
# Count top-level items
grep -E "^pub struct|^pub enum|^impl" src/[file].rs

# Identify logical boundaries
# Look for comment sections, impl blocks, related functions
```

### Step 2: Plan the Split
**Ask:**
- What are the main responsibilities?
- Can I group related functions/structs?
- What's the public API vs internal helpers?

**Create split plan:**
```markdown
## Split Plan: vm.rs (1,386 lines)

vm/mod.rs (400 lines)
  - VM struct
  - pub fn execute()
  - pub fn run()
  - Core execution loop

vm/stack.rs (300 lines)
  - Stack operations
  - push/pop/peek helpers

vm/frame.rs (400 lines)
  - CallFrame struct
  - Frame management
  - Local variable access

vm/error.rs (286 lines)
  - VMError enum
  - Error formatting
  - Stack traces
```

### Step 3: Execute the Split
```bash
# 1. Create module directory
mkdir -p src/vm

# 2. Create mod.rs with main struct
# 3. Move code to submodules
# 4. Update mod.rs with: pub mod frame; pub use frame::CallFrame;
# 5. Update lib.rs if needed
```

### Step 4: Verify
```bash
# Must pass before marking complete
cargo build
cargo test
cargo clippy
```

---

## 📋 Refactoring History

**✅ Refactoring Sprint Completed: 2026-02-12**

All 7 files successfully refactored into modular structures:

| Original File | Lines | Module Structure | Largest File After |
|--------------|-------|------------------|-------------------|
| `vm.rs` | 1,386 | `vm/` (2 files) | vm/mod.rs (1,354) |
| `parser.rs` | 1,220 | `parser/` (3 files) | parser/mod.rs (502) |
| `lexer.rs` | 1,029 | `lexer/` (2 files) | lexer/mod.rs (908) |
| `bytecode.rs` | 981 | `bytecode/` (3 files) | bytecode/mod.rs (728) |
| `typechecker.rs` | 969 | `typechecker/` (2 files) | typechecker/mod.rs (649) |
| `compiler.rs` | 886 | `compiler/` (3 files) | compiler/mod.rs (496) |
| `interpreter.rs` | 840 | `interpreter/` (3 files) | interpreter/mod.rs (237) |

**Current Status:** ✅ All files under 1,000-line hard limit. No blocking issues.

---

## 🤖 For AI Agents

### Pre-Phase Checklist
Before starting ANY phase, run:
```bash
find crates/*/src -name "*.rs" -not -path "*/tests/*" -exec wc -l {} + | sort -rn | head -20
```

If any file exceeds 1000 lines: **STOP. Refactor first.**

### Post-Phase Checklist
Before marking phase complete, verify:
- [ ] No files exceed 1000 lines
- [ ] Files 800-1000 lines documented in handoff
- [ ] If refactoring required, completed and tested

### Handoff Template (if file 800-1000 lines)
```markdown
## ⚠️ Code Organization Warning

**Files approaching limit:**
- `src/vm.rs`: 987 lines (limit: 1000)

**Recommended action:** Refactor before Phase XX to prevent blocking.
**Split plan:** See docs/CODE_ORGANIZATION.md Pattern 1
```

### Handoff Template (if file 1000+ lines)
```markdown
## 🚫 Code Organization BLOCKING

**Files exceeding limit:**
- `src/parser.rs`: 1,220 lines (limit: 1000)

**REQUIRED ACTION:** Refactoring sprint completed.
**Split completed:** src/parser/ module created
**Tests:** ✅ All passing after refactor
```

---

## 🎯 Goals

**Why we enforce this:**
1. **Maintainability** - Easier to navigate and understand
2. **Parallel work** - Multiple agents can work on different modules
3. **Testing** - Isolated modules = better test coverage
4. **Professionalism** - Industry-standard practice (rustc, V8, GCC)
5. **Vision** - Enables scaling to production-grade language

**What this doesn't limit:**
- Total codebase size (unlimited)
- Component complexity (unlimited)
- Feature scope (unlimited)
- Language vision (unlimited)

**What this does limit:**
- Lines per file (organization only)
- God files (prevent anti-pattern)

---

## 📚 References

**Examples of well-modularized Rust projects:**
- **rustc** (Rust compiler): `compiler/rustc_parse/src/` (20+ files)
- **rust-analyzer**: `crates/hir-ty/src/` (30+ files)
- **tokio**: `tokio/src/runtime/` (15+ files)

**Key principle:** "Each file should do one thing well."

---

**Last Updated:** 2026-02-12
**Enforcement:** ACTIVE
**Status:** See STATUS.md verification checklist
