# Phase 10: Composability & Piping

**Objective:** Enable command composition, piping, chaining, batch operations.

**Priority:** MEDIUM
**Depends On:** Phases 1-9

---

## Deliverables

1. ✅ Stdin input support (--stdin flag)
2. ✅ JSON streaming (output → input)
3. ✅ Command chaining (&&, ||)
4. ✅ Batch operations (xargs integration)
5. ✅ Parallel execution (xargs -P)
6. ✅ Pipeline error handling
7. ✅ Progress reporting for batch ops
8. ✅ Dry-run for pipelines

---

## The Power of Composition

**Problem:** AI agents often need to do multiple things:
- Search decisions → read matching ones
- List features → validate each
- Find incomplete phases → show context
- Validate all → fix errors

**Without composition:**
```bash
# AI needs to:
# 1. Call atlas-dev decision search "hash"
# 2. Parse JSON
# 3. Extract IDs
# 4. For each ID, call atlas-dev decision read <id>
# → Multiple tool calls, complex logic
```

**With composition:**
```bash
# AI does:
atlas-dev decision search "hash" | atlas-dev decision read --stdin
# → One pipeline, simple, efficient
```

---

## Implementation

### 1. Stdin Support

**File:** `internal/compose/stdin.go`

```go
func ReadStdin() ([]byte, error)
func ParseJSONFromStdin() (interface{}, error)
func ExtractIDs(data interface{}) []string
func ExtractPaths(data interface{}) []string
```

**All commands support `--stdin` flag:**
```bash
# Instead of: atlas-dev decision read DR-001 DR-002 DR-003
# Do: echo '["DR-001", "DR-002", "DR-003"]' | atlas-dev decision read --stdin
```

### 2. JSON Streaming

**All commands output JSON by default:**
```bash
atlas-dev decision list | atlas-dev decision read --stdin
# decision list outputs JSON array
# decision read reads from stdin, outputs results
```

### 3. Command Chaining

**Shell built-in (&&, ||):**
```bash
# Run if previous succeeds
atlas-dev phase complete "..." && atlas-dev validate parity

# Run if previous fails
atlas-dev validate parity || atlas-dev validate parity --detailed

# Always run
atlas-dev phase complete "..."; atlas-dev summary
```

### 4. Batch Operations

**File:** `internal/compose/batch.go`

```go
func BatchProcess(items []string, operation func(string) error) error {
    // Process items in batch
    // Report progress
    // Handle errors
}

func BatchValidate(items []string) (*BatchResult, error)
func BatchUpdate(items []string, updates map[string]string) error
```

**Usage with xargs:**
```bash
# Validate all features
atlas-dev feature list --json | jq -r '.[].name' | xargs -I {} atlas-dev feature validate {}

# Validate phases in parallel
atlas-dev phase list --status=complete --json | jq -r '.[].path' | xargs -P8 -I {} atlas-dev validate phase {}

# Update multiple features
echo '["HashMap", "HashSet"]' | atlas-dev feature update --stdin --status="Implemented"
```

### 5. Parallel Execution

**Built-in parallel support:**
```bash
# Using xargs -P (parallel)
atlas-dev feature list --json | jq -r '.[].name' | xargs -P4 -I {} atlas-dev feature validate {}
# Validates 4 features at a time in parallel

# Using GNU parallel
atlas-dev feature list --json | jq -r '.[].name' | parallel -j8 atlas-dev feature validate {}
```

### 6. Pipeline Error Handling

**File:** `internal/compose/pipeline.go`

```go
func Pipeline(steps []PipelineStep) (*PipelineResult, error) {
    // Run steps in sequence
    // If any step fails, stop and report error
    // Return results from all steps
}

type PipelineStep struct {
    Command string
    Args    []string
    Input   interface{}
}
```

**Error propagation:**
```bash
# If any command fails, pipeline stops
atlas-dev phase complete "..." | atlas-dev feature update "..." | atlas-dev validate parity
# If phase complete fails → stops
# If feature update fails → stops
# If validate fails → reports error
```

### 7. Progress Reporting

**For batch operations:**
```bash
# With --progress flag
atlas-dev feature list --json | atlas-dev feature validate --stdin --progress

# Output (stderr, doesn't interfere with JSON):
# [1/10] Validating HashMap... OK
# [2/10] Validating HashSet... OK
# [3/10] Validating Queue... FAIL
# ...
```

### 8. Dry-Run for Pipelines

**Preview what would happen:**
```bash
atlas-dev phase complete "..." --dry-run | atlas-dev validate parity --dry-run
# Shows what each command would do without actually doing it
```

---

## Example Workflows

### Workflow 1: Search & Read

```bash
# Find all decisions about "hash", read them
atlas-dev decision search "hash" --json | jq -r '.[].id' | xargs -I {} atlas-dev decision read {}

# Or with stdin:
atlas-dev decision search "hash" | atlas-dev decision read --stdin
```

### Workflow 2: Validate All Features

```bash
# Validate all features in parallel
atlas-dev feature list --json | jq -r '.[].name' | xargs -P8 -I {} atlas-dev feature validate {}

# With progress:
atlas-dev feature list | atlas-dev feature validate --stdin --progress
```

### Workflow 3: Find & Fix Broken Links

```bash
# Find broken links, list files, fix each
atlas-dev validate links --json | jq -r '.broken[] | .file' | uniq | xargs -I {} atlas-dev check-links --fix {}
```

### Workflow 4: Complete Phase Pipeline

```bash
# Complete phase → update feature → validate parity → commit (if valid)
atlas-dev phase complete "phases/stdlib/phase-07c.md" \
  --desc "Queue+Stack, 36 tests" \
  --json | \
atlas-dev feature update "queue" --stdin | \
atlas-dev feature update "stack" --stdin | \
atlas-dev validate parity --stdin && \
atlas-dev commit --message "Complete phase-07c"

# Each step validates, passes data to next
# If validation fails, commit doesn't happen
```

### Workflow 5: Batch Update Features

```bash
# Update status for multiple features
echo '["HashMap", "HashSet", "Queue"]' | \
  atlas-dev feature update --stdin --status="Implemented" --progress
```

### Workflow 6: Find Incomplete, Show Context

```bash
# List incomplete phases, show context for each
atlas-dev phase list --status=pending --json | \
  jq -r '.[].path' | \
  head -5 | \
  xargs -I {} atlas-dev context phase {}
```

---

## Command Flag Additions

**All commands get:**
- `--stdin` - Read input from stdin (JSON)
- `--json` - Output JSON (default, but explicit)
- `--progress` - Show progress for batch ops
- `--dry-run` - Preview without executing
- `--parallel <n>` - Process items in parallel (built-in)

**Example:**
```bash
atlas-dev feature validate --stdin --progress --parallel 4
```

---

## Testing

```bash
# Test stdin support
echo '["DR-001", "DR-002"]' | atlas-dev decision read --stdin

# Test piping
atlas-dev decision search "hash" | atlas-dev decision read --stdin

# Test xargs
atlas-dev feature list --json | jq -r '.[].name' | xargs -I {} atlas-dev feature validate {}

# Test parallel
atlas-dev feature list --json | jq -r '.[].name' | xargs -P4 -I {} atlas-dev feature validate {}

# Test chaining
atlas-dev phase complete "..." && atlas-dev validate parity

# Test dry-run
atlas-dev phase complete "..." --dry-run | atlas-dev validate parity --dry-run

# Test progress
atlas-dev feature list | atlas-dev feature validate --stdin --progress
```

---

## Acceptance Criteria

- [x] All commands support --stdin flag
- [x] JSON output is parseable by next command
- [x] Piping works (command1 | command2)
- [x] xargs integration works
- [x] Parallel execution works (xargs -P)
- [x] Error handling stops pipeline on failure
- [x] Progress reporting works for batch ops
- [x] Dry-run shows what would happen
- [x] Exit codes propagate correctly

---

## Impact

**Before (no composition):**
```bash
# AI agent needs to:
# 1. Call atlas-dev decision search "hash"
# 2. Parse JSON response
# 3. Extract IDs: ["DR-001", "DR-003"]
# 4. For each ID:
#    - Call atlas-dev decision read <id>
#    - Parse response
#    - Store results
# → 5 tool calls, complex logic
```

**After (with composition):**
```bash
# AI agent does:
atlas-dev decision search "hash" | atlas-dev decision read --stdin
# → 1 pipeline, simple, efficient
```

**Benefits:**
- ✅ Fewer tool calls (1 pipeline vs N individual calls)
- ✅ Simpler logic (no parsing/looping in AI)
- ✅ More powerful (can do complex workflows)
- ✅ Faster (parallel execution)
- ✅ Unix philosophy (do one thing well, compose)

---

## Completion

**After Phase 10:**
- ✅ atlas-dev is 100% feature-complete
- ✅ 82+ commands implemented
- ✅ Full docs management
- ✅ Parity validation
- ✅ Composable & pipeable
- ✅ AI-optimized (JSON, compact, efficient)
- ✅ World-class unified development platform

**atlas-dev is READY FOR PRODUCTION.**

**Total implementation: 36-47 hours (4.5-6 days)**
