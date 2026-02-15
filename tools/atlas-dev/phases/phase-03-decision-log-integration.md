# Phase 03: Decision Log Integration

**Objective:** Full decision log management - create, read, list, search.

**Priority:** HIGH
**Depends On:** Phases 1-2

---

## Deliverables

1. ✅ Decision log parser (markdown)
2. ✅ Next ID calculator (DR-XXX)
3. ✅ Template generator
4. ✅ Search indexer
5. ✅ `decision create` command
6. ✅ `decision list` command
7. ✅ `decision read <id>` command
8. ✅ `decision search <query>` command
9. ✅ `decision next-id <component>` command

---

## Implementation

### 1. Decision Log Parser

**File:** `internal/decision/parser.go`

Parse decision log markdown files.

**Functions:**
- `Parse(path string) (*Decision, error)` - Parse DR-XXX file
- `ParseFrontmatter(content string) map[string]string` - Extract metadata
- `ExtractSections(content string) map[string]string` - Get Context, Decision, etc.

**Struct:**
```go
type Decision struct {
    ID           string   // DR-001
    Title        string
    Date         string   // YYYY-MM-DD
    Status       int      // 0=Accepted, 1=Superseded, 2=Deprecated
    Component    string
    Context      string
    Decision     string
    Rationale    string
    Alternatives []string
    Benefits     []string
    Tradeoffs    []string
    Costs        []string
    ImplNotes    string
    References   []string
    Supersedes   string
    SupersededBy string
    Path         string
}
```

### 2. Next ID Calculator

**File:** `internal/decision/next_id.go`

Find next DR-XXX number for a component.

**Functions:**
- `NextID(component string) (string, error)` - Get next DR-XXX
- `ScanDirectory(dir string) []string` - List all DR-XXX in directory
- `ParseID(filename string) int` - Extract number from DR-XXX

**Algorithm:**
1. List files in `docs/decision-logs/{component}/`
2. Parse all DR-XXX numbers
3. Find max number
4. Return max + 1 (formatted as DR-XXX)

### 3. Template Generator

**File:** `internal/decision/template.go`

Generate decision log from template.

**Functions:**
- `Generate(id, title, component, date string) string` - Create markdown
- `FillTemplate(data map[string]string) string` - Fill template with data

**Template:**
```markdown
# DR-{ID}: {Title}

**Date:** {Date}
**Status:** Accepted
**Component:** {Component}

## Context
{Context}

## Decision
{Decision}

## Rationale
{Rationale}

## Alternatives Considered
{Alternatives}

## Consequences
- ✅ **Benefits:** {Benefits}
- ⚠️  **Trade-offs:** {Tradeoffs}
- ❌ **Costs:** {Costs}

## Implementation Notes
{ImplNotes}

## References
{References}
```

### 4. Search Indexer

**File:** `internal/decision/search.go`

Search decision logs by keyword.

**Functions:**
- `BuildIndex() (*Index, error)` - Scan all decision logs, build index
- `Search(query string) []Decision` - Find decisions matching query
- `ByComponent(component string) []Decision` - Filter by component
- `ByDate(startDate, endDate string) []Decision` - Filter by date range

### 5. Implement Commands

**`decision create`:**
```go
// cmd/atlas-dev/decision_create.go
func runDecisionCreate(cmd *cobra.Command, args []string) {
    // Get flags
    component := cmd.Flag("component").Value.String()
    title := cmd.Flag("title").Value.String()
    interactive := cmd.Flag("interactive").Changed

    // Get next ID
    nextID := decision.NextID(component)

    if interactive {
        // Prompt for fields
        context := prompt("Context: ")
        decisionText := prompt("Decision: ")
        rationale := prompt("Rationale: ")
        // ...
    }

    // Generate markdown from template
    content := decision.Generate(nextID, title, component, date)

    // Write file
    path := fmt.Sprintf("docs/decision-logs/%s/%s-%s.md",
        component, nextID, slugify(title))
    os.WriteFile(path, []byte(content), 0644)

    // Output JSON
    out.Success(map[string]interface{}{
        "ok": true,
        "decision": map[string]string{
            "id":    nextID,
            "path":  path,
            "title": title,
        },
    })
}
```

**`decision list`:**
```json
{
  "ok": true,
  "decisions": [
    {"id": "DR-001", "title": "...", "comp": "runtime", "date": "2026-01-15", "status": 0},
    {"id": "DR-002", "title": "...", "comp": "stdlib", "date": "2026-01-20", "status": 0}
  ],
  "cnt": 2,
  "by_comp": {"runtime": 2, "stdlib": 4}
}
```

**`decision read DR-001`:**
```json
{
  "ok": true,
  "decision": {
    "id": "DR-001",
    "title": "Value representation",
    "date": "2026-01-15",
    "status": 0,
    "comp": "runtime",
    "context": "...",
    "decision": "...",
    "rationale": "...",
    ...
  }
}
```

**`decision search "hash function"`:**
```json
{
  "ok": true,
  "query": "hash function",
  "results": [
    {"id": "DR-003", "title": "Hash function design", "comp": "stdlib", "matches": 3}
  ],
  "cnt": 1
}
```

**`decision next-id stdlib`:**
```json
{
  "ok": true,
  "component": "stdlib",
  "next_id": "DR-007"
}
```

---

## Testing

```bash
# Test next-id
atlas-dev decision next-id stdlib
# Expected: {"ok":true,"component":"stdlib","next_id":"DR-007"}

# Test list
atlas-dev decision list | jq '.decisions | length'
# Expected: 16 (current count)

# Test read
atlas-dev decision read DR-001 | jq '.decision.title'
# Expected: "Value representation"

# Test search
atlas-dev decision search "hash" | jq '.cnt'
# Expected: 3+ results

# Test create (dry-run)
atlas-dev decision create \
  --component "stdlib" \
  --title "Iterator protocol" \
  --dry-run
# Expected: Shows what would be created
```

---

## Acceptance Criteria

- [x] Parser correctly reads existing decision logs
- [x] Next ID calculator finds correct DR-XXX number
- [x] Template generator creates valid markdown
- [x] Search finds decisions by keyword
- [x] `decision create` creates new decision log file
- [x] `decision list` returns all decisions (JSON)
- [x] `decision read` returns full decision data (JSON)
- [x] `decision search` finds matching decisions
- [x] `decision next-id` returns correct next number
- [x] All output is valid, compact JSON

---

## Next Phase

**Phase 4:** Progress Analytics & Validation
- Progress calculator
- Velocity tracker
- Blocker analyzer
- Test coverage
- Timeline
