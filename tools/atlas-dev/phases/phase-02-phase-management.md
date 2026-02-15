# Phase 02: Phase Management System

**Objective:** Implement core phase tracking - complete, current, next, validate.

**Priority:** CRITICAL
**Depends On:** Phase 1

---

## Deliverables

1. ✅ Phase path parser
2. ✅ Tracker file reader/writer (markdown)
3. ✅ STATUS.md reader/writer
4. ✅ Percentage calculator
5. ✅ Next phase finder
6. ✅ Sync validator
7. ✅ Git commit automation
8. ✅ `phase complete` works end-to-end
9. ✅ `phase current` works
10. ✅ `phase next` works
11. ✅ `validate` works

---

## Implementation Steps

### Step 1: Implement Phase Path Parser

**File:** `internal/phase/parser.go`

Parse phase paths like `phases/stdlib/phase-07b-hashset.md` into components.

**Functions:**
- `Parse(path string) (*PhaseInfo, error)` - Parse phase path
- `ExtractCategory(path string) string` - Get category from path
- `ExtractName(path string) string` - Get phase name
- `MapCategoryToTracker(category string) int` - Map stdlib→1, etc.

### Step 2: Implement Tracker File Handler

**File:** `internal/tracker/tracker.go`

Read/write tracker markdown files.

**Functions:**
- `Read(path string) (*Tracker, error)` - Parse tracker file
- `Write(path string, tracker *Tracker) error` - Write tracker
- `MarkComplete(tracker *Tracker, phaseName string, desc string, date string)` - Mark ✅
- `CountCompleted(tracker *Tracker) int` - Count ✅ lines
- `CountTotal(tracker *Tracker) int` - Count all phases
- `FindNext(tracker *Tracker, currentPhase string) string` - Find next ⬜

### Step 3: Implement STATUS.md Handler

**File:** `internal/status/status.go`

Read/write STATUS.md fields.

**Functions:**
- `Read(path string) (*Status, error)` - Parse STATUS.md
- `Write(path string, status *Status) error` - Write STATUS.md
- `UpdateCurrentPhase(status *Status, completed string, next string, date string)` - Update lines 10-12
- `UpdateProgress(status *Status, completed int, total int)` - Update line 13
- `UpdateCategoryRow(status *Status, category string, progress [3]int)` - Update table row
- `UpdateLastUpdated(status *Status, date string)` - Update line 3

### Step 4: Implement Percentage Calculator

**File:** `internal/calc/percentage.go`

Calculate percentages correctly (rounding).

**Functions:**
- `Calculate(completed int, total int) int` - Returns rounded percentage
- `Format(completed int, total int) string` - Returns "X/Y (Z%)"

### Step 5: Implement Sync Validator

**File:** `internal/validator/sync.go`

Validate STATUS.md matches trackers.

**Functions:**
- `ValidateSync(statusPath string, trackerPaths []string) error` - Verify counts match
- `ValidatePercentages(status *Status) error` - Check percentage math
- `Report() *ValidationReport` - Detailed validation report

### Step 6: Implement Git Automation

**File:** `internal/git/commit.go`

Create atomic commits.

**Functions:**
- `AddFiles(paths []string) error` - Stage files
- `Commit(message string) (sha string, error)` - Create commit
- `AtomicCommit(files []string, message string) (sha string, error)` - Stage + commit atomically

### Step 7: Implement `phase complete` Command

**File:** `cmd/atlas-dev/phase_complete.go`

**Algorithm:**
1. Parse phase path → category, name
2. Find tracker file (`status/trackers/{N}-{category}.md`)
3. Read tracker, mark phase complete
4. Count completed, calculate percentages
5. Find next phase in tracker
6. Read STATUS.md
7. Update STATUS.md (5 fields)
8. Write tracker file
9. Write STATUS.md
10. Validate sync
11. Git commit (if --commit)
12. Return JSON

**JSON Output:**
```json
{
  "ok": true,
  "phase": "phase-07b",
  "cat": "stdlib",
  "progress": {
    "cat": [10, 21, 48],
    "total": [31, 78, 40]
  },
  "next": "phase-07c",
  "mod": ["status/trackers/1-stdlib.md", "STATUS.md"],
  "commit": "a1b2c3d",
  "ts": 1708012800
}
```

### Step 8: Implement `phase current`

**File:** `cmd/atlas-dev/phase_current.go`

Read STATUS.md current phase section, return JSON.

**JSON Output:**
```json
{
  "ok": true,
  "current": "phase-07c",
  "cat": "stdlib",
  "path": "phases/stdlib/phase-07c-queue-stack.md"
}
```

### Step 9: Implement `phase next`

**File:** `cmd/atlas-dev/phase_next.go`

Read STATUS.md next phase, return JSON.

**JSON Output:**
```json
{
  "ok": true,
  "next": "phase-07c",
  "cat": "stdlib",
  "path": "phases/stdlib/phase-07c-queue-stack.md",
  "desc": "Queue (FIFO) + Stack (LIFO), ~690 lines, 36+ tests"
}
```

### Step 10: Implement `validate`

**File:** `cmd/atlas-dev/validate.go`

Run full sync validation, report results.

**JSON Output (success):**
```json
{
  "ok": true,
  "trackers": 31,
  "status": 31,
  "cats": {
    "foundation": [21, 21, 100],
    "stdlib": [10, 21, 48]
  }
}
```

**JSON Output (failure):**
```json
{
  "ok": false,
  "err": "Sync validation failed",
  "details": {
    "trackers": 31,
    "status": 30,
    "mismatch": "Trackers show 31, STATUS.md shows 30"
  }
}
```

---

## Testing

### Unit Tests

Create tests for each module:
- `internal/phase/parser_test.go`
- `internal/tracker/tracker_test.go`
- `internal/status/status_test.go`
- `internal/calc/percentage_test.go`
- `internal/validator/sync_test.go`

### Integration Test

**Test with dummy data:**
```bash
# Create test fixture
cp -r status/ testdata/status-snapshot/
cp STATUS.md testdata/STATUS-snapshot.md

# Run phase complete on dummy phase
atlas-dev phase complete "phases/test/phase-00-dummy.md" \
  --desc "Test phase" \
  --dry-run

# Verify output is valid JSON
atlas-dev phase complete "..." --dry-run | jq .

# Verify validation works
atlas-dev validate
```

### Real-World Test

**Use on REAL phase (phase-07c):**
```bash
# After completing phase-07c implementation
atlas-dev phase complete "phases/stdlib/phase-07c-queue-stack.md" \
  --desc "Queue + Stack implementation, 36 tests, 100% parity" \
  --commit

# Verify:
# 1. Tracker updated (status/trackers/1-stdlib.md shows ✅ phase-07c)
# 2. STATUS.md updated (current=07c, next=07d, progress=11/21)
# 3. Git commit created
# 4. Validation passes
```

---

## Acceptance Criteria

- [x] Phase path parser correctly extracts category and name
- [x] Tracker reader parses markdown, counts ✅/⬜ correctly
- [x] Tracker writer updates phase status (⬜ → ✅)
- [x] STATUS.md reader parses all necessary fields
- [x] STATUS.md writer updates all 5 fields correctly
- [x] Percentage calculator rounds correctly (10/21 = 48%, not 47%)
- [x] Next phase finder locates next ⬜ in tracker
- [x] Sync validator detects mismatches
- [x] Git commit creates atomic commit with both files
- [x] `phase complete` works end-to-end with --dry-run
- [x] `phase complete --commit` creates real commit
- [x] `phase current` returns correct current phase
- [x] `phase next` returns correct next phase
- [x] `validate` detects sync errors
- [x] All output is valid JSON
- [x] Unit tests pass
- [x] Integration test passes
- [x] Real-world test (phase-07c) passes

---

## Files Created

```
internal/
├── phase/
│   ├── parser.go
│   ├── parser_test.go
│   └── types.go
├── tracker/
│   ├── tracker.go
│   ├── tracker_test.go
│   ├── reader.go
│   └── writer.go
├── status/
│   ├── status.go
│   ├── status_test.go
│   ├── reader.go
│   └── writer.go
├── calc/
│   ├── percentage.go
│   └── percentage_test.go
├── validator/
│   ├── sync.go
│   └── sync_test.go
└── git/
    ├── commit.go
    └── commit_test.go

cmd/atlas-dev/
├── phase_complete.go
├── phase_current.go
├── phase_next.go
└── validate.go
```

---

## Next Phase

**Phase 3:** Decision Log Integration
- Decision log parser
- Create new decision logs
- Search/list decision logs
- Get next DR-XXX number
