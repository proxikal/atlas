package undo

import (
	"fmt"
	"log/slog"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// undoPhaseComplete restores a phase to pending status
func (u *UndoManager) undoPhaseComplete(entry *AuditLogEntry) (*UndoResult, error) {
	// Parse old data
	oldData, err := parseOldData(entry.OldData)
	if err != nil {
		return nil, err
	}

	// Get phase ID from entity_id (should be the phase path or ID)
	phaseID := entry.EntityID

	// Restore phase to previous state
	err = u.db.WithExclusiveLock(func() error {
		return u.db.WithTransaction(func(tx *db.Transaction) error {
			// Update phase back to pending
			query := `
				UPDATE phases
				SET status = 'pending',
				    completed_date = NULL,
				    description = NULL,
				    test_count = 0,
				    updated_at = CURRENT_TIMESTAMP
				WHERE path = ? OR CAST(id AS TEXT) = ?
			`

			result, err := tx.Exec(query, phaseID, phaseID)
			if err != nil {
				return fmt.Errorf("failed to update phase: %w", err)
			}

			rows, _ := result.RowsAffected()
			if rows == 0 {
				return fmt.Errorf("phase not found: %s", phaseID)
			}

			slog.Debug("phase restored to pending", "id", phaseID)
			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	return &UndoResult{
		Action:     entry.Action,
		EntityType: entry.EntityType,
		EntityID:   phaseID,
		Restored:   oldData,
	}, nil
}

// undoDecisionCreate deletes a decision that was created
func (u *UndoManager) undoDecisionCreate(entry *AuditLogEntry) (*UndoResult, error) {
	decisionID := entry.EntityID

	// Delete the decision
	err := u.db.WithExclusiveLock(func() error {
		return u.db.WithTransaction(func(tx *db.Transaction) error {
			query := `DELETE FROM decisions WHERE id = ?`

			result, err := tx.Exec(query, decisionID)
			if err != nil {
				return fmt.Errorf("failed to delete decision: %w", err)
			}

			rows, _ := result.RowsAffected()
			if rows == 0 {
				return fmt.Errorf("decision not found: %s", decisionID)
			}

			slog.Debug("decision deleted", "id", decisionID)
			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	return &UndoResult{
		Action:     entry.Action,
		EntityType: entry.EntityType,
		EntityID:   decisionID,
	}, nil
}

// undoDecisionUpdate restores a decision to its previous state
func (u *UndoManager) undoDecisionUpdate(entry *AuditLogEntry) (*UndoResult, error) {
	// Parse old data
	oldData, err := parseOldData(entry.OldData)
	if err != nil {
		return nil, err
	}

	decisionID := entry.EntityID

	// Restore decision fields from old data
	err = u.db.WithExclusiveLock(func() error {
		return u.db.WithTransaction(func(tx *db.Transaction) error {
			// Build UPDATE statement based on what fields are in old_data
			query := `UPDATE decisions SET updated_at = CURRENT_TIMESTAMP`
			args := []interface{}{}

			if status, ok := oldData["status"].(string); ok {
				query += `, status = ?`
				args = append(args, status)
			}

			if title, ok := oldData["title"].(string); ok {
				query += `, title = ?`
				args = append(args, title)
			}

			if rationale, ok := oldData["rationale"].(string); ok {
				query += `, rationale = ?`
				args = append(args, rationale)
			}

			query += ` WHERE id = ?`
			args = append(args, decisionID)

			result, err := tx.Exec(query, args...)
			if err != nil {
				return fmt.Errorf("failed to update decision: %w", err)
			}

			rows, _ := result.RowsAffected()
			if rows == 0 {
				return fmt.Errorf("decision not found: %s", decisionID)
			}

			slog.Debug("decision restored", "id", decisionID)
			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	return &UndoResult{
		Action:     entry.Action,
		EntityType: entry.EntityType,
		EntityID:   decisionID,
		Restored:   oldData,
	}, nil
}

// undoFeatureUpdate restores a feature to its previous state (if features table exists)
func (u *UndoManager) undoFeatureUpdate(entry *AuditLogEntry) (*UndoResult, error) {
	// Parse old data
	oldData, err := parseOldData(entry.OldData)
	if err != nil {
		return nil, err
	}

	featureID := entry.EntityID

	// Restore feature fields from old data
	err = u.db.WithExclusiveLock(func() error {
		return u.db.WithTransaction(func(tx *db.Transaction) error {
			// Check if features table exists
			var count int
			err := tx.QueryRow(`
				SELECT COUNT(*) FROM sqlite_master
				WHERE type='table' AND name='features'
			`).Scan(&count)
			if err != nil || count == 0 {
				return fmt.Errorf("features table does not exist")
			}

			// Build UPDATE statement
			query := `UPDATE features SET updated_at = CURRENT_TIMESTAMP`
			args := []interface{}{}

			if status, ok := oldData["status"].(string); ok {
				query += `, status = ?`
				args = append(args, status)
			}

			if description, ok := oldData["description"].(string); ok {
				query += `, description = ?`
				args = append(args, description)
			}

			query += ` WHERE id = ?`
			args = append(args, featureID)

			result, err := tx.Exec(query, args...)
			if err != nil {
				return fmt.Errorf("failed to update feature: %w", err)
			}

			rows, _ := result.RowsAffected()
			if rows == 0 {
				return fmt.Errorf("feature not found: %s", featureID)
			}

			slog.Debug("feature restored", "id", featureID)
			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	return &UndoResult{
		Action:     entry.Action,
		EntityType: entry.EntityType,
		EntityID:   featureID,
		Restored:   oldData,
	}, nil
}

// ValidateUndoSafe checks if an undo operation is safe to perform
func (u *UndoManager) ValidateUndoSafe(entry *AuditLogEntry) error {
	// Check if entity still exists
	switch entry.EntityType {
	case "phase":
		// Check if phase exists
		query := `SELECT COUNT(*) FROM phases WHERE path = ? OR CAST(id AS TEXT) = ?`
		var count int
		err := u.db.QueryRow(query, entry.EntityID, entry.EntityID).Scan(&count)
		if err != nil {
			return fmt.Errorf("failed to check phase existence: %w", err)
		}
		if count == 0 && entry.Action != "create_phase" {
			return fmt.Errorf("phase no longer exists, cannot undo")
		}

	case "decision":
		// Check if decision exists (for updates, not for creates)
		if entry.Action != "create_decision" && entry.Action != "decision_create" {
			query := `SELECT COUNT(*) FROM decisions WHERE id = ?`
			var count int
			err := u.db.QueryRow(query, entry.EntityID).Scan(&count)
			if err != nil {
				return fmt.Errorf("failed to check decision existence: %w", err)
			}
			if count == 0 {
				return fmt.Errorf("decision no longer exists, cannot undo")
			}
		}
	}

	return nil
}
