package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"strings"
	"time"
)

// Feature represents a feature record in the database
type Feature struct {
	ID                  int
	Name                string
	DisplayName         string
	Version             string
	Status              string
	Description         sql.NullString
	ImplementationNotes sql.NullString
	RelatedPhases       sql.NullString
	SpecPath            sql.NullString
	APIPath             sql.NullString
	CreatedAt           string
	UpdatedAt           string
}

// FeatureListItem represents a feature in list view
type FeatureListItem struct {
	Name        string
	DisplayName string
	Version     string
	Status      string
}

// CreateFeatureRequest represents parameters for creating a feature
type CreateFeatureRequest struct {
	Name        string
	DisplayName string
	Version     string // default: "v0.1"
	Status      string // default: "Planned"
	Description string
	SpecPath    string
	APIPath     string
}

// UpdateFeatureRequest represents parameters for updating a feature
type UpdateFeatureRequest struct {
	Name                string
	Version             string
	Status              string
	Description         string
	ImplementationNotes string
	RelatedPhases       string
	SpecPath            string
	APIPath             string
}

// CreateFeature creates a new feature record
func (db *DB) CreateFeature(req CreateFeatureRequest) (*Feature, error) {
	start := time.Now()

	// Validate name not empty
	if req.Name == "" {
		return nil, fmt.Errorf("feature name cannot be empty")
	}

	// Default version to "v0.1"
	if req.Version == "" {
		req.Version = "v0.1"
	}

	// Default status to "Planned"
	if req.Status == "" {
		req.Status = "Planned"
	}

	// Validate status
	validStatuses := map[string]bool{
		"Planned":     true,
		"InProgress":  true,
		"Implemented": true,
		"Deprecated":  true,
	}
	if !validStatuses[req.Status] {
		return nil, fmt.Errorf("%w: %s (valid: Planned, InProgress, Implemented, Deprecated)", ErrInvalidStatus, req.Status)
	}

	// Default display name to name if not provided
	if req.DisplayName == "" {
		req.DisplayName = req.Name
	}

	var featureID int64

	// Use exclusive lock + transaction for atomic creation
	err := db.WithExclusiveLock(func() error {
		return db.WithTransaction(func(tx *Transaction) error {
			// Insert feature
			result, err := tx.Exec(`
				INSERT INTO features (
					name, display_name, version, status,
					description, spec_path, api_path
				) VALUES (?, ?, ?, ?, ?, ?, ?)
			`,
				req.Name, req.DisplayName, req.Version, req.Status,
				nullString(req.Description),
				nullString(req.SpecPath),
				nullString(req.APIPath),
			)
			if err != nil {
				if strings.Contains(err.Error(), "UNIQUE constraint failed") {
					return fmt.Errorf("feature with name '%s' already exists", req.Name)
				}
				return fmt.Errorf("failed to insert feature: %w", err)
			}

			// Get inserted ID
			id, err := result.LastInsertId()
			if err != nil {
				return fmt.Errorf("failed to get last insert ID: %w", err)
			}
			featureID = id

			// Insert audit log
			changes := fmt.Sprintf(`{"name":"%s","version":"%s","status":"%s"}`,
				req.Name, req.Version, req.Status)
			_, err = tx.Exec(`
				INSERT INTO audit_log (action, entity_type, entity_id, changes, agent)
				VALUES (?, ?, ?, ?, ?)
			`, "create", "feature", featureID, changes, "atlas-dev")
			if err != nil {
				return fmt.Errorf("failed to insert audit log: %w", err)
			}

			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	// Fetch and return created feature (AFTER transaction)
	feature, err := db.GetFeature(req.Name)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch created feature: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("feature created",
		"name", req.Name,
		"version", req.Version,
		"status", req.Status,
		"duration_ms", duration.Milliseconds(),
	)

	return feature, nil
}

// GetFeature retrieves a feature by name
func (db *DB) GetFeature(name string) (*Feature, error) {
	start := time.Now()

	query := `
		SELECT id, name, display_name, version, status,
		       description, implementation_notes, related_phases,
		       spec_path, api_path, created_at, updated_at
		FROM features
		WHERE name = ?
	`

	var f Feature
	err := db.conn.QueryRow(query, name).Scan(
		&f.ID, &f.Name, &f.DisplayName, &f.Version, &f.Status,
		&f.Description, &f.ImplementationNotes, &f.RelatedPhases,
		&f.SpecPath, &f.APIPath, &f.CreatedAt, &f.UpdatedAt,
	)

	if err != nil {
		if err == sql.ErrNoRows {
			return nil, ErrFeatureNotFound
		}
		return nil, fmt.Errorf("failed to query feature: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("feature retrieved",
		"name", name,
		"duration_ms", duration.Milliseconds(),
	)

	return &f, nil
}

// ListFeatures retrieves features with optional filters
func (db *DB) ListFeatures(category, status string) ([]*FeatureListItem, error) {
	start := time.Now()

	query := `
		SELECT name, display_name, version, status
		FROM features
		WHERE 1=1
	`
	args := []interface{}{}

	// Note: category filter would require joining with a category mapping
	// For now, we'll skip category filtering at DB level

	if status != "" {
		query += " AND status = ?"
		args = append(args, status)
	}

	query += " ORDER BY name"

	rows, err := db.conn.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query features: %w", err)
	}
	defer rows.Close()

	features := []*FeatureListItem{}
	for rows.Next() {
		var f FeatureListItem
		err := rows.Scan(&f.Name, &f.DisplayName, &f.Version, &f.Status)
		if err != nil {
			return nil, fmt.Errorf("failed to scan feature: %w", err)
		}
		features = append(features, &f)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("failed to iterate features: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("features listed",
		"count", len(features),
		"status_filter", status,
		"duration_ms", duration.Milliseconds(),
	)

	return features, nil
}

// UpdateFeature updates a feature record
func (db *DB) UpdateFeature(req UpdateFeatureRequest) (*Feature, error) {
	start := time.Now()

	// Validate status if provided
	if req.Status != "" {
		validStatuses := map[string]bool{
			"Planned":     true,
			"InProgress":  true,
			"Implemented": true,
			"Deprecated":  true,
		}
		if !validStatuses[req.Status] {
			return nil, fmt.Errorf("%w: %s", ErrInvalidStatus, req.Status)
		}
	}

	// Get current feature for audit log
	oldFeature, err := db.GetFeature(req.Name)
	if err != nil {
		return nil, err
	}

	// Build update query dynamically based on provided fields
	updates := []string{}
	args := []interface{}{}

	if req.Version != "" {
		updates = append(updates, "version = ?")
		args = append(args, req.Version)
	}
	if req.Status != "" {
		updates = append(updates, "status = ?")
		args = append(args, req.Status)
	}
	if req.Description != "" {
		updates = append(updates, "description = ?")
		args = append(args, req.Description)
	}
	if req.ImplementationNotes != "" {
		updates = append(updates, "implementation_notes = ?")
		args = append(args, req.ImplementationNotes)
	}
	if req.RelatedPhases != "" {
		updates = append(updates, "related_phases = ?")
		args = append(args, req.RelatedPhases)
	}
	if req.SpecPath != "" {
		updates = append(updates, "spec_path = ?")
		args = append(args, req.SpecPath)
	}
	if req.APIPath != "" {
		updates = append(updates, "api_path = ?")
		args = append(args, req.APIPath)
	}

	if len(updates) == 0 {
		return nil, fmt.Errorf("no fields to update")
	}

	// Add name to args for WHERE clause
	args = append(args, req.Name)

	// Use exclusive lock + transaction
	err = db.WithExclusiveLock(func() error {
		return db.WithTransaction(func(tx *Transaction) error {
			// Update feature
			query := fmt.Sprintf("UPDATE features SET %s WHERE name = ?", strings.Join(updates, ", "))
			_, err := tx.Exec(query, args...)
			if err != nil {
				return fmt.Errorf("failed to update feature: %w", err)
			}

			// Create audit log
			changes := fmt.Sprintf(`{"old_version":"%s","old_status":"%s"}`,
				oldFeature.Version, oldFeature.Status)
			_, err = tx.Exec(`
				INSERT INTO audit_log (action, entity_type, entity_id, changes, agent)
				VALUES (?, ?, ?, ?, ?)
			`, "update", "feature", oldFeature.ID, changes, "atlas-dev")
			if err != nil {
				return fmt.Errorf("failed to insert audit log: %w", err)
			}

			return nil
		})
	})

	if err != nil {
		return nil, err
	}

	// Fetch and return updated feature (AFTER transaction)
	feature, err := db.GetFeature(req.Name)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch updated feature: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("feature updated",
		"name", req.Name,
		"duration_ms", duration.Milliseconds(),
	)

	return feature, nil
}

// DeleteFeature deletes a feature record
func (db *DB) DeleteFeature(name string) error {
	start := time.Now()

	// Get feature for audit log
	feature, err := db.GetFeature(name)
	if err != nil {
		return err
	}

	// Use exclusive lock + transaction
	err = db.WithExclusiveLock(func() error {
		return db.WithTransaction(func(tx *Transaction) error {
			// Delete feature
			result, err := tx.Exec("DELETE FROM features WHERE name = ?", name)
			if err != nil {
				return fmt.Errorf("failed to delete feature: %w", err)
			}

			rows, _ := result.RowsAffected()
			if rows == 0 {
				return ErrFeatureNotFound
			}

			// Insert audit log
			changes := fmt.Sprintf(`{"name":"%s","version":"%s"}`,
				feature.Name, feature.Version)
			_, err = tx.Exec(`
				INSERT INTO audit_log (action, entity_type, entity_id, changes, agent)
				VALUES (?, ?, ?, ?, ?)
			`, "delete", "feature", feature.ID, changes, "atlas-dev")
			if err != nil {
				return fmt.Errorf("failed to insert audit log: %w", err)
			}

			return nil
		})
	})

	if err != nil {
		return err
	}

	duration := time.Since(start)
	slog.Debug("feature deleted",
		"name", name,
		"duration_ms", duration.Milliseconds(),
	)

	return nil
}

// ToCompactJSON converts Feature to compact JSON representation
func (f *Feature) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"name":    f.Name,
		"display": f.DisplayName,
		"ver":     f.Version,
		"stat":    f.Status,
	}

	if f.Description.Valid && f.Description.String != "" {
		result["desc"] = f.Description.String
	}
	if f.ImplementationNotes.Valid && f.ImplementationNotes.String != "" {
		result["notes"] = f.ImplementationNotes.String
	}
	if f.SpecPath.Valid && f.SpecPath.String != "" {
		result["spec"] = f.SpecPath.String
	}
	if f.APIPath.Valid && f.APIPath.String != "" {
		result["api"] = f.APIPath.String
	}
	if f.RelatedPhases.Valid && f.RelatedPhases.String != "" {
		result["phases"] = f.RelatedPhases.String
	}

	return result
}

// SearchFeatures searches features by query
func (db *DB) SearchFeatures(query string) ([]*FeatureListItem, error) {
	start := time.Now()

	searchQuery := `
		SELECT name, display_name, version, status
		FROM features
		WHERE name LIKE ? OR display_name LIKE ? OR description LIKE ?
		ORDER BY
			CASE
				WHEN name LIKE ? THEN 1
				WHEN display_name LIKE ? THEN 2
				ELSE 3
			END,
			name
	`

	pattern := "%" + query + "%"
	exactPattern := query + "%"

	rows, err := db.conn.Query(searchQuery,
		pattern, pattern, pattern, // WHERE clauses
		exactPattern, exactPattern, // ORDER BY clauses
	)
	if err != nil {
		return nil, fmt.Errorf("failed to search features: %w", err)
	}
	defer rows.Close()

	features := []*FeatureListItem{}
	for rows.Next() {
		var f FeatureListItem
		err := rows.Scan(&f.Name, &f.DisplayName, &f.Version, &f.Status)
		if err != nil {
			return nil, fmt.Errorf("failed to scan feature: %w", err)
		}
		features = append(features, &f)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("failed to iterate search results: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("features searched",
		"query", query,
		"count", len(features),
		"duration_ms", duration.Milliseconds(),
	)

	return features, nil
}
