# Phase 19: Interpreter/VM Parity Verification

**Block:** 1 (Memory Model)
**Depends on:** Phases 07, 10, 12, 13, 14, 18 complete (both engines fully migrated, tests fixed)

---

## Objective

Systematically verify that interpreter and VM produce identical output for every array
and collection operation. Parity is a non-negotiable requirement of the Atlas project.
This phase is a dedicated verification pass — not a fix phase.

---

## Parity Test Matrix

For each operation below, run the Atlas program in both engines and compare output:

### Array Operations
| Operation | Test Program | Expected |
|-----------|-------------|----------|
| Index read | `print([10,20,30][1])` | `20` |
| Length | `print([1,2,3].len())` | `3` |
| Push (CoW) | `let a=[1];let b=a;b.push(2);print(a.len())` | `1` |
| Pop | `let a=[1,2,3];a.pop();print(a.len())` | `2` |
| Slice | `print([1,2,3,4,5].slice(1,3))` | `[2, 3]` |
| Sort | `print([3,1,2].sort())` | `[1, 2, 3]` |
| Map | `print([1,2,3].map(fn(x) x*2))` | `[2, 4, 6]` |
| Filter | `print([1,2,3,4].filter(fn(x) x>2))` | `[3, 4]` |
| Concat | `print([1,2]+[3,4])` | `[1, 2, 3, 4]` |
| For-each | `for x in [1,2,3] { print(x) }` | `1\n2\n3` |

### Map Operations
| Operation | Test Program | Expected |
|-----------|-------------|----------|
| Get | `let m={"a":1};print(m["a"])` | `1` |
| Set (CoW) | `let m={"a":1};let n=m;n["b"]=2;print(m.len())` | `1` |
| Keys | `print({"a":1,"b":2}.keys().sort())` | `["a", "b"]` |
| Delete | `let m={"a":1,"b":2};m.remove("a");print(m.len())` | `1` |

---

## Execution Method

Write a parity test harness (or extend existing one):
```rust
fn assert_parity(program: &str, expected_output: &str) {
    let interp_out = run_interpreter(program).unwrap();
    let vm_out = run_vm(program).unwrap();
    assert_eq!(interp_out, expected_output, "interpreter output mismatch");
    assert_eq!(vm_out, expected_output, "vm output mismatch");
    assert_eq!(interp_out, vm_out, "parity failure: engines diverge");
}
```

Check `crates/atlas-runtime/tests/` for existing parity test patterns.

---

## Parity Failures are BLOCKING

If any parity test fails:
1. Identify which engine produces wrong output
2. File as a BLOCKING bug — do not proceed to Phase 20
3. Fix the divergence before continuing

---

## Acceptance Criteria

- [ ] All 10 array operations produce identical interpreter/VM output
- [ ] All 4 map operations produce identical interpreter/VM output
- [ ] Queue and Stack operations produce identical output (at least 3 each)
- [ ] CoW semantics verified identically in both engines
- [ ] Zero parity failures
- [ ] Parity test suite added to permanent test suite (not deleted after this phase)
