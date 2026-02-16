package context

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// Helper to create test database
func newTestDB(t *testing.T) *db.DB {
	t.Helper()

	// Create temp database
	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "test.db")

	database, err := db.New(dbPath)
	if err != nil {
		t.Fatalf("failed to create test db: %v", err)
	}

	// Initialize schema
	if err := database.InitSchema(); err != nil {
		t.Fatalf("failed to init schema: %v", err)
	}

	// Prepare statements
	if err := database.Prepare(); err != nil {
		t.Fatalf("failed to prepare statements: %v", err)
	}

	t.Cleanup(func() {
		_ = database.Close()
	})

	return database
}

// Helper to create test phase file
func createTestPhaseFile(t *testing.T, path, content string) {
	t.Helper()

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		t.Fatalf("failed to create directory: %v", err)
	}

	if err := os.WriteFile(path, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write phase file: %v", err)
	}

	t.Cleanup(func() {
		_ = os.RemoveAll(filepath.Dir(path))
	})
}

const testPhaseContent = `# Phase Test

## Objective
Test phase for aggregator testing.

## Files
**Create:** ` + "`internal/test/file.go`" + `

## Dependencies
- Phase 1 (infrastructure)
- Phase 2 (management)

## Implementation

### Feature One
Implement feature one.

### Feature Two
Implement feature two.

## Acceptance
- First acceptance criterion
- Second acceptance criterion
- Third acceptance criterion
`

func TestGetPhaseContext(t *testing.T) {
	database := newTestDB(t)

	// Insert test phase
	phaseID, err := database.InsertPhase("phases/test.md", "test-phase", "test")
	if err != nil {
		t.Fatalf("failed to insert phase: %v", err)
	}
	if phaseID == 0 {
		t.Fatal("expected non-zero phase ID")
	}

	// Create phase file
	createTestPhaseFile(t, "phases/test.md", testPhaseContent)

	// Create aggregator
	agg := NewAggregator(database)

	// Get phase context
	ctx, err := agg.GetPhaseContext("phases/test.md")
	if err != nil {
		t.Fatalf("GetPhaseContext() error = %v", err)
	}

	// Verify basic fields
	if ctx.Path != "phases/test.md" {
		t.Errorf("path = %q, want %q", ctx.Path, "phases/test.md")
	}
	if ctx.Name != "test-phase" {
		t.Errorf("name = %q, want %q", ctx.Name, "test-phase")
	}
	if ctx.Category != "test" {
		t.Errorf("category = %q, want %q", ctx.Category, "test")
	}
	if ctx.Status != "pending" {
		t.Errorf("status = %q, want %q", ctx.Status, "pending")
	}

	// Verify parsed phase file data
	if ctx.Objective == "" {
		t.Error("expected objective to be populated")
	}
	if len(ctx.Deliverables) != 2 {
		t.Errorf("deliverables = %d, want 2", len(ctx.Deliverables))
	}
	if len(ctx.AcceptanceCriteria) != 3 {
		t.Errorf("acceptance criteria = %d, want 3", len(ctx.AcceptanceCriteria))
	}
	if len(ctx.Files) != 1 {
		t.Errorf("files = %d, want 1", len(ctx.Files))
	}
	if len(ctx.Dependencies) != 2 {
		t.Errorf("dependencies = %d, want 2", len(ctx.Dependencies))
	}
}

func TestGetPhaseContext_MissingPhaseFile(t *testing.T) {
	database := newTestDB(t)

	// Insert phase without creating file
	phaseID, err := database.InsertPhase("phases/missing.md", "missing-phase", "test")
	if err != nil {
		t.Fatalf("failed to insert phase: %v", err)
	}
	if phaseID == 0 {
		t.Fatal("expected non-zero phase ID")
	}

	// Create aggregator
	agg := NewAggregator(database)

	// Get phase context (should work even without file)
	ctx, err := agg.GetPhaseContext("phases/missing.md")
	if err != nil {
		t.Fatalf("GetPhaseContext() error = %v", err)
	}

	// Verify DB fields still populated
	if ctx.Path != "phases/missing.md" {
		t.Errorf("path = %q, want %q", ctx.Path, "phases/missing.md")
	}

	// Verify phase file fields are empty
	if ctx.Objective != "" {
		t.Error("expected empty objective for missing file")
	}
	if len(ctx.Deliverables) != 0 {
		t.Error("expected no deliverables for missing file")
	}
}

func TestGetPhaseContext_NotFound(t *testing.T) {
	database := newTestDB(t)
	agg := NewAggregator(database)

	// Try to get non-existent phase
	_, err := agg.GetPhaseContext("phases/nonexistent.md")
	if err == nil {
		t.Error("expected error for non-existent phase")
	}
}

func TestGetCurrentPhaseContext(t *testing.T) {
	database := newTestDB(t)

	// Insert phases
	_, err := database.InsertPhase("phases/phase-01.md", "phase-01", "cat1")
	if err != nil {
		t.Fatalf("failed to insert phase 1: %v", err)
	}

	_, err = database.InsertPhase("phases/phase-02.md", "phase-02", "cat1")
	if err != nil {
		t.Fatalf("failed to insert phase 2: %v", err)
	}

	// Create phase files
	createTestPhaseFile(t, "phases/phase-01.md", testPhaseContent)
	createTestPhaseFile(t, "phases/phase-02.md", testPhaseContent)

	// Create aggregator
	agg := NewAggregator(database)

	// Get current phase context (should return first pending)
	ctx, err := agg.GetCurrentPhaseContext()
	if err != nil {
		t.Fatalf("GetCurrentPhaseContext() error = %v", err)
	}

	// Should return first pending phase
	if ctx.Path != "phases/phase-01.md" {
		t.Errorf("path = %q, want %q", ctx.Path, "phases/phase-01.md")
	}
}

func TestGetCurrentPhaseContext_NoPhases(t *testing.T) {
	database := newTestDB(t)
	agg := NewAggregator(database)

	// Try to get current when no phases exist
	_, err := agg.GetCurrentPhaseContext()
	if err == nil {
		t.Error("expected error when no phases exist")
	}
}

func TestToCompactJSON(t *testing.T) {
	ctx := &PhaseContext{
		ID:           1,
		Path:         "test.md",
		Name:         "test",
		Category:     "cat",
		Status:       "pending",
		Objective:    "Test objective",
		Deliverables: []string{"Del 1", "Del 2"},
		CategoryProgress: &ProgressData{
			Completed:  5,
			Total:      10,
			Percentage: 50,
		},
	}

	result := ctx.ToCompactJSON()

	// Check required fields
	if result["ok"] != true {
		t.Error("expected ok = true")
	}
	if result["path"] != "test.md" {
		t.Error("expected path in result")
	}

	// Check optional fields
	if result["obj"] != "Test objective" {
		t.Error("expected objective in result")
	}

	// Check array format for progress
	if prog, ok := result["catProg"].([]int); !ok || len(prog) != 3 {
		t.Errorf("expected catProg as [int,int,int], got %v", result["catProg"])
	}
}

func TestToCompactJSON_OmitsEmpty(t *testing.T) {
	ctx := &PhaseContext{
		Path:     "test.md",
		Name:     "test",
		Category: "cat",
		Status:   "pending",
		// No optional fields set
	}

	result := ctx.ToCompactJSON()

	// Check that empty fields are omitted
	if _, exists := result["obj"]; exists {
		t.Error("expected empty objective to be omitted")
	}
	if _, exists := result["dlv"]; exists {
		t.Error("expected empty deliverables to be omitted")
	}
	if _, exists := result["catProg"]; exists {
		t.Error("expected nil category progress to be omitted")
	}
}

func TestGetNavigation(t *testing.T) {
	database := newTestDB(t)

	// Insert phases
	id1, _ := database.InsertPhase("phases/phase-01.md", "phase-01", "cat1")
	id2, _ := database.InsertPhase("phases/phase-02.md", "phase-02", "cat1")
	id3, _ := database.InsertPhase("phases/phase-03.md", "phase-03", "cat1")

	// Mark phase 1 as completed
	_, _ = database.Exec(`
		UPDATE phases SET status = 'completed', completed_date = '2024-01-01'
		WHERE id = ?
	`, id1)

	// Get phase 2
	phase, err := database.GetPhaseByPath("phases/phase-02.md")
	if err != nil {
		t.Fatalf("failed to get phase: %v", err)
	}

	// Create aggregator and get navigation
	agg := NewAggregator(database)
	nav, err := agg.getNavigation(phase)
	if err != nil {
		t.Fatalf("getNavigation() error = %v", err)
	}

	// Should have previous (phase-01) since it's completed
	if nav.Previous == nil {
		t.Error("expected previous phase")
	} else if nav.Previous.Path != "phases/phase-01.md" {
		t.Errorf("previous = %q, want %q", nav.Previous.Path, "phases/phase-01.md")
	}

	// Should have next (phase-03) since it's pending
	if nav.Next == nil {
		t.Error("expected next phase")
	} else if nav.Next.Path != "phases/phase-03.md" {
		t.Errorf("next = %q, want %q", nav.Next.Path, "phases/phase-03.md")
	}

	_ = id2
	_ = id3
}

func TestGetCurrentPhaseContext_AfterCompletion(t *testing.T) {
	database := newTestDB(t)

	// Insert phases in same category
	id1, _ := database.InsertPhase("phases/phase-01.md", "phase-01", "cat1")
	_, _ = database.InsertPhase("phases/phase-02.md", "phase-02", "cat1")

	// Create phase files
	createTestPhaseFile(t, "phases/phase-01.md", testPhaseContent)
	createTestPhaseFile(t, "phases/phase-02.md", testPhaseContent)

	// Mark phase 1 as completed
	_, _ = database.Exec(`
		UPDATE phases SET status = 'completed', completed_date = '2024-01-01'
		WHERE id = ?
	`, id1)

	// Create aggregator
	agg := NewAggregator(database)

	// Get current phase context (should return next in same category)
	ctx, err := agg.GetCurrentPhaseContext()
	if err != nil {
		t.Fatalf("GetCurrentPhaseContext() error = %v", err)
	}

	// Should return phase-02 (next in same category)
	if ctx.Path != "phases/phase-02.md" {
		t.Errorf("path = %q, want %q", ctx.Path, "phases/phase-02.md")
	}
}

func TestGetCurrentPhaseContext_DifferentCategory(t *testing.T) {
	database := newTestDB(t)

	// Insert phase in cat1
	id1, _ := database.InsertPhase("phases/cat1-phase.md", "cat1-phase", "cat1")

	// Insert phase in cat2
	_, _ = database.InsertPhase("phases/cat2-phase.md", "cat2-phase", "cat2")

	// Create phase files
	createTestPhaseFile(t, "phases/cat1-phase.md", testPhaseContent)
	createTestPhaseFile(t, "phases/cat2-phase.md", testPhaseContent)

	// Mark cat1 phase as completed
	_, _ = database.Exec(`
		UPDATE phases SET status = 'completed', completed_date = '2024-01-01'
		WHERE id = ?
	`, id1)

	// Create aggregator
	agg := NewAggregator(database)

	// Get current phase context (should return first pending overall)
	ctx, err := agg.GetCurrentPhaseContext()
	if err != nil {
		t.Fatalf("GetCurrentPhaseContext() error = %v", err)
	}

	// Should return cat2-phase (first pending overall)
	if ctx.Path != "phases/cat2-phase.md" {
		t.Errorf("path = %q, want %q", ctx.Path, "phases/cat2-phase.md")
	}
}

func TestToCompactJSON_AllFields(t *testing.T) {
	ctx := &PhaseContext{
		ID:                 1,
		Path:               "test.md",
		Name:               "test",
		Category:           "cat",
		Status:             "pending",
		Objective:          "Test objective",
		Priority:           "HIGH",
		Deliverables:       []string{"Del 1", "Del 2"},
		AcceptanceCriteria: []string{"Crit 1"},
		Files:              []string{"file.go"},
		Dependencies:       []string{"Phase 1"},
		EstimatedTime:      "2 hours",
		CategoryProgress: &ProgressData{
			Completed:  5,
			Total:      10,
			Percentage: 50,
		},
		RelatedDecisions: []*DecisionSummary{
			{ID: "DR-001", Title: "Decision"},
		},
		TestCoverage: &TestCoverageData{
			Count:  10,
			Target: 15,
		},
		Navigation: &NavigationData{
			Previous: &PhaseSummary{Path: "prev.md", Name: "prev"},
			Next:     &PhaseSummary{Path: "next.md", Name: "next"},
		},
	}

	result := ctx.ToCompactJSON()

	// Check all fields present
	fields := []string{"ok", "path", "name", "cat", "stat", "id", "obj", "pri",
		"dlv", "acc", "files", "deps", "est", "catProg", "decisions", "tests", "nav"}

	for _, field := range fields {
		if _, exists := result[field]; !exists {
			t.Errorf("expected field %q to be present", field)
		}
	}
}
