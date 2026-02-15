# Phase 04: Progress Analytics & Validation

**Objective:** Analytics, statistics, validation, blocker detection.

**Priority:** HIGH
**Depends On:** Phases 1-2

---

## Deliverables

1. ✅ Progress calculator
2. ✅ Velocity tracker
3. ✅ Blocker analyzer
4. ✅ Test coverage tracker
5. ✅ Timeline generator
6. ✅ `summary` command
7. ✅ `stats` command
8. ✅ `blockers` command
9. ✅ `test-coverage` command
10. ✅ `timeline` command
11. ✅ Enhanced `validate` command

---

## Implementation

### 1. Progress Calculator

**File:** `internal/analytics/progress.go`

**Functions:**
- `CalculateOverall() ProgressData` - Overall 31/78
- `CalculateByCategory() map[string]ProgressData` - Per category
- `CalculateVelocity(days int) float64` - Phases per day
- `EstimateCompletion() time.Time` - Estimated finish date

### 2. Blocker Analyzer

**File:** `internal/analytics/blockers.go`

Scan trackers for 🚨 blocked phases.

**Functions:**
- `FindBlockers() []BlockedPhase` - All blocked phases
- `AnalyzeBlockerChains() [][]string` - Dependency chains
- `CriticalPath() []string` - Longest dependency chain

**JSON Output:**
```json
{
  "ok": true,
  "blockers": [
    {
      "phase": "phase-10-network-http",
      "cat": "stdlib",
      "reason": "needs foundation/09 + foundation/15",
      "blocking": ["phase-11-async-io"]
    }
  ],
  "cnt": 5,
  "critical_path": ["foundation/09", "stdlib/10", "stdlib/11"]
}
```

### 3. Test Coverage Tracker

**File:** `internal/analytics/tests.go`

Track test counts from phase files and git history.

**Functions:**
- `CountTests() TestStats` - Current test counts
- `TrackTestGrowth() []TestDataPoint` - Test count over time
- `CoverageByCategory() map[string]int` - Tests per category

**JSON Output:**
```json
{
  "ok": true,
  "tests": {
    "total": 1547,
    "by_cat": {
      "foundation": 767,
      "stdlib": 445,
      "vm": 0
    },
    "target": 2000,
    "pct": 77
  }
}
```

### 4. Timeline Generator

**File:** `internal/analytics/timeline.go`

Generate completion timeline from git history.

**Functions:**
- `BuildTimeline() []TimelineEvent` - Parse git log
- `PhaseCompletionDates() map[string]time.Time` - Phase → date
- `VelocityOverTime() []VelocityPoint` - Phases/week over time

**JSON Output:**
```json
{
  "ok": true,
  "timeline": [
    {"date": "2026-02-15", "phase": "phase-07a", "cat": "stdlib", "tests": 17},
    {"date": "2026-02-14", "phase": "phase-06c", "cat": "stdlib", "tests": 0}
  ],
  "velocity": {
    "last_7_days": 2.5,
    "last_30_days": 1.8,
    "all_time": 1.2
  },
  "eta": "2026-03-30"
}
```

### 5. Implement Commands

**`summary`:**
```json
{
  "ok": true,
  "progress": [31, 78, 40],
  "last": "phase-07b",
  "next": "phase-07c",
  "cats": [
    {"name": "foundation", "prog": [21, 21, 100], "status": 3},
    {"name": "stdlib", "prog": [10, 21, 48], "status": 1}
  ]
}
```

**`stats`:**
```json
{
  "ok": true,
  "velocity": {
    "phases_per_day": 0.5,
    "days_remaining": 94,
    "eta": "2026-05-20"
  },
  "tests": {
    "total": 1547,
    "target": 2000,
    "pct": 77
  },
  "categories": {
    "complete": 1,
    "active": 1,
    "pending": 7
  }
}
```

**`blockers`:**
```json
{
  "ok": true,
  "blocked": [
    {"phase": "stdlib/phase-10", "reason": "needs foundation/09,15"},
    {"phase": "frontend/phase-01", "reason": "needs foundation/04"}
  ],
  "cnt": 5,
  "critical": ["foundation/09", "stdlib/10", "stdlib/11"]
}
```

**`test-coverage`:**
```json
{
  "ok": true,
  "total": 1547,
  "by_cat": {
    "foundation": 767,
    "stdlib": 445
  },
  "target": 2000,
  "pct": 77
}
```

**`timeline`:**
```json
{
  "ok": true,
  "events": [
    {"date": "2026-02-15", "phase": "phase-07a", "tests": 17}
  ],
  "velocity": [0.8, 1.2, 0.5, 2.1],
  "eta": "2026-03-30"
}
```

---

## Testing

```bash
# Test summary
atlas-dev summary | jq '.progress'
# Expected: [31, 78, 40]

# Test stats
atlas-dev stats | jq '.velocity.eta'
# Expected: Future date

# Test blockers
atlas-dev blockers | jq '.cnt'
# Expected: 5+

# Test test-coverage
atlas-dev test-coverage | jq '.total'
# Expected: 1547

# Test timeline
atlas-dev timeline | jq '.events | length'
# Expected: 31 (one per completed phase)
```

---

## Acceptance Criteria

- [x] Progress calculator accurate (matches STATUS.md)
- [x] Velocity tracker calculates phases/day correctly
- [x] ETA estimation reasonable (based on velocity)
- [x] Blocker analyzer finds all 🚨 phases
- [x] Test coverage tracks test counts accurately
- [x] Timeline parses git history correctly
- [x] All commands return valid JSON
- [x] Percentages match manual calculations

---

## Next Phase

**Phase 5:** Documentation & Context System
- Doc indexer
- Context aggregator for phases
- Related doc/decision finder
