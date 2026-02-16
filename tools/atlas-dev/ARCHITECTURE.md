# Atlas-Dev Architecture
**Canonical reference for all implementation patterns**

**Purpose:** Define world-class, AI-optimized architectural patterns for atlas-dev tool.

**Audience:** AI agents implementing phases (100% AI consumption, zero human use)

---

## ⚠️ Critical: Read Decision Log First

**Before implementing, read [DECISION-LOG.md](DECISION-LOG.md) for critical anti-patterns.**

Most common mistakes that cause deadlocks:
- ❌ Querying database inside transaction
- ❌ Using db methods inside transaction (use tx.Exec instead)

**See DECISION-LOG.md for complete list and examples.**

---

## Core Principles

1. **AI-Optimized** - Token efficient, structured JSON, deterministic
2. **Concurrent-Safe** - Multiple AI agents can run simultaneously
3. **ACID Guarantees** - Atomic transactions, no partial states
4. **Performance** - < 1ms queries (indexed, prepared statements)
5. **Reliable** - 80%+ test coverage, comprehensive error handling
6. **Observable** - Structured logging for AI debugging

---

## Database Architecture

### Pattern: Struct-Based DB (Not Globals)

**❌ WRONG (old pattern):**
```go
package db

var DB *sql.DB  // Global state, not concurrent-safe, hard to test

func Open(path string) error {
    DB, err := sql.Open(...)
}

func GetPhase(id int) (*Phase, error) {
    return DB.QueryRow("SELECT ...", id)  // Direct query, no caching
}
```

**✅ CORRECT (canonical pattern):**
```go
package db

import (
    "database/sql"
    "sync"
    _ "github.com/mattn/go-sqlite3"
)

// DB is the database handle with prepared statements
type DB struct {
    conn  *sql.DB
    stmts map[string]*sql.Stmt
    mu    sync.RWMutex
}

// New creates a new database connection
func New(path string) (*DB, error) {
    // Open with WAL mode + foreign keys + busy timeout
    conn, err := sql.Open("sqlite3",
        path+"?_journal_mode=WAL&_foreign_keys=ON&_busy_timeout=5000")
    if err != nil {
        return nil, fmt.Errorf("failed to open database: %w", err)
    }

    // SQLite: single writer (prevents lock contention)
    conn.SetMaxOpenConns(1)

    // Test connection
    if err := conn.Ping(); err != nil {
        return nil, fmt.Errorf("database ping failed: %w", err)
    }

    db := &DB{
        conn:  conn,
        stmts: make(map[string]*sql.Stmt),
    }

    // Prepare common queries (cache for performance)
    if err := db.prepare(); err != nil {
        return nil, err
    }

    return db, nil
}

// prepare caches common queries as prepared statements
func (db *DB) prepare() error {
    queries := map[string]string{
        "getPhase": "SELECT id, path, name, category, status, completed_date, description, test_count FROM phases WHERE id = ?",
        "getPhaseByPath": "SELECT id, path, name, category, status, completed_date, description, test_count FROM phases WHERE path = ?",
        "updatePhaseStatus": "UPDATE phases SET status = ?, completed_date = ?, description = ?, test_count = ? WHERE id = ?",
        "listPhases": "SELECT id, path, name, category, status FROM phases WHERE category = ? ORDER BY id",
        "getCategory": "SELECT completed, total, percentage FROM categories WHERE name = ?",
    }

    for name, query := range queries {
        stmt, err := db.conn.Prepare(query)
        if err != nil {
            return fmt.Errorf("failed to prepare %s: %w", name, err)
        }
        db.stmts[name] = stmt
    }

    return nil
}

// Close closes database and prepared statements
func (db *DB) Close() error {
    // Close prepared statements
    for _, stmt := range db.stmts {
        stmt.Close()
    }

    // Close connection
    if db.conn != nil {
        return db.conn.Close()
    }
    return nil
}

// GetPhase retrieves phase by ID (uses prepared statement)
func (db *DB) GetPhase(id int) (*Phase, error) {
    stmt := db.stmts["getPhase"]

    var p Phase
    err := stmt.QueryRow(id).Scan(
        &p.ID, &p.Path, &p.Name, &p.Category,
        &p.Status, &p.CompletedDate, &p.Description, &p.TestCount,
    )
    if err != nil {
        return nil, err
    }

    return &p, nil
}
```

**Why:**
- ✅ Testable (inject mock DB)
- ✅ Prepared statements (< 1ms queries)
- ✅ No global state (concurrent-safe)
- ✅ Clean lifecycle (New/Close)

---

## Transaction Handling

### Pattern: Correct Transaction with Defer

**❌ WRONG:**
```go
func WithTransaction(fn func(*sql.Tx) error) error {
    tx, _ := db.Begin()
    defer tx.Rollback()  // ❌ ALWAYS runs, even after Commit()

    if err := fn(tx); err != nil {
        return err
    }

    return tx.Commit()
}
```

**✅ CORRECT:**
```go
// Transaction wraps sql.Tx with proper cleanup
type Transaction struct {
    tx *sql.Tx
}

// WithTransaction executes fn within a transaction
// Commits on success, rollback on error/panic
func (db *DB) WithTransaction(fn func(*Transaction) error) error {
    tx, err := db.conn.Begin()
    if err != nil {
        return fmt.Errorf("failed to begin transaction: %w", err)
    }

    t := &Transaction{tx: tx}

    // Handle panic (rollback and re-panic)
    defer func() {
        if p := recover(); p != nil {
            tx.Rollback()
            panic(p)
        }
    }()

    // Execute function
    err = fn(t)
    if err != nil {
        tx.Rollback()
        return err
    }

    // Commit
    if err := tx.Commit(); err != nil {
        return fmt.Errorf("failed to commit: %w", err)
    }

    return nil
}

// Exec executes query within transaction
func (t *Transaction) Exec(query string, args ...interface{}) (sql.Result, error) {
    return t.tx.Exec(query, args...)
}

// QueryRow queries single row within transaction
func (t *Transaction) QueryRow(query string, args ...interface{}) *sql.Row {
    return t.tx.QueryRow(query, args...)
}

// Query queries multiple rows within transaction
func (t *Transaction) Query(query string, args ...interface{}) (*sql.Rows, error) {
    return t.tx.Query(query, args...)
}
```

**Why:**
- ✅ Only rollback on error/panic
- ✅ Commit happens once
- ✅ Panic-safe

---

## Concurrent Access (Multi-Agent)

### Pattern: Exclusive Lock for Writes

**Problem:** Multiple AI agents might run `phase complete` simultaneously

**Solution:** Exclusive lock + WAL mode

```go
// WithExclusiveLock ensures single writer
func (db *DB) WithExclusiveLock(fn func() error) error {
    db.mu.Lock()
    defer db.mu.Unlock()

    return fn()
}

// CompletePhase atomically completes a phase
func (db *DB) CompletePhase(phasePath, description, date string, testCount int) error {
    return db.WithExclusiveLock(func() error {
        return db.WithTransaction(func(tx *Transaction) error {
            // 1. Update phase
            // 2. Triggers update categories
            // 3. Audit log
            // All atomic
            return nil
        })
    })
}
```

**Why:**
- ✅ WAL mode: concurrent reads (multiple agents can query)
- ✅ Exclusive lock: single writer (prevents race conditions)
- ✅ ACID: all-or-nothing updates

---

## Error Handling

### Pattern: Structured Errors with Codes

**Exit codes (already defined in AI-OPTIMIZATION.md):**
```
0 = Success
1 = Invalid arguments
2 = Not found
3 = Validation failed
4 = Git operation failed
5 = Cache error
6 = Permission denied
```

**Sentinel errors:**
```go
// internal/db/errors.go
package db

import "errors"

var (
    ErrPhaseNotFound     = errors.New("phase not found")
    ErrPhaseAlreadyDone  = errors.New("phase already completed")
    ErrInvalidStatus     = errors.New("invalid status")
    ErrCategoryNotFound  = errors.New("category not found")
)
```

**Error to JSON mapping:**
```go
// internal/output/error.go
package output

import (
    "encoding/json"
    "errors"
    "os"

    "github.com/atlas-lang/atlas-dev/internal/db"
)

// Error outputs structured JSON error
func Error(err error) int {
    code := 1  // Default: invalid arguments
    response := map[string]interface{}{
        "ok":  false,
        "err": err.Error(),
    }

    // Map error to exit code
    switch {
    case errors.Is(err, db.ErrPhaseNotFound):
        code = 2
    case errors.Is(err, db.ErrPhaseAlreadyDone):
        code = 3
    // ... etc
    }

    // Output JSON
    json.NewEncoder(os.Stdout).Encode(response)

    return code
}

// ErrorWithDetails outputs error with extra context
func ErrorWithDetails(err error, details map[string]interface{}) int {
    code := 1
    response := map[string]interface{}{
        "ok":  false,
        "err": err.Error(),
    }

    // Add details
    for k, v := range details {
        response[k] = v
    }

    // Map error to exit code
    switch {
    case errors.Is(err, db.ErrPhaseNotFound):
        code = 2
        // Add suggestion
        if available, ok := details["available"].([]string); ok && len(available) > 0 {
            response["suggestion"] = "Did you mean: " + available[0] + "?"
        }
    }

    json.NewEncoder(os.Stdout).Encode(response)
    return code
}
```

**Usage in commands:**
```go
// cmd/atlas-dev/phase_complete.go
func runPhaseComplete(cmd *cobra.Command, args []string) {
    result, err := db.CompletePhase(...)
    if err != nil {
        exitCode := output.Error(err)
        os.Exit(exitCode)
    }

    output.Success(result)
}
```

---

## Logging (AI Debugging)

### Pattern: Structured Logging with slog

**Setup:**
```go
// cmd/atlas-dev/main.go
package main

import (
    "log/slog"
    "os"

    "github.com/spf13/cobra"
)

var (
    debug bool
)

func main() {
    // Configure logging based on --debug flag
    if debug {
        // JSON handler for AI parsing
        handler := slog.NewJSONHandler(os.Stderr, &slog.HandlerOptions{
            Level: slog.LevelDebug,
        })
        slog.SetDefault(slog.New(handler))
    } else {
        // INFO level by default (errors only)
        handler := slog.NewJSONHandler(os.Stderr, &slog.HandlerOptions{
            Level: slog.LevelInfo,
        })
        slog.SetDefault(slog.New(handler))
    }

    rootCmd := &cobra.Command{...}
    rootCmd.PersistentFlags().BoolVar(&debug, "debug", false, "Enable debug logging")

    if err := rootCmd.Execute(); err != nil {
        os.Exit(1)
    }
}
```

**Usage in code:**
```go
// internal/db/db.go
import "log/slog"

func (db *DB) GetPhase(id int) (*Phase, error) {
    slog.Debug("querying phase", "id", id)

    start := time.Now()
    stmt := db.stmts["getPhase"]

    var p Phase
    err := stmt.QueryRow(id).Scan(...)

    duration := time.Since(start)
    slog.Debug("query completed",
        "id", id,
        "duration_ms", duration.Milliseconds(),
        "found", err == nil,
    )

    if err != nil {
        slog.Error("query failed", "id", id, "error", err)
        return nil, err
    }

    return &p, nil
}
```

**AI usage:**
```bash
# Normal mode (errors only to stderr)
atlas-dev phase complete ...
# Stdout: {"ok":true,"phase":"phase-07b",...}
# Stderr: (empty unless error)

# Debug mode (all logs to stderr)
atlas-dev --debug phase complete ...
# Stdout: {"ok":true,"phase":"phase-07b",...}
# Stderr: {"level":"DEBUG","msg":"querying phase","id":1}
#         {"level":"DEBUG","msg":"query completed","id":1,"duration_ms":0.8}
```

**Why:**
- ✅ Structured (AI can parse)
- ✅ Separates logs (stderr) from output (stdout)
- ✅ Debug on-demand (--debug flag)

---

## Testing Strategy

### Pattern: Comprehensive Testing for AI Reliability

**1. Unit Tests (Table-Driven)**

```go
// internal/db/db_test.go
package db

import (
    "testing"
)

func TestGetPhase(t *testing.T) {
    tests := []struct {
        name    string
        phaseID int
        want    string
        wantErr bool
    }{
        {"valid phase", 1, "phase-01-core", false},
        {"invalid phase", 999, "", true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            db := newTestDB(t)

            phase, err := db.GetPhase(tt.phaseID)

            if tt.wantErr {
                if err == nil {
                    t.Error("expected error, got nil")
                }
                return
            }

            if err != nil {
                t.Fatalf("unexpected error: %v", err)
            }

            if phase.Name != tt.want {
                t.Errorf("got %s, want %s", phase.Name, tt.want)
            }
        })
    }
}
```

**2. Test Helpers**

```go
// internal/db/testing.go
package db

import (
    "testing"
)

// newTestDB creates in-memory database for tests
func newTestDB(t *testing.T) *DB {
    t.Helper()

    // Use :memory: for speed
    db, err := New(":memory:")
    if err != nil {
        t.Fatalf("failed to create test db: %v", err)
    }

    // Create schema
    if err := db.InitSchema(); err != nil {
        t.Fatalf("failed to init schema: %v", err)
    }

    // Auto-cleanup
    t.Cleanup(func() {
        db.Close()
    })

    return db
}

// seedTestPhase inserts test phase
func seedTestPhase(t *testing.T, db *DB, path, category, name string) int64 {
    t.Helper()

    result, err := db.conn.Exec(`
        INSERT INTO phases (path, name, category, status)
        VALUES (?, ?, ?, 'pending')
    `, path, name, category)
    if err != nil {
        t.Fatalf("failed to seed phase: %v", err)
    }

    id, _ := result.LastInsertId()
    return id
}
```

**3. CLI Integration Tests**

```go
// cmd/atlas-dev/phase_test.go
package main

import (
    "bytes"
    "encoding/json"
    "os/exec"
    "testing"
)

func TestPhaseCompleteCommand(t *testing.T) {
    // Setup test database
    setupTestDB(t)

    // Execute command
    cmd := exec.Command("atlas-dev",
        "--db", "test.db",
        "phase", "complete",
        "phases/test/phase-01.md",
        "-d", "Test phase completed",
    )

    output, err := cmd.CombinedOutput()
    if err != nil {
        t.Fatalf("command failed: %v\nOutput: %s", err, output)
    }

    // Parse JSON output
    var result map[string]interface{}
    if err := json.Unmarshal(output, &result); err != nil {
        t.Fatalf("failed to parse JSON: %v\nOutput: %s", err, output)
    }

    // Verify response
    if ok, _ := result["ok"].(bool); !ok {
        t.Errorf("expected ok=true, got %v", result)
    }

    // Verify database state
    verifyPhaseCompleted(t, "phases/test/phase-01.md")
}
```

**4. Benchmark Tests (Performance)**

```go
// internal/db/bench_test.go
package db

import "testing"

func BenchmarkGetPhase(b *testing.B) {
    db := newTestDB(&testing.T{})
    seedTestPhase(&testing.T{}, db, "test.md", "test", "test")

    b.ResetTimer()

    for i := 0; i < b.N; i++ {
        _, err := db.GetPhase(1)
        if err != nil {
            b.Fatal(err)
        }
    }
}

// Run: go test -bench=. -benchmem
// Assert: < 1ms per op (< 1000000 ns/op)
```

**Coverage target:** 80%+ on critical paths

```bash
go test ./... -cover
# Verify coverage >= 80%
```

---

## JSON Output (Token Efficiency)

### Pattern: Compact, Structured, Parseable

**Already defined in TOKEN-EFFICIENCY.md - follow exactly:**

```go
// internal/output/json.go
package output

import (
    "encoding/json"
    "os"
)

// Success outputs compact JSON
func Success(data map[string]interface{}) error {
    // Add ok=true
    data["ok"] = true

    // Remove null/empty fields
    cleaned := removeEmpty(data)

    // Compact encoding (no spaces)
    encoder := json.NewEncoder(os.Stdout)
    encoder.SetEscapeHTML(false)
    return encoder.Encode(cleaned)
}

// removeEmpty removes null/empty values
func removeEmpty(m map[string]interface{}) map[string]interface{} {
    result := make(map[string]interface{})
    for k, v := range m {
        if !isEmpty(v) {
            result[k] = v
        }
    }
    return result
}

// isEmpty checks if value is empty/null
func isEmpty(v interface{}) bool {
    if v == nil {
        return true
    }

    switch val := v.(type) {
    case string:
        return val == ""
    case []interface{}:
        return len(val) == 0
    case map[string]interface{}:
        return len(val) == 0
    default:
        return false
    }
}
```

**Field abbreviations (from TOKEN-EFFICIENCY.md):**
```
ok   = success
err  = error
msg  = message
cat  = category
pct  = percentage
cnt  = count
tot  = total
cmp  = completed
mod  = modified
dep  = dependencies
blk  = blockers
desc = description
ts   = timestamp
```

---

## Flags: AI-Only (No Human Options)

**CRITICAL:** This tool is 100% AI-only. No human mode exists.

### Flags that MUST exist:
```
--db <path>      # Database path (default: atlas-dev.db)
--debug          # Enable debug logging (slog DEBUG level)
--dry-run        # Preview changes without writing
-c, --commit     # Auto-commit to git
-d, --desc       # Description (for phase complete)
```

### Flags that MUST NOT exist:
```
❌ --json        # JSON is ALWAYS on (not optional)
❌ --human       # No human mode (AI-only)
❌ --pretty      # Always compact (not optional)
❌ --verbose     # Structured errors only (not optional)
❌ --color       # No colors (not optional)
```

**If you see these flags in scaffold code (cmd/atlas-dev/main.go), DELETE them.**

---

## Summary: Canonical Patterns

**When implementing ANY phase:**

1. ✅ Use struct-based DB (never globals)
2. ✅ Use prepared statements (cache queries)
3. ✅ Use correct transaction pattern (defer only on error)
4. ✅ Use exclusive locks for writes (concurrent-safe)
5. ✅ Use structured errors with codes (0-6)
6. ✅ Use slog for logging (--debug flag ONLY)
7. ✅ Write table-driven tests (80%+ coverage)
8. ✅ Write CLI integration tests
9. ✅ Write benchmark tests (< 1ms)
10. ✅ Use compact JSON output (TOKEN-EFFICIENCY.md)
11. ✅ **NO --json, --human, --pretty, --verbose flags**

**Reference this file when implementing phases 1-10.**

**This is the canonical architecture. Follow it exactly.**
