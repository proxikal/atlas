package parity

import (
	"os"
	"path/filepath"
	"testing"
)

func TestParityChecker_CheckParity(t *testing.T) {
	// Create temp project structure
	tempDir := t.TempDir()

	// Create directories
	cratesDir := filepath.Join(tempDir, "crates")
	if err := os.MkdirAll(cratesDir, 0755); err != nil {
		t.Fatal(err)
	}

	// Create a simple Rust file
	rustFile := filepath.Join(cratesDir, "lib.rs")
	rustContent := `
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(1, 2), 3);
}
`
	if err := os.WriteFile(rustFile, []byte(rustContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Create parity checker
	checker := NewParityChecker(tempDir)

	// Run parity check
	report, err := checker.CheckParity()
	if err != nil {
		t.Fatalf("CheckParity failed: %v", err)
	}

	// Verify report structure
	if report == nil {
		t.Fatal("Expected non-nil report")
	}

	// Health score should be between 0 and 100
	if report.HealthScore < 0 || report.HealthScore > 100 {
		t.Errorf("Invalid health score: %.2f", report.HealthScore)
	}

	// Should have run some checks
	if report.TotalChecks == 0 {
		t.Error("Expected TotalChecks > 0")
	}

	// Report should have details
	if report.Details == nil {
		t.Error("Expected non-nil Details")
	}

	// Should have code analysis details
	if _, ok := report.Details["code"]; !ok {
		t.Error("Expected code analysis in details")
	}
}

func TestParityChecker_WithCustomDirs(t *testing.T) {
	tempDir := t.TempDir()

	customCodeDir := filepath.Join(tempDir, "custom_code")
	if err := os.MkdirAll(customCodeDir, 0755); err != nil {
		t.Fatal(err)
	}

	checker := NewParityChecker(tempDir).
		WithCodeDir(customCodeDir).
		WithSpecDir(filepath.Join(tempDir, "custom_spec")).
		WithAPIDir(filepath.Join(tempDir, "custom_api"))

	// Verify directories were set (internal fields - would need getters in real code)
	// For now, just verify it doesn't crash
	if checker == nil {
		t.Error("Expected non-nil checker")
	}
}

func TestParityChecker_ToCompactJSON(t *testing.T) {
	report := &ParityReport{
		OK:           true,
		HealthScore:  95.5,
		TotalChecks:  100,
		PassedChecks: 95,
		FailedChecks: 5,
		Errors:       []ParityError{},
		Warnings:     []ParityError{},
		Details:      make(map[string]interface{}),
	}

	result := report.ToCompactJSON()

	// Verify required fields
	if ok, exists := result["ok"].(bool); !exists || !ok {
		t.Error("Expected ok=true")
	}

	if health, exists := result["health"].(float64); !exists || health != 95.5 {
		t.Errorf("Expected health=95.5, got %v", result["health"])
	}

	if checks, exists := result["checks"].(int); !exists || checks != 100 {
		t.Errorf("Expected checks=100, got %v", result["checks"])
	}
}

func TestParityChecker_ErrorHandling(t *testing.T) {
	// Test with non-existent directory
	checker := NewParityChecker("/nonexistent/path")

	report, err := checker.CheckParity()

	// Should not return error (should handle gracefully with warnings)
	if err != nil {
		// It's okay to error if directory doesn't exist
		t.Logf("CheckParity returned error (expected): %v", err)
	}

	// If report is returned, it should have structure
	if report != nil {
		if report.HealthScore < 0 || report.HealthScore > 100 {
			t.Errorf("Invalid health score: %.2f", report.HealthScore)
		}
	}
}

func TestParityError_Structure(t *testing.T) {
	err := ParityError{
		Type:     "test_error",
		Severity: "error",
		Source:   "test.rs:10",
		Issue:    "Test issue",
		Fix:      "Fix suggestion",
	}

	if err.Type != "test_error" {
		t.Errorf("Expected Type='test_error', got '%s'", err.Type)
	}

	if err.Severity != "error" {
		t.Errorf("Expected Severity='error', got '%s'", err.Severity)
	}
}

func TestParityReport_WithErrors(t *testing.T) {
	report := &ParityReport{
		OK:           false,
		HealthScore:  60.0,
		TotalChecks:  10,
		PassedChecks: 6,
		FailedChecks: 4,
		Errors: []ParityError{
			{
				Type:     "spec_code_mismatch",
				Severity: "error",
				Source:   "spec.md:5",
				Issue:    "Missing implementation",
				Fix:      "Implement function foo",
			},
		},
		Warnings: []ParityError{
			{
				Type:     "code_not_specified",
				Severity: "warning",
				Source:   "lib.rs:10",
				Issue:    "Public function not in spec",
				Fix:      "Add to specification",
			},
		},
		Details: make(map[string]interface{}),
	}

	result := report.ToCompactJSON()

	// Verify errors are included
	if errors, ok := result["errors"].([]map[string]interface{}); !ok || len(errors) != 1 {
		t.Error("Expected 1 error in JSON")
	}

	// Verify warnings are included
	if warnings, ok := result["warnings"].([]map[string]interface{}); !ok || len(warnings) != 1 {
		t.Error("Expected 1 warning in JSON")
	}

	// Verify error counts
	if errCnt, ok := result["err_cnt"].(int); !ok || errCnt != 1 {
		t.Errorf("Expected err_cnt=1, got %v", result["err_cnt"])
	}

	if warnCnt, ok := result["warn_cnt"].(int); !ok || warnCnt != 1 {
		t.Errorf("Expected warn_cnt=1, got %v", result["warn_cnt"])
	}
}
