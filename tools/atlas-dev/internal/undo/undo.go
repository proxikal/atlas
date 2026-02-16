package undo

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log/slog"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// UndoManager handles undo operations
type UndoManager struct {
	db *db.DB
}

// NewUndoManager creates a new undo manager
func NewUndoManager(database *db.DB) *UndoManager {
	return &UndoManager{db: database}
}

// UndoResult represents the result of an undo operation
type UndoResult struct {
	Action     string                 `json:"action"`
	EntityType string                 `json:"entity_type"`
	EntityID   string                 `json:"entity_id"`
	Restored   map[string]interface{} `json:"restored,omitempty"`
}

// AuditLogEntry represents an audit log record
type AuditLogEntry struct {
	ID         int
	Action     string
	EntityType string
	EntityID   string
	OldData    sql.NullString
	Changes    sql.NullString
	CommitSHA  sql.NullString
	Agent      string
	CreatedAt  string
}

// Undo rolls back the last operation
func (u *UndoManager) Undo() (*UndoResult, error) {
	// Get most recent audit log entry
	entry, err := u.getLastAuditEntry()
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("nothing to undo")
		}
		return nil, fmt.Errorf("failed to get audit entry: %w", err)
	}

	slog.Debug("undoing operation", "action", entry.Action, "entity", entry.EntityType, "id", entry.EntityID)

	// Execute operation-specific undo
	var result *UndoResult
	switch entry.Action {
	case "complete_phase", "phase_complete":
		result, err = u.undoPhaseComplete(entry)
	case "create_decision", "decision_create":
		result, err = u.undoDecisionCreate(entry)
	case "update_decision", "decision_update":
		result, err = u.undoDecisionUpdate(entry)
	default:
		return nil, fmt.Errorf("unsupported action type: %s", entry.Action)
	}

	if err != nil {
		return nil, fmt.Errorf("failed to undo %s: %w", entry.Action, err)
	}

	// Delete audit log entry after successful undo
	if err := u.deleteAuditEntry(entry.ID); err != nil {
		slog.Warn("failed to delete audit entry after undo", "id", entry.ID, "error", err)
		// Don't fail the undo operation if audit deletion fails
	}

	slog.Info("undo completed", "action", entry.Action, "entity", entry.EntityID)

	return result, nil
}

// getLastAuditEntry retrieves the most recent audit log entry
func (u *UndoManager) getLastAuditEntry() (*AuditLogEntry, error) {
	query := `
		SELECT id, action, entity_type, entity_id, old_data, changes, commit_sha, agent, created_at
		FROM audit_log
		ORDER BY id DESC
		LIMIT 1
	`

	var entry AuditLogEntry
	err := u.db.QueryRow(query).Scan(
		&entry.ID,
		&entry.Action,
		&entry.EntityType,
		&entry.EntityID,
		&entry.OldData,
		&entry.Changes,
		&entry.CommitSHA,
		&entry.Agent,
		&entry.CreatedAt,
	)

	if err != nil {
		return nil, err
	}

	return &entry, nil
}

// deleteAuditEntry removes an audit log entry
func (u *UndoManager) deleteAuditEntry(id int) error {
	query := `DELETE FROM audit_log WHERE id = ?`
	_, err := u.db.Exec(query, id)
	return err
}

// CanUndo checks if there are operations that can be undone
func (u *UndoManager) CanUndo() (bool, error) {
	query := `SELECT COUNT(*) FROM audit_log`
	var count int
	err := u.db.QueryRow(query).Scan(&count)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

// GetUndoHistory returns the last N audit log entries
func (u *UndoManager) GetUndoHistory(limit int) ([]*AuditLogEntry, error) {
	query := `
		SELECT id, action, entity_type, entity_id, old_data, changes, commit_sha, agent, created_at
		FROM audit_log
		ORDER BY id DESC
		LIMIT ?
	`

	rows, err := u.db.Query(query, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var entries []*AuditLogEntry
	for rows.Next() {
		var entry AuditLogEntry
		err := rows.Scan(
			&entry.ID,
			&entry.Action,
			&entry.EntityType,
			&entry.EntityID,
			&entry.OldData,
			&entry.Changes,
			&entry.CommitSHA,
			&entry.Agent,
			&entry.CreatedAt,
		)
		if err != nil {
			continue
		}
		entries = append(entries, &entry)
	}

	return entries, rows.Err()
}

// parseOldData parses the old_data JSON field
func parseOldData(oldDataJSON sql.NullString) (map[string]interface{}, error) {
	if !oldDataJSON.Valid || oldDataJSON.String == "" {
		return nil, fmt.Errorf("no old data available for undo")
	}

	var oldData map[string]interface{}
	if err := json.Unmarshal([]byte(oldDataJSON.String), &oldData); err != nil {
		return nil, fmt.Errorf("failed to parse old data: %w", err)
	}

	return oldData, nil
}
