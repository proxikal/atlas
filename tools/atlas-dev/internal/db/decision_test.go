package db

import (
	"strings"
	"testing"
)

func TestGetNextDecisionID(t *testing.T) {
	tests := []struct {
		name       string
		existing   []string
		wantNextID string
	}{
		{
			name:       "first decision",
			existing:   []string{},
			wantNextID: "DR-001",
		},
		{
			name:       "sequential IDs",
			existing:   []string{"DR-001", "DR-002"},
			wantNextID: "DR-003",
		},
		{
			name:       "zero padding",
			existing:   []string{"DR-001", "DR-002", "DR-003", "DR-004", "DR-005", "DR-006", "DR-007", "DR-008", "DR-009"},
			wantNextID: "DR-010",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db := NewTestDB(t)
			SeedTestCategory(t, db, 100, "test", "Test Category", 1)

			// Seed existing decisions
			for _, id := range tt.existing {
				SeedTestDecision(t, db, id, "test", "Test Decision")
			}

			// Get next ID
			nextID, err := db.GetNextDecisionID()
			if err != nil {
				t.Fatalf("GetNextDecisionID() error: %v", err)
			}

			if nextID != tt.wantNextID {
				t.Errorf("GetNextDecisionID() = %q, want %q", nextID, tt.wantNextID)
			}
		})
	}
}

func TestCreateDecision(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 101, "stdlib", "Standard Library", 1)

	req := CreateDecisionRequest{
		Component:     "stdlib",
		Title:         "Hash function design",
		DecisionText:  "Use FNV-1a for HashMap",
		Rationale:     "Fast, simple, good distribution",
		Alternatives:  "MurmurHash, CityHash",
		Consequences:  "Good performance for most use cases",
		Status:        "accepted",
		RelatedPhases: "phase-07b",
		Tags:          "performance,stdlib",
	}

	decision, err := db.CreateDecision(req)
	if err != nil {
		t.Fatalf("CreateDecision() error: %v", err)
	}

	// Verify ID format
	if !strings.HasPrefix(decision.ID, "DR-") {
		t.Errorf("expected ID to start with 'DR-', got %q", decision.ID)
	}

	// Verify fields
	if decision.Component != "stdlib" {
		t.Errorf("expected component = 'stdlib', got %q", decision.Component)
	}

	if decision.Title != "Hash function design" {
		t.Errorf("expected title = 'Hash function design', got %q", decision.Title)
	}

	if decision.Status != "accepted" {
		t.Errorf("expected status = 'accepted', got %q", decision.Status)
	}
}

func TestCreateDecisionAutoIncrementID(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 102, "test", "Test", 1)

	req := CreateDecisionRequest{
		Component:    "test",
		Title:        "Decision 1",
		DecisionText: "Decision text",
		Rationale:    "Rationale",
	}

	// Create first decision
	d1, err := db.CreateDecision(req)
	if err != nil {
		t.Fatalf("CreateDecision() error: %v", err)
	}

	if d1.ID != "DR-001" {
		t.Errorf("expected first ID = 'DR-001', got %q", d1.ID)
	}

	// Create second decision
	req.Title = "Decision 2"
	d2, err := db.CreateDecision(req)
	if err != nil {
		t.Fatalf("CreateDecision() error: %v", err)
	}

	if d2.ID != "DR-002" {
		t.Errorf("expected second ID = 'DR-002', got %q", d2.ID)
	}

	// Create third decision
	req.Title = "Decision 3"
	d3, err := db.CreateDecision(req)
	if err != nil {
		t.Fatalf("CreateDecision() error: %v", err)
	}

	if d3.ID != "DR-003" {
		t.Errorf("expected third ID = 'DR-003', got %q", d3.ID)
	}
}

func TestCreateDecisionDefaultStatus(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 103, "test", "Test", 1)

	req := CreateDecisionRequest{
		Component:    "test",
		Title:        "Test Decision",
		DecisionText: "Test",
		Rationale:    "Test",
		// Status not set - should default to "accepted"
	}

	decision, err := db.CreateDecision(req)
	if err != nil {
		t.Fatalf("CreateDecision() error: %v", err)
	}

	if decision.Status != "accepted" {
		t.Errorf("expected default status = 'accepted', got %q", decision.Status)
	}
}

func TestCreateDecisionInvalidComponent(t *testing.T) {
	db := NewTestDB(t)

	req := CreateDecisionRequest{
		Component:    "nonexistent",
		Title:        "Test",
		DecisionText: "Test",
		Rationale:    "Test",
	}

	_, err := db.CreateDecision(req)
	if err == nil {
		t.Error("expected error for invalid component, got nil")
	}

	if !strings.Contains(err.Error(), "invalid component") {
		t.Errorf("expected 'invalid component' error, got %v", err)
	}
}

func TestCreateDecisionInvalidStatus(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 104, "test", "Test", 1)

	req := CreateDecisionRequest{
		Component:    "test",
		Title:        "Test",
		DecisionText: "Test",
		Rationale:    "Test",
		Status:       "invalid-status",
	}

	_, err := db.CreateDecision(req)
	if err == nil {
		t.Error("expected error for invalid status, got nil")
	}

	if !strings.Contains(err.Error(), "invalid status") {
		t.Errorf("expected 'invalid status' error, got %v", err)
	}
}

func TestGetDecision(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 105, "test", "Test", 1)

	// Seed decision
	SeedTestDecision(t, db, "DR-001", "test", "Test Decision")

	// Get decision
	decision, err := db.GetDecision("DR-001")
	if err != nil {
		t.Fatalf("GetDecision() error: %v", err)
	}

	if decision.ID != "DR-001" {
		t.Errorf("expected ID = 'DR-001', got %q", decision.ID)
	}

	if decision.Component != "test" {
		t.Errorf("expected component = 'test', got %q", decision.Component)
	}

	if decision.Title != "Test Decision" {
		t.Errorf("expected title = 'Test Decision', got %q", decision.Title)
	}
}

func TestGetDecisionNotFound(t *testing.T) {
	db := NewTestDB(t)

	_, err := db.GetDecision("DR-999")
	if err != ErrDecisionNotFound {
		t.Errorf("expected ErrDecisionNotFound, got %v", err)
	}
}

func TestListDecisions(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 106, "stdlib", "Stdlib", 1)
	SeedTestCategory(t, db, 107, "frontend", "Frontend", 1)

	// Seed decisions
	SeedTestDecision(t, db, "DR-001", "stdlib", "Decision 1")
	SeedTestDecision(t, db, "DR-002", "stdlib", "Decision 2")
	SeedTestDecision(t, db, "DR-003", "frontend", "Decision 3")

	// List all
	all, err := db.ListDecisions(ListDecisionsOptions{})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(all) != 3 {
		t.Errorf("expected 3 decisions, got %d", len(all))
	}
}

func TestListDecisionsFilterByComponent(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 108, "stdlib", "Stdlib", 1)
	SeedTestCategory(t, db, 109, "frontend", "Frontend", 1)

	// Seed decisions
	SeedTestDecision(t, db, "DR-001", "stdlib", "Decision 1")
	SeedTestDecision(t, db, "DR-002", "stdlib", "Decision 2")
	SeedTestDecision(t, db, "DR-003", "frontend", "Decision 3")

	// Filter by component
	stdlibDecisions, err := db.ListDecisions(ListDecisionsOptions{
		Component: "stdlib",
	})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(stdlibDecisions) != 2 {
		t.Errorf("expected 2 stdlib decisions, got %d", len(stdlibDecisions))
	}

	for _, d := range stdlibDecisions {
		if d.Component != "stdlib" {
			t.Errorf("expected component = 'stdlib', got %q", d.Component)
		}
	}
}

func TestListDecisionsFilterByStatus(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 110, "test", "Test", 1)

	// Seed decisions with different statuses
	SeedTestDecisionWithStatus(t, db, "DR-001", "test", "Decision 1", "accepted")
	SeedTestDecisionWithStatus(t, db, "DR-002", "test", "Decision 2", "accepted")
	SeedTestDecisionWithStatus(t, db, "DR-003", "test", "Decision 3", "proposed")

	// Filter by status
	accepted, err := db.ListDecisions(ListDecisionsOptions{
		Status: "accepted",
	})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(accepted) != 2 {
		t.Errorf("expected 2 accepted decisions, got %d", len(accepted))
	}

	for _, d := range accepted {
		if d.Status != "accepted" {
			t.Errorf("expected status = 'accepted', got %q", d.Status)
		}
	}
}

func TestListDecisionsCombinedFilters(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 111, "stdlib", "Stdlib", 1)
	SeedTestCategory(t, db, 112, "frontend", "Frontend", 1)

	// Seed decisions
	SeedTestDecisionWithStatus(t, db, "DR-001", "stdlib", "Decision 1", "accepted")
	SeedTestDecisionWithStatus(t, db, "DR-002", "stdlib", "Decision 2", "proposed")
	SeedTestDecisionWithStatus(t, db, "DR-003", "frontend", "Decision 3", "accepted")

	// Filter by component and status
	results, err := db.ListDecisions(ListDecisionsOptions{
		Component: "stdlib",
		Status:    "accepted",
	})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(results) != 1 {
		t.Errorf("expected 1 result, got %d", len(results))
	}

	if len(results) > 0 {
		if results[0].Component != "stdlib" || results[0].Status != "accepted" {
			t.Errorf("unexpected result: %+v", results[0])
		}
	}
}

func TestListDecisionsLimit(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 113, "test", "Test", 1)

	// Seed 30 decisions
	for i := 1; i <= 30; i++ {
		id := "DR-" + padZero(i, 3)
		SeedTestDecision(t, db, id, "test", "Decision "+id)
	}

	// List with limit
	results, err := db.ListDecisions(ListDecisionsOptions{
		Limit: 10,
	})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(results) != 10 {
		t.Errorf("expected 10 results (limit), got %d", len(results))
	}
}

func TestListDecisionsEmpty(t *testing.T) {
	db := NewTestDB(t)

	results, err := db.ListDecisions(ListDecisionsOptions{})
	if err != nil {
		t.Fatalf("ListDecisions() error: %v", err)
	}

	if len(results) != 0 {
		t.Errorf("expected 0 results, got %d", len(results))
	}
}

func TestSearchDecisions(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 114, "test", "Test", 1)

	// Seed decisions
	_, err := db.conn.Exec(`
		INSERT INTO decisions (id, component, title, decision, rationale, date, status)
		VALUES
			('DR-001', 'test', 'Hash function design', 'Use FNV-1a', 'Fast and simple', '2024-01-01', 'accepted'),
			('DR-002', 'test', 'Error handling', 'Use Result type', 'Explicit errors', '2024-01-02', 'accepted'),
			('DR-003', 'test', 'HashMap implementation', 'Open addressing with linear probing', 'Simple and fast', '2024-01-03', 'accepted')
	`)
	if err != nil {
		t.Fatalf("failed to seed decisions: %v", err)
	}

	tests := []struct {
		name      string
		query     string
		wantCount int
	}{
		{
			name:      "search by title",
			query:     "Hash",
			wantCount: 2, // "Hash function design" and "HashMap implementation"
		},
		{
			name:      "search by decision text",
			query:     "FNV-1a",
			wantCount: 1,
		},
		{
			name:      "search by rationale",
			query:     "simple",
			wantCount: 2, // "Fast and simple" and "Simple and fast"
		},
		{
			name:      "case insensitive",
			query:     "hash",
			wantCount: 2,
		},
		{
			name:      "no matches",
			query:     "nonexistent",
			wantCount: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			results, err := db.SearchDecisions(tt.query, 20)
			if err != nil {
				t.Fatalf("SearchDecisions() error: %v", err)
			}

			if len(results) != tt.wantCount {
				t.Errorf("SearchDecisions(%q) = %d results, want %d", tt.query, len(results), tt.wantCount)
			}
		})
	}
}

func TestUpdateDecisionStatus(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 115, "test", "Test", 1)

	// Seed decision with "proposed" status
	SeedTestDecisionWithStatus(t, db, "DR-001", "test", "Test", "proposed")

	// Update to accepted
	req := UpdateDecisionRequest{
		ID:     "DR-001",
		Status: "accepted",
	}

	decision, err := db.UpdateDecision(req)
	if err != nil {
		t.Fatalf("UpdateDecision() error: %v", err)
	}

	if decision.Status != "accepted" {
		t.Errorf("expected status = 'accepted', got %q", decision.Status)
	}

	// Verify in database
	updated, err := db.GetDecision("DR-001")
	if err != nil {
		t.Fatalf("GetDecision() error: %v", err)
	}

	if updated.Status != "accepted" {
		t.Errorf("expected status = 'accepted' in DB, got %q", updated.Status)
	}
}

func TestUpdateDecisionSuperseded(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 116, "test", "Test", 1)

	// Seed decision
	SeedTestDecisionWithStatus(t, db, "DR-001", "test", "Test", "accepted")
	SeedTestDecisionWithStatus(t, db, "DR-002", "test", "Test 2", "accepted")

	// Mark as superseded
	req := UpdateDecisionRequest{
		ID:           "DR-001",
		SupersededBy: "DR-002",
	}

	decision, err := db.UpdateDecision(req)
	if err != nil {
		t.Fatalf("UpdateDecision() error: %v", err)
	}

	if decision.Status != "superseded" {
		t.Errorf("expected status = 'superseded', got %q", decision.Status)
	}

	if !decision.SupersededBy.Valid || decision.SupersededBy.String != "DR-002" {
		t.Errorf("expected superseded_by = 'DR-002', got %v", decision.SupersededBy)
	}
}

func TestUpdateDecisionInvalidTransition(t *testing.T) {
	db := NewTestDB(t)
	SeedTestCategory(t, db, 117, "test", "Test", 1)

	// Seed decision with "rejected" status
	SeedTestDecisionWithStatus(t, db, "DR-001", "test", "Test", "rejected")

	// Try to transition from rejected (not allowed)
	req := UpdateDecisionRequest{
		ID:     "DR-001",
		Status: "accepted",
	}

	_, err := db.UpdateDecision(req)
	if err == nil {
		t.Error("expected error for invalid transition, got nil")
	}

	if !strings.Contains(err.Error(), "invalid status") {
		t.Errorf("expected 'invalid status' error, got %v", err)
	}
}

func TestUpdateDecisionNotFound(t *testing.T) {
	db := NewTestDB(t)

	req := UpdateDecisionRequest{
		ID:     "DR-999",
		Status: "accepted",
	}

	_, err := db.UpdateDecision(req)
	if err != ErrDecisionNotFound {
		t.Errorf("expected ErrDecisionNotFound, got %v", err)
	}
}

func TestDecisionToCompactJSON(t *testing.T) {
	decision := &Decision{
		ID:            "DR-001",
		Component:     "stdlib",
		Title:         "Test Decision",
		DecisionText:  "Test",
		Rationale:     "Test rationale",
		Date:          "2024-01-01",
		Status:        "accepted",
		Alternatives:  nullString("Alt 1"),
		Consequences:  nullString(""),
		SupersededBy:  nullString(""),
		RelatedPhases: nullString(""),
		Tags:          nullString(""),
	}

	json := decision.ToCompactJSON()

	// Check required fields
	if json["id"] != "DR-001" {
		t.Errorf("expected id = 'DR-001', got %v", json["id"])
	}

	if json["comp"] != "stdlib" {
		t.Errorf("expected comp = 'stdlib', got %v", json["comp"])
	}

	// Check abbreviated field names
	if _, ok := json["component"]; ok {
		t.Error("should use 'comp' not 'component'")
	}

	if _, ok := json["stat"]; !ok {
		t.Error("should have 'stat' field")
	}

	// Check null fields are omitted
	if _, ok := json["cons"]; ok {
		t.Error("empty consequences should be omitted")
	}

	if _, ok := json["super"]; ok {
		t.Error("empty superseded_by should be omitted")
	}
}

// Helper functions

func SeedTestDecision(t *testing.T, db *DB, id, component, title string) {
	t.Helper()

	_, err := db.conn.Exec(`
		INSERT INTO decisions (id, component, title, decision, rationale, date, status)
		VALUES (?, ?, ?, 'Test decision', 'Test rationale', '2024-01-01', 'accepted')
	`, id, component, title)
	if err != nil {
		t.Fatalf("failed to seed decision: %v", err)
	}
}

func SeedTestDecisionWithStatus(t *testing.T, db *DB, id, component, title, status string) {
	t.Helper()

	_, err := db.conn.Exec(`
		INSERT INTO decisions (id, component, title, decision, rationale, date, status)
		VALUES (?, ?, ?, 'Test decision', 'Test rationale', '2024-01-01', ?)
	`, id, component, title, status)
	if err != nil {
		t.Fatalf("failed to seed decision: %v", err)
	}
}

func padZero(n, width int) string {
	s := ""
	for i := 0; i < width; i++ {
		s = "0" + s
	}

	// Convert n to string and overlay
	ns := ""
	for n > 0 {
		ns = string(rune('0'+(n%10))) + ns
		n /= 10
	}

	if len(ns) >= width {
		return ns
	}

	return s[:width-len(ns)] + ns
}
