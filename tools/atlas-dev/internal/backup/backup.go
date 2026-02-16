package backup

import (
	"database/sql"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// BackupManager handles database backups
type BackupManager struct {
	dbPath        string
	backupDir     string
	retentionDays int
	maxBackups    int
}

// NewBackupManager creates a new backup manager
func NewBackupManager(dbPath string) *BackupManager {
	return &BackupManager{
		dbPath:        dbPath,
		backupDir:     ".backups",
		retentionDays: 30,
		maxBackups:    10,
	}
}

// BackupResult represents the result of a backup operation
type BackupResult struct {
	BackupPath string `json:"backup_path"`
	SizeBytes  int64  `json:"size_bytes"`
	Timestamp  string `json:"timestamp"`
}

// BackupInfo represents information about a backup file
type BackupInfo struct {
	Path      string
	SizeBytes int64
	Timestamp time.Time
}

// Create creates a new timestamped backup of the database
func (bm *BackupManager) Create() (*BackupResult, error) {
	// Create backup directory if not exists
	if err := os.MkdirAll(bm.backupDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create backup directory: %w", err)
	}

	// Generate backup filename with timestamp
	timestamp := time.Now().Format("20060102-150405")
	filename := fmt.Sprintf("atlas-dev-%s.db", timestamp)
	backupPath := filepath.Join(bm.backupDir, filename)

	// Copy database file to backup location
	if err := bm.copyDatabase(bm.dbPath, backupPath); err != nil {
		return nil, fmt.Errorf("failed to copy database: %w", err)
	}

	// Get backup file info
	info, err := os.Stat(backupPath)
	if err != nil {
		return nil, fmt.Errorf("failed to get backup info: %w", err)
	}

	// Cleanup old backups
	if err := bm.CleanupOldBackups(); err != nil {
		// Log warning but don't fail the backup operation
		fmt.Fprintf(os.Stderr, "Warning: failed to cleanup old backups: %v\n", err)
	}

	return &BackupResult{
		BackupPath: backupPath,
		SizeBytes:  info.Size(),
		Timestamp:  timestamp,
	}, nil
}

// copyDatabase copies a SQLite database file safely
func (bm *BackupManager) copyDatabase(src, dst string) error {
	// Open source database
	srcDB, err := sql.Open("sqlite3", src)
	if err != nil {
		return fmt.Errorf("failed to open source database: %w", err)
	}
	defer srcDB.Close()

	// Use SQLite backup API for safe copying
	dstDB, err := sql.Open("sqlite3", dst)
	if err != nil {
		return fmt.Errorf("failed to open destination database: %w", err)
	}
	defer dstDB.Close()

	// Perform backup using VACUUM INTO (SQLite 3.27+)
	// This is safer than file copy as it handles locks properly
	query := fmt.Sprintf("VACUUM INTO '%s'", dst)
	if _, err := srcDB.Exec(query); err != nil {
		// Fallback to file copy if VACUUM INTO not supported
		return bm.fileCopy(src, dst)
	}

	return nil
}

// fileCopy performs a simple file copy
func (bm *BackupManager) fileCopy(src, dst string) error {
	srcFile, err := os.Open(src)
	if err != nil {
		return fmt.Errorf("failed to open source: %w", err)
	}
	defer srcFile.Close()

	dstFile, err := os.Create(dst)
	if err != nil {
		return fmt.Errorf("failed to create destination: %w", err)
	}
	defer dstFile.Close()

	if _, err := io.Copy(dstFile, srcFile); err != nil {
		return fmt.Errorf("failed to copy: %w", err)
	}

	return dstFile.Sync()
}

// ListBackups returns a list of all backup files
func (bm *BackupManager) ListBackups() ([]*BackupInfo, error) {
	if _, err := os.Stat(bm.backupDir); os.IsNotExist(err) {
		return []*BackupInfo{}, nil
	}

	entries, err := os.ReadDir(bm.backupDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read backup directory: %w", err)
	}

	var backups []*BackupInfo
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}

		// Only include .db files with timestamp pattern
		if !strings.HasSuffix(entry.Name(), ".db") {
			continue
		}

		path := filepath.Join(bm.backupDir, entry.Name())
		info, err := os.Stat(path)
		if err != nil {
			continue
		}

		backups = append(backups, &BackupInfo{
			Path:      path,
			SizeBytes: info.Size(),
			Timestamp: info.ModTime(),
		})
	}

	// Sort by timestamp (newest first)
	sort.Slice(backups, func(i, j int) bool {
		return backups[i].Timestamp.After(backups[j].Timestamp)
	})

	return backups, nil
}

// CleanupOldBackups removes old backups beyond retention limit
func (bm *BackupManager) CleanupOldBackups() error {
	backups, err := bm.ListBackups()
	if err != nil {
		return err
	}

	// Keep only maxBackups most recent
	if len(backups) <= bm.maxBackups {
		return nil
	}

	// Delete old backups
	for i := bm.maxBackups; i < len(backups); i++ {
		if err := os.Remove(backups[i].Path); err != nil {
			return fmt.Errorf("failed to remove old backup %s: %w", backups[i].Path, err)
		}
	}

	return nil
}

// GetBackupInfo retrieves information about a specific backup
func (bm *BackupManager) GetBackupInfo(backupPath string) (*BackupInfo, error) {
	info, err := os.Stat(backupPath)
	if err != nil {
		return nil, fmt.Errorf("failed to stat backup: %w", err)
	}

	return &BackupInfo{
		Path:      backupPath,
		SizeBytes: info.Size(),
		Timestamp: info.ModTime(),
	}, nil
}

// VerifyIntegrity checks if a backup is a valid SQLite database
func (bm *BackupManager) VerifyIntegrity(backupPath string) error {
	db, err := sql.Open("sqlite3", backupPath)
	if err != nil {
		return fmt.Errorf("failed to open backup: %w", err)
	}
	defer db.Close()

	// Run integrity check
	var result string
	err = db.QueryRow("PRAGMA integrity_check").Scan(&result)
	if err != nil {
		return fmt.Errorf("integrity check failed: %w", err)
	}

	if result != "ok" {
		return fmt.Errorf("database integrity check failed: %s", result)
	}

	return nil
}

// SetRetention sets the backup retention policy
func (bm *BackupManager) SetRetention(days, maxBackups int) {
	bm.retentionDays = days
	bm.maxBackups = maxBackups
}
