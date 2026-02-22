# Phase 14 — VM: Trait Method Dispatch + Parity Verification

**Block:** 3 (Trait System)
**Depends on:** Phase 13 complete
**Estimated tests added:** 20–25

---

## Objective

Verify VM correctly executes trait method calls compiled in Phase 12. Run all trait
tests against both engines and confirm identical output. This phase combines VM
execution verification with parity testing.

Note: The VM itself needs minimal changes — Phase 12 compiled impl methods to regular
named functions using existing `Call` opcode. This phase is primarily verification +
any fixes needed.

---

## Current State (verified after Phase 13)

- Compiler (Phase 12): compiles `impl` blocks as mangled `__impl__Type__Trait__Method` functions
- Interpreter (Phase 13): dispatches trait methods by looking up `ImplMethod` in `impl_methods` map
- VM: executes bytecode — if Phase 12 compiled correctly, VM just runs the `Call` opcode

The parity requirement: interpreter returns `42` → VM must also return `42`.

---

## VM Verification

Run the existing Phase 12 VM tests. If they pass, the VM needs no changes.
If they fail, diagnose why:

**Likely failure modes:**
1. Mangled function name not found at Call site → function lookup issue in VM
2. `self` argument count mismatch → arg_count byte wrong in Call opcode
3. Scope/local variable indexing off → local count wrong in compiled function

For each failure, investigate the compiled bytecode using the disassembler:
```bash
cargo run -- disasm --file test.atl
```

Fix any compilation bugs found in Phase 12's compiler code.

---

## Parity Tests

Add to `crates/atlas-runtime/tests/interpreter.rs` and `tests/vm.rs` — one test per scenario,
both engines:

```rust
// In interpreter.rs:
#[test]
fn test_parity_trait_basic_method_call_interpreter() {
    let atlas = Atlas::new();
    let result = atlas.eval("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        x.display();
    ");
    assert_eq!(result.unwrap().to_string(), "42");
}

// In vm.rs:
#[test]
fn test_parity_trait_basic_method_call_vm() {
    let bytecode = common::compile_source("
        trait Display { fn display(self: Display) -> string; }
        impl Display for number {
            fn display(self: number) -> string { return str(self); }
        }
        let x: number = 42;
        x.display();
    ").unwrap();
    let result = common::run_bytecode(bytecode);
    assert_eq!(result.unwrap().to_string(), "42");
}
```

Full parity test matrix — each scenario run in BOTH engines:

| Scenario | Interpreter Test | VM Test |
|---------|-----------------|---------|
| Basic method call, returns string | ✓ | ✓ |
| Method call with extra args | ✓ | ✓ |
| Method call accesses self value | ✓ | ✓ |
| Return value from method body | ✓ | ✓ |
| Two impls for different types, correct dispatch | ✓ | ✓ |
| Chained method calls | ✓ | ✓ |
| Marker trait (no methods) — no crash | ✓ | ✓ |
| Impl method calls another function | ✓ | ✓ |
| Impl method with conditional | ✓ | ✓ |
| Impl method with loop | ✓ | ✓ |

**Total: 20 parity tests (10 scenarios × 2 engines)**

---

## Parity Test Implementation

```rust
// All parity tests follow this pattern:
// 1. Same source code
// 2. Interpreter result via atlas.eval()
// 3. VM result via compile + run_bytecode()
// 4. Assert both equal the expected value

macro_rules! parity_test {
    ($name:ident, $src:expr, $expected:expr) => {
        paste::paste! {
            #[test]
            fn [<test_parity_ $name _interpreter>]() {
                let atlas = Atlas::new();
                let result = atlas.eval($src);
                assert_eq!(result.unwrap().to_string(), $expected);
            }

            #[test]
            fn [<test_parity_ $name _vm>]() {
                let bytecode = common::compile_source($src).unwrap();
                let result = common::run_bytecode(bytecode);
                assert_eq!(result.unwrap().to_string(), $expected);
            }
        }
    };
}
```

(Use `paste` crate if available, or write tests manually if not.)

---

## Full Parity Test List

```rust
parity_test!(
    trait_return_string,
    "trait Display { fn display(self: Display) -> string; }
     impl Display for number { fn display(self: number) -> string { return str(self); } }
     let x: number = 42; x.display();",
    "42"
);

parity_test!(
    trait_with_args,
    "trait Adder { fn add(self: Adder, n: number) -> number; }
     impl Adder for number { fn add(self: number, n: number) -> number { return self + n; } }
     let x: number = 10; x.add(32);",
    "42"
);

parity_test!(
    trait_self_access,
    "trait Doubler { fn double(self: Doubler) -> number; }
     impl Doubler for number { fn double(self: number) -> number { return self * 2; } }
     let x: number = 21; x.double();",
    "42"
);

parity_test!(
    trait_return_bool,
    "trait Checker { fn is_positive(self: Checker) -> bool; }
     impl Checker for number { fn is_positive(self: number) -> bool { return self > 0; } }
     let x: number = 5; x.is_positive();",
    "true"
);

parity_test!(
    trait_two_type_dispatch,
    "trait Tag { fn tag(self: Tag) -> string; }
     impl Tag for number { fn tag(self: number) -> string { return \"num\"; } }
     impl Tag for bool { fn tag(self: bool) -> string { return \"bool\"; } }
     let b: bool = true; b.tag();",
    "bool"
);

parity_test!(
    trait_method_with_conditional,
    "trait Abs { fn abs(self: Abs) -> number; }
     impl Abs for number {
         fn abs(self: number) -> number { if (self < 0) { return -self; } return self; }
     }
     let x: number = -5; x.abs();",
    "5"
);

parity_test!(
    trait_method_calls_stdlib,
    "trait Str { fn to_str(self: Str) -> string; }
     impl Str for number { fn to_str(self: number) -> string { return str(self); } }
     let x: number = 99; x.to_str();",
    "99"
);

parity_test!(
    trait_method_with_loop,
    "trait Multiply { fn times(self: Multiply, n: number) -> number; }
     impl Multiply for number {
         fn times(self: number, n: number) -> number {
             var result: number = 0;
             var i: number = 0;
             while (i < n) { result = result + self; i = i + 1; }
             return result;
         }
     }
     let x: number = 6; x.times(7);",
    "42"
);

parity_test!(
    trait_method_returns_null,
    "trait Cleaner { fn clean(self: Cleaner) -> void; }
     impl Cleaner for number { fn clean(self: number) -> void { } }
     let x: number = 1; x.clean();",
    "null"
);

parity_test!(
    marker_trait_no_crash,
    "trait Marker { }
     impl Marker for number { }
     let x: number = 1;
     x;",
    "1"
);
```

---

## Acceptance Criteria

- [ ] All 10 parity scenarios produce identical output in interpreter and VM
- [ ] Zero parity divergences
- [ ] All 20 parity tests pass (10 × 2 engines)
- [ ] VM test failures (if any) investigated and compiler bugs fixed
- [ ] All existing VM tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- If VM and interpreter produce different output for the same source, the VM is the
  likely culprit (interpreter was tested in Phase 13). Start debugging with the compiler
  disassembler to inspect what bytecode was generated.
- Parity divergence = BLOCKING. Do not hand off Phase 14 with any known divergence.
- The `paste` macro is used in some Atlas test files — check if it's available before
  using the macro approach. If not, write all 20 tests individually.
- Common cause of parity bugs: arg count off by one (self not counted in compiler's
  arg_count byte), or local variable index wrong in compiled impl method.
