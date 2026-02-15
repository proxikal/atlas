# Token Efficiency Standards

**Every command MUST meet token budgets. No exceptions.**

---

## Global Standards

### Default Output: JSON
```bash
# DEFAULT (no flags needed)
atlas-dev summary
# Returns compact JSON

# Human mode (opt-in)
atlas-dev summary --human
# Returns formatted text
```

**Rationale:** AI agents parse JSON, humans read formatted text.

---

### Help Text Budgets

| Context | Token Limit | Example |
|---------|-------------|---------|
| Top-level help (`atlas-dev --help`) | < 100 tokens | See below |
| Command help (`phase --help`) | < 80 tokens | See below |
| Subcommand help (`phase complete --help`) | < 60 tokens | See below |
| Full help (opt-in with `--help-full`) | < 200 tokens | Detailed examples |

---

### Top-Level Help (< 100 tokens)

```bash
$ atlas-dev --help
atlas-dev - Atlas development automation

Commands:
  phase      Phase tracking (complete, current, next, info)
  decision   Decision logs (create, list, search)
  validate   Sync validation (status, parity, tests)
  context    Phase context (current, phase <path>)
  doc        Documentation (search, read, index)
  summary    Progress dashboard

Use: atlas-dev <cmd> --help
Quick start: atlas-dev context current
```

**Token count: ~80 tokens** ✅

---

### Command Help (< 80 tokens)

```bash
$ atlas-dev phase --help
Phase tracking and management

Commands:
  complete   Mark phase complete, update STATUS.md
  current    Show current phase
  next       Show next phase(s)
  info       Get phase metadata

Use: atlas-dev phase <cmd> --help
```

**Token count: ~50 tokens** ✅

---

### Subcommand Help (< 60 tokens)

```bash
$ atlas-dev phase complete --help
Mark phase complete, update STATUS.md + tracker

Usage: atlas-dev phase complete <path> -d "desc" [-c]
Flags: -d description, -c commit, --dry-run

Full: atlas-dev phase complete --help-full
```

**Token count: ~45 tokens** ✅

---

### Full Help (< 200 tokens, opt-in)

```bash
$ atlas-dev phase complete --help-full
Mark phase as complete and update all tracking files.

Usage:
  atlas-dev phase complete <phase-path> -d "description" [flags]

Flags:
  -d, --desc string   Phase completion description (required)
  -c, --commit        Auto-commit changes (default: false)
      --dry-run       Preview changes without writing files
      --date string   Completion date (default: today, YYYY-MM-DD)

Examples:
  # Complete phase with commit
  atlas-dev phase complete "phases/stdlib/phase-07b.md" \
    -d "HashSet with 25 tests, 100% parity" -c

  # Preview changes
  atlas-dev phase complete "phases/stdlib/phase-07b.md" \
    -d "..." --dry-run

Output: Compact JSON (default) or human-readable (--human)
```

**Token count: ~180 tokens** ✅

---

## JSON Output Standards

### Compact Notation

**Use abbreviated field names:**
```json
{
  "ok": true,         // success (not "success")
  "err": null,        // error (not "error_message")
  "msg": "...",       // message (not "message")
  "cat": "stdlib",    // category (not "category")
  "pct": 48,          // percentage (not "percentage")
  "cnt": 10,          // count (not "count")
  "tot": 78,          // total (not "total")
  "cmp": 31,          // completed (not "completed")
  "mod": [],          // modified (not "modified_files")
  "dep": [],          // dependencies (not "dependencies")
  "blk": [],          // blockers (not "blockers")
  "desc": "...",      // description (not "description")
  "ts": 1708012800    // timestamp (not "timestamp")
}
```

---

### Array Notation for Tuples

**Instead of objects, use arrays for fixed-size data:**
```json
// BAD (verbose)
{
  "progress": {
    "completed": 31,
    "total": 78,
    "percentage": 40
  }
}

// GOOD (compact)
{
  "progress": [31, 78, 40]  // [completed, total, percentage]
}
```

**Other examples:**
```json
{
  "category": ["stdlib", "active", 10, 21, 48],  // [name, status, cmp, tot, pct]
  "phase": ["phase-07b", "stdlib", "HashSet"],   // [name, category, description]
  "next": ["phase-07c", "Queue + Stack"]          // [name, description]
}
```

---

### Omit Null/Empty Fields

**Don't include fields with null/empty values:**
```json
// BAD (wasteful)
{
  "ok": true,
  "err": null,
  "warning": null,
  "blockers": [],
  "dependencies": []
}

// GOOD (compact)
{
  "ok": true
}
```

**Only include fields with actual data.**

---

### Boolean Flags

**Use single-letter keys for flags:**
```json
{
  "ok": true,   // success
  "d": false,   // dry-run
  "v": false,   // verbose
  "c": true,    // commit
  "h": false    // human mode
}
```

---

### Numeric Enums

**Use integers instead of strings for status:**
```json
// BAD (verbose)
{"status": "pending"}
{"status": "in_progress"}
{"status": "complete"}

// GOOD (compact)
{"status": 0}  // 0=pending, 1=in_progress, 2=complete
```

**Document enum mapping in API docs.**

---

## Token Budgets per Command

| Command | JSON Output (tokens) | Help Text (tokens) | Total Budget |
|---------|----------------------|--------------------|--------------|
| `phase complete` | < 120 | < 60 | < 180 |
| `phase current` | < 80 | < 50 | < 130 |
| `phase next` | < 60 | < 50 | < 110 |
| `context current` | < 200 | < 60 | < 260 |
| `decision create` | < 100 | < 60 | < 160 |
| `decision list` | < 150 | < 50 | < 200 |
| `validate` | < 80 | < 50 | < 130 |
| `validate parity` | < 200 | < 60 | < 260 |
| `summary` | < 150 | < 50 | < 200 |

---

## Measurement & Enforcement

### Token Counting
```bash
# Count tokens in help text
atlas-dev phase complete --help | wc -w
# Must be < 60 words (~45 tokens)

# Count tokens in JSON output
atlas-dev phase complete ... | jq -c | wc -c
# Divide by 4 for rough token count
```

### Acceptance Criteria Template

**Every phase MUST include:**
```markdown
## Token Efficiency Acceptance Criteria

- [ ] `<command> --help` output < X tokens
- [ ] JSON output uses compact notation (abbreviated fields)
- [ ] No emoji/colors in default output (JSON mode)
- [ ] Null/empty fields omitted
- [ ] Arrays used for tuples (not objects)
- [ ] Help examples < 50 tokens each
- [ ] Default output is JSON (not human-readable)
```

---

## Examples: Before/After

### Example 1: Phase Complete

**BEFORE (verbose, 180 tokens):**
```
✅ Phase marked complete: phase-07b-hashset.md

📊 Progress Update:
   Category: Standard Library
   Category Progress: 10 completed out of 21 total phases (48%)
   Overall Progress: 31 completed out of 78 total phases (40%)

📝 Files Updated:
   ✅ status/trackers/1-stdlib.md (updated phase status)
   ✅ STATUS.md (updated 5 fields)

🔍 Validation: PASSED
   All percentages match tracker counts

⏭️  Next Phase: phases/stdlib/phase-07c-queue-stack.md
   Description: Queue (FIFO) + Stack (LIFO), ~690 lines, 36+ tests

📦 Git Commit: a1b2c3d4e5f
   Message: Mark phase-07b-hashset.md complete (31/78)
```

**AFTER (compact JSON, 80 tokens):**
```json
{
  "ok": true,
  "phase": "phase-07b",
  "cat": "stdlib",
  "progress": {
    "cat": [10, 21, 48],
    "tot": [31, 78, 40]
  },
  "next": ["phase-07c", "Queue + Stack, ~690 lines, 36+ tests"],
  "mod": ["status/trackers/1-stdlib.md", "STATUS.md"],
  "commit": "a1b2c3d"
}
```

**Savings: 55% reduction** ✅

---

### Example 2: Context Current

**BEFORE (verbose, 250 tokens):**
```
📋 Current Phase Information

Phase: phase-07c-queue-stack.md
Category: Standard Library
Description: Queue (FIFO) + Stack (LIFO), ~690 lines, 36+ tests

Files to Create/Modify:
  - crates/atlas-runtime/src/stdlib/collections/queue.rs
  - crates/atlas-runtime/src/stdlib/collections/stack.rs
  - crates/atlas-runtime/tests/queue_tests.rs
  - crates/atlas-runtime/tests/stack_tests.rs

Dependencies:
  - phase-07a-hash-infrastructure-hashmap (✅ Complete)
  - phase-07b-hashset (✅ Complete)

Test Target: 36+ tests
Acceptance Criteria:
  - 36+ tests passing
  - Queue implements FIFO semantics
  - Stack implements LIFO semantics
  - 100% interpreter/VM parity

Progress:
  Category: 10/21 (48%)
  Overall: 31/78 (40%)

Related Decision Logs:
  - DR-003: Hash function design
  - DR-005: Collection API design
```

**AFTER (compact JSON, 120 tokens):**
```json
{
  "ok": true,
  "phase": ["phase-07c", "stdlib", "Queue + Stack, ~690 lines, 36+ tests"],
  "files": [
    "crates/atlas-runtime/src/stdlib/collections/queue.rs",
    "crates/atlas-runtime/src/stdlib/collections/stack.rs",
    "crates/atlas-runtime/tests/queue_tests.rs",
    "crates/atlas-runtime/tests/stack_tests.rs"
  ],
  "deps": ["phase-07a", "phase-07b"],
  "tests": [0, 36],
  "acceptance": [
    "36+ tests passing",
    "FIFO semantics",
    "LIFO semantics",
    "100% parity"
  ],
  "progress": {"cat": [10, 21, 48], "tot": [31, 78, 40]},
  "decisions": [
    ["DR-003", "Hash function design"],
    ["DR-005", "Collection API design"]
  ]
}
```

**Savings: 52% reduction** ✅

---

## Summary

**Every command MUST:**
1. ✅ Output JSON by default (not human-readable)
2. ✅ Use compact notation (abbreviated fields, arrays for tuples)
3. ✅ Omit null/empty fields
4. ✅ Have concise help (< 60 tokens for subcommands)
5. ✅ Meet token budgets in acceptance criteria
6. ✅ No emoji/colors in default output

**Failure to meet budgets = phase incomplete.**

**Token efficiency is NOT optional. It's REQUIRED.**
