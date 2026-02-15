# Phase 05: Documentation & Context System

**Objective:** Doc search, context aggregation, intelligent phase preview.

**Priority:** MEDIUM
**Depends On:** Phases 1-3

---

## Deliverables

1. ✅ Doc indexer (hierarchy)
2. ✅ Doc search engine
3. ✅ Phase context aggregator
4. ✅ Related doc/decision finder
5. ✅ `doc search` command
6. ✅ `doc read` command
7. ✅ `doc index` command
8. ✅ `context phase` command
9. ✅ `context current` command (MOST IMPORTANT)

---

## Implementation

### 1. Doc Indexer

**File:** `internal/docs/indexer.go`

Build searchable index of docs.

**Functions:**
- `BuildIndex() (*DocIndex, error)` - Scan docs/, build index
- `IndexDirectory(dir string) []Doc` - Recursively index directory
- `ExtractMetadata(path string) Metadata` - Parse frontmatter/headers
- `BuildHierarchy() DocTree` - Create doc tree structure

### 2. Doc Search

**File:** `internal/docs/search.go`

Search docs by keyword/path.

**Functions:**
- `Search(query string) []Doc` - Find docs matching query
- `SearchByPath(pattern string) []Doc` - Glob-style search
- `FindRelated(keywords []string) []Doc` - Find related docs

### 3. Context Aggregator

**File:** `internal/context/aggregator.go`

Get EVERYTHING needed for a phase.

**Functions:**
- `GetPhaseContext(phasePath string) (*PhaseContext, error)` - Complete context
- `ExtractFilesToEdit(phaseContent string) []string` - Parse Files section
- `ExtractDependencies(phaseContent string) []string` - Parse deps
- `ExtractAcceptanceCriteria(phaseContent string) []string` - Parse acceptance
- `FindRelatedDecisions(phase *Phase) []Decision` - Keyword match
- `FindRelatedDocs(phase *Phase) []string` - Spec refs

**PhaseContext struct:**
```go
type PhaseContext struct {
    Phase      PhaseInfo
    Files      []string        // Files to create/modify
    Deps       []string        // Dependencies (must be complete)
    Blockers   []string        // Blockers
    Tests      TestTarget      // Target and current count
    Accept     []string        // Acceptance criteria
    Decisions  []DecisionRef   // Related decision logs
    Docs       []string        // Related doc paths
    Progress   ProgressData    // Current progress
}
```

### 4. Implement Commands

**`doc search "generic types"`:**
```json
{
  "ok": true,
  "query": "generic types",
  "results": [
    {"path": "docs/specification/types.md", "section": "Generic Types", "score": 95},
    {"path": "docs/implementation/07-typechecker.md", "section": "Generics", "score": 80}
  ],
  "cnt": 2
}
```

**`doc read docs/api/stdlib.md`:**
```json
{
  "ok": true,
  "path": "docs/api/stdlib.md",
  "sections": [
    {"title": "String Functions", "content": "..."},
    {"title": "Array Functions", "content": "..."}
  ]
}
```

**`doc index`:**
```json
{
  "ok": true,
  "tree": {
    "docs": {
      "specification": ["types.md", "syntax.md", "runtime.md"],
      "api": ["stdlib.md", "runtime-api.md"],
      "implementation": ["01-project-structure.md", "..."]
    }
  },
  "cnt": 47
}
```

**`context current` (MOST IMPORTANT):**
```json
{
  "ok": true,
  "phase": {
    "path": "phases/stdlib/phase-07c-queue-stack.md",
    "name": "phase-07c",
    "cat": "stdlib",
    "desc": "Queue (FIFO) + Stack (LIFO)",
    "files": [
      "crates/atlas-runtime/src/stdlib/collections/queue.rs",
      "crates/atlas-runtime/src/stdlib/collections/stack.rs",
      "crates/atlas-runtime/tests/queue_tests.rs",
      "crates/atlas-runtime/tests/stack_tests.rs"
    ],
    "dep": ["phase-07a", "phase-07b"],
    "blk": [],
    "tests": [36, 0],
    "accept": [
      "36+ tests passing",
      "Queue implements FIFO",
      "Stack implements LIFO",
      "100% parity"
    ]
  },
  "progress": {
    "cat": [10, 21, 48],
    "total": [31, 78, 40]
  },
  "decisions": [
    {"id": "DR-003", "title": "Hash function design", "path": "..."},
    {"id": "DR-005", "title": "Collection API design", "path": "..."}
  ],
  "docs": [
    "docs/api/stdlib.md#collections",
    "docs/specification/types.md#generic-types"
  ]
}
```

This command gives AI agent EVERYTHING needed to start work:
- What phase to work on
- What files to create/edit
- What dependencies must be complete
- What tests to write
- What acceptance criteria to meet
- Related decision logs for context
- Related docs to reference

---

## Testing

```bash
# Test doc search
atlas-dev doc search "generics" | jq '.cnt'
# Expected: 5+ results

# Test doc read
atlas-dev doc read docs/api/stdlib.md | jq '.sections | length'
# Expected: 10+ sections

# Test doc index
atlas-dev doc index | jq '.cnt'
# Expected: 47+ docs

# Test context current (CRITICAL)
atlas-dev context current | jq '.phase.files | length'
# Expected: 4+ files

# Test context has all needed info
atlas-dev context current | jq 'has("phase") and has("decisions") and has("docs")'
# Expected: true
```

---

## Acceptance Criteria

- [x] Doc indexer scans all docs/ directory
- [x] Doc search finds relevant docs by keyword
- [x] Context aggregator extracts phase metadata correctly
- [x] Related decision finder matches keywords
- [x] Related doc finder extracts spec references
- [x] `context current` returns comprehensive context
- [x] All commands return valid JSON
- [x] Context has everything AI needs (files, deps, tests, criteria)

---

## Next Phase

**Phase 6:** Polish & Advanced Features
- Undo/redo
- Export functionality
- Cache system
- Human mode output
