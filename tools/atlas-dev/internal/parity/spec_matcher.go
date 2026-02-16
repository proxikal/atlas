package parity

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/spec"
)

// SpecMatch represents a match between spec and code
type SpecMatch struct {
	SpecItem     string
	SpecSection  string
	CodeItem     *CodeItem
	MatchType    string // "exact", "partial", "none"
	Confidence   float64 // 0.0 to 1.0
}

// SpecMismatch represents a spec requirement without implementation
type SpecMismatch struct {
	SpecItem    string
	SpecSection string
	Expected    string
	Issue       string
	FixSuggestion string
	FilePath    string
	Line        int
}

// SpecMatchReport contains the complete spec-to-code matching results
type SpecMatchReport struct {
	Matches         []SpecMatch
	Mismatches      []SpecMismatch
	Unspecified     []CodeItem // Code items without spec
	MatchPercentage float64
	TotalSpec       int
	TotalMatched    int
}

// SpecMatcher matches spec requirements to code
type SpecMatcher struct {
	specDir  string
	codeAnalysis *CodeAnalysis
}

// NewSpecMatcher creates a new spec matcher
func NewSpecMatcher(specDir string, codeAnalysis *CodeAnalysis) *SpecMatcher {
	return &SpecMatcher{
		specDir:      specDir,
		codeAnalysis: codeAnalysis,
	}
}

// MatchSpecs matches all specs to code
func (m *SpecMatcher) MatchSpecs() (*SpecMatchReport, error) {
	report := &SpecMatchReport{
		Matches:     []SpecMatch{},
		Mismatches:  []SpecMismatch{},
		Unspecified: []CodeItem{},
	}

	// Find all spec files
	specFiles, err := filepath.Glob(filepath.Join(m.specDir, "*.md"))
	if err != nil {
		return nil, fmt.Errorf("failed to glob spec files: %w", err)
	}

	// Parse and match each spec
	for _, specFile := range specFiles {
		if err := m.matchSpecFile(specFile, report); err != nil {
			// Log error but continue
			continue
		}
	}

	// Calculate match percentage
	report.TotalSpec = len(report.Matches) + len(report.Mismatches)
	report.TotalMatched = len(report.Matches)
	if report.TotalSpec > 0 {
		report.MatchPercentage = float64(report.TotalMatched) / float64(report.TotalSpec) * 100.0
	}

	// Find unspecified code items (public items without spec)
	m.findUnspecified(report)

	return report, nil
}

// matchSpecFile matches a single spec file to code
func (m *SpecMatcher) matchSpecFile(specFile string, report *SpecMatchReport) error {
	// Parse spec
	parsedSpec, err := spec.Parse(specFile)
	if err != nil {
		return fmt.Errorf("failed to parse spec: %w", err)
	}

	// Extract requirements from spec
	requirements := m.extractRequirements(parsedSpec)

	// Match each requirement to code
	for _, req := range requirements {
		if match := m.findCodeMatch(req); match != nil {
			report.Matches = append(report.Matches, *match)
		} else {
			// No match found - create mismatch
			mismatch := SpecMismatch{
				SpecItem:      req.Name,
				SpecSection:   req.Section,
				Expected:      req.Description,
				Issue:         "Spec requirement not implemented in code",
				FixSuggestion: m.generateFixSuggestion(req),
				FilePath:      specFile,
				Line:          req.Line,
			}
			report.Mismatches = append(report.Mismatches, mismatch)
		}
	}

	return nil
}

// SpecRequirement represents a requirement extracted from spec
type SpecRequirement struct {
	Name        string
	Type        string // "function", "struct", "enum", "trait"
	Section     string
	Description string
	Line        int
}

// extractRequirements extracts requirements from spec
func (m *SpecMatcher) extractRequirements(s *spec.Spec) []SpecRequirement {
	requirements := []SpecRequirement{}

	// Look for code blocks that define interfaces
	for _, block := range s.CodeBlocks {
		if block.Language == "atlas" || block.Language == "rust" {
			reqs := m.parseCodeBlock(block)
			requirements = append(requirements, reqs...)
		}
	}

	// Look for explicit type definitions in sections
	for _, section := range s.Sections {
		reqs := m.extractFromSection(section)
		requirements = append(requirements, reqs...)
	}

	return requirements
}

// parseCodeBlock extracts requirements from code blocks
func (m *SpecMatcher) parseCodeBlock(block *spec.CodeBlock) []SpecRequirement {
	requirements := []SpecRequirement{}
	lines := strings.Split(block.Code, "\n")

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Match function definitions: fn name(...) -> ReturnType
		if strings.HasPrefix(trimmed, "fn ") || strings.HasPrefix(trimmed, "pub fn ") {
			name := extractFunctionName(trimmed)
			if name != "" {
				requirements = append(requirements, SpecRequirement{
					Name:        name,
					Type:        "function",
					Section:     block.Section,
					Description: trimmed,
					Line:        0,
				})
			}
		}

		// Match struct definitions
		if strings.HasPrefix(trimmed, "struct ") || strings.HasPrefix(trimmed, "pub struct ") {
			name := extractTypeName(trimmed, "struct")
			if name != "" {
				requirements = append(requirements, SpecRequirement{
					Name:        name,
					Type:        "struct",
					Section:     block.Section,
					Description: trimmed,
					Line:        0,
				})
			}
		}

		// Match enum definitions
		if strings.HasPrefix(trimmed, "enum ") || strings.HasPrefix(trimmed, "pub enum ") {
			name := extractTypeName(trimmed, "enum")
			if name != "" {
				requirements = append(requirements, SpecRequirement{
					Name:        name,
					Type:        "enum",
					Section:     block.Section,
					Description: trimmed,
					Line:        0,
				})
			}
		}

		// Match trait definitions
		if strings.HasPrefix(trimmed, "trait ") || strings.HasPrefix(trimmed, "pub trait ") {
			name := extractTypeName(trimmed, "trait")
			if name != "" {
				requirements = append(requirements, SpecRequirement{
					Name:        name,
					Type:        "trait",
					Section:     block.Section,
					Description: trimmed,
					Line:        0,
				})
			}
		}
	}

	return requirements
}

// extractFromSection extracts requirements from section content
func (m *SpecMatcher) extractFromSection(section *spec.Section) []SpecRequirement {
	requirements := []SpecRequirement{}

	// Recursively process subsections
	for _, subsection := range section.Subsections {
		reqs := m.extractFromSection(subsection)
		requirements = append(requirements, reqs...)
	}

	return requirements
}

// findCodeMatch finds a matching code item for a requirement
func (m *SpecMatcher) findCodeMatch(req SpecRequirement) *SpecMatch {
	var bestMatch *CodeItem
	bestConfidence := 0.0

	// Search appropriate code items based on type
	var items []CodeItem
	switch req.Type {
	case "function":
		items = m.codeAnalysis.Functions
	case "struct":
		items = m.codeAnalysis.Structs
	case "enum":
		items = m.codeAnalysis.Enums
	case "trait":
		items = m.codeAnalysis.Traits
	default:
		// Search all items
		items = append(items, m.codeAnalysis.Functions...)
		items = append(items, m.codeAnalysis.Structs...)
		items = append(items, m.codeAnalysis.Enums...)
		items = append(items, m.codeAnalysis.Traits...)
	}

	// Find best match
	for i := range items {
		item := &items[i]
		confidence := m.calculateMatchConfidence(req, item)
		if confidence > bestConfidence {
			bestConfidence = confidence
			bestMatch = item
		}
	}

	// Require at least 70% confidence for a match
	if bestConfidence < 0.7 {
		return nil
	}

	matchType := "exact"
	if bestConfidence < 1.0 {
		matchType = "partial"
	}

	return &SpecMatch{
		SpecItem:    req.Name,
		SpecSection: req.Section,
		CodeItem:    bestMatch,
		MatchType:   matchType,
		Confidence:  bestConfidence,
	}
}

// calculateMatchConfidence calculates confidence that code matches spec requirement
func (m *SpecMatcher) calculateMatchConfidence(req SpecRequirement, item *CodeItem) float64 {
	confidence := 0.0

	// Name match (most important)
	if strings.EqualFold(req.Name, item.Name) {
		confidence += 0.6
	} else if strings.Contains(strings.ToLower(item.Name), strings.ToLower(req.Name)) {
		confidence += 0.3
	} else if strings.Contains(strings.ToLower(req.Name), strings.ToLower(item.Name)) {
		confidence += 0.2
	}

	// Type match
	if req.Type == item.Type {
		confidence += 0.3
	}

	// Visibility (public items are more likely to be specified)
	if item.Public {
		confidence += 0.1
	}

	return confidence
}

// generateFixSuggestion generates a suggestion for implementing missing requirement
func (m *SpecMatcher) generateFixSuggestion(req SpecRequirement) string {
	switch req.Type {
	case "function":
		return fmt.Sprintf("Implement function '%s' as specified in section '%s'", req.Name, req.Section)
	case "struct":
		return fmt.Sprintf("Define struct '%s' as specified in section '%s'", req.Name, req.Section)
	case "enum":
		return fmt.Sprintf("Define enum '%s' as specified in section '%s'", req.Name, req.Section)
	case "trait":
		return fmt.Sprintf("Define trait '%s' as specified in section '%s'", req.Name, req.Section)
	default:
		return fmt.Sprintf("Implement '%s' as specified in section '%s'", req.Name, req.Section)
	}
}

// findUnspecified finds public code items without spec
func (m *SpecMatcher) findUnspecified(report *SpecMatchReport) {
	// Build set of matched items
	matched := make(map[string]bool)
	for _, match := range report.Matches {
		if match.CodeItem != nil {
			key := fmt.Sprintf("%s:%s", match.CodeItem.Type, match.CodeItem.Name)
			matched[key] = true
		}
	}

	// Find unmatched public items
	checkUnmatched := func(items []CodeItem) {
		for _, item := range items {
			if !item.Public {
				continue
			}
			key := fmt.Sprintf("%s:%s", item.Type, item.Name)
			if !matched[key] {
				report.Unspecified = append(report.Unspecified, item)
			}
		}
	}

	checkUnmatched(m.codeAnalysis.Functions)
	checkUnmatched(m.codeAnalysis.Structs)
	checkUnmatched(m.codeAnalysis.Enums)
	checkUnmatched(m.codeAnalysis.Traits)
}

// Helper functions

func extractFunctionName(line string) string {
	// Remove pub, fn keywords
	line = strings.TrimPrefix(line, "pub ")
	line = strings.TrimPrefix(line, "fn ")
	// Get name (up to < or ()
	if idx := strings.IndexAny(line, "(<"); idx != -1 {
		return strings.TrimSpace(line[:idx])
	}
	return ""
}

func extractTypeName(line, keyword string) string {
	// Remove pub, keyword
	line = strings.TrimPrefix(line, "pub ")
	line = strings.TrimPrefix(line, keyword+" ")
	// Get name (up to < or { or whitespace)
	if idx := strings.IndexAny(line, "<{ \t"); idx != -1 {
		return strings.TrimSpace(line[:idx])
	}
	return strings.TrimSpace(line)
}

// ToCompactJSON returns compact JSON representation
func (r *SpecMatchReport) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"match_cnt":   len(r.Matches),
		"mismatch_cnt": len(r.Mismatches),
		"unspec_cnt":  len(r.Unspecified),
		"match_pct":   r.MatchPercentage,
		"tot_spec":    r.TotalSpec,
	}
}
