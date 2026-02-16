package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"time"
)

// Component represents a decision component/category
type Component struct {
	ID          int
	Name        string
	DisplayName string
	Description sql.NullString
	CreatedAt   string
}

// CreateComponentRequest represents parameters for creating a component
type CreateComponentRequest struct {
	Name        string
	DisplayName string
	Description string
}

// CreateComponent creates a new component
func (db *DB) CreateComponent(req CreateComponentRequest) (*Component, error) {
	start := time.Now()

	result, err := db.Exec(`
		INSERT INTO components (name, display_name, description)
		VALUES (?, ?, ?)
	`, req.Name, req.DisplayName, nullString(req.Description))

	if err != nil {
		return nil, fmt.Errorf("failed to create component: %w", err)
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, fmt.Errorf("failed to get component ID: %w", err)
	}

	duration := time.Since(start)
	slog.Debug("component created",
		"name", req.Name,
		"id", id,
		"duration_ms", duration.Milliseconds(),
	)

	return db.GetComponent(req.Name)
}

// GetComponent retrieves a component by name
func (db *DB) GetComponent(name string) (*Component, error) {
	start := time.Now()

	var c Component
	err := db.conn.QueryRow(`
		SELECT id, name, display_name, description, created_at
		FROM components
		WHERE name = ?
	`, name).Scan(&c.ID, &c.Name, &c.DisplayName, &c.Description, &c.CreatedAt)

	duration := time.Since(start)
	slog.Debug("component retrieved",
		"name", name,
		"duration_ms", duration.Milliseconds(),
	)

	if err == sql.ErrNoRows {
		return nil, fmt.Errorf("component not found: %s", name)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to query component: %w", err)
	}

	return &c, nil
}

// ListComponents returns all components
func (db *DB) ListComponents() ([]*Component, error) {
	start := time.Now()

	rows, err := db.conn.Query(`
		SELECT id, name, display_name, description, created_at
		FROM components
		ORDER BY name
	`)
	if err != nil {
		return nil, fmt.Errorf("failed to query components: %w", err)
	}
	defer rows.Close()

	components := []*Component{}
	for rows.Next() {
		var c Component
		if err := rows.Scan(&c.ID, &c.Name, &c.DisplayName, &c.Description, &c.CreatedAt); err != nil {
			return nil, fmt.Errorf("failed to scan component: %w", err)
		}
		components = append(components, &c)
	}

	duration := time.Since(start)
	slog.Debug("components listed",
		"count", len(components),
		"duration_ms", duration.Milliseconds(),
	)

	return components, rows.Err()
}

// ComponentExists checks if a component exists
func (db *DB) ComponentExists(name string) (bool, error) {
	var exists int
	err := db.conn.QueryRow(`
		SELECT 1 FROM components WHERE name = ?
	`, name).Scan(&exists)

	if err == sql.ErrNoRows {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	return true, nil
}
