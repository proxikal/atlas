package parity

import (
	"testing"
)

func TestAPIMatcher_ToCompactJSON(t *testing.T) {
	report := &APIMatchReport{
		Matches:          make([]APIMatch, 8),
		Mismatches:       make([]APIMismatch, 2),
		Coverage:         80.0,
		TotalDocumented:  10,
		TotalImplemented: 15,
		TotalMatched:     8,
	}

	result := report.ToCompactJSON()

	if matchCnt, ok := result["match_cnt"].(int); !ok || matchCnt != 8 {
		t.Errorf("Expected match_cnt=8, got %v", result["match_cnt"])
	}

	if coverage, ok := result["coverage"].(float64); !ok || coverage != 80.0 {
		t.Errorf("Expected coverage=80.0, got %v", result["coverage"])
	}
}

func TestNormalizeSignature(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"pub fn add(x: i32) -> i32", "add(x: i32) -> i32"},
		{"fn  subtract(x: i32,  y: i32)", "subtract(x: i32, y: i32)"},
	}

	for _, tt := range tests {
		result := normalizeSignature(tt.input)
		if result != tt.expected {
			t.Errorf("normalizeSignature(%q) = %q, want %q",
				tt.input, result, tt.expected)
		}
	}
}

func TestSimilarSignatures(t *testing.T) {
	tests := []struct {
		sig1     string
		sig2     string
		expected bool
	}{
		{"add(x: i32, y: i32)", "add(x: i32, y: i32)", true},
		{"add(x: i32, y: i32)", "add(a: i32, b: i32)", true},  // Same name, same param count
		{"add(x: i32)", "add(x: i32, y: i32)", false},         // Different param count
		{"add(x: i32)", "subtract(x: i32)", false},            // Different name
	}

	for _, tt := range tests {
		result := similarSignatures(tt.sig1, tt.sig2)
		if result != tt.expected {
			t.Errorf("similarSignatures(%q, %q) = %v, want %v",
				tt.sig1, tt.sig2, result, tt.expected)
		}
	}
}

func TestSimilarTypes(t *testing.T) {
	tests := []struct {
		type1    string
		type2    string
		expected bool
	}{
		{"i32", "i32", true},
		{"String", "string", true},       // Case insensitive
		{"Vec<T>", "vec<t>", true},       // Case insensitive
		{"Vec<T>", "Vec < T >", true},    // Whitespace normalized
		{"i32", "i64", false},
	}

	for _, tt := range tests {
		result := similarTypes(tt.type1, tt.type2)
		if result != tt.expected {
			t.Errorf("similarTypes(%q, %q) = %v, want %v",
				tt.type1, tt.type2, result, tt.expected)
		}
	}
}

func TestExtractParameterCount(t *testing.T) {
	tests := []struct {
		sig      string
		expected int
	}{
		{"fn add(x: i32, y: i32)", 2},
		{"fn no_params()", 0},
		{"fn one_param(x: i32)", 1},
		{"fn three(a: i32, b: i32, c: i32)", 3},
		{"fn invalid", 0},
	}

	for _, tt := range tests {
		result := extractParameterCount(tt.sig)
		if result != tt.expected {
			t.Errorf("extractParameterCount(%q) = %d, want %d",
				tt.sig, result, tt.expected)
		}
	}
}

func TestAPIMatch_Structure(t *testing.T) {
	match := APIMatch{
		APIFunc: nil,  // Would be actual Function in real usage
		CodeItem: &CodeItem{
			Name: "test_fn",
		},
		Verified: true,
		Issues:   []string{},
	}

	if !match.Verified {
		t.Error("Expected Verified=true")
	}

	if len(match.Issues) != 0 {
		t.Errorf("Expected 0 issues, got %d", len(match.Issues))
	}
}

func TestAPIMismatch_Structure(t *testing.T) {
	mismatch := APIMismatch{
		Type:          "not_implemented",
		APIFunc:       "test_function",
		Expected:      "fn test_function()",
		Issue:         "Function not found",
		FixSuggestion: "Implement the function",
		FilePath:      "api.md",
		Line:          20,
	}

	if mismatch.Type != "not_implemented" {
		t.Errorf("Expected Type='not_implemented', got '%s'", mismatch.Type)
	}

	if mismatch.Line != 20 {
		t.Errorf("Expected Line=20, got %d", mismatch.Line)
	}
}
