# Phase Infra-04: Ignore Audit — Zero Unexplained Skips

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Infra-01, 02, 03 complete. Suite green. ~17 test binaries.

**Verification:**
```bash
cargo nextest run -p atlas-runtime 2>&1 | tail -5
grep -rn "^#\[ignore\]$" crates/atlas-runtime/tests/ --include="*.rs"
grep -rn "^#\[ignore\]$" crates/atlas-runtime/src/ --include="*.rs"
```
The second and third commands show bare `#[ignore]` with no reason string — these are the problem. Every line in that output is a hidden broken or forgotten test.

---

## Objective
Every `#[ignore]` in the codebase must have an explicit, honest reason string. Bare `#[ignore]` is a code smell — it means someone knew a test was broken and buried it. Audit all ~40+ bare ignores: fix and re-enable, or delete with a git commit message explaining why. Zero unexplained skips by end of phase.

## Files
**Audit scope:** All `*.rs` files in `crates/atlas-runtime/tests/` and `crates/atlas-runtime/src/`
**No new files created.** Edits are surgical — each `#[ignore]` becomes one of three outcomes.

## Dependencies
- Infra-01, 02, 03 complete
- Understanding of WHY each test was originally ignored (check git blame)

## Implementation

### Step 1: Full inventory
Run the audit command and collect every bare `#[ignore]` with file and line number:
```bash
grep -rn "^#\[ignore\]$" crates/atlas-runtime/ --include="*.rs" -B2
```
Categorize each one before touching any code. Three buckets:
- **Bucket A — Broken:** Test fails because the feature isn't implemented yet or has a bug
- **Bucket B — Environment:** Test requires something not available (network, platform, tokio runtime, real library)
- **Bucket C — Dead:** Test is for behavior that no longer exists or was superseded

### Step 2: Bucket B — Add proper reason strings
For tests that legitimately need to be skipped, change bare `#[ignore]` to `#[ignore = "reason"]`. Use consistent reason strings: `"requires network"`, `"requires tokio LocalSet context"`, `"requires platform: linux"`, `"requires external library: libm"`. These tests stay skipped but are now self-documenting. Run: `cargo nextest run -p atlas-runtime` to confirm suite still green.

### Step 3: Bucket A — Fix or delete
For broken tests: attempt to fix each one. If the feature IS implemented and the test is just wrong, fix the test. If the feature is genuinely not implemented yet, add `#[ignore = "not yet implemented: feature-name"]` so it is tracked. If the test is permanently superseded by better tests, `git rm` it. Do NOT leave `#[ignore]` without a reason. Document each decision in the commit message.

### Step 4: Bucket C — Delete dead tests
Dead tests with no path to re-enablement get deleted. Use `git rm` with a descriptive commit message. A deleted test is honest. A silently skipped test is a lie.

### Step 5: Verify ignored tests run correctly when enabled
For all tests that now have `#[ignore = "reason"]`, verify the reason is accurate by attempting to run them:
```bash
cargo nextest run -p atlas-runtime --run-ignored all 2>&1 | grep -E "FAIL|ERROR"
```
Network tests will fail (expected). Platform tests may fail (expected). Any failure NOT explained by the ignore reason is a bug — fix it.

### Step 6: Final audit
```bash
grep -rn "^#\[ignore\]$" crates/atlas-runtime/ --include="*.rs"
```
This command must produce zero output. Zero. If any bare `#[ignore]` remains, the phase is not complete.

### Step 7: Document the outcome
Add a brief comment block at the top of files that have many intentional ignores (e.g., `async_runtime.rs`, `ffi.rs`) explaining the ignore pattern and when those tests will be re-enabled. One paragraph per file is enough.

## Tests
No new tests. Metric: bare `#[ignore]` count goes from ~40 to 0. All remaining ignores have explicit reason strings. Suite remains green.

## Integration Points
- No runtime code changes
- No API changes
- Changes are purely test metadata and deletion of dead tests

## Acceptance
- `grep -rn "^#\[ignore\]$" crates/atlas-runtime/ --include="*.rs"` returns zero lines
- Every remaining `#[ignore]` has a non-empty reason string
- `cargo nextest run -p atlas-runtime` suite green
- No test count regression (only deletions of genuinely dead tests)
- `cargo clippy -p atlas-runtime -- -D warnings` zero warnings
- Each bucket handled in a separate git commit with clear message
- Final commit: `fix(tests): Audit complete — zero unexplained #[ignore] annotations`
