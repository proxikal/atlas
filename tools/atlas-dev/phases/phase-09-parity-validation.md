# Phase 09: Parity Validation (CRITICAL)

**Objective:** Cross-system validation - ensure code, specs, docs, tests all match.

**Priority:** CRITICAL
**Depends On:** Phases 1-8

---

## Deliverables

1. ✅ Code analyzer (parse Rust code)
2. ✅ Spec matcher (spec → code)
3. ✅ API matcher (API docs → code)
4. ✅ Test analyzer (test coverage)
5. ✅ Cross-reference validator
6. ✅ Parity checker (comprehensive)
7. ✅ `validate all` command
8. ✅ `validate parity` command (KILLER FEATURE)
9. ✅ `validate consistency` command
10. ✅ `validate tests` command
11. ✅ `validate grammar` command

---

## The Parity Problem

**Atlas has many interconnected systems:**
- Phase files reference specs
- Specs define language semantics
- Code implements specs
- API docs describe code
- Feature docs describe functionality
- Tests verify implementation
- Decision logs record choices

**If any of these drift out of sync → BUGS.**

**Example drift:**
- Spec says HashMap has 12 functions
- Code implements 11 functions
- API doc lists 13 functions
- Feature doc says 10 functions
- Tests cover 9 functions

**Which is correct? NO ONE KNOWS.**

**Solution: Automated parity validation.**

---

## Implementation

### 1. Code Analyzer

**File:** `internal/parity/code_analyzer.go`

Parse Rust code to extract structure.

```go
type CodeAnalysis struct {
    Functions   []Function
    Structs     []Struct
    Enums       []Enum
    Traits      []Trait
    Tests       []Test
}

type Function struct {
    Name       string
    Signature  string
    Params     []Param
    Returns    string
    File       string
    Line       int
    DocComment string
}

func AnalyzeRustCode(path string) (*CodeAnalysis, error) {
    // Use tree-sitter or regex parsing
    // Extract function signatures
    // Extract struct/enum definitions
    // Count tests
}

func FindFunction(analysis *CodeAnalysis, name string) *Function
func CountPublicFunctions(analysis *CodeAnalysis) int
func CountTests(analysis *CodeAnalysis) int
```

### 2. Spec Matcher

**File:** `internal/parity/spec_matcher.go`

Match spec definitions to code.

```go
func MatchSpecToCode(spec *Spec, code *CodeAnalysis) (*SpecMatch, error) {
    // For each spec definition:
    // 1. Find corresponding code
    // 2. Compare signatures/structure
    // 3. Report mismatches
}

type SpecMatch struct {
    Matches    []Match
    Mismatches []Mismatch
}

type Mismatch struct {
    Type     string  // "missing_impl", "signature_mismatch", etc.
    Spec     string  // Spec reference
    Code     string  // Code location
    Issue    string  // Description
    Fix      string  // Suggested fix
}
```

### 3. API Matcher

**File:** `internal/parity/api_matcher.go`

Match API docs to code.

```go
func MatchAPIToCode(apiDoc *APIDoc, code *CodeAnalysis) (*APIMatch, error) {
    // For each API function:
    // 1. Find in code
    // 2. Compare signature
    // 3. Verify return types
    // 4. Check error types
}
```

### 4. Test Analyzer

**File:** `internal/parity/test_analyzer.go`

Analyze test coverage.

```go
func AnalyzeTests(codeDir string) (*TestAnalysis, error) {
    // Parse test files
    // Count tests per module
    // Extract test names
    // Determine coverage
}

func CompareTestsToPhase(phase *Phase, tests *TestAnalysis) (*TestMatch, error) {
    // Phase says "35+ tests"
    // Count actual tests
    // Report if mismatch
}
```

### 5. Cross-Reference Validator

**File:** `internal/parity/ref_validator.go`

Validate cross-references.

```go
func ValidateReferences(docs []Document) (*RefValidation, error) {
    // For each reference (docs/spec/X.md):
    // 1. Check file exists
    // 2. Check section exists (if #section)
    // 3. Report broken refs
}
```

### 6. Parity Checker

**File:** `internal/parity/parity_checker.go`

Comprehensive parity validation.

```go
func CheckParity() (*ParityReport, error) {
    // Run ALL validators:
    // 1. Spec → Code parity
    // 2. API → Code parity
    // 3. Feature → Code parity
    // 4. Test count parity
    // 5. Cross-references
    // 6. Grammar consistency

    // Return comprehensive report
}

type ParityReport struct {
    OK              bool
    TotalChecks     int
    Passed          int
    Failed          int
    Warnings        int
    Errors          []ParityError
    Warnings        []ParityWarning
    Details         map[string]interface{}
}

type ParityError struct {
    Type     string  // "spec_code_mismatch", etc.
    Severity string  // "error", "warning"
    Source   string  // Where the error is
    Issue    string  // What's wrong
    Fix      string  // How to fix it
}
```

---

## Commands

### `validate parity` (THE KILLER FEATURE)

```bash
atlas-dev validate parity
```

**JSON Output (errors found):**
```json
{
  "ok": false,
  "checks": 127,
  "passed": 119,
  "failed": 8,
  "errors": [
    {
      "type": "spec_code_mismatch",
      "severity": "error",
      "spec": "docs/specification/types.md#Result",
      "code": "crates/atlas-runtime/src/value.rs:142",
      "issue": "Spec defines Result::map() method, code missing",
      "fix": "Add Result::map() method to Value::Result variant"
    },
    {
      "type": "api_code_mismatch",
      "severity": "error",
      "api": "docs/api/stdlib.md#HashMap",
      "code": "crates/atlas-runtime/src/stdlib/collections/hashmap.rs",
      "issue": "API doc lists 12 functions, code implements 11",
      "fix": "Add HashMap.drain() function or remove from API doc"
    },
    {
      "type": "test_count_mismatch",
      "severity": "error",
      "phase": "phases/stdlib/phase-07a.md",
      "target": 35,
      "actual": 17,
      "issue": "Phase requires 35+ tests, only 17 found",
      "fix": "Add 18 more tests to hashmap_tests.rs"
    },
    {
      "type": "feature_code_mismatch",
      "severity": "warning",
      "feature": "docs/features/hashmap.md",
      "code": "src/stdlib/collections/hashmap.rs",
      "issue": "Feature doc says '12 functions', code has 11",
      "fix": "Update docs/features/hashmap.md or add missing function"
    },
    {
      "type": "broken_reference",
      "severity": "warning",
      "source": "phases/stdlib/phase-07a.md:45",
      "ref": "docs/api/hashmap.md",
      "issue": "Referenced file does not exist",
      "fix": "Create docs/api/hashmap.md or fix reference"
    }
  ],
  "warnings": [
    {
      "type": "doc_stale",
      "severity": "warning",
      "doc": "docs/features/hashmap.md",
      "last_update": "2026-02-01",
      "code_update": "2026-02-15",
      "issue": "Doc not updated in 14 days, code changed 2 days ago",
      "fix": "Review and update docs/features/hashmap.md"
    }
  ],
  "details": {
    "spec_parity": {"checks": 45, "passed": 43, "failed": 2},
    "api_parity": {"checks": 30, "passed": 29, "failed": 1},
    "test_coverage": {"checks": 25, "passed": 20, "failed": 5},
    "references": {"checks": 15, "passed": 14, "failed": 1},
    "features": {"checks": 12, "passed": 11, "failed": 1}
  }
}
```

### `validate all`

```bash
atlas-dev validate all --detailed
```

Runs ALL validators:
- `validate parity` (spec/code/api/docs)
- `validate refs` (cross-references)
- `validate links` (broken links)
- `validate tests` (test coverage)
- `validate grammar` (EBNF)
- `validate consistency` (internal consistency)

**JSON Output:**
```json
{
  "ok": true,
  "health": 95,
  "checks": {
    "parity": {"ok": true, "score": 100},
    "refs": {"ok": true, "broken": 0},
    "links": {"ok": true, "broken": 0},
    "tests": {"ok": true, "coverage": 95},
    "grammar": {"ok": true, "valid": true},
    "consistency": {"ok": true, "issues": 0}
  },
  "summary": "All systems green"
}
```

### `validate tests`

```bash
atlas-dev validate tests
```

**JSON Output:**
```json
{
  "ok": false,
  "mismatches": [
    {
      "phase": "phases/stdlib/phase-07a.md",
      "target": 35,
      "actual": 17,
      "deficit": 18
    }
  ],
  "total_tests": 1547,
  "coverage_by_category": {
    "foundation": {"target": 750, "actual": 767, "ok": true},
    "stdlib": {"target": 445, "actual": 445, "ok": true}
  }
}
```

### `validate consistency`

Check internal consistency (e.g., feature doc says X, spec says Y).

```bash
atlas-dev validate consistency
```

**JSON Output:**
```json
{
  "ok": false,
  "conflicts": [
    {
      "type": "function_count_conflict",
      "sources": [
        {"doc": "docs/features/hashmap.md", "value": "12 functions"},
        {"doc": "docs/api/stdlib.md", "value": "11 functions"},
        {"doc": "code", "value": "11 actual functions"}
      ],
      "issue": "Feature doc and API doc disagree on function count",
      "fix": "Update docs/features/hashmap.md to say 11 functions"
    }
  ]
}
```

---

## Testing

```bash
# Run parity validation
atlas-dev validate parity

# Check exit code
echo $?
# 0 if all checks pass, 1 if any fail

# Validate all systems
atlas-dev validate all

# Check specific system
atlas-dev validate tests

# Get detailed report
atlas-dev validate parity --detailed | jq '.errors | length'
```

---

## Acceptance Criteria

- [x] Code analyzer parses Rust code correctly
- [x] Spec matcher finds spec/code mismatches
- [x] API matcher validates API docs against code
- [x] Test analyzer counts tests accurately
- [x] Cross-reference validator finds broken refs
- [x] Parity checker runs comprehensive validation
- [x] `validate parity` returns actionable errors
- [x] `validate all` checks everything
- [x] Exit codes correct (0=pass, 1=fail)
- [x] Error messages include fix suggestions

---

## Impact

**This is THE feature that keeps Atlas consistent.**

**Before:**
- Spec drifts from code (manual checking)
- API docs get stale (no validation)
- Test counts unverified (trust but don't verify)
- References break (find out later)

**After:**
- `atlas-dev validate parity` catches ALL drift
- Run before every commit (pre-commit hook)
- CI/CD runs validation automatically
- 100% confidence in consistency

**This makes Atlas world-class.**

---

## Next Phase

**Phase 10:** Composability & Piping
- Stdin/stdout support
- Command chaining
- Batch operations
- Parallel execution
