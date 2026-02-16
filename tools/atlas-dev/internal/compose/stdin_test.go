package compose

import (
	"strings"
	"testing"
)

func TestParseJSONFromStdin_Object(t *testing.T) {
	data := []byte(`{"id":"DR-001","title":"Test"}`)

	input, err := ParseJSONFromStdin(data)
	if err != nil {
		t.Fatalf("ParseJSONFromStdin failed: %v", err)
	}

	if input.IsArray {
		t.Error("Expected IsArray=false for single object")
	}

	if len(input.Items) != 1 {
		t.Errorf("Expected 1 item, got %d", len(input.Items))
	}

	if id, ok := input.Items[0]["id"].(string); !ok || id != "DR-001" {
		t.Errorf("Expected id='DR-001', got %v", input.Items[0]["id"])
	}
}

func TestParseJSONFromStdin_Array(t *testing.T) {
	data := []byte(`[{"id":"DR-001"},{"id":"DR-002"}]`)

	input, err := ParseJSONFromStdin(data)
	if err != nil {
		t.Fatalf("ParseJSONFromStdin failed: %v", err)
	}

	if !input.IsArray {
		t.Error("Expected IsArray=true for array")
	}

	if len(input.Items) != 2 {
		t.Errorf("Expected 2 items, got %d", len(input.Items))
	}
}

func TestParseJSONFromStdin_StringArray(t *testing.T) {
	data := []byte(`["DR-001","DR-002","DR-003"]`)

	input, err := ParseJSONFromStdin(data)
	if err != nil {
		t.Fatalf("ParseJSONFromStdin failed: %v", err)
	}

	if !input.IsArray {
		t.Error("Expected IsArray=true")
	}

	if len(input.Items) != 3 {
		t.Errorf("Expected 3 items, got %d", len(input.Items))
	}

	// String arrays are converted to objects with "id" field
	if id, ok := input.Items[0]["id"].(string); !ok || id != "DR-001" {
		t.Errorf("Expected id='DR-001', got %v", input.Items[0]["id"])
	}
}

func TestParseJSONFromStdin_Invalid(t *testing.T) {
	data := []byte(`not valid json`)

	_, err := ParseJSONFromStdin(data)
	if err == nil {
		t.Error("Expected error for invalid JSON")
	}
}

func TestExtractIDs(t *testing.T) {
	input := &StdinInput{
		Items: []map[string]interface{}{
			{"id": "DR-001"},
			{"id": "DR-002"},
			{"phase_id": "phase-03"},
		},
	}

	ids := ExtractIDs(input)

	if len(ids) != 3 {
		t.Errorf("Expected 3 IDs, got %d", len(ids))
	}

	expected := []string{"DR-001", "DR-002", "phase-03"}
	for i, exp := range expected {
		if ids[i] != exp {
			t.Errorf("Expected id[%d]='%s', got '%s'", i, exp, ids[i])
		}
	}
}

func TestExtractPaths(t *testing.T) {
	input := &StdinInput{
		Items: []map[string]interface{}{
			{"path": "phases/test/phase-01.md"},
			{"file_path": "docs/spec.md"},
			{"phase_path": "phases/stdlib/phase-02.md"},
		},
	}

	paths := ExtractPaths(input)

	if len(paths) != 3 {
		t.Errorf("Expected 3 paths, got %d", len(paths))
	}
}

func TestExtractFirstID(t *testing.T) {
	input := &StdinInput{
		Items: []map[string]interface{}{
			{"id": "DR-001"},
			{"id": "DR-002"},
		},
	}

	id, err := ExtractFirstID(input)
	if err != nil {
		t.Fatalf("ExtractFirstID failed: %v", err)
	}

	if id != "DR-001" {
		t.Errorf("Expected 'DR-001', got '%s'", id)
	}
}

func TestExtractFirstID_NoID(t *testing.T) {
	input := &StdinInput{
		Items: []map[string]interface{}{
			{"name": "test"},
		},
	}

	_, err := ExtractFirstID(input)
	if err == nil {
		t.Error("Expected error when no ID found")
	}
}

func TestExtractField(t *testing.T) {
	input := &StdinInput{
		Items: []map[string]interface{}{
			{"name": "First"},
			{"name": "Second"},
			{"name": "Third"},
		},
	}

	names := ExtractField(input, "name")

	if len(names) != 3 {
		t.Errorf("Expected 3 names, got %d", len(names))
	}

	if names[0] != "First" {
		t.Errorf("Expected 'First', got '%s'", names[0])
	}
}

func TestFormatAsLines(t *testing.T) {
	values := []string{"DR-001", "DR-002", "DR-003"}

	result := FormatAsLines(values)

	expected := "DR-001\nDR-002\nDR-003"
	if result != expected {
		t.Errorf("Expected '%s', got '%s'", expected, result)
	}
}

func TestFormatAsJSON(t *testing.T) {
	values := []string{"DR-001", "DR-002"}

	result, err := FormatAsJSON(values)
	if err != nil {
		t.Fatalf("FormatAsJSON failed: %v", err)
	}

	expected := `["DR-001","DR-002"]`
	if result != expected {
		t.Errorf("Expected '%s', got '%s'", expected, result)
	}
}

func TestExtractIDs_VariousFields(t *testing.T) {
	tests := []struct {
		name     string
		item     map[string]interface{}
		expected string
	}{
		{"id field", map[string]interface{}{"id": "test-id"}, "test-id"},
		{"ID field", map[string]interface{}{"ID": "TEST-ID"}, "TEST-ID"},
		{"phase_id field", map[string]interface{}{"phase_id": "phase-01"}, "phase-01"},
		{"decision_id field", map[string]interface{}{"decision_id": "DR-001"}, "DR-001"},
		{"feature_id field", map[string]interface{}{"feature_id": "FT-001"}, "FT-001"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := &StdinInput{
				Items: []map[string]interface{}{tt.item},
			}

			ids := ExtractIDs(input)

			if len(ids) != 1 {
				t.Fatalf("Expected 1 ID, got %d", len(ids))
			}

			if ids[0] != tt.expected {
				t.Errorf("Expected '%s', got '%s'", tt.expected, ids[0])
			}
		})
	}
}

func TestHasStdin(t *testing.T) {
	// This test would require mocking stdin
	// For now, just verify it doesn't panic
	_ = HasStdin()
}

func TestParseJSONFromStdin_Empty(t *testing.T) {
	data := []byte(``)

	_, err := ParseJSONFromStdin(data)
	if err == nil {
		t.Error("Expected error for empty JSON")
	}
}

func TestParseJSONFromStdin_Whitespace(t *testing.T) {
	data := []byte(`   {"id":"test"}   `)

	input, err := ParseJSONFromStdin(data)
	if err != nil {
		t.Fatalf("ParseJSONFromStdin failed: %v", err)
	}

	if len(input.Items) != 1 {
		t.Errorf("Expected 1 item, got %d", len(input.Items))
	}
}

func TestExtractPaths_VariousFields(t *testing.T) {
	tests := []struct {
		name     string
		item     map[string]interface{}
		expected string
	}{
		{"path field", map[string]interface{}{"path": "test/path.md"}, "test/path.md"},
		{"file_path field", map[string]interface{}{"file_path": "file.md"}, "file.md"},
		{"phase_path field", map[string]interface{}{"phase_path": "phases/p1.md"}, "phases/p1.md"},
		{"spec_path field", map[string]interface{}{"spec_path": "docs/spec.md"}, "docs/spec.md"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input := &StdinInput{
				Items: []map[string]interface{}{tt.item},
			}

			paths := ExtractPaths(input)

			if len(paths) != 1 {
				t.Fatalf("Expected 1 path, got %d", len(paths))
			}

			if paths[0] != tt.expected {
				t.Errorf("Expected '%s', got '%s'", tt.expected, paths[0])
			}
		})
	}
}

func TestFormatAsLines_Empty(t *testing.T) {
	values := []string{}
	result := FormatAsLines(values)

	if result != "" {
		t.Errorf("Expected empty string, got '%s'", result)
	}
}

func TestFormatAsLines_Single(t *testing.T) {
	values := []string{"single"}
	result := FormatAsLines(values)

	if result != "single" {
		t.Errorf("Expected 'single', got '%s'", result)
	}
}

func TestFormatAsJSON_Empty(t *testing.T) {
	values := []string{}
	result, err := FormatAsJSON(values)

	if err != nil {
		t.Fatalf("FormatAsJSON failed: %v", err)
	}

	if !strings.Contains(result, "[]") {
		t.Errorf("Expected empty array, got '%s'", result)
	}
}
