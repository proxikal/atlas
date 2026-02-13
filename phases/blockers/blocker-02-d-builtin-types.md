# BLOCKER 02-D: Built-in Generic Types & Integration

**Part:** 4 of 4 (Built-in Types)
**Category:** Foundation - Type System Extension
**Estimated Effort:** 1 week
**Complexity:** Medium

---

## 🚨 BLOCKERS - CHECK BEFORE STARTING

**REQUIRED:** BLOCKER 02-C must be complete.

**Verification:**
```bash
# 02-C complete
cargo test generics_runtime_tests --no-fail-fast
cargo test vm_generics_runtime_tests --no-fail-fast
grep -n "Monomorphizer" crates/atlas-runtime/src/typechecker/generics.rs
```

**What's needed:**
- ✅ Monomorphization works
- ✅ Interpreter executes generics
- ✅ VM executes generics
- ✅ 100% parity

**If missing:** Complete BLOCKER 02-C first.

---

## Objective

**THIS PHASE:** Add built-in generic types (Option<T>, Result<T,E>) and migrate Array to generic syntax. Makes generics immediately useful.

**Success criteria:** Can use `Option<number>`, `Result<T, E>` in Atlas code. Array uses `number[]` or `Array<number>` syntax.

---

## Implementation

### Step 1: Define Built-in Generic Types (Days 1-2)

**Register Option<T> and Result<T,E>:**
```rust
// In type system initialization
pub fn register_builtin_generics() {
    register_generic_type("Option", vec!["T"]);
    register_generic_type("Result", vec!["T", "E"]);
    register_generic_type("Array", vec!["T"]);  // For future migration
}
```

### Step 2: Option<T> Implementation (Days 3-4)

**Option constructors:**
```atlas
// Some constructor
fn Some<T>(value: T) -> Option<T>;

// None value
let none: Option<T> = None;
```

**Add to stdlib:**
```rust
// Builtin functions
"Some" -> construct Option with value
"is_some" -> check if Option has value
"is_none" -> check if Option is None
"unwrap" -> get value or panic
"unwrap_or" -> get value or default
```

### Step 3: Result<T,E> Implementation (Days 5-6)

**Result constructors:**
```atlas
fn Ok<T>(value: T) -> Result<T, E>;
fn Err<E>(error: E) -> Result<T, E>;
```

**Add to stdlib:**
```rust
"Ok" -> construct success Result
"Err" -> construct error Result
"is_ok" -> check if Result is Ok
"is_err" -> check if Result is Err
"unwrap" -> get value or panic
"unwrap_or" -> get value or default
```

### Step 4: Testing (Day 7)

Test Option and Result usage:
```atlas
// Option tests
let some_val: Option<number> = Some(42);
let none_val: Option<string> = None;

// Result tests
let ok_val: Result<number, string> = Ok(42);
let err_val: Result<number, string> = Err("failed");
```

---

## Acceptance Criteria

- ✅ Option<T> works
- ✅ Result<T,E> works
- ✅ Constructors (Some, None, Ok, Err) work
- ✅ Helper functions work
- ✅ 30+ tests pass (both engines)
- ✅ Parity maintained

**Next:** BLOCKER 03 (Pattern Matching) to use these types ergonomically

---

**This completes BLOCKER 02! Generics are now fully functional.**
