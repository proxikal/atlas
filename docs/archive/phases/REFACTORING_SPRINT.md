# Refactoring Sprint - Phase 10.5

**Status:** REQUIRED before Phase 11
**Reason:** 7 files exceed or approach 1000-line limit
**Enforcement:** See `docs/CODE_ORGANIZATION.md` and `STATUS.md` verification checklist
**Date Created:** 2026-02-12

---

## 🎯 Objective

Refactor all files approaching or exceeding 1000 lines into logical submodules.

**Files to refactor:**
1. `vm.rs` (1,386 lines) - 🚫 BLOCKING
2. `parser.rs` (1,220 lines) - 🚫 BLOCKING
3. `lexer.rs` (1,029 lines) - 🚫 BLOCKING
4. `bytecode.rs` (981 lines) - ⚠️ WARNING
5. `typechecker.rs` (969 lines) - ⚠️ WARNING
6. `compiler.rs` (886 lines) - ⚠️ WARNING
7. `interpreter.rs` (840 lines) - ⚠️ WARNING

---

## 📋 Refactoring Tasks

### Task 1: vm.rs (1,386 lines) → vm/ module

**Priority:** CRITICAL (largest file)

**Proposed structure:**
```
src/vm/
  mod.rs          - VM struct, execute(), run(), core loop (~400 lines)
  stack.rs        - Stack operations, push/pop helpers (~300 lines)
  frame.rs        - CallFrame struct, frame management (~400 lines)
  error.rs        - VMError, error formatting, traces (~286 lines)
```

**Steps:**
1. Create `src/vm/` directory
2. Create `mod.rs` with VM struct and public API
3. Move CallFrame to `frame.rs`
4. Move VMError to `error.rs`
5. Move stack operations to `stack.rs`
6. Update `src/lib.rs`: `pub mod vm;`
7. Add re-exports in `vm/mod.rs`: `pub use frame::CallFrame; pub use error::VMError;`
8. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file in `vm/` exceeds 500 lines
- [ ] Public API unchanged (no breaking changes)

---

### Task 2: parser.rs (1,220 lines) → parser/ module

**Priority:** CRITICAL

**Proposed structure:**
```
src/parser/
  mod.rs          - Parser struct, public API, core logic (~400 lines)
  expr.rs         - Expression parsing methods (~400 lines)
  stmt.rs         - Statement parsing methods (~300 lines)
  error.rs        - Parser error recovery (~120 lines)
```

**Steps:**
1. Create `src/parser/` directory
2. Create `mod.rs` with Parser struct
3. Move expression parsing to `expr.rs`
4. Move statement parsing to `stmt.rs`
5. Move error recovery to `error.rs`
6. Update `src/lib.rs`: change `mod parser;` to use parser module
7. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file in `parser/` exceeds 500 lines
- [ ] Public API unchanged

---

### Task 3: lexer.rs (1,029 lines) → lexer/ module

**Priority:** CRITICAL

**Proposed structure:**
```
src/lexer/
  mod.rs          - Lexer struct, main tokenization loop (~400 lines)
  token.rs        - Move Token enum here from token.rs (~300 lines)
  scanner.rs      - Character scanning, peek/advance (~200 lines)
  literal.rs      - Number/string literal parsing (~129 lines)
```

**Alternative:** If token.rs is separate, keep it:
```
src/lexer/
  mod.rs          - Lexer struct, main tokenization (~500 lines)
  scanner.rs      - Character operations (~300 lines)
  literal.rs      - Literal parsing (~229 lines)
```

**Steps:**
1. Check if `token.rs` exists and its size
2. Create `src/lexer/` directory
3. Create `mod.rs` with Lexer struct
4. Split based on structure found
5. Update imports in `src/lib.rs`
6. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file in `lexer/` exceeds 500 lines
- [ ] Public API unchanged

---

### Task 4: bytecode.rs (981 lines) → bytecode/ module

**Priority:** HIGH (approaching limit, new features coming)

**Proposed structure:**
```
src/bytecode/
  mod.rs          - Bytecode struct, emit/patch methods (~300 lines)
  opcode.rs       - Opcode enum, TryFrom<u8> (~150 lines)
  serialize.rs    - to_bytes/from_bytes, helpers (~400 lines)
  debug.rs        - DebugSpan, debug formatting (~131 lines)
```

**Steps:**
1. Create `src/bytecode/` directory
2. Split Opcode enum to `opcode.rs`
3. Move serialization to `serialize.rs`
4. Move DebugSpan to `debug.rs`
5. Keep Bytecode struct in `mod.rs`
6. Update `src/lib.rs`
7. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file exceeds 500 lines
- [ ] Ready for Phase 11 (versioning) additions

---

### Task 5: typechecker.rs (969 lines) → typechecker/ module

**Priority:** HIGH (approaching limit)

**Proposed structure:**
```
src/typechecker/
  mod.rs          - TypeChecker struct, public API (~300 lines)
  expr.rs         - Expression type checking (~350 lines)
  stmt.rs         - Statement type checking (~200 lines)
  inference.rs    - Type inference logic (~119 lines)
```

**Steps:**
1. Create `src/typechecker/` directory
2. Analyze impl blocks and split by expression/statement
3. Move type inference to separate file if applicable
4. Update `src/lib.rs`
5. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file exceeds 500 lines
- [ ] Public API unchanged

---

### Task 6: compiler.rs (886 lines) → compiler/ module

**Priority:** MEDIUM (will grow with optimizer phases)

**Proposed structure:**
```
src/compiler/
  mod.rs          - Compiler struct, public API (~300 lines)
  expr.rs         - Expression compilation (~300 lines)
  stmt.rs         - Statement compilation (~200 lines)
  scope.rs        - Scope management (~86 lines)
```

**Steps:**
1. Create `src/compiler/` directory
2. Split expression/statement compilation
3. Extract scope management if present
4. Update `src/lib.rs`
5. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file exceeds 500 lines
- [ ] Ready for optimizer hooks (Phase 05)

---

### Task 7: interpreter.rs (840 lines) → interpreter/ module

**Priority:** MEDIUM (stable, not growing in upcoming phases)

**Proposed structure:**
```
src/interpreter/
  mod.rs          - Interpreter struct, public API (~300 lines)
  expr.rs         - Expression evaluation (~300 lines)
  stmt.rs         - Statement execution (~200 lines)
  env.rs          - Environment management (~40 lines)
```

**Steps:**
1. Create `src/interpreter/` directory
2. Split expression/statement logic
3. Extract environment if separate
4. Update `src/lib.rs`
5. Run: `cargo test`

**Exit criteria:**
- [ ] All tests pass
- [ ] No file exceeds 500 lines
- [ ] Public API unchanged

---

## 🔄 Execution Order

**Recommended order (by priority):**
1. vm.rs (largest, most critical)
2. parser.rs (large, foundational)
3. lexer.rs (large, foundational)
4. bytecode.rs (growing soon with Phase 11+)
5. typechecker.rs (stable but large)
6. compiler.rs (growing with optimizer phases)
7. interpreter.rs (stable, lowest priority)

**Parallel execution:** Tasks 1-3 can be done in parallel (different components). Tasks 4-7 can be done in parallel after 1-3.

---

## ✅ Global Exit Criteria

**Before resuming Phase 11, verify:**
- [ ] All 7 files refactored into modules
- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo test` passes (all existing tests)
- [ ] `cargo clippy` clean
- [ ] No .rs file in `src/` exceeds 1000 lines
- [ ] No .rs file in submodules exceeds 800 lines
- [ ] Public APIs unchanged (no breaking changes)
- [ ] STATUS.md updated with refactoring completion

**Test command:**
```bash
# Verify file sizes after refactoring
find crates/atlas-runtime/src -name "*.rs" -not -path "*/tests/*" -exec wc -l {} + | sort -rn | head -20

# Expected: No file over 800 lines
```

---

## 📝 STATUS.md Handoff Template

**After completing refactoring sprint, update STATUS.md:**

```markdown
**Last Completed:** REFACTORING SPRINT (Phase 10.5)
**Next Phase:** `phases/bytecode-vm/phase-11-bytecode-versioning.md`

**Refactoring Completed:**
- ✅ vm.rs → vm/ module (1,386 → largest file 400 lines)
- ✅ parser.rs → parser/ module (1,220 → largest file 400 lines)
- ✅ lexer.rs → lexer/ module (1,029 → largest file 500 lines)
- ✅ bytecode.rs → bytecode/ module (981 → largest file 400 lines)
- ✅ typechecker.rs → typechecker/ module (969 → largest file 350 lines)
- ✅ compiler.rs → compiler/ module (886 → largest file 300 lines)
- ✅ interpreter.rs → interpreter/ module (840 → largest file 300 lines)

**Tests:** All passing ✅
**File size check:** No file exceeds 500 lines ✅
```

---

## 🎯 Goals Achieved

**After completion:**
- ✅ No god files (all under 500 lines per file)
- ✅ Clear module boundaries
- ✅ Easier navigation and maintenance
- ✅ Ready for parallel development
- ✅ Scalable for upcoming phases (optimizer, debugger, profiler)
- ✅ Professional codebase structure

**Enforcement:** `docs/CODE_ORGANIZATION.md` prevents regression.

---

**Ready to execute? Start with Task 1 (vm.rs) - highest priority.**

**Last Updated:** 2026-02-12
