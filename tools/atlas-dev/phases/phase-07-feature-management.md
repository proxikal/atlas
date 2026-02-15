# Phase 07: Feature Management

**Objective:** Manage feature documentation (docs/features/) - create, read, update, validate, sync.

**Priority:** HIGH
**Depends On:** Phases 1-6

---

## Deliverables

1. ✅ Feature doc parser (markdown)
2. ✅ Feature CRUD operations
3. ✅ Feature validation (against code/spec)
4. ✅ Feature sync (auto-update from code)
5. ✅ `feature create` command
6. ✅ `feature list` command
7. ✅ `feature read` command
8. ✅ `feature update` command
9. ✅ `feature validate` command
10. ✅ `feature sync` command
11. ✅ `feature delete` command
12. ✅ `feature search` command

---

## Feature Doc Format

**Example: `docs/features/hashmap.md`**

```markdown
# HashMap

**Category:** Collections
**Status:** Implemented
**Since:** v0.2.0
**Spec:** docs/specification/types.md#HashMap
**API:** docs/api/stdlib.md#HashMap

## Overview
HashMap provides O(1) average-case key-value storage.

## Functions
- `HashMap.new() -> HashMap<K, V>`
- `HashMap.insert(map, key, value) -> HashMap<K, V>`
- `HashMap.get(map, key) -> Option<V>`
- ... (12 total)

## Implementation
- File: `crates/atlas-runtime/src/stdlib/collections/hashmap.rs`
- Tests: `crates/atlas-runtime/tests/hashmap_tests.rs`
- Test Count: 17
- Parity: 100%

## Related
- Decision: DR-006
- Features: HashSet, Queue, Stack
```

---

## Implementation

### 1. Feature Parser

**File:** `internal/feature/parser.go`

```go
type Feature struct {
    Name         string
    Category     string
    Status       string   // Implemented, In Progress, Planned
    Since        string   // v0.2.0
    Spec         string   // Spec reference
    API          string   // API reference
    Overview     string
    Functions    []string
    Implementation struct {
        File      string
        Tests     string
        TestCount int
        Parity    int    // Percentage
    }
    Related      struct {
        Decisions []string
        Features  []string
    }
    Path         string
}

func Parse(path string) (*Feature, error)
func List(dir string) ([]Feature, error)
```

### 2. Feature Validator

**File:** `internal/feature/validator.go`

Validate feature against code and spec.

```go
func Validate(feature *Feature) (*ValidationResult, error) {
    // Check:
    // 1. Spec reference exists
    // 2. API reference exists
    // 3. Implementation file exists
    // 4. Test file exists
    // 5. Function count matches code
    // 6. Test count matches actual tests
    // 7. Parity claim is accurate
}
```

### 3. Feature Sync

**File:** `internal/feature/sync.go`

Auto-update feature docs from code.

```go
func Sync(featureName string) error {
    // Parse code to extract:
    // - Function count
    // - Test count
    // - Last modified date
    // Update feature doc automatically
}
```

### 4. Commands

**`feature create`:**
```bash
atlas-dev feature create "Iterator" --category "collections"
```

**JSON Output:**
```json
{
  "ok": true,
  "feature": {
    "name": "Iterator",
    "path": "docs/features/iterator.md",
    "category": "collections",
    "status": "Planned"
  }
}
```

**`feature list`:**
```bash
atlas-dev feature list --category "collections"
```

**JSON Output:**
```json
{
  "ok": true,
  "features": [
    {"name": "HashMap", "category": "collections", "status": "Implemented"},
    {"name": "HashSet", "category": "collections", "status": "Planned"},
    {"name": "Iterator", "category": "collections", "status": "Planned"}
  ],
  "cnt": 3
}
```

**`feature read HashMap`:**
```json
{
  "ok": true,
  "feature": {
    "name": "HashMap",
    "category": "collections",
    "status": "Implemented",
    "functions": 12,
    "test_count": 17,
    "parity": 100
  }
}
```

**`feature validate HashMap`:**
```json
{
  "ok": true,
  "checks": [
    {"name": "spec_ref_exists", "ok": true},
    {"name": "api_ref_exists", "ok": true},
    {"name": "impl_file_exists", "ok": true},
    {"name": "test_file_exists", "ok": true},
    {"name": "function_count", "ok": true, "expected": 12, "actual": 12},
    {"name": "test_count", "ok": true, "expected": 17, "actual": 17},
    {"name": "parity", "ok": true, "expected": 100, "actual": 100}
  ],
  "passed": 7,
  "failed": 0
}
```

**`feature sync HashMap`:**
```bash
atlas-dev feature sync HashMap

# Parses code, updates feature doc with latest counts
```

---

## Testing

```bash
# Create feature
atlas-dev feature create "Iterator" --category "collections"

# List features
atlas-dev feature list | jq '.cnt'
# Expected: 3+

# Read feature
atlas-dev feature read HashMap | jq '.feature.functions'
# Expected: 12

# Validate feature
atlas-dev feature validate HashMap | jq '.ok'
# Expected: true

# Sync feature
atlas-dev feature sync HashMap
# Expected: Updates doc with current code stats
```

---

## Acceptance Criteria

- [x] Parser correctly reads feature docs
- [x] CRUD operations work (create, read, update, delete)
- [x] Validator checks all criteria (spec refs, impl files, counts)
- [x] Sync auto-updates from code
- [x] All commands return valid JSON
- [x] Feature validation catches mismatches

---

## Next Phase

**Phase 8:** Spec & API Management
- Spec parsing and validation
- API doc management
- Grammar validation (EBNF)
