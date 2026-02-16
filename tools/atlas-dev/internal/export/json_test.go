package export

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestJSONExport(t *testing.T) {
	database := newTestDB(t)
	exporter := NewJSONExporter(database)

	outputPath := filepath.Join(t.TempDir(), "export.json")

	result, err := exporter.Export(outputPath)
	if err != nil {
		t.Fatalf("Export() error = %v", err)
	}

	// Verify file created
	if _, err := os.Stat(result.FilePath); os.IsNotExist(err) {
		t.Error("export file not created")
	}

	// Verify result
	if result.SizeBytes == 0 {
		t.Error("export file is empty")
	}

	if result.Tables == 0 {
		t.Error("no tables exported")
	}
}

func TestJSONExportValid(t *testing.T) {
	database := newTestDB(t)
	exporter := NewJSONExporter(database)

	outputPath := filepath.Join(t.TempDir(), "export.json")

	_, err := exporter.Export(outputPath)
	if err != nil {
		t.Fatalf("Export() error = %v", err)
	}

	// Read and parse JSON
	data, err := os.ReadFile(outputPath)
	if err != nil {
		t.Fatalf("failed to read export: %v", err)
	}

	var parsed map[string]interface{}
	if err := json.Unmarshal(data, &parsed); err != nil {
		t.Fatalf("invalid JSON: %v", err)
	}

	// Verify structure
	if _, ok := parsed["exported_at"]; !ok {
		t.Error("missing exported_at field")
	}

	if _, ok := parsed["tables"]; !ok {
		t.Error("missing tables field")
	}
}

func TestGenerateBackupFilename(t *testing.T) {
	filename := GenerateBackupFilename("test")

	if filename == "" {
		t.Error("filename is empty")
	}

	if !contains(filename, "test-") {
		t.Error("filename missing prefix")
	}

	if !contains(filename, ".json") {
		t.Error("filename missing .json extension")
	}
}

func TestValidateJSON(t *testing.T) {
	database := newTestDB(t)
	exporter := NewJSONExporter(database)

	outputPath := filepath.Join(t.TempDir(), "export.json")
	_, err := exporter.Export(outputPath)
	if err != nil {
		t.Fatalf("Export() error = %v", err)
	}

	// Validate should pass
	if err := ValidateJSON(outputPath); err != nil {
		t.Errorf("ValidateJSON() error = %v", err)
	}
}

func TestValidateJSON_Invalid(t *testing.T) {
	invalidPath := filepath.Join(t.TempDir(), "invalid.json")
	_ = os.WriteFile(invalidPath, []byte("not json"), 0644)

	err := ValidateJSON(invalidPath)
	if err == nil {
		t.Error("expected error for invalid JSON")
	}
}

func contains(s, substr string) bool {
	return len(s) > 0 && len(substr) > 0 && (s == substr || len(s) > len(substr) &&
		(s[:len(substr)] == substr || s[len(s)-len(substr):] == substr ||
			len(s) > len(substr)*2 && containsMiddle(s, substr)))
}

func containsMiddle(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
