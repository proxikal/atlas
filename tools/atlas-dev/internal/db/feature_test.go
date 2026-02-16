package db

import (
	"testing"
)

func TestCreateFeature(t *testing.T) {
	db := NewTestDB(t)

	tests := []struct {
		name    string
		req     CreateFeatureRequest
		wantErr bool
	}{
		{
			name: "valid feature",
			req: CreateFeatureRequest{
				Name:        "test-feature",
				DisplayName: "Test Feature",
				Version:     "v0.1",
				Status:      "Planned",
				Description: "Test description",
			},
			wantErr: false,
		},
		{
			name: "duplicate name",
			req: CreateFeatureRequest{
				Name:        "test-feature",
				DisplayName: "Duplicate",
				Version:     "v0.1",
				Status:      "Planned",
			},
			wantErr: true,
		},
		{
			name: "invalid status",
			req: CreateFeatureRequest{
				Name:   "test-invalid",
				Status: "InvalidStatus",
			},
			wantErr: true,
		},
		{
			name: "default values",
			req: CreateFeatureRequest{
				Name:        "test-defaults",
				DisplayName: "Test Defaults",
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			feature, err := db.CreateFeature(tt.req)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if feature.Name != tt.req.Name {
				t.Errorf("Name = %s, want %s", feature.Name, tt.req.Name)
			}

			// Verify defaults
			if tt.req.Version == "" && feature.Version != "v0.1" {
				t.Errorf("Default version = %s, want v0.1", feature.Version)
			}
			if tt.req.Status == "" && feature.Status != "Planned" {
				t.Errorf("Default status = %s, want Planned", feature.Status)
			}
		})
	}
}

func TestGetFeature(t *testing.T) {
	db := NewTestDB(t)

	// Create test feature
	req := CreateFeatureRequest{
		Name:        "get-test",
		DisplayName: "Get Test",
		Version:     "v0.2",
		Status:      "Implemented",
	}
	created, err := db.CreateFeature(req)
	if err != nil {
		t.Fatalf("failed to create test feature: %v", err)
	}

	// Get feature
	feature, err := db.GetFeature("get-test")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if feature.Name != created.Name {
		t.Errorf("Name = %s, want %s", feature.Name, created.Name)
	}
	if feature.Version != created.Version {
		t.Errorf("Version = %s, want %s", feature.Version, created.Version)
	}

	// Test not found
	_, err = db.GetFeature("nonexistent")
	if err != ErrFeatureNotFound {
		t.Errorf("expected ErrFeatureNotFound, got %v", err)
	}
}

func TestListFeatures(t *testing.T) {
	db := NewTestDB(t)

	// Create test features
	features := []CreateFeatureRequest{
		{Name: "f1", DisplayName: "Feature 1", Status: "Planned"},
		{Name: "f2", DisplayName: "Feature 2", Status: "InProgress"},
		{Name: "f3", DisplayName: "Feature 3", Status: "Implemented"},
	}

	for _, req := range features {
		_, err := db.CreateFeature(req)
		if err != nil {
			t.Fatalf("failed to create feature: %v", err)
		}
	}

	tests := []struct {
		name       string
		category   string
		status     string
		wantCount  int
	}{
		{"all features", "", "", 3},
		{"planned only", "", "Planned", 1},
		{"implemented only", "", "Implemented", 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			list, err := db.ListFeatures(tt.category, tt.status)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if len(list) != tt.wantCount {
				t.Errorf("got %d features, want %d", len(list), tt.wantCount)
			}
		})
	}
}

func TestUpdateFeature(t *testing.T) {
	db := NewTestDB(t)

	// Create test feature
	_, err := db.CreateFeature(CreateFeatureRequest{
		Name:    "update-test",
		Version: "v0.1",
		Status:  "Planned",
	})
	if err != nil {
		t.Fatalf("failed to create feature: %v", err)
	}

	// Update
	req := UpdateFeatureRequest{
		Name:    "update-test",
		Version: "v0.2",
		Status:  "InProgress",
	}
	updated, err := db.UpdateFeature(req)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if updated.Version != "v0.2" {
		t.Errorf("Version = %s, want v0.2", updated.Version)
	}
	if updated.Status != "InProgress" {
		t.Errorf("Status = %s, want InProgress", updated.Status)
	}

	// Test invalid status
	req.Status = "InvalidStatus"
	_, err = db.UpdateFeature(req)
	if err == nil {
		t.Error("expected error for invalid status, got nil")
	}

	// Test nonexistent feature
	req.Name = "nonexistent"
	req.Status = "Planned"
	_, err = db.UpdateFeature(req)
	if err != ErrFeatureNotFound {
		t.Errorf("expected ErrFeatureNotFound, got %v", err)
	}
}

func TestDeleteFeature(t *testing.T) {
	db := NewTestDB(t)

	// Create test feature
	_, err := db.CreateFeature(CreateFeatureRequest{
		Name: "delete-test",
	})
	if err != nil {
		t.Fatalf("failed to create feature: %v", err)
	}

	// Delete
	err = db.DeleteFeature("delete-test")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Verify deleted
	_, err = db.GetFeature("delete-test")
	if err != ErrFeatureNotFound {
		t.Errorf("expected ErrFeatureNotFound, got %v", err)
	}

	// Test delete nonexistent
	err = db.DeleteFeature("nonexistent")
	if err != ErrFeatureNotFound {
		t.Errorf("expected ErrFeatureNotFound, got %v", err)
	}
}

func TestSearchFeatures(t *testing.T) {
	tests := []struct {
		name      string
		features  []CreateFeatureRequest
		query     string
		wantCount int
	}{
		{
			name: "pattern",
			features: []CreateFeatureRequest{
				{Name: "pattern-matching", DisplayName: "Pattern Matching"},
				{Name: "error-handling", DisplayName: "Error Handling"},
			},
			query:     "pattern",
			wantCount: 1,
		},
		{
			name: "error",
			features: []CreateFeatureRequest{
				{Name: "pattern-matching", DisplayName: "Pattern Matching"},
				{Name: "error-handling", DisplayName: "Error Handling"},
			},
			query:     "error",
			wantCount: 1,
		},
		{
			name: "management",
			features: []CreateFeatureRequest{
				{Name: "error-handling", DisplayName: "Error Handling"},
				{Name: "memory-management", DisplayName: "Memory Management"},
				{Name: "state-management", DisplayName: "State Management"},
			},
			query:     "management",
			wantCount: 2,
		},
		{
			name: "nonexistent",
			features: []CreateFeatureRequest{
				{Name: "test", DisplayName: "Test"},
			},
			query:     "nonexistent",
			wantCount: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db := NewTestDB(t)

			// Create test features
			for _, req := range tt.features {
				_, err := db.CreateFeature(req)
				if err != nil {
					t.Fatalf("failed to create feature: %v", err)
				}
			}

			// Search
			results, err := db.SearchFeatures(tt.query)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if len(results) != tt.wantCount {
				t.Errorf("got %d results, want %d", len(results), tt.wantCount)
			}
		})
	}
}

func TestFeatureToCompactJSON(t *testing.T) {
	feature := &Feature{
		Name:        "test",
		DisplayName: "Test Feature",
		Version:     "v0.1",
		Status:      "Planned",
	}

	json := feature.ToCompactJSON()

	if json["name"] != "test" {
		t.Errorf("name = %v, want test", json["name"])
	}
	if json["display"] != "Test Feature" {
		t.Errorf("display = %v, want Test Feature", json["display"])
	}
	if json["ver"] != "v0.1" {
		t.Errorf("ver = %v, want v0.1", json["ver"])
	}
	if json["stat"] != "Planned" {
		t.Errorf("stat = %v, want Planned", json["stat"])
	}

	// Verify empty fields omitted
	if _, exists := json["desc"]; exists {
		t.Error("desc should be omitted when empty")
	}
}
