# Pipeline Patterns for Atlas-Dev

**Purpose:** Command composition templates for AI agents

**Audience:** AI agents using atlas-dev for automated workflows

---

## Core Concepts

### Stdin Support

All atlas-dev commands support `--stdin` flag to read input from piped JSON:

```bash
# Without stdin
atlas-dev decision read DR-001

# With stdin
echo '{"id":"DR-001"}' | atlas-dev decision read --stdin
```

### JSON Piping

Commands output JSON that can be piped to other commands:

```bash
# Output from one command flows to next
atlas-dev decision search "hash" | atlas-dev decision read --stdin
```

### Batch Processing

Process multiple items efficiently:

```bash
# List all decisions and validate each
atlas-dev decision list | atlas-dev validate --stdin
```

---

## Common Patterns

### Pattern 1: Search → Read

**Use Case:** Find decisions by keyword, then read full details

```bash
# Search for decisions about "hash functions"
atlas-dev decision search "hash" | atlas-dev decision read --stdin
```

**Output:** Full details of first matching decision

**AI Usage:** Use when you need to find and read specific decisions

---

### Pattern 2: List → Context

**Use Case:** Get context for next phases in a category

```bash
# List pending phases in stdlib, get context for next
atlas-dev phase list -c stdlib -s pending | atlas-dev context phase --stdin
```

**Output:** Comprehensive context for next stdlib phase

**AI Usage:** Use to get detailed context before starting work

---

### Pattern 3: Validate → Report

**Use Case:** Run validation and generate report

```bash
# Validate parity and extract errors
atlas-dev validate parity --detailed | jq '.errors[]'
```

**Output:** Array of validation errors with file:line locations

**AI Usage:** Use to find quality issues that need fixing

---

### Pattern 4: Complete Workflow

**Use Case:** Complete phase, update feature, validate, commit if valid

```bash
# Complex workflow in one pipeline
atlas-dev phase complete "phases/stdlib/phase-07.md" \
  --description "HashMap implementation with 25 tests" \
  --tests 25 | \
jq -r '.ok' | \
xargs -I {} atlas-dev validate parity
```

**Output:** Parity validation results after phase completion

**AI Usage:** Automate multi-step workflows

---

## Advanced Patterns

### Pattern 5: Parallel Batch Processing

**Use Case:** Validate multiple features in parallel

```bash
# Get all features and validate in parallel
atlas-dev feature list | \
  atlas-dev validate --stdin --parallel --workers 4
```

**Benefits:** 4x faster than sequential processing

---

### Pattern 6: Progress Tracking

**Use Case:** Process many items with progress updates

```bash
# Process with progress to stderr (doesn't interfere with JSON output)
atlas-dev phase list | \
  atlas-dev context phase --stdin --progress
```

**Output:**
- stdout: JSON results
- stderr: Progress updates

---

### Pattern 7: Dry-Run Preview

**Use Case:** Preview changes before executing

```bash
# See what would change without modifying data
atlas-dev phase complete "phases/test/phase-01.md" \
  --description "Test" \
  --dry-run
```

**Output:** JSON with `before` and `after` fields showing changes

---

### Pattern 8: Error Handling

**Use Case:** Continue processing even if some items fail

```bash
# Process all items, collect errors
atlas-dev decision list | \
  atlas-dev decision read --stdin --continue-on-error
```

**Output:** Results + errors array for failed items

---

### Pattern 9: xargs Integration

**Use Case:** Convert JSON to line-separated for xargs

```bash
# Extract IDs and pipe to xargs
atlas-dev decision list --format=lines | \
  xargs -I {} atlas-dev decision read {}
```

**Output:** Details for each decision

---

### Pattern 10: jq Filtering

**Use Case:** Filter and transform JSON between commands

```bash
# Get pending phases, extract paths, get context
atlas-dev phase list -s pending | \
  jq -r '.phases[].path' | \
  xargs -I {} atlas-dev context phase {}
```

**Output:** Context for each pending phase

---

## Field Reference

### Common Input Fields

Commands with `--stdin` extract these fields:

| Field | Used By | Example |
|-------|---------|---------|
| `id` | decision read, feature read | `{"id":"DR-001"}` |
| `phase_id` | phase info | `{"phase_id":"phase-07"}` |
| `decision_id` | decision read | `{"decision_id":"DR-001"}` |
| `path` | phase context | `{"path":"phases/stdlib/phase-07.md"}` |
| `file_path` | spec read | `{"file_path":"docs/spec.md"}` |
| `phase_path` | phase complete | `{"phase_path":"phases/test.md"}` |

### Common Output Fields

All commands return consistent JSON:

| Field | Type | Description |
|-------|------|-------------|
| `ok` | boolean | Success/failure |
| `error` | string | Error message if ok=false |
| `items` | array | List results |
| `results` | array | Batch operation results |
| `total` | int | Total item count |
| `processed` | int | Items processed |

---

## AI Agent Templates

### Template 1: Find and Fix Decision

```bash
# 1. Search for decision
DECISION=$(atlas-dev decision search "performance")

# 2. Read details
echo "$DECISION" | atlas-dev decision read --stdin

# 3. Update if needed
# (update logic here)
```

### Template 2: Batch Validate Features

```bash
# 1. List all features
FEATURES=$(atlas-dev feature list)

# 2. Validate each in parallel
echo "$FEATURES" | \
  atlas-dev validate --stdin --parallel --continue-on-error

# 3. Report errors
# (error handling here)
```

### Template 3: Complete Phase Workflow

```bash
# 1. Complete phase
RESULT=$(atlas-dev phase complete "phases/stdlib/phase-07.md" \
  --description "HashMap: 25 tests, 100% parity" \
  --tests 25)

# 2. Check if successful
if echo "$RESULT" | jq -e '.ok' > /dev/null; then
  # 3. Validate parity
  atlas-dev validate parity

  # 4. Commit if validation passes
  # (git commit logic here)
fi
```

---

## Performance Tips

### 1. Use Parallel Processing

```bash
# 4x faster with 4 workers
--parallel --workers 4
```

### 2. Continue on Error

```bash
# Process all items even if some fail
--continue-on-error
```

### 3. Suppress Progress for Scripts

```bash
# No progress output for automated scripts
--no-progress
```

### 4. Use Compact Output

```bash
# Default is already compact (no --pretty flag exists)
# Output is optimized for piping
```

---

## Debugging Pipelines

### Enable Debug Mode

```bash
# See detailed logs
atlas-dev --debug validate parity 2>&1 | grep ERROR
```

### Check Exit Codes

```bash
# Verify command succeeded
atlas-dev phase complete "test.md" --dry-run
echo $?  # Should be 0 for success
```

### Inspect JSON Structure

```bash
# See full JSON structure
atlas-dev decision list | jq '.'
```

---

## Error Handling Best Practices

### 1. Always Check `ok` Field

```json
{
  "ok": false,
  "error": "Phase not found"
}
```

### 2. Collect Errors from Batch Operations

```json
{
  "ok": true,
  "total": 10,
  "succeeded": 8,
  "failed": 2,
  "errors": [
    {"index": 3, "item": "...", "error": "..."},
    {"index": 7, "item": "...", "error": "..."}
  ]
}
```

### 3. Use Exit Codes

- `0` = Success
- `1` = Invalid arguments
- `2` = Not found
- `3` = Validation failed

---

## Token Efficiency

### Before Composability

```
AI: Read decision DR-001
atlas-dev decision read DR-001
(35 tokens input + 150 tokens output = 185 tokens)

AI: Read decision DR-002
atlas-dev decision read DR-002
(35 tokens input + 150 tokens output = 185 tokens)

Total: 370 tokens for 2 decisions
```

### After Composability

```
AI: Read all matching decisions
atlas-dev decision search "hash" | atlas-dev decision read --stdin
(50 tokens input + 300 tokens output = 350 tokens)

Total: 350 tokens for 2 decisions (6% savings)
```

**For N items:** `(N × 185) → (50 + N × 150)` = **18% average savings**

---

## Summary

Composability enables:

1. **Chained operations:** Multiple commands in one pipeline
2. **Batch processing:** Process many items efficiently
3. **Parallel execution:** 4x+ speedup with workers
4. **Error resilience:** Continue on error, collect failures
5. **Dry-run preview:** See changes before applying
6. **Token efficiency:** Reduce AI tool calls by 18%+

**Use these patterns to automate atlas-dev workflows efficiently!**
