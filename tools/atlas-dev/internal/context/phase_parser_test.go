package context

import (
	"os"
	"path/filepath"
	"testing"
)

// Sample phase file content for testing
const samplePhaseFile = `# Phase 05: Context System

## 🚨 BLOCKERS - CHECK BEFORE STARTING
**REQUIRED:** Phase 4 must be 100% complete

**Verification:**
` + "```bash" + `
atlas-dev summary
atlas-dev stats
` + "```" + `

**What's needed:**
- Phase 4 analytics system complete
- All database queries optimized

**If missing:** Complete Phase 4 first

---

## Objective
Aggregate comprehensive phase context from database and phase markdown files - combining structured DB data with phase instructions, dependencies, related decisions, and navigation - providing AI agents with everything needed to start work in a single < 10ms query returning ~200 tokens.

## Files
**Create:** ` + "`cmd/atlas-dev/context.go`" + ` (~150 lines - context command group)
**Create:** ` + "`cmd/atlas-dev/context_current.go`" + ` (~100 lines - current command)
**Update:** ` + "`cmd/atlas-dev/main.go`" + ` (add context command group)

## Dependencies
- Phase 1 infrastructure (DB, JSON output)
- Phase 2 phase management (phase tracking)
- Phase 3 decision logs (related decisions)
- Phase 4 analytics (category progress)
- All Phase 1-4 acceptance criteria met

## Implementation

### Phase File Parser
Implement markdown parser in internal/context/phase_parser.go to extract structured data from phase markdown files.

### Context Aggregator
Implement context aggregator in internal/context/aggregator.go to combine DB data with parsed phase files.

### Context Current Command
Implement context current subcommand to show context for the next phase to work on.

## Acceptance
- atlas-dev context current returns next phase context
- Context includes all necessary fields
- Phase file parsing extracts objectives/deliverables/criteria
- All commands return compact JSON
- Null/empty fields omitted
- Exit codes correct (0-6)
- 35+ tests pass
- 80%+ coverage on context/parser
`

func createPhaseTestFile(t *testing.T, content string) string {
	t.Helper()

	dir := t.TempDir()
	path := filepath.Join(dir, "phase-05-test.md")

	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		t.Fatalf("failed to create test file: %v", err)
	}

	return path
}

func TestParsePhaseFile(t *testing.T) {
	path := createPhaseTestFile(t, samplePhaseFile)

	phaseFile, err := ParsePhaseFile(path)
	if err != nil {
		t.Fatalf("ParsePhaseFile() error = %v", err)
	}

	// Check objective
	if phaseFile.Objective == "" {
		t.Error("expected objective to be extracted")
	}
	if !contains(phaseFile.Objective, "Aggregate comprehensive phase context") {
		t.Errorf("objective doesn't contain expected text: %s", phaseFile.Objective)
	}

	// Check files
	if len(phaseFile.Files) == 0 {
		t.Error("expected files to be extracted")
	}
	expectedFiles := []string{
		"cmd/atlas-dev/context.go",
		"cmd/atlas-dev/context_current.go",
		"cmd/atlas-dev/main.go",
	}
	for _, expected := range expectedFiles {
		if !containsString(phaseFile.Files, expected) {
			t.Errorf("expected file %q not found in %v", expected, phaseFile.Files)
		}
	}

	// Check dependencies
	if len(phaseFile.Dependencies) == 0 {
		t.Log("Warning: no dependencies extracted (this is OK if none specified)")
	}

	// Check deliverables
	if len(phaseFile.Deliverables) == 0 {
		t.Error("expected deliverables to be extracted")
	}
	expectedDeliverables := []string{
		"Phase File Parser",
		"Context Aggregator",
		"Context Current Command",
	}
	for _, expected := range expectedDeliverables {
		if !containsString(phaseFile.Deliverables, expected) {
			t.Errorf("expected deliverable %q not found in %v", expected, phaseFile.Deliverables)
		}
	}

	// Check acceptance criteria
	if len(phaseFile.AcceptanceCriteria) == 0 {
		t.Error("expected acceptance criteria to be extracted")
	}
}

func TestParsePhaseFile_MissingFile(t *testing.T) {
	_, err := ParsePhaseFile("/nonexistent/path.md")
	if err == nil {
		t.Error("expected error for missing file")
	}
}

func TestParsePhaseFile_EmptyFile(t *testing.T) {
	path := createPhaseTestFile(t, "")

	phaseFile, err := ParsePhaseFile(path)
	if err != nil {
		t.Fatalf("ParsePhaseFile() error = %v", err)
	}

	// Should handle empty file gracefully
	if phaseFile.Objective != "" {
		t.Error("expected empty objective for empty file")
	}
	if len(phaseFile.Deliverables) != 0 {
		t.Error("expected no deliverables for empty file")
	}
}

func TestParsePhaseFile_MinimalFile(t *testing.T) {
	minimal := `# Phase Test

## Objective
Test objective here.

## Acceptance
- First criterion
- Second criterion
`
	path := createPhaseTestFile(t, minimal)

	phaseFile, err := ParsePhaseFile(path)
	if err != nil {
		t.Fatalf("ParsePhaseFile() error = %v", err)
	}

	if phaseFile.Objective == "" {
		t.Error("expected objective to be extracted")
	}
	if len(phaseFile.AcceptanceCriteria) != 2 {
		t.Errorf("expected 2 acceptance criteria, got %d", len(phaseFile.AcceptanceCriteria))
	}
}

func TestExtractFirstParagraph(t *testing.T) {
	tests := []struct {
		name    string
		content string
		want    string
	}{
		{
			name:    "single line",
			content: "This is a single line.",
			want:    "This is a single line.",
		},
		{
			name:    "multiple lines",
			content: "Line one\nLine two\n\nNew paragraph",
			want:    "Line one Line two",
		},
		{
			name:    "with leading whitespace",
			content: "\n\nFirst paragraph here",
			want:    "First paragraph here",
		},
		{
			name:    "empty",
			content: "",
			want:    "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractFirstParagraph(tt.content)
			if got != tt.want {
				t.Errorf("got %q, want %q", got, tt.want)
			}
		})
	}
}

func TestExtractTimeEstimate(t *testing.T) {
	tests := []struct {
		name string
		text string
		want string
	}{
		{
			name: "hours range",
			text: "Estimated time: 4-6 hours",
			want: "4-6 hour",
		},
		{
			name: "single hours",
			text: "This will take 2 hours",
			want: "2 hour",
		},
		{
			name: "minutes",
			text: "About 30 minutes",
			want: "30 minute",
		},
		{
			name: "no estimate",
			text: "No time mentioned",
			want: "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := extractTimeEstimate(tt.text)
			if got != tt.want {
				t.Errorf("got %q, want %q", got, tt.want)
			}
		})
	}
}

// Helper functions
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

func containsString(slice []string, s string) bool {
	for _, item := range slice {
		if item == s {
			return true
		}
	}
	return false
}
