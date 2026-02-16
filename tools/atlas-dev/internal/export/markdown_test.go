package export

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

func newTestDB(t *testing.T) *db.DB {
	t.Helper()

	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	database, err := db.New(dbPath)
	if err != nil {
		t.Fatalf("failed to create test db: %v", err)
	}

	if err := database.InitSchema(); err != nil {
		t.Fatalf("failed to init schema: %v", err)
	}

	if err := database.Prepare(); err != nil {
		t.Fatalf("failed to prepare statements: %v", err)
	}

	t.Cleanup(func() {
		_ = database.Close()
	})

	return database
}

func TestMarkdownExport(t *testing.T) {
	database := newTestDB(t)
	exporter := NewMarkdownExporter(database)

	outputDir := t.TempDir()

	result, err := exporter.Export(outputDir)
	if err != nil {
		t.Fatalf("Export() error = %v", err)
	}

	// Verify files created
	if result.FileCount == 0 {
		t.Error("expected files to be created")
	}

	// Verify STATUS.md exists
	statusPath := filepath.Join(outputDir, "STATUS.md")
	if _, err := os.Stat(statusPath); os.IsNotExist(err) {
		t.Error("STATUS.md not created")
	}

	// Verify trackers directory exists
	trackersDir := filepath.Join(outputDir, "trackers")
	if _, err := os.Stat(trackersDir); os.IsNotExist(err) {
		t.Error("trackers directory not created")
	}
}

func TestGenerateStatusMD(t *testing.T) {
	database := newTestDB(t)
	exporter := NewMarkdownExporter(database)

	outputPath := filepath.Join(t.TempDir(), "STATUS.md")

	err := exporter.generateStatusMD(outputPath)
	if err != nil {
		t.Fatalf("generateStatusMD() error = %v", err)
	}

	// Read generated file
	content, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatalf("failed to read STATUS.md: %v", err)
	}

	contentStr := string(content)

	// Verify content
	if !strings.Contains(contentStr, "# Atlas Development Status") {
		t.Error("STATUS.md missing title")
	}

	if !strings.Contains(contentStr, "Overall Progress") {
		t.Error("STATUS.md missing progress section")
	}

	if !strings.Contains(contentStr, "Category Breakdown") {
		t.Error("STATUS.md missing category breakdown")
	}
}

func TestGenerateTrackerFiles(t *testing.T) {
	database := newTestDB(t)

	// Insert a test phase
	_, _ = database.InsertPhase("phases/test.md", "test-phase", "foundation")

	exporter := NewMarkdownExporter(database)
	trackersDir := t.TempDir()

	files, err := exporter.generateTrackerFiles(trackersDir)
	if err != nil {
		t.Fatalf("generateTrackerFiles() error = %v", err)
	}

	// Should have created tracker files for all categories
	if len(files) == 0 {
		t.Error("expected tracker files to be created")
	}

	// Verify at least one file exists and has content
	for _, file := range files {
		if _, err := os.Stat(file); os.IsNotExist(err) {
			t.Errorf("tracker file not created: %s", file)
		}

		content, err := os.ReadFile(file)
		if err != nil {
			continue
		}

		if len(content) == 0 {
			t.Errorf("tracker file is empty: %s", file)
		}
	}
}

func TestExportWithPhases(t *testing.T) {
	database := newTestDB(t)

	// Insert test data
	_, _ = database.InsertPhase("phases/phase-01.md", "phase-01", "foundation")
	_, _ = database.InsertPhase("phases/phase-02.md", "phase-02", "stdlib")

	exporter := NewMarkdownExporter(database)
	outputDir := t.TempDir()

	result, err := exporter.Export(outputDir)
	if err != nil {
		t.Fatalf("Export() error = %v", err)
	}

	// Verify result
	if result.FileCount < 2 {
		t.Errorf("expected at least 2 files (STATUS.md + trackers), got %d", result.FileCount)
	}

	if result.OutputDir != outputDir {
		t.Errorf("output_dir = %s, want %s", result.OutputDir, outputDir)
	}
}
