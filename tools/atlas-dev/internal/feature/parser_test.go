package feature

import (
	"os"
	"path/filepath"
	"testing"
)

func TestParse(t *testing.T) {
	tests := []struct {
		name        string
		content     string
		wantName    string
		wantVersion string
		wantStatus  string
		wantErr     bool
	}{
		{
			name: "complete feature file",
			content: `# Error Handling

**Version:** v0.2
**Status:** Implemented
**Category:** core
**Spec:** spec/error-handling.md
**API:** api/error-handling.md

---

## Overview

This is a test feature for error handling.

## Functions

- error_new
- error_propagate
`,
			wantName:    "error-handling",
			wantVersion: "v0.2",
			wantStatus:  "Implemented",
			wantErr:     false,
		},
		{
			name: "minimal feature file",
			content: `# Test Feature

**Version:** v0.1
**Status:** Planned

---

## Overview

Minimal feature.
`,
			wantName:    "test-feature",
			wantVersion: "v0.1",
			wantStatus:  "Planned",
			wantErr:     false,
		},
		{
			name: "missing version",
			content: `# Test Feature

**Status:** Planned

---
`,
			wantErr: true,
		},
		{
			name: "missing status",
			content: `# Test Feature

**Version:** v0.1

---
`,
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Create temp file
			tmpDir := t.TempDir()
			tmpFile := filepath.Join(tmpDir, "test.md")
			err := os.WriteFile(tmpFile, []byte(tt.content), 0644)
			if err != nil {
				t.Fatalf("failed to create temp file: %v", err)
			}

			// Parse
			feature, err := Parse(tmpFile)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if feature.Name != tt.wantName {
				t.Errorf("name: got %s, want %s", feature.Name, tt.wantName)
			}
			if feature.Version != tt.wantVersion {
				t.Errorf("version: got %s, want %s", feature.Version, tt.wantVersion)
			}
			if feature.Status != tt.wantStatus {
				t.Errorf("status: got %s, want %s", feature.Status, tt.wantStatus)
			}
		})
	}
}

func TestGenerateFeatureName(t *testing.T) {
	tests := []struct {
		displayName string
		want        string
	}{
		{"Error Handling in Atlas", "error-handling"},
		{"Error Handling", "error-handling"},
		{"Pattern Matching", "pattern-matching"},
		{"FFI Guide", "ffi-guide"},
		{"First-Class Functions", "first-class-functions"},
	}

	for _, tt := range tests {
		t.Run(tt.displayName, func(t *testing.T) {
			got := generateFeatureName(tt.displayName)
			if got != tt.want {
				t.Errorf("generateFeatureName(%q) = %q, want %q", tt.displayName, got, tt.want)
			}
		})
	}
}

func TestExtractValue(t *testing.T) {
	tests := []struct {
		line  string
		label string
		want  string
	}{
		{"**Version:** v0.2", "**Version:**", "v0.2"},
		{"**Status:** Implemented", "**Status:**", "Implemented"},
		{"**Spec:** `spec/error.md`", "**Spec:**", "spec/error.md"},
		{"**Empty:**", "**Empty:**", ""},
	}

	for _, tt := range tests {
		t.Run(tt.line, func(t *testing.T) {
			got := extractValue(tt.line, tt.label)
			if got != tt.want {
				t.Errorf("extractValue(%q, %q) = %q, want %q", tt.line, tt.label, got, tt.want)
			}
		})
	}
}

func TestExtractInt(t *testing.T) {
	tests := []struct {
		line  string
		label string
		want  int
	}{
		{"**Test Count:** 42", "**Test Count:**", 42},
		{"**Function Count:** 10", "**Function Count:**", 10},
		{"**Empty:**", "**Empty:**", 0},
	}

	for _, tt := range tests {
		t.Run(tt.line, func(t *testing.T) {
			got := extractInt(tt.line, tt.label)
			if got != tt.want {
				t.Errorf("extractInt(%q, %q) = %d, want %d", tt.line, tt.label, got, tt.want)
			}
		})
	}
}

func TestExtractFloat(t *testing.T) {
	tests := []struct {
		line  string
		label string
		want  float64
	}{
		{"**Parity:** 95.5%", "**Parity:**", 95.5},
		{"**Parity:** 100", "**Parity:**", 100.0},
		{"**Empty:**", "**Empty:**", 0.0},
	}

	for _, tt := range tests {
		t.Run(tt.line, func(t *testing.T) {
			got := extractFloat(tt.line, tt.label)
			if got != tt.want {
				t.Errorf("extractFloat(%q, %q) = %f, want %f", tt.line, tt.label, got, tt.want)
			}
		})
	}
}

func TestParseCommaSeparated(t *testing.T) {
	tests := []struct {
		text string
		want []string
	}{
		{"one, two, three", []string{"one", "two", "three"}},
		{"single", []string{"single"}},
		{"", []string{}},
		{"  spaced  ,  items  ", []string{"spaced", "items"}},
	}

	for _, tt := range tests {
		t.Run(tt.text, func(t *testing.T) {
			got := parseCommaSeparated(tt.text)
			if len(got) != len(tt.want) {
				t.Errorf("parseCommaSeparated(%q) length = %d, want %d", tt.text, len(got), len(tt.want))
				return
			}
			for i := range got {
				if got[i] != tt.want[i] {
					t.Errorf("parseCommaSeparated(%q)[%d] = %q, want %q", tt.text, i, got[i], tt.want[i])
				}
			}
		})
	}
}
