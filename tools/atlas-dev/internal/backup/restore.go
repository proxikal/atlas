package backup

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// RestoreResult represents the result of a restore operation
type RestoreResult struct {
	RestoredFrom  string `json:"restored_from"`
	SafetyBackup  string `json:"safety_backup"`
	Timestamp     string `json:"timestamp"`
	SizeBytes     int64  `json:"size_bytes"`
	IntegrityOK   bool   `json:"integrity_ok"`
}

// Restore restores the database from a backup file
func (bm *BackupManager) Restore(backupPath string, confirm bool) (*RestoreResult, error) {
	// Require confirmation flag
	if !confirm {
		return nil, fmt.Errorf("restore requires --confirm flag to prevent accidental data loss")
	}

	// Verify backup file exists
	if _, err := os.Stat(backupPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("backup file not found: %s", backupPath)
	}

	// Verify backup integrity
	if err := bm.VerifyIntegrity(backupPath); err != nil {
		return nil, fmt.Errorf("backup integrity check failed: %w", err)
	}

	// Create safety backup of current database before restore
	safetyBackup, err := bm.createSafetyBackup()
	if err != nil {
		return nil, fmt.Errorf("failed to create safety backup: %w", err)
	}

	// Get backup file size
	backupInfo, err := os.Stat(backupPath)
	if err != nil {
		return nil, fmt.Errorf("failed to stat backup file: %w", err)
	}

	// Perform restore (copy backup over current database)
	if err := bm.performRestore(backupPath); err != nil {
		// Try to restore from safety backup if restore fails
		if restoreErr := bm.performRestore(safetyBackup); restoreErr != nil {
			return nil, fmt.Errorf("restore failed and safety restore also failed: %w (original error: %v)", restoreErr, err)
		}
		return nil, fmt.Errorf("restore failed, database restored from safety backup: %w", err)
	}

	// Verify restored database integrity
	if err := bm.VerifyIntegrity(bm.dbPath); err != nil {
		// Restore from safety backup
		_ = bm.performRestore(safetyBackup)
		return nil, fmt.Errorf("restored database failed integrity check, rolled back to safety backup: %w", err)
	}

	return &RestoreResult{
		RestoredFrom:  backupPath,
		SafetyBackup:  safetyBackup,
		Timestamp:     time.Now().Format(time.RFC3339),
		SizeBytes:     backupInfo.Size(),
		IntegrityOK:   true,
	}, nil
}

// createSafetyBackup creates a backup before restore
func (bm *BackupManager) createSafetyBackup() (string, error) {
	timestamp := time.Now().Format("20060102-150405")
	filename := fmt.Sprintf("atlas-dev-safety-%s.db", timestamp)
	safetyPath := filepath.Join(bm.backupDir, filename)

	// Create backup directory if not exists
	if err := os.MkdirAll(bm.backupDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create backup directory: %w", err)
	}

	// Copy current database to safety backup
	if err := bm.fileCopy(bm.dbPath, safetyPath); err != nil {
		return "", fmt.Errorf("failed to create safety backup: %w", err)
	}

	return safetyPath, nil
}

// performRestore performs the actual restore operation
func (bm *BackupManager) performRestore(backupPath string) error {
	// Copy backup file to database location
	if err := bm.fileCopy(backupPath, bm.dbPath); err != nil {
		return fmt.Errorf("failed to copy backup to database location: %w", err)
	}

	return nil
}

// RestoreFromLatest restores from the most recent backup
func (bm *BackupManager) RestoreFromLatest(confirm bool) (*RestoreResult, error) {
	backups, err := bm.ListBackups()
	if err != nil {
		return nil, fmt.Errorf("failed to list backups: %w", err)
	}

	if len(backups) == 0 {
		return nil, fmt.Errorf("no backups found")
	}

	// Use most recent backup (backups are sorted newest first)
	return bm.Restore(backups[0].Path, confirm)
}

// ValidateBackupFile checks if a file is a valid SQLite database backup
func ValidateBackupFile(path string) error {
	// Check file exists
	info, err := os.Stat(path)
	if err != nil {
		return fmt.Errorf("backup file not found: %w", err)
	}

	// Check it's a file (not directory)
	if info.IsDir() {
		return fmt.Errorf("backup path is a directory, not a file")
	}

	// Check file extension
	if !strings.HasSuffix(path, ".db") {
		return fmt.Errorf("backup file must have .db extension")
	}

	// Try to open as SQLite database
	db, err := sql.Open("sqlite3", path)
	if err != nil {
		return fmt.Errorf("failed to open as SQLite database: %w", err)
	}
	defer db.Close()

	// Verify it's a valid database
	var result string
	if err := db.QueryRow("SELECT 1").Scan(&result); err != nil {
		return fmt.Errorf("invalid SQLite database: %w", err)
	}

	return nil
}
