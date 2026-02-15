# STATUS.md Format Specification

**This document defines the EXACT format atlas-dev must parse and update.**

---

## Current STATUS.md Structure

**Location:** `/Users/proxikal/dev/projects/atlas/STATUS.md`

### Line Numbers (CRITICAL)

```markdown
1   # Atlas Implementation Status
2
3   **Last Updated:** 2026-02-15
4   **Version:** v0.2 (building production infrastructure)
5
6   ---
7
8   ## 🎯 Current Phase
9
10  **Last Completed:** phases/stdlib/phase-07a-hash-infrastructure-hashmap.md (verified 2026-02-15)
11  **Next Phase:** phases/stdlib/phase-07b-hashset.md
12  **Real Progress:** 30/78 phases complete (38%)
13
14  ---
15
16  ## 📊 Category Progress
17
18  | Category | Progress | Status |
19  |----------|----------|--------|
20  | **[0. Foundation](status/trackers/0-foundation.md)** | 21/21 (100%) | ✅ COMPLETE |
21  | **[1. Stdlib](status/trackers/1-stdlib.md)** | 9/21 (43%) | 🔨 ACTIVE (⚠️ blockers at phase-10+) |
22  | **[2. Bytecode-VM](status/trackers/2-bytecode-vm.md)** | 0/8 (0%) | ⬜ Pending |
23  | **[3. Frontend](status/trackers/3-frontend.md)** | 0/5 (0%) | 🚨 BLOCKED (needs foundation/04) |
24  | **[4. Typing](status/trackers/4-typing.md)** | 0/7 (0%) | ⬜ Pending |
25  | **[5. Interpreter](status/trackers/5-interpreter.md)** | 0/2 (0%) | ⬜ Pending |
26  | **[6. CLI](status/trackers/6-cli.md)** | 0/6 (0%) | 🚨 BLOCKED (needs foundation phases) |
27  | **[7. LSP](status/trackers/7-lsp.md)** | 0/5 (0%) | ⬜ Pending |
28  | **[8. Polish](status/trackers/8-polish.md)** | 0/5 (0%) | ⬜ Pending |
```

---

## Update Locations

### 1. Last Updated (Line 3)

**Pattern:** `**Last Updated:** YYYY-MM-DD`

**Update:** Replace date with current date when phase completed.

**Example:**
```markdown
**Last Updated:** 2026-02-15
```

---

### 2. Last Completed (Line 10)

**Pattern:** `**Last Completed:** phases/{category}/{phase-name}.md (verified YYYY-MM-DD)`

**Update:** Replace phase path and date when marking complete.

**Example:**
```markdown
**Last Completed:** phases/stdlib/phase-07b-hashset.md (verified 2026-02-15)
```

---

### 3. Next Phase (Line 11)

**Pattern:** `**Next Phase:** phases/{category}/{phase-name}.md`

**Update:** Replace with next pending phase from tracker.

**Example:**
```markdown
**Next Phase:** phases/stdlib/phase-07c-queue-stack.md
```

---

### 4. Real Progress (Line 12)

**Pattern:** `**Real Progress:** X/78 phases complete (Z%)`

**Update:** Recalculate from all trackers, update count and percentage.

**Example:**
```markdown
**Real Progress:** 31/78 phases complete (40%)
```

---

### 5. Category Table Row (Lines 20-28)

**Pattern:** `| **[N. Name](path)** | X/Y (Z%) | Status |`

**Update:** Find row matching category, update progress and percentage.

**Example (stdlib row, line 21):**
```markdown
| **[1. Stdlib](status/trackers/1-stdlib.md)** | 10/21 (48%) | 🔨 ACTIVE (⚠️ blockers at phase-10+) |
```

**Status column rules:**
- 0% → `⬜ Pending`
- 1-99% → `🔨 ACTIVE` (keep existing notes if present)
- 100% → `✅ COMPLETE`

---

## Tracker File Structure

**Location:** `status/trackers/{N}-{category}.md`

**Format:**
```markdown
# Category Name (X/Y) - Description

**Status:** ...
**Progress:** X/Y phases (Z%)

---

## Completed Phases

- ✅ phase-name.md **[Description, YYYY-MM-DD]**

---

## Pending Phases

- ⬜ phase-name.md **[Description]**
- 🚨 phase-name.md **[BLOCKED: reason]**
```

**Parsing rules:**
- Completed count: Count lines matching `^- ✅`
- Pending count: Count lines matching `^- [⬜🚨]`
- Total: Completed + Pending
- Next phase: First line matching `^- ⬜` after current completed phase

---

## Category to Tracker Mapping

```go
var categoryMap = map[string]int{
    "foundation": 0,
    "stdlib": 1,
    "bytecode-vm": 2,
    "frontend": 3,
    "typing": 4,
    "interpreter": 5,
    "cli": 6,
    "lsp": 7,
    "polish": 8,
}
```

**Usage:**
```go
category := "stdlib"  // extracted from "phases/stdlib/phase-07b.md"
trackerNum := categoryMap[category]  // 1
trackerPath := fmt.Sprintf("status/trackers/%d-%s.md", trackerNum, category)
// → "status/trackers/1-stdlib.md"
```

---

## Phase Completion Algorithm

**Input:**
- Phase path: `phases/stdlib/phase-07b-hashset.md`
- Description: `HashSet with 25 tests, 100% parity`
- Date: `2026-02-15`

**Steps:**

### 1. Parse Phase Path
```go
path := "phases/stdlib/phase-07b-hashset.md"
category := extractCategory(path)  // "stdlib"
phaseName := extractName(path)     // "phase-07b-hashset.md"
trackerNum := categoryMap[category] // 1
trackerPath := "status/trackers/1-stdlib.md"
```

### 2. Update Tracker
```markdown
# Before (in status/trackers/1-stdlib.md)
- ⬜ phase-07b-hashset.md **[HashSet + set operations, ~610 lines, 25+ tests]**

# After
- ✅ phase-07b-hashset.md **[HashSet with 25 tests, 100% parity, 2026-02-15]**
```

### 3. Count Completed Phases
```go
// In tracker file
completedInTracker := countLines("^- ✅")  // 10
totalInTracker := countLines("^- [✅⬜🚨]")  // 21
categoryPercent := round(10.0 / 21.0 * 100)  // 48%

// Across all trackers
completedTotal := 0
for _, tracker := range allTrackers {
    completedTotal += countCompleted(tracker)
}
// completedTotal = 31
totalPercent := round(31.0 / 78.0 * 100)  // 40%
```

### 4. Find Next Phase
```go
// In tracker file, find first ⬜ after current
nextPhase := findFirstMatch("^- ⬜")  // "phase-07c-queue-stack.md"
nextPath := fmt.Sprintf("phases/%s/%s", category, nextPhase)
// → "phases/stdlib/phase-07c-queue-stack.md"
```

### 5. Update STATUS.md

**Line 3:** `**Last Updated:** 2026-02-15`

**Line 10:** `**Last Completed:** phases/stdlib/phase-07b-hashset.md (verified 2026-02-15)`

**Line 11:** `**Next Phase:** phases/stdlib/phase-07c-queue-stack.md`

**Line 12:** `**Real Progress:** 31/78 phases complete (40%)`

**Line 21:** `| **[1. Stdlib](status/trackers/1-stdlib.md)** | 10/21 (48%) | 🔨 ACTIVE (⚠️ blockers at phase-10+) |`

### 6. Write Files
- Write `status/trackers/1-stdlib.md`
- Write `STATUS.md`

### 7. Validate Sync
```go
// Verify counts match
trackerTotal := sumAllTrackerCompleted()  // 31
statusTotal := parseStatusRealProgress()   // 31
if trackerTotal != statusTotal {
    return error("Sync mismatch")
}
```

### 8. Git Commit (if --commit)
```bash
git add status/trackers/1-stdlib.md STATUS.md
git commit -m "Mark phase-07b-hashset.md complete (31/78)"
```

---

## JSON Output Format

**Success:**
```json
{
  "ok": true,
  "phase": "phase-07b-hashset",
  "cat": "stdlib",
  "progress": {
    "cat": [10, 21, 48],
    "tot": [31, 78, 40]
  },
  "next": "phase-07c-queue-stack",
  "mod": [
    "status/trackers/1-stdlib.md",
    "STATUS.md"
  ],
  "commit": "a1b2c3d",
  "ts": 1708012800
}
```

**Failure:**
```json
{
  "ok": false,
  "err": "Phase not found in tracker",
  "phase": "phase-07b-hashset",
  "cat": "stdlib",
  "tracker": "status/trackers/1-stdlib.md"
}
```

---

## Validation Rules

### Sync Validation

**Rule 1:** Total completed in all trackers MUST match STATUS.md line 12

```go
trackerSum := 0
for _, path := range trackerPaths {
    trackerSum += countCompleted(path)
}

statusTotal := parseStatusLine12()  // "30/78" → 30

if trackerSum != statusTotal {
    return error("Sync mismatch: trackers=%d, status=%d", trackerSum, statusTotal)
}
```

**Rule 2:** Each category progress in table MUST match its tracker

```go
// For stdlib (line 21)
trackerCompleted := countCompleted("status/trackers/1-stdlib.md")  // 10
trackerTotal := countTotal("status/trackers/1-stdlib.md")           // 21
trackerPercent := round(10.0 / 21.0 * 100)                          // 48

statusProgress := parseStatusLine21()  // "10/21 (48%)"

if statusProgress != [10, 21, 48] {
    return error("Stdlib category mismatch")
}
```

### Percentage Calculation

**Rounding rule:** Round to nearest integer, 0.5 rounds up

```go
func calcPercent(completed, total int) int {
    return int(math.Round(float64(completed) / float64(total) * 100))
}

// Examples:
calcPercent(10, 21)  // 47.619... → 48
calcPercent(31, 78)  // 39.744... → 40
calcPercent(21, 21)  // 100.0 → 100
calcPercent(0, 21)   // 0.0 → 0
```

---

## Error Handling

### Phase Not Found in Tracker

```json
{
  "ok": false,
  "err": "Phase not found in tracker",
  "phase": "phase-07b-hashset.md",
  "tracker": "status/trackers/1-stdlib.md",
  "available": [
    "phase-07c-queue-stack.md",
    "phase-07d-collection-integration.md"
  ]
}
```

### Category Not Recognized

```json
{
  "ok": false,
  "err": "Unknown category",
  "phase": "phases/unknown/phase-01.md",
  "category": "unknown",
  "valid": ["foundation", "stdlib", "bytecode-vm", "frontend", "typing", "interpreter", "cli", "lsp", "polish"]
}
```

### Sync Validation Failed

```json
{
  "ok": false,
  "err": "Sync validation failed",
  "trackers": 31,
  "status": 30,
  "mismatch": [
    {
      "cat": "stdlib",
      "tracker": 10,
      "status": 9
    }
  ]
}
```

---

## Implementation Checklist

Phase 2 must implement:

- [ ] Parse phase path → extract category and name
- [ ] Map category → tracker number (0-8)
- [ ] Read tracker markdown, parse ✅/⬜ lines
- [ ] Write tracker markdown, update ⬜ → ✅
- [ ] Count completed/total phases in tracker
- [ ] Calculate percentage with correct rounding
- [ ] Find next ⬜ phase in tracker
- [ ] Read STATUS.md, parse lines 3, 10, 11, 12, 21-28
- [ ] Write STATUS.md, update 5 locations
- [ ] Validate sync (tracker sum = STATUS.md total)
- [ ] Git commit (atomic, both files)
- [ ] Output compact JSON

---

## Example Test Case

**Before:**
- `STATUS.md` line 10: `phases/stdlib/phase-07a-hash-infrastructure-hashmap.md`
- `STATUS.md` line 11: `phases/stdlib/phase-07b-hashset.md`
- `STATUS.md` line 12: `30/78 phases complete (38%)`
- `STATUS.md` line 21: `| **[1. Stdlib](...)** | 9/21 (43%) | ...`
- `status/trackers/1-stdlib.md`: 9 ✅, 12 ⬜ (21 total)

**Command:**
```bash
atlas-dev phase complete "phases/stdlib/phase-07b-hashset.md" \
  -d "HashSet with 25 tests, 100% parity" \
  --commit
```

**After:**
- `STATUS.md` line 3: `**Last Updated:** 2026-02-15`
- `STATUS.md` line 10: `phases/stdlib/phase-07b-hashset.md (verified 2026-02-15)`
- `STATUS.md` line 11: `phases/stdlib/phase-07c-queue-stack.md`
- `STATUS.md` line 12: `31/78 phases complete (40%)`
- `STATUS.md` line 21: `| **[1. Stdlib](...)** | 10/21 (48%) | ...`
- `status/trackers/1-stdlib.md`: 10 ✅, 11 ⬜ (21 total)
- Git commit: `Mark phase-07b-hashset.md complete (31/78)`

**JSON Output:**
```json
{
  "ok": true,
  "phase": "phase-07b-hashset",
  "cat": "stdlib",
  "progress": {
    "cat": [10, 21, 48],
    "tot": [31, 78, 40]
  },
  "next": "phase-07c-queue-stack",
  "mod": ["status/trackers/1-stdlib.md", "STATUS.md"],
  "commit": "a1b2c3d",
  "ts": 1708012800
}
```

---

## CRITICAL: This Format is SACRED

**DO NOT deviate from this format when implementing Phase 2.**

- Line numbers are FIXED (don't change STATUS.md structure)
- Patterns are EXACT (regex must match precisely)
- Rounding is SPECIFIC (use math.Round, not floor/ceil)
- Category mapping is CANONICAL (0-8, no changes)
- Tracker path format is FIXED (`status/trackers/{N}-{category}.md`)

**If implementation doesn't match this spec, STATUS.md will break.**
