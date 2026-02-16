package parity

import (
	"testing"
)

func TestReferenceReport_ToCompactJSON(t *testing.T) {
	report := &ReferenceReport{
		TotalRefs:    100,
		ValidRefs:    95,
		BrokenRefs:   []BrokenReference{{}, {}, {}, {}, {}},
		OrphanedDocs: []string{"orphan1.md", "orphan2.md"},
	}

	result := report.ToCompactJSON()

	if total, ok := result["total"].(int); !ok || total != 100 {
		t.Errorf("Expected total=100, got %v", result["total"])
	}

	if valid, ok := result["valid"].(int); !ok || valid != 95 {
		t.Errorf("Expected valid=95, got %v", result["valid"])
	}

	if brokenCnt, ok := result["broken_cnt"].(int); !ok || brokenCnt != 5 {
		t.Errorf("Expected broken_cnt=5, got %v", result["broken_cnt"])
	}

	if orphanedCnt, ok := result["orphaned_cnt"].(int); !ok || orphanedCnt != 2 {
		t.Errorf("Expected orphaned_cnt=2, got %v", result["orphaned_cnt"])
	}
}

func TestReference_Structure(t *testing.T) {
	ref := Reference{
		SourceFile:   "README.md",
		SourceLine:   10,
		TargetPath:   "docs/guide.md",
		TargetAnchor: "getting-started",
		Text:         "Getting Started",
		Type:         "markdown",
	}

	if ref.SourceFile != "README.md" {
		t.Errorf("Expected SourceFile='README.md', got '%s'", ref.SourceFile)
	}

	if ref.SourceLine != 10 {
		t.Errorf("Expected SourceLine=10, got %d", ref.SourceLine)
	}

	if ref.TargetAnchor != "getting-started" {
		t.Errorf("Expected TargetAnchor='getting-started', got '%s'", ref.TargetAnchor)
	}

	if ref.Type != "markdown" {
		t.Errorf("Expected Type='markdown', got '%s'", ref.Type)
	}
}

func TestBrokenReference_Structure(t *testing.T) {
	broken := BrokenReference{
		Ref: Reference{
			SourceFile: "doc.md",
			TargetPath: "missing.md",
		},
		ErrorType:     "file_missing",
		FixSuggestion: "Create missing.md",
	}

	if broken.ErrorType != "file_missing" {
		t.Errorf("Expected ErrorType='file_missing', got '%s'", broken.ErrorType)
	}

	if broken.FixSuggestion == "" {
		t.Error("Expected non-empty FixSuggestion")
	}
}
