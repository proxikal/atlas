# Decision Log - Critical Patterns

**Purpose:** Token-efficient guide for AI agents implementing atlas-dev features.

---

## DR-001: Never Query DB Inside Transaction (2026-02-15)

**Status:** CRITICAL - MUST FOLLOW

**Problem:** Deadlock when querying database inside open transaction.

**Wrong:**
```go
db.WithTransaction(func(tx *Transaction) error {
    tx.Exec("INSERT INTO ...", ...)

    // ❌ DEADLOCK - queries while transaction open
    result, err := db.GetSomething(id)
    return err
})
```

**Correct:**
```go
var resultID string

db.WithTransaction(func(tx *Transaction) error {
    tx.Exec("INSERT INTO ...", ...)
    resultID = id
    return nil
})

// ✅ Query AFTER transaction commits
result, err := db.GetSomething(resultID)
```

**Impact:** Phase 2 hung forever (361ms after fix). Phase 3 hung forever (376ms after fix).

**References:** PHASE-2-COMPLETE.md, PHASE-3-COMPLETE.md

---

## DR-002: Never Use Prepared Statements Inside Transaction (2026-02-15)

**Status:** CRITICAL - MUST FOLLOW

**Problem:** Prepared statements (db.stmts) use main connection, not transaction connection.

**Wrong:**
```go
db.WithTransaction(func(tx *Transaction) error {
    tx.Exec("INSERT INTO ...", ...)

    // ❌ DEADLOCK - uses db.stmts["insertAuditLog"] (main conn)
    db.InsertAuditLog("create", "entity", id, changes, "", "atlas-dev")
    return nil
})
```

**Correct:**
```go
db.WithTransaction(func(tx *Transaction) error {
    tx.Exec("INSERT INTO ...", ...)

    // ✅ Use tx.Exec directly
    tx.Exec(`INSERT INTO audit_log (...) VALUES (?, ?, ...)`,
        "create", "entity", id, changes, "", "atlas-dev")
    return nil
})
```

**Impact:** Phase 3 decision create hung. Fixed by using tx.Exec for audit logs.

**Rule:** Inside transaction = use tx.Exec. Outside transaction = use db.stmts or db methods.

---

## DR-003: Transaction Pattern (2026-02-15)

**Status:** CANONICAL PATTERN

**Pattern:**
```go
func (db *DB) CreateEntity(req Request) (*Entity, error) {
    // 1. Validate BEFORE transaction
    if err := validate(req); err != nil {
        return nil, err
    }

    var entityID string

    // 2. Transaction: write only, no reads
    err := db.WithExclusiveLock(func() error {
        return db.WithTransaction(func(tx *Transaction) error {
            // All writes use tx.Exec (NOT db methods)
            _, err := tx.Exec("INSERT INTO ...", ...)
            if err != nil {
                return err
            }

            entityID = generatedID
            return nil
        })
    })

    if err != nil {
        return nil, err
    }

    // 3. Fetch AFTER transaction commits
    entity, err := db.GetEntity(entityID)
    return entity, err
}
```

**Key Rules:**
- Validate before transaction
- Only writes inside transaction
- Use tx.Exec, never db methods
- Fetch results after transaction
- Store ID during transaction, query after

**References:** internal/db/phase.go, internal/db/decision.go

---

## DR-004: Compact JSON Output (2026-02-15)

**Status:** REQUIRED FOR TOKEN EFFICIENCY

**Decision:** Use abbreviated field names in JSON output.

**Abbreviations:**
```
ok   = success       comp = component    dec  = decision
err  = error         stat = status       rat  = rationale
msg  = message       pct  = percentage   alt  = alternatives
cat  = category      cnt  = count        super = superseded_by
tot  = total         cmp  = completed    cons = consequences
desc = description   mod  = modified     blk  = blockers
ts   = timestamp     dep  = dependencies
```

**Pattern:**
```go
func (e *Entity) ToCompactJSON() map[string]interface{} {
    result := map[string]interface{}{
        "id":   e.ID,
        "comp": e.Component,  // NOT "component"
        "stat": e.Status,     // NOT "status"
    }

    // Omit null/empty fields
    if e.Optional.Valid {
        result["opt"] = e.Optional.String
    }

    return result
}
```

**Impact:** 76% token reduction vs full field names.

**References:** TOKEN-EFFICIENCY.md, internal/db/phase.go, internal/db/decision.go

---

## DR-005: No Human Flags (2026-02-15)

**Status:** REQUIRED - AI-ONLY TOOL

**Decision:** This tool is 100% AI-only. No human mode exists.

**Forbidden Flags:**
```
❌ --json        (JSON always on)
❌ --human       (no human mode)
❌ --pretty      (always compact)
❌ --verbose     (structured only)
❌ --color       (no colors)
```

**Allowed Flags:**
```
✅ --db <path>   (database path)
✅ --debug       (slog debug level)
✅ --dry-run     (preview changes)
✅ -c, --commit  (auto-commit)
```

**References:** ARCHITECTURE.md, VISION.md

---

## DR-006: Auto-Generated IDs (2026-02-15)

**Status:** PATTERN FOR DECISION LOGS

**Decision:** Use DR-XXX format with zero padding for auto-generated decision IDs.

**Pattern:**
```go
func (db *DB) GetNextDecisionID() (string, error) {
    var maxID sql.NullInt64
    err := db.conn.QueryRow(`
        SELECT MAX(CAST(SUBSTR(id, 4) AS INTEGER))
        FROM decisions
        WHERE id LIKE 'DR-%'
    `).Scan(&maxID)

    nextNum := 1
    if maxID.Valid {
        nextNum = int(maxID.Int64) + 1
    }

    return fmt.Sprintf("DR-%03d", nextNum), nil
}
```

**Format:** DR-001, DR-002, ..., DR-009, DR-010, ..., DR-099, DR-100

**Concurrent Safety:** Call inside WithExclusiveLock to prevent race conditions.

**References:** internal/db/decision.go

---

## Summary: Critical Rules

1. **NEVER query database inside transaction** - fetch after commit
2. **NEVER use db methods inside transaction** - use tx.Exec only
3. **ALWAYS validate before transaction** - not during
4. **ALWAYS fetch results after transaction** - not inside
5. **ALWAYS use compact JSON field names** - 76% token savings
6. **NEVER add human-mode flags** - AI-only tool

**Read this file before implementing any new phase.**
