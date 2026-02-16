package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"strconv"
	"time"
)

// Decision represents a decision record
type Decision struct {
	ID            string
	Component     string
	Title         string
	DecisionText  string
	Rationale     string
	Alternatives  sql.NullString
	Consequences  sql.NullString
	Date          string
	Status        string
	SupersededBy  sql.NullString
	RelatedPhases sql.NullString
	Tags          sql.NullString
	CreatedAt     string
	UpdatedAt     string
}

// DecisionListItem represents a decision in list view
type DecisionListItem struct {
	ID        string
	Component string
	Title     string
	Date      string
	Status    string
}

// CreateDecisionRequest represents parameters for creating a decision
type CreateDecisionRequest struct {
	Component     string
	Title         string
	DecisionText  string
	Rationale     string
	Alternatives  string
	Consequences  string
	Status        string // default: "accepted"
	RelatedPhases string
	Tags          string
}

// UpdateDecisionRequest represents parameters for updating a decision
type UpdateDecisionRequest struct {
	ID           string
	Status       string
	SupersededBy string
}

// GetNextDecisionID returns the next auto-generated decision ID
func (db *DB) GetNextDecisionID() (string, error) {
	start := time.Now()

	// Query max ID number from decisions table
	query := `
		SELECT MAX(CAST(SUBSTR(id, 4) AS INTEGER))
		FROM decisions
		WHERE id LIKE 'DR-%'
	`

	var maxID sql.NullInt64
	err := db.conn.QueryRow(query).Scan(&maxID)
	if err != nil && err != sql.ErrNoRows {
		return "", fmt.Errorf("failed to query max decision ID: %w", err)
	}

	// Generate next ID with zero padding
	nextNum := 1
	if maxID.Valid {
		nextNum = int(maxID.Int64) + 1
	}

	nextID := fmt.Sprintf("DR-%03d", nextNum)

	duration := time.Since(start)
	slog.Debug("next decision ID generated",
		"next_id", nextID,
		"duration_ms", duration.Milliseconds(),
	)

	return nextID, nil
}

// CreateDecision creates a new decision with auto-generated ID
func (db *DB) CreateDecision(req CreateDecisionRequest) (*Decision, error) {
	start := time.Now()

	// Validate component exists
	_, err := db.GetCategory(req.Component)
	if err != nil {
		if err == ErrCategoryNotFound {
			return nil, fmt.Errorf("invalid component: %s (category not found)", req.Component)
		}
		return nil, err
	}

	// Default status to "accepted"
	if req.Status == "" {
		req.Status = "accepted"
	}

	// Validate status
	validStatuses := map[string]bool{
		"proposed":   true,
		"accepted":   true,
		"rejected":   true,
		"superseded": true,
	}
	if !validStatuses[req.Status] {
		return nil, fmt.Errorf("%w: %s (valid: proposed, accepted, rejected, superseded)", ErrInvalidStatus, req.Status)
	}

	var decisionID string

	// Use exclusive lock + transaction for atomic creation
	err = db.WithExclusiveLock(func() error {
		// Get next ID inside lock to prevent race
		nextID, err := db.GetNextDecisionID()
		if err != nil {
			return err
		}

		// Current date
		date := time.Now().Format("2006-01-02")

		err = db.WithTransaction(func(tx *Transaction) error {
			// Insert decision
			_, err := tx.Exec(`
				INSERT INTO decisions (
					id, component, title, decision, rationale,
					alternatives, consequences, date, status,
					related_phases, tags
				) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
			`,
				nextID, req.Component, req.Title, req.DecisionText, req.Rationale,
				nullString(req.Alternatives), nullString(req.Consequences),
				date, req.Status,
				nullString(req.RelatedPhases), nullString(req.Tags),
			)
			if err != nil {
				return fmt.Errorf("failed to insert decision: %w", err)
			}

			// Insert audit log (use tx.Exec, not db.InsertAuditLog which uses prepared stmt)
			changes := fmt.Sprintf(`{"action":"created","component":"%s","title":"%s","status":"%s"}`,
				req.Component, req.Title, req.Status)
			_, err = tx.Exec(`
				INSERT INTO audit_log (action, entity_type, entity_id, changes, commit_sha, agent)
				VALUES (?, ?, ?, ?, ?, ?)
			`, "create", "decision", nextID, changes, "", "atlas-dev")
			if err != nil {
				slog.Warn("failed to insert audit log", "error", err)
			}

			return nil
		})

		if err != nil {
			return err
		}

		// Store ID for fetching after transaction
		decisionID = nextID
		return nil
	})

	if err != nil {
		return nil, err
	}

	// Fetch created decision AFTER transaction completes
	decision, err := db.GetDecision(decisionID)
	if err != nil {
		return nil, err
	}

	duration := time.Since(start)
	slog.Debug("decision created",
		"id", decision.ID,
		"component", decision.Component,
		"duration_ms", duration.Milliseconds(),
	)

	return decision, nil
}

// GetDecision retrieves a decision by ID
func (db *DB) GetDecision(id string) (*Decision, error) {
	start := time.Now()

	query := `
		SELECT id, component, title, decision, rationale,
		       alternatives, consequences, date, status,
		       superseded_by, related_phases, tags,
		       created_at, updated_at
		FROM decisions
		WHERE id = ?
	`

	var d Decision
	err := db.conn.QueryRow(query, id).Scan(
		&d.ID, &d.Component, &d.Title, &d.DecisionText, &d.Rationale,
		&d.Alternatives, &d.Consequences, &d.Date, &d.Status,
		&d.SupersededBy, &d.RelatedPhases, &d.Tags,
		&d.CreatedAt, &d.UpdatedAt,
	)

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "getDecision",
		"id", id,
		"duration_ms", duration.Milliseconds(),
		"found", err == nil,
	)

	if err == sql.ErrNoRows {
		return nil, ErrDecisionNotFound
	}

	if err != nil {
		return nil, err
	}

	return &d, nil
}

// ListDecisionsOptions represents filtering options for listing decisions
type ListDecisionsOptions struct {
	Component string
	Status    string
	Limit     int
	Offset    int
}

// ListDecisions returns decisions matching the filter criteria
func (db *DB) ListDecisions(opts ListDecisionsOptions) ([]*DecisionListItem, error) {
	start := time.Now()

	query := `
		SELECT id, component, title, date, status
		FROM decisions
		WHERE 1=1
	`
	args := []interface{}{}

	// Add filters
	if opts.Component != "" {
		query += " AND component = ?"
		args = append(args, opts.Component)
	}

	if opts.Status != "" {
		query += " AND status = ?"
		args = append(args, opts.Status)
	}

	// Order by date DESC (most recent first)
	query += " ORDER BY date DESC, id DESC"

	// Add pagination
	limit := opts.Limit
	if limit == 0 {
		limit = 20 // Default limit
	}
	query += " LIMIT ?"
	args = append(args, limit)

	if opts.Offset > 0 {
		query += " OFFSET ?"
		args = append(args, opts.Offset)
	}

	rows, err := db.conn.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	var decisions []*DecisionListItem
	for rows.Next() {
		var d DecisionListItem
		if err := rows.Scan(&d.ID, &d.Component, &d.Title, &d.Date, &d.Status); err != nil {
			return nil, err
		}
		decisions = append(decisions, &d)
	}

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "listDecisions",
		"count", len(decisions),
		"duration_ms", duration.Milliseconds(),
	)

	return decisions, rows.Err()
}

// SearchDecisions searches decisions by query string
func (db *DB) SearchDecisions(query string, limit int) ([]*DecisionListItem, error) {
	start := time.Now()

	if limit == 0 {
		limit = 20
	}

	// Use LIKE for simple full-text search across title, decision, rationale
	searchQuery := `
		SELECT id, component, title, date, status
		FROM decisions
		WHERE title LIKE ? OR decision LIKE ? OR rationale LIKE ?
		ORDER BY date DESC, id DESC
		LIMIT ?
	`

	pattern := "%" + query + "%"
	rows, err := db.conn.Query(searchQuery, pattern, pattern, pattern, limit)
	if err != nil {
		return nil, err
	}
	defer func() { _ = rows.Close() }()

	var decisions []*DecisionListItem
	for rows.Next() {
		var d DecisionListItem
		if err := rows.Scan(&d.ID, &d.Component, &d.Title, &d.Date, &d.Status); err != nil {
			return nil, err
		}
		decisions = append(decisions, &d)
	}

	duration := time.Since(start)
	slog.Debug("query completed",
		"query", "searchDecisions",
		"search_query", query,
		"count", len(decisions),
		"duration_ms", duration.Milliseconds(),
	)

	return decisions, rows.Err()
}

// UpdateDecision updates a decision's status or superseded_by
func (db *DB) UpdateDecision(req UpdateDecisionRequest) (*Decision, error) {
	start := time.Now()

	// Validate decision exists
	existing, err := db.GetDecision(req.ID)
	if err != nil {
		return nil, err
	}

	// Validate status transition if status provided
	if req.Status != "" {
		validTransitions := map[string][]string{
			"proposed":   {"accepted", "rejected"},
			"accepted":   {"superseded"},
			"rejected":   {}, // Cannot transition from rejected
			"superseded": {}, // Cannot transition from superseded
		}

		allowed, exists := validTransitions[existing.Status]
		if !exists {
			return nil, fmt.Errorf("%w: unknown current status: %s", ErrInvalidStatus, existing.Status)
		}

		isValid := false
		for _, valid := range allowed {
			if req.Status == valid {
				isValid = true
				break
			}
		}

		if !isValid && req.Status != existing.Status {
			return nil, fmt.Errorf("%w: cannot transition from %s to %s", ErrInvalidStatus, existing.Status, req.Status)
		}
	}

	// Use exclusive lock + transaction
	err = db.WithExclusiveLock(func() error {
		return db.WithTransaction(func(tx *Transaction) error {
			// Build update query dynamically
			updates := []string{}
			args := []interface{}{}

			if req.Status != "" {
				updates = append(updates, "status = ?")
				args = append(args, req.Status)
			}

			if req.SupersededBy != "" {
				updates = append(updates, "superseded_by = ?", "status = 'superseded'")
				args = append(args, req.SupersededBy)
			}

			if len(updates) == 0 {
				return fmt.Errorf("no fields to update")
			}

			// Add ID to args
			args = append(args, req.ID)

			// Execute update
			query := fmt.Sprintf("UPDATE decisions SET %s WHERE id = ?", joinStrings(updates, ", "))
			_, err := tx.Exec(query, args...)
			if err != nil {
				return fmt.Errorf("failed to update decision: %w", err)
			}

			// Insert audit log (use tx.Exec, not db.InsertAuditLog which uses prepared stmt)
			changes := fmt.Sprintf(`{"action":"updated","old_status":"%s","new_status":"%s","superseded_by":"%s"}`,
				existing.Status, req.Status, req.SupersededBy)
			_, err = tx.Exec(`
				INSERT INTO audit_log (action, entity_type, entity_id, changes, commit_sha, agent)
				VALUES (?, ?, ?, ?, ?, ?)
			`, "update", "decision", req.ID, changes, "", "atlas-dev")
			if err != nil {
				slog.Warn("failed to insert audit log", "error", err)
			}

			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	// Fetch updated decision AFTER transaction completes
	decision, err := db.GetDecision(req.ID)
	if err != nil {
		return nil, err
	}

	duration := time.Since(start)
	slog.Debug("decision updated",
		"id", decision.ID,
		"duration_ms", duration.Milliseconds(),
	)

	return decision, nil
}

// ToCompactJSON converts Decision to compact JSON map
func (d *Decision) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"id":    d.ID,
		"comp":  d.Component,
		"title": d.Title,
		"dec":   d.DecisionText,
		"rat":   d.Rationale,
		"date":  d.Date,
		"stat":  d.Status,
	}

	if d.Alternatives.Valid && d.Alternatives.String != "" {
		result["alt"] = d.Alternatives.String
	}

	if d.Consequences.Valid && d.Consequences.String != "" {
		result["cons"] = d.Consequences.String
	}

	if d.SupersededBy.Valid && d.SupersededBy.String != "" {
		result["super"] = d.SupersededBy.String
	}

	if d.RelatedPhases.Valid && d.RelatedPhases.String != "" {
		result["phases"] = d.RelatedPhases.String
	}

	if d.Tags.Valid && d.Tags.String != "" {
		result["tags"] = d.Tags.String
	}

	return result
}

// ToCompactJSON converts DecisionListItem to compact JSON map
func (d *DecisionListItem) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"id":    d.ID,
		"comp":  d.Component,
		"title": d.Title,
		"date":  d.Date,
		"stat":  d.Status,
	}
}

// nullString converts empty string to sql.NullString
func nullString(s string) sql.NullString {
	if s == "" {
		return sql.NullString{Valid: false}
	}
	return sql.NullString{String: s, Valid: true}
}

// joinStrings joins a slice of strings with separator
func joinStrings(strs []string, sep string) string {
	if len(strs) == 0 {
		return ""
	}
	result := strs[0]
	for i := 1; i < len(strs); i++ {
		result += sep + strs[i]
	}
	return result
}

// GetDecisionIDNumber extracts the numeric part from decision ID (e.g., "DR-001" -> 1)
func GetDecisionIDNumber(id string) (int, error) {
	if len(id) < 4 || id[0:3] != "DR-" {
		return 0, fmt.Errorf("invalid decision ID format: %s", id)
	}
	return strconv.Atoi(id[3:])
}
