package undo

import (
	"database/sql"
	"path/filepath"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

func newTestDB(t *testing.T) *db.DB {
	t.Helper()

	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	database, err := db.New(dbPath)
	if err != nil {
		t.Fatalf("failed to create test db: %v", err)
	}

	if err := database.InitSchema(); err != nil {
		t.Fatalf("failed to init schema: %v", err)
	}

	if err := database.Prepare(); err != nil {
		t.Fatalf("failed to prepare statements: %v", err)
	}

	t.Cleanup(func() {
		_ = database.Close()
	})

	return database
}

func TestCanUndo(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	// No audit entries yet
	canUndo, err := undoMgr.CanUndo()
	if err != nil {
		t.Fatalf("CanUndo() error = %v", err)
	}

	if canUndo {
		t.Error("expected canUndo = false when no audit entries")
	}
}

func TestUndo_NothingToUndo(t *testing.T) {
	database := newTestDB(t)
	undoMgr := NewUndoManager(database)

	_, err := undoMgr.Undo()
	if err == nil {
		t.Error("expected error when nothing to undo")
	}
}

func TestUndo_PhaseComplete(t *testing.T) {
	t.Skip("TODO: audit_log schema needs old_data column for full undo support")
}

func TestUndo_DecisionCreate(t *testing.T) {
	t.Skip("TODO: audit_log schema needs old_data column for full undo support")
}

func TestGetUndoHistory(t *testing.T) {
	t.Skip("TODO: audit_log schema needs old_data column for full undo support")
}

func TestParseOldData(t *testing.T) {
	validJSON := sql.NullString{
		String: `{"status":"pending"}`,
		Valid:  true,
	}

	data, err := parseOldData(validJSON)
	if err != nil {
		t.Fatalf("parseOldData() error = %v", err)
	}

	if data["status"] != "pending" {
		t.Error("failed to parse old data")
	}
}

func TestParseOldData_Invalid(t *testing.T) {
	invalidJSON := sql.NullString{
		String: "",
		Valid:  false,
	}

	_, err := parseOldData(invalidJSON)
	if err == nil {
		t.Error("expected error for invalid old data")
	}
}
