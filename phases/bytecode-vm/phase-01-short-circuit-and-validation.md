# Phase 01: Short-Circuit Evaluation & Bytecode Validation

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** VM must have LogicalAnd/LogicalOr opcodes with TODO markers.

**Verification:**
```bash
grep -n "LogicalAnd\|LogicalOr" crates/atlas-runtime/src/bytecode/mod.rs
grep -n "TODO.*short.*circuit" crates/atlas-runtime/src/vm/mod.rs
cargo test vm_logical
```

**What's needed:**
- Opcodes LogicalAnd/LogicalOr exist from v0.1
- VM executes them (currently evaluates both sides - wrong)
- TODO comments indicate need for implementation

**If missing:** Check v0.1 phase bytecode-vm/phase-08 completed

---

## Objective
Implement proper short-circuit evaluation for && and || operators in VM, and add comprehensive bytecode validation to detect malformed bytecode before execution.

## Files
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~50 lines changed)
**Update:** `crates/atlas-runtime/src/compiler/mod.rs` (~100 lines changed)
**Create:** `crates/atlas-runtime/src/bytecode/validator.rs` (~600 lines)
**Update:** `crates/atlas-runtime/src/bytecode/mod.rs` (add validator module)
**Tests:** `crates/atlas-runtime/tests/vm_short_circuit_tests.rs` (~300 lines)
**Tests:** `crates/atlas-runtime/tests/bytecode_validator_tests.rs` (~400 lines)

## Dependencies
- v0.1 complete with VM, compiler, bytecode format
- Existing LogicalAnd/LogicalOr opcodes
- Atlas-SPEC.md defines short-circuit semantics

## Implementation

### Short-Circuit Evaluation
Current v0.1 evaluates both operands which is wrong. Implement jump-based short-circuiting. Compiler emits conditional jumps instead of logical opcodes. For AND: jump if left false skipping right evaluation. For OR: jump if left true skipping right. Add JumpIfTrue/JumpIfFalse/Dup opcodes if needed. Remove old LogicalAnd/LogicalOr opcodes.

### Bytecode Validator
Create comprehensive validation before execution. Check jump targets within bounds. Verify constant indices not excessive. Simulate stack depth detecting underflow. Detect unreachable code after returns/jumps. Build reachability graph marking dead code. Update jump targets after dead code removal.

### VM Integration
Integrate validator in VM constructor. Reject invalid bytecode before execution. Add stack_effect method to Opcode calculating push/pop delta. Use in validator for depth simulation.

## Tests (TDD - Use rstest)

**Short-circuit tests:**
1. AND with false left - right not evaluated
2. OR with true left - right not evaluated
3. Non-short-circuit cases - right evaluated when needed
4. Nested logical operators
5. Side effects only when evaluated
6. Function calls in logical expressions

**Validator tests:**
1. Invalid jump targets - out of bounds
2. Stack underflow detection
3. Stack overflow warnings
4. Invalid constant indices
5. Unreachable code detection
6. Valid bytecode passes all checks
7. Validator performance acceptable

**Minimum test count:** 80 tests (40 short-circuit, 40 validator)

## Integration Points
- Uses: Opcode enum from bytecode/mod.rs
- Uses: VM from vm/mod.rs
- Uses: Compiler from compiler/mod.rs
- Updates: Compiler jump-based logical ops
- Updates: VM with new jump opcodes
- Creates: Validator for bytecode verification
- Output: Correct short-circuit semantics, validated bytecode

## Acceptance
- Short-circuit evaluation works - right side skipped when unnecessary
- Side effects only when operand evaluated
- Bytecode validator catches all malformed categories
- Valid bytecode passes validation
- VM rejects invalid bytecode pre-execution
- 80+ tests pass
- Interpreter/VM parity both short-circuit correctly
- No performance regression
- Files under 100 lines changed each
- validator.rs under 700 lines
- No clippy warnings
- cargo test passes
