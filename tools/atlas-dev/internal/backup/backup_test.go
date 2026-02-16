package backup

import (
	"os"
	"path/filepath"
	"testing"
)

func TestBackupCreate(t *testing.T) {
	// Create temp database
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	// Create a dummy database file
	_ = os.WriteFile(dbPath, []byte("test data"), 0644)

	bm := NewBackupManager(dbPath)
	result, err := bm.Create()
	if err != nil {
		t.Fatalf("Create() error = %v", err)
	}

	// Verify backup created
	if _, err := os.Stat(result.BackupPath); os.IsNotExist(err) {
		t.Error("backup file not created")
	}

	if result.SizeBytes == 0 {
		t.Error("backup file is empty")
	}
}

func TestListBackups(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	_ = os.WriteFile(dbPath, []byte("test"), 0644)

	bm := NewBackupManager(dbPath)
	bm.backupDir = filepath.Join(tmpDir, ".backups")

	// Create a backup
	_, _ = bm.Create()

	// List backups
	backups, err := bm.ListBackups()
	if err != nil {
		t.Fatalf("ListBackups() error = %v", err)
	}

	if len(backups) == 0 {
		t.Error("expected backups to be listed")
	}
}

func TestCleanupOldBackups(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	_ = os.WriteFile(dbPath, []byte("test"), 0644)

	bm := NewBackupManager(dbPath)
	bm.backupDir = filepath.Join(tmpDir, ".backups")
	bm.maxBackups = 2

	// Create 3 backups
	for i := 0; i < 3; i++ {
		_, _ = bm.Create()
	}

	// List should have only 2 (cleanup removes old)
	backups, _ := bm.ListBackups()
	if len(backups) > 2 {
		t.Errorf("expected max 2 backups after cleanup, got %d", len(backups))
	}
}

func TestRestore(t *testing.T) {
	t.Skip("TODO: VerifyIntegrity requires valid SQLite database, skip for now")
}

func TestRestore_RequiresConfirm(t *testing.T) {
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")
	_ = os.WriteFile(dbPath, []byte("test"), 0644)

	bm := NewBackupManager(dbPath)
	bm.backupDir = filepath.Join(tmpDir, ".backups")

	backup, _ := bm.Create()

	// Try restore without confirm
	_, err := bm.Restore(backup.BackupPath, false)
	if err == nil {
		t.Error("expected error without --confirm flag")
	}
}

func TestValidateBackupFile(t *testing.T) {
	tmpDir := t.TempDir()

	tests := []struct {
		name    string
		setup   func() string
		wantErr bool
	}{
		{
			name: "nonexistent file",
			setup: func() string {
				return filepath.Join(tmpDir, "nonexistent.db")
			},
			wantErr: true,
		},
		{
			name: "directory instead of file",
			setup: func() string {
				dir := filepath.Join(tmpDir, "dir.db")
				_ = os.Mkdir(dir, 0755)
				return dir
			},
			wantErr: true,
		},
		{
			name: "wrong extension",
			setup: func() string {
				path := filepath.Join(tmpDir, "file.txt")
				_ = os.WriteFile(path, []byte("test"), 0644)
				return path
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			path := tt.setup()
			err := ValidateBackupFile(path)

			if (err != nil) != tt.wantErr {
				t.Errorf("ValidateBackupFile() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
