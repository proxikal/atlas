# Phase 17 — Parity Hardening: Extended Edge Cases

**Block:** 3 (Trait System)
**Depends on:** Phase 16 complete
**Estimated tests added:** 15–20

---

## Objective

Run a comprehensive parity audit across all trait system features. This phase goes
beyond the basic parity tests in Phase 14 to cover:
- Edge cases in trait dispatch
- Multiple trait implementations on the same type
- Trait methods that call other trait methods
- Generic functions with trait bounds
- Nested trait calls
- Error paths (method not found, bounds not satisfied)

Any divergence found here is BLOCKING — fix before handoff.

---

## Extended Parity Scenarios

Each scenario is tested in both interpreter and VM:

### Scenario A: Multiple traits on same type
```atlas
trait Addable { fn add(self: Addable, n: number) -> number; }
trait Subtractable { fn sub(self: Subtractable, n: number) -> number; }
impl Addable for number { fn add(self: number, n: number) -> number { return self + n; } }
impl Subtractable for number { fn sub(self: number, n: number) -> number { return self - n; } }
let x: number = 10;
let a: number = x.add(5);
let b: number = a.sub(3);
b;
// Expected: 12
```

### Scenario B: Trait method returning bool, used in condition
```atlas
trait Comparable { fn greater_than(self: Comparable, other: number) -> bool; }
impl Comparable for number {
    fn greater_than(self: number, other: number) -> bool { return self > other; }
}
let x: number = 10;
if (x.greater_than(5)) { "yes"; } else { "no"; }
// Expected: "yes"
```

### Scenario C: Trait method calling stdlib function
```atlas
trait Formatted { fn fmt(self: Formatted) -> string; }
impl Formatted for number {
    fn fmt(self: number) -> string {
        return "Value: " + str(self);
    }
}
let x: number = 42;
x.fmt();
// Expected: "Value: 42"
```

### Scenario D: Chained trait method calls
```atlas
trait Inc { fn inc(self: Inc) -> number; }
impl Inc for number { fn inc(self: number) -> number { return self + 1; } }
let x: number = 40;
// Note: chaining requires result type to also support Inc
// Use a variable intermediate instead
let y: number = x.inc();
let z: number = y.inc();
z;
// Expected: 42
```

### Scenario E: Trait method with multiple parameters
```atlas
trait Interpolator {
    fn interpolate(self: Interpolator, t: number, other: number) -> number;
}
impl Interpolator for number {
    fn interpolate(self: number, t: number, other: number) -> number {
        return self + (other - self) * t;
    }
}
let a: number = 0;
a.interpolate(0.5, 100);
// Expected: 50
```

### Scenario F: Trait method with conditional return paths
```atlas
trait Clamp { fn clamp(self: Clamp, min: number, max: number) -> number; }
impl Clamp for number {
    fn clamp(self: number, min: number, max: number) -> number {
        if (self < min) { return min; }
        if (self > max) { return max; }
        return self;
    }
}
let x: number = 150;
x.clamp(0, 100);
// Expected: 100
```

### Scenario G: Impl method modifying local state (no side effects on caller)
```atlas
trait Counter { fn count_to(self: Counter, n: number) -> number; }
impl Counter for number {
    fn count_to(self: number, n: number) -> number {
        var total: number = 0;
        var i: number = self;
        while (i <= n) { total = total + i; i = i + 1; }
        return total;
    }
}
let x: number = 1;
x.count_to(10);
// Expected: 55 (sum 1..10)
```

### Scenario H: String type impl
```atlas
trait Shouter { fn shout(self: Shouter) -> string; }
impl Shouter for string {
    fn shout(self: string) -> string { return self + "!!!"; }
}
let s: string = "hello";
s.shout();
// Expected: "hello!!!"
```

### Scenario I: Bool type impl
```atlas
trait Toggle { fn toggle(self: Toggle) -> bool; }
impl Toggle for bool { fn toggle(self: bool) -> bool { return !self; } }
let b: bool = true;
b.toggle();
// Expected: false
```

### Scenario J: Trait method returning array
```atlas
trait Pair { fn pair(self: Pair) -> number[]; }
impl Pair for number { fn pair(self: number) -> number[] { return [self, self * 2]; } }
let x: number = 7;
let p: number[] = x.pair();
p[1];
// Expected: 14
```

---

## Tests

Add all 10 scenarios × 2 engines = 20 parity tests.

Naming convention:
- `test_parity_block03_scenario_a_interpreter`
- `test_parity_block03_scenario_a_vm`
- ... through `_j_`

Add to:
- `crates/atlas-runtime/tests/interpreter.rs` — all 10 interpreter variants
- `crates/atlas-runtime/tests/vm.rs` — all 10 VM variants

---

## Parity Check Protocol

After adding all 20 tests:

1. Run `cargo test` — all 20 must pass
2. If any VM test fails while interpreter passes → compiler bug (Phase 12 code)
3. If any interpreter test fails while VM passes → interpreter bug (Phase 13 code)
4. If both fail → typechecker issue upstream

For each failure: investigate bytecode disassembly, fix, re-run.

---

## Acceptance Criteria

- [ ] All 10 scenarios pass in interpreter
- [ ] All 10 scenarios pass in VM
- [ ] Zero parity divergences across all 20 tests
- [ ] Any bugs found are fixed in their source phase (12, 13, or upstream)
- [ ] Total new tests: 15–20
- [ ] All existing tests pass (no regressions)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt` clean

---

## Notes

- This phase is the last safety net before the spec update. If any scenario reveals a
  fundamental architecture problem (e.g., "impl method can't access outer scope"), fix
  it here before closing the block.
- Known edge case to watch: Scenario G (local state in impl method). The interpreter
  uses dynamic scoping — ensure the impl method's local `total` and `i` don't leak
  into the caller's scope. The scope push/pop in `eval_impl_method_body()` must be correct.
- Scenario J (returning array) verifies that CoW value semantics (Block 1) work correctly
  inside impl methods. The returned array should be a fresh CoW array, not aliased.
