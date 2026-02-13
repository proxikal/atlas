# Phase 03: Complete VM Profiler

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Profiler hooks must exist from v0.1.

**Verification:**
```bash
grep -n "profiler\|TODO.*profil" crates/atlas-runtime/src/vm/mod.rs
ls crates/atlas-runtime/src/profiler/ 2>/dev/null || echo "Need to create"
grep -n "pub fn run\|pub fn execute" crates/atlas-runtime/src/vm/mod.rs
```

**What's needed:**
- VM has hooks/TODOs for profiling
- VM execution loop accessible for instrumentation
- Opcode execution can be measured

**If missing:** Check v0.1 phase bytecode-vm/phase-12

---

## Objective
Implement complete VM profiler with instruction counting, hotspot detection, and performance analysis tools for identifying optimization opportunities.

## Files
**Create:** `crates/atlas-runtime/src/profiler/mod.rs` (~800 lines)
**Create:** `crates/atlas-runtime/src/profiler/collector.rs` (~400 lines)
**Create:** `crates/atlas-runtime/src/profiler/hotspots.rs` (~350 lines)
**Create:** `crates/atlas-runtime/src/profiler/report.rs` (~250 lines)
**Update:** `crates/atlas-runtime/src/vm/mod.rs` (~100 lines add instrumentation)
**Update:** `crates/atlas-runtime/src/lib.rs` (add profiler module)
**Tests:** `crates/atlas-runtime/tests/profiler_tests.rs` (~500 lines)
**Create:** `crates/atlas-cli/src/commands/profile.rs` (~300 lines CLI integration)

## Dependencies
- v0.1 complete with VM execution
- VM allows instrumentation hooks
- CLI structure for atlas profile command

## Implementation

### Profiler Infrastructure
Create Profiler struct managing collection and reporting. Enable/disable profiling flag. Start timer on profiling begin. Record every instruction execution with opcode type and instruction pointer. Stop timer and generate report on finish.

### Profile Collector
Collect execution statistics during VM run. Count total instructions executed. Track counts per opcode type. Track counts per instruction location. Monitor call stack depth recording maximum. Track function call counts when available.

### Hotspot Detection
Identify performance bottlenecks in bytecode. Find instruction locations executed above threshold percentage. Rank hotspots by execution count descending. Identify top opcodes by frequency. Calculate percentage of total execution for each hotspot.

### Profile Report Generation
Generate human-readable performance reports. Show total execution time and instruction count. Calculate instructions per second. List top opcodes with counts and percentages. Show hotspots above threshold with locations. Include maximum stack depth reached.

### VM Integration
Add optional profiler to VM struct. Initialize profiler when requested. Record instruction before each execution. Retrieve profile report after execution completes. Measure profiling overhead keeping under ten percent.

### CLI Integration
Add atlas profile command. Read and compile source file. Run in VM with profiling enabled. Display formatted report. Save report to file. Show program result after profiling.

## Tests (TDD - Use rstest)

**Profiler tests:**
1. Instruction counting accuracy
2. Opcode breakdown correctness
3. Hotspot detection in loops
4. Stack depth tracking
5. Performance measurement IPS calculation
6. Report formatting readability
7. CLI command functionality
8. Profiling overhead acceptable

**Minimum test count:** 60 tests (40 profiler, 20 integration)

## Integration Points
- Uses: VM from vm/mod.rs
- Uses: Opcode from bytecode/mod.rs
- Updates: VM with profiler hooks
- Creates: Complete profiler with hotspot detection
- Creates: CLI profile command
- Output: Performance reports identifying bottlenecks

## Acceptance
- Instruction counting accurate
- Opcode breakdown correct
- Hotspots detected above one percent threshold
- Performance metrics calculated IPS
- Reports readable and informative
- CLI command works atlas profile program.at
- 60+ tests pass
- Profiling overhead under ten percent
- No clippy warnings
- cargo test passes
