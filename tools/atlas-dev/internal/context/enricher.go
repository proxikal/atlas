package context

import (
	"database/sql"
	"encoding/json"
	"log/slog"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// Enricher adds supplementary data to phase context
type Enricher struct {
	db *db.DB
}

// NewEnricher creates a new context enricher
func NewEnricher(database *db.DB) *Enricher {
	return &Enricher{db: database}
}

// EnrichPhaseContext adds supplementary data to phase context
func (e *Enricher) EnrichPhaseContext(ctx *PhaseContext) error {
	// Add related features (if features table exists and has data)
	if features := e.getRelatedFeatures(ctx.Category); len(features) > 0 {
		// Could add features field to PhaseContext if needed
		slog.Debug("found related features", "count", len(features), "category", ctx.Category)
	}

	// Add blocking/blocked phases info (parse from blockers field)
	if blockers := e.getBlockingPhases(ctx.ID); len(blockers) > 0 {
		// Could add blockers field to PhaseContext if needed
		slog.Debug("found blocking phases", "count", len(blockers), "id", ctx.ID)
	}

	// Add recent audit log entries
	if auditEntries := e.getRecentAuditLog(ctx.ID); len(auditEntries) > 0 {
		// Could add audit field to PhaseContext if needed
		slog.Debug("found audit entries", "count", len(auditEntries), "id", ctx.ID)
	}

	return nil
}

// getRelatedFeatures queries features table for related features
func (e *Enricher) getRelatedFeatures(category string) []string {
	// Query features table (if it exists)
	query := `
		SELECT name
		FROM features
		WHERE category = ?
		ORDER BY created_at DESC
		LIMIT 5
	`

	rows, err := e.db.Query(query, category)
	if err != nil {
		// Table might not exist yet (Phase 7), silently skip
		return nil
	}
	defer rows.Close()

	var features []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			continue
		}
		features = append(features, name)
	}

	return features
}

// getBlockingPhases gets phases that block this phase
func (e *Enricher) getBlockingPhases(phaseID int) []BlockerInfo {
	// Get phase with blockers field
	query := `
		SELECT blockers
		FROM phases
		WHERE id = ?
	`

	var blockersJSON sql.NullString
	err := e.db.QueryRow(query, phaseID).Scan(&blockersJSON)
	if err != nil || !blockersJSON.Valid {
		return nil
	}

	// Parse blockers JSON
	var blockers []string
	if err := json.Unmarshal([]byte(blockersJSON.String), &blockers); err != nil {
		return nil
	}

	// Look up blocker phase names
	var blockerInfos []BlockerInfo
	for _, blockerPath := range blockers {
		var name string
		err := e.db.QueryRow(`SELECT name FROM phases WHERE path = ?`, blockerPath).Scan(&name)
		if err == nil {
			blockerInfos = append(blockerInfos, BlockerInfo{
				Path: blockerPath,
				Name: name,
			})
		}
	}

	return blockerInfos
}

// getRecentAuditLog gets recent audit log entries for this phase
func (e *Enricher) getRecentAuditLog(phaseID int) []AuditEntry {
	query := `
		SELECT action, entity_type, created_at
		FROM audit_log
		WHERE entity_id = ?
		ORDER BY created_at DESC
		LIMIT 3
	`

	rows, err := e.db.Query(query, phaseID)
	if err != nil {
		return nil
	}
	defer rows.Close()

	var entries []AuditEntry
	for rows.Next() {
		var entry AuditEntry
		if err := rows.Scan(&entry.Action, &entry.EntityType, &entry.CreatedAt); err != nil {
			continue
		}
		entries = append(entries, entry)
	}

	return entries
}

// BlockerInfo represents a blocking phase
type BlockerInfo struct {
	Path string `json:"path"`
	Name string `json:"name"`
}

// AuditEntry represents an audit log entry
type AuditEntry struct {
	Action     string `json:"action"`
	EntityType string `json:"entity_type"`
	CreatedAt  string `json:"created_at"`
}
