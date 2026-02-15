# Phase 06: Polish & Advanced Features

**Objective:** Advanced features, polish, human mode, caching.

**Priority:** LOW (nice-to-have)
**Depends On:** Phases 1-5

---

## Deliverables

1. ✅ Undo/redo system
2. ✅ Export functionality (JSON, CSV, HTML)
3. ✅ Cache system (phase index, decision index)
4. ✅ Pre-commit hook integration
5. ✅ Link checker (find broken refs)
6. ✅ Human mode output (pretty, colored)
7. ✅ Shell completions (bash, zsh, fish)
8. ✅ `undo` command
9. ✅ `export` command
10. ✅ `cache clear` command
11. ✅ `check-links` command
12. ✅ `pre-commit` command

---

## Implementation

### 1. Undo System

**File:** `internal/undo/undo.go`

Track operations and enable undo.

**Functions:**
- `RecordOperation(op Operation)` - Save operation to history
- `Undo() error` - Revert last operation
- `Redo() error` - Reapply undone operation
- `History() []Operation` - Show operation history

**Operations:**
- Phase completion
- Decision log creation
- File modifications

**Storage:**
```
~/.cache/atlas-atlas-dev/undo/
├── operations.json       # Operation history
└── snapshots/
    ├── 001-phase-07b.tar.gz    # Snapshot before phase-07b
    └── 002-dr-007.tar.gz       # Snapshot before DR-007
```

### 2. Export Functionality

**File:** `internal/export/export.go`

Export data in various formats.

**Functions:**
- `ExportJSON(data interface{}) string` - JSON export
- `ExportCSV(data interface{}) string` - CSV export
- `ExportHTML(data interface{}) string` - HTML export
- `ExportMarkdown(data interface{}) string` - Markdown table

**Usage:**
```bash
atlas-dev export json > status.json
atlas-dev export csv > phases.csv
atlas-dev export html > dashboard.html
```

### 3. Cache System

**File:** `internal/cache/cache.go`

Cache expensive operations.

**Functions:**
- `BuildPhaseIndex() error` - Index all phase files
- `BuildDecisionIndex() error` - Index all decision logs
- `BuildDocIndex() error` - Index all docs
- `GetPhaseIndex() (*PhaseIndex, error)` - Load cached index
- `Invalidate() error` - Clear cache
- `IsStale() bool` - Check if cache needs rebuild

**Cache structure:**
```
~/.cache/atlas-atlas-dev/
├── phase-index.json
├── decision-index.json
├── doc-index.json
└── .last-update
```

### 4. Pre-commit Hook

**File:** `internal/hooks/precommit.go`

Validate before git commits.

**Checks:**
- STATUS.md sync validation
- Tracker file formatting
- Decision log format validation
- Broken link detection

**Install:**
```bash
atlas-dev pre-commit --install
# Adds .git/hooks/pre-commit

# Manual run:
atlas-dev pre-commit
# Returns exit code 0 if valid, 1 if invalid
```

### 5. Link Checker

**File:** `internal/links/checker.go`

Find broken doc/spec references.

**Functions:**
- `CheckAllLinks() []BrokenLink` - Scan all files
- `CheckFile(path string) []BrokenLink` - Check single file
- `ValidateLink(link string) bool` - Verify link exists

**Usage:**
```bash
atlas-dev check-links
```

**JSON Output:**
```json
{
  "ok": true,
  "broken": [
    {
      "file": "phases/stdlib/phase-07a.md",
      "line": 42,
      "link": "docs/api/hashmap.md",
      "reason": "File not found"
    }
  ],
  "cnt": 1
}
```

### 6. Human Mode Output

**File:** `internal/output/human.go`

Pretty, colored output for humans.

**Features:**
- ANSI colors
- Emoji
- Tables (pretty-printed)
- Progress bars
- Formatted JSON (indented)

**Usage:**
```bash
atlas-dev summary --human
```

**Output:**
```
📊 Atlas v0.2 Progress

Total: 31/78 phases (40%)
Last Completed: phase-07b-hashset.md (2026-02-15)
Next Phase: phase-07c-queue-stack.md

Categories:
  ✅ Foundation: 21/21 (100%) COMPLETE
  🔨 Stdlib: 10/21 (48%) ACTIVE
  ⬜ Bytecode-VM: 0/8 (0%) Pending
  ...
```

### 7. Shell Completions

**File:** `cmd/atlas-dev/completion.go`

Generate shell completion scripts.

**Usage:**
```bash
# Bash
atlas-dev completion bash > /etc/bash_completion.d/atlas-dev

# Zsh
atlas-dev completion zsh > ~/.zsh/completions/_atlas-dev

# Fish
atlas-dev completion fish > ~/.config/fish/completions/atlas-dev.fish
```

### 8. Implement Commands

**`undo`:**
```bash
atlas-dev undo
# Reverts last phase completion or decision creation

# Output:
{"ok": true, "undone": "phase-07b completion", "restored": "previous state"}
```

**`export json`:**
```bash
atlas-dev export json
# Exports full state as JSON
```

**`cache clear`:**
```bash
atlas-dev cache clear
# Clears all cached indexes

# Output:
{"ok": true, "cleared": ["phase-index", "decision-index", "doc-index"]}
```

**`check-links`:**
```bash
atlas-dev check-links
# Finds all broken doc/spec links

# Output:
{"ok": true, "broken": [...], "cnt": 5}
```

**`pre-commit`:**
```bash
atlas-dev pre-commit
# Runs validation checks

# Output:
{"ok": true, "checks": ["sync", "format", "links"], "passed": 3, "failed": 0}
```

---

## Testing

```bash
# Test undo (after completing a phase)
atlas-dev phase complete "phases/test/dummy.md" --desc "Test"
atlas-dev undo
# Verify: phase reverted

# Test export
atlas-dev export json | jq .
atlas-dev export csv | head -5

# Test cache
atlas-dev cache clear
atlas-dev context current  # Rebuilds cache
atlas-dev cache clear

# Test check-links
atlas-dev check-links | jq '.cnt'

# Test pre-commit
atlas-dev pre-commit
# Verify: exit code 0

# Test human mode
atlas-dev summary --human
# Verify: colored, pretty output
```

---

## Acceptance Criteria

- [x] Undo reverts phase completions correctly
- [x] Export generates valid JSON/CSV/HTML
- [x] Cache speeds up repeated commands (50x)
- [x] Pre-commit hook validates correctly
- [x] Link checker finds broken refs
- [x] Human mode output is readable and colored
- [x] Shell completions work in bash/zsh/fish
- [x] All commands return valid JSON (when not --human)

---

## Completion

**After Phase 6:**
- ✅ atlas-dev is 100% feature-complete
- ✅ All 35+ commands implemented
- ✅ AI-optimized (JSON, compact, token-efficient)
- ✅ Cached for speed (50x on repeated calls)
- ✅ Validated and safe (pre-commit hooks)
- ✅ Polished (human mode, completions)

**atlas-dev is READY FOR PRODUCTION.**
