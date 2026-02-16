package parity

import (
	"testing"
)

func TestSpecMatcher_ToCompactJSON(t *testing.T) {
	report := &SpecMatchReport{
		Matches:         make([]SpecMatch, 10),
		Mismatches:      make([]SpecMismatch, 3),
		Unspecified:     make([]CodeItem, 5),
		MatchPercentage: 76.9,
		TotalSpec:       13,
		TotalMatched:    10,
	}

	result := report.ToCompactJSON()

	if matchCnt, ok := result["match_cnt"].(int); !ok || matchCnt != 10 {
		t.Errorf("Expected match_cnt=10, got %v", result["match_cnt"])
	}

	if mismatchCnt, ok := result["mismatch_cnt"].(int); !ok || mismatchCnt != 3 {
		t.Errorf("Expected mismatch_cnt=3, got %v", result["mismatch_cnt"])
	}

	if matchPct, ok := result["match_pct"].(float64); !ok || matchPct != 76.9 {
		t.Errorf("Expected match_pct=76.9, got %v", result["match_pct"])
	}
}

func TestSpecMatcher_calculateMatchConfidence(t *testing.T) {
	codeAnalysis := &CodeAnalysis{}
	matcher := NewSpecMatcher("", codeAnalysis)

	tests := []struct {
		name       string
		req        SpecRequirement
		item       CodeItem
		minConf    float64
		maxConf    float64
	}{
		{
			name: "exact match",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add",
				Type:   "function",
				Public: true,
			},
			minConf: 0.9,
			maxConf: 1.0,
		},
		{
			name: "type mismatch",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add",
				Type:   "struct",
				Public: true,
			},
			minConf: 0.0,
			maxConf: 0.8,
		},
		{
			name: "name contains",
			req: SpecRequirement{
				Name: "add",
				Type: "function",
			},
			item: CodeItem{
				Name:   "add_numbers",
				Type:   "function",
				Public: true,
			},
			minConf: 0.0,
			maxConf: 0.7,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			conf := matcher.calculateMatchConfidence(tt.req, &tt.item)
			if conf < tt.minConf || conf > tt.maxConf {
				t.Errorf("Expected confidence between %.2f and %.2f, got %.2f",
					tt.minConf, tt.maxConf, conf)
			}
		})
	}
}

func TestExtractFunctionName(t *testing.T) {
	tests := []struct {
		line     string
		expected string
	}{
		{"pub fn add(x: i32) -> i32", "add"},
		{"fn subtract(x: i32, y: i32)", "subtract"},
		{"pub fn generic<T>(val: T) -> T", "generic"},
		{"fn no_params()", "no_params"},
		{"fn", ""}, // edge case
	}

	for _, tt := range tests {
		result := extractFunctionName(tt.line)
		if result != tt.expected {
			t.Errorf("extractFunctionName(%q) = %q, want %q",
				tt.line, result, tt.expected)
		}
	}
}

func TestExtractTypeName(t *testing.T) {
	tests := []struct {
		line     string
		keyword  string
		expected string
	}{
		{"pub struct Point { x: i32 }", "struct", "Point"},
		{"enum Color { Red, Green }", "enum", "Color"},
		{"pub trait Display {", "trait", "Display"},
		{"struct Generic<T> {", "struct", "Generic"},
	}

	for _, tt := range tests {
		result := extractTypeName(tt.line, tt.keyword)
		if result != tt.expected {
			t.Errorf("extractTypeName(%q, %q) = %q, want %q",
				tt.line, tt.keyword, result, tt.expected)
		}
	}
}

func TestSpecMatcher_generateFixSuggestion(t *testing.T) {
	matcher := NewSpecMatcher("", &CodeAnalysis{})

	tests := []struct {
		reqType  string
		reqName  string
		contains string
	}{
		{"function", "add", "Implement function 'add'"},
		{"struct", "Point", "Define struct 'Point'"},
		{"enum", "Color", "Define enum 'Color'"},
		{"trait", "Display", "Define trait 'Display'"},
	}

	for _, tt := range tests {
		req := SpecRequirement{
			Name:    tt.reqName,
			Type:    tt.reqType,
			Section: "test_section",
		}

		suggestion := matcher.generateFixSuggestion(req)
		if suggestion == "" {
			t.Errorf("Expected non-empty suggestion for %s", tt.reqType)
		}
	}
}

func TestSpecRequirement_Structure(t *testing.T) {
	req := SpecRequirement{
		Name:        "test_function",
		Type:        "function",
		Section:     "Core Functions",
		Description: "A test function",
		Line:        42,
	}

	if req.Name != "test_function" {
		t.Errorf("Expected Name='test_function', got '%s'", req.Name)
	}

	if req.Type != "function" {
		t.Errorf("Expected Type='function', got '%s'", req.Type)
	}

	if req.Line != 42 {
		t.Errorf("Expected Line=42, got %d", req.Line)
	}
}

func TestSpecMatch_Structure(t *testing.T) {
	match := SpecMatch{
		SpecItem:    "add",
		SpecSection: "Math Functions",
		CodeItem: &CodeItem{
			Name: "add",
			Type: "function",
		},
		MatchType:  "exact",
		Confidence: 1.0,
	}

	if match.SpecItem != "add" {
		t.Errorf("Expected SpecItem='add', got '%s'", match.SpecItem)
	}

	if match.MatchType != "exact" {
		t.Errorf("Expected MatchType='exact', got '%s'", match.MatchType)
	}

	if match.Confidence != 1.0 {
		t.Errorf("Expected Confidence=1.0, got %.2f", match.Confidence)
	}
}

func TestSpecMismatch_Structure(t *testing.T) {
	mismatch := SpecMismatch{
		SpecItem:      "multiply",
		SpecSection:   "Math Functions",
		Expected:      "fn multiply(a: i32, b: i32) -> i32",
		Issue:         "Function not implemented",
		FixSuggestion: "Implement multiply function",
		FilePath:      "spec.md",
		Line:          10,
	}

	if mismatch.SpecItem != "multiply" {
		t.Errorf("Expected SpecItem='multiply', got '%s'", mismatch.SpecItem)
	}

	if mismatch.Line != 10 {
		t.Errorf("Expected Line=10, got %d", mismatch.Line)
	}
}
