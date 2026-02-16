package db

import (
	"database/sql"
	"fmt"
)

// IsMigrated checks if the database has already been migrated from markdown files
func (db *DB) IsMigrated() (bool, error) {
	var value string
	err := db.conn.QueryRow(`
		SELECT value FROM metadata
		WHERE key = 'migrated'
	`).Scan(&value)

	if err == sql.ErrNoRows {
		return false, nil
	}
	if err != nil {
		return false, fmt.Errorf("failed to check migration status: %w", err)
	}

	return value == "true", nil
}

// MarkAsMigrated marks the database as migrated
func (db *DB) MarkAsMigrated() error {
	_, err := db.conn.Exec(`
		INSERT INTO metadata (key, value, updated_at)
		VALUES ('migrated', 'true', datetime('now'))
		ON CONFLICT(key) DO UPDATE SET
			value = 'true',
			updated_at = datetime('now')
	`)
	if err != nil {
		return fmt.Errorf("failed to mark as migrated: %w", err)
	}

	return nil
}

// UnmarkMigration removes the migration marker (DANGEROUS - only for testing/force re-migration)
func (db *DB) UnmarkMigration() error {
	_, err := db.conn.Exec(`
		DELETE FROM metadata WHERE key = 'migrated'
	`)
	if err != nil {
		return fmt.Errorf("failed to unmark migration: %w", err)
	}

	return nil
}
