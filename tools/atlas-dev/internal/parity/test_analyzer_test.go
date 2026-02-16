package parity

import (
	"testing"
)

func TestTestAnalysisReport_ToCompactJSON(t *testing.T) {
	report := &TestAnalysisReport{
		Requirements:  []TestRequirement{{Required: 10, Actual: 8}},
		TotalRequired: 50,
		TotalActual:   45,
		TotalMet:      8,
		TotalDeficit:  5,
		Coverage:      90.0,
	}

	result := report.ToCompactJSON()

	if required, ok := result["required"].(int); !ok || required != 50 {
		t.Errorf("Expected required=50, got %v", result["required"])
	}

	if coverage, ok := result["coverage"].(float64); !ok || coverage != 90.0 {
		t.Errorf("Expected coverage=90.0, got %v", result["coverage"])
	}
}

func TestTestAnalysisReport_GetDeficits(t *testing.T) {
	report := &TestAnalysisReport{
		Requirements: []TestRequirement{
			{PhaseID: "phase-01", Required: 10, Actual: 10, Met: true},
			{PhaseID: "phase-02", Required: 20, Actual: 15, Met: false, Deficit: 5},
			{PhaseID: "phase-03", Required: 5, Actual: 3, Met: false, Deficit: 2},
		},
	}

	deficits := report.GetDeficits()

	if len(deficits) != 2 {
		t.Errorf("Expected 2 deficits, got %d", len(deficits))
	}

	// Verify all deficits are !Met
	for _, d := range deficits {
		if d.Met {
			t.Errorf("Deficit should have Met=false, got true for %s", d.PhaseID)
		}
	}
}

func TestTestRequirement_Structure(t *testing.T) {
	req := TestRequirement{
		PhasePath: "phases/test/phase-01.md",
		PhaseID:   "phase-01",
		Category:  "test",
		Required:  25,
		Actual:    20,
		Deficit:   5,
		Met:       false,
		TestFiles: []string{"test1.rs", "test2.rs"},
	}

	if req.PhaseID != "phase-01" {
		t.Errorf("Expected PhaseID='phase-01', got '%s'", req.PhaseID)
	}

	if req.Deficit != 5 {
		t.Errorf("Expected Deficit=5, got %d", req.Deficit)
	}

	if req.Met {
		t.Error("Expected Met=false")
	}

	if len(req.TestFiles) != 2 {
		t.Errorf("Expected 2 test files, got %d", len(req.TestFiles))
	}
}
