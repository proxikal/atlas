package parity

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/api"
)

// APIMatch represents a successful API-to-code match
type APIMatch struct {
	APIFunc  *api.Function
	CodeItem *CodeItem
	Verified bool
	Issues   []string
}

// APIMismatch represents an API documentation issue
type APIMismatch struct {
	Type          string // "not_implemented", "not_documented", "signature_diff"
	APIFunc       string
	CodeItem      string
	Expected      string
	Actual        string
	Issue         string
	FixSuggestion string
	FilePath      string
	Line          int
}

// APIMatchReport contains complete API-to-code matching results
type APIMatchReport struct {
	Matches          []APIMatch
	Mismatches       []APIMismatch
	Coverage         float64
	TotalDocumented  int
	TotalImplemented int
	TotalMatched     int
}

// APIMatcher matches API documentation to code
type APIMatcher struct {
	apiDir       string
	codeAnalysis *CodeAnalysis
}

// NewAPIMatcher creates a new API matcher
func NewAPIMatcher(apiDir string, codeAnalysis *CodeAnalysis) *APIMatcher {
	return &APIMatcher{
		apiDir:       apiDir,
		codeAnalysis: codeAnalysis,
	}
}

// MatchAPIs matches all API docs to code
func (m *APIMatcher) MatchAPIs() (*APIMatchReport, error) {
	report := &APIMatchReport{
		Matches:    []APIMatch{},
		Mismatches: []APIMismatch{},
	}

	// Find all API doc files
	apiFiles, err := filepath.Glob(filepath.Join(m.apiDir, "*.md"))
	if err != nil {
		return nil, fmt.Errorf("failed to glob API files: %w", err)
	}

	// Parse and match each API doc
	for _, apiFile := range apiFiles {
		if err := m.matchAPIFile(apiFile, report); err != nil {
			// Log error but continue
			continue
		}
	}

	// Find undocumented public functions
	m.findUndocumented(report)

	// Calculate coverage
	report.TotalDocumented = len(report.Matches) + len(report.Mismatches)
	report.TotalImplemented = len(m.codeAnalysis.Functions)
	report.TotalMatched = len(report.Matches)

	if report.TotalDocumented > 0 {
		report.Coverage = float64(report.TotalMatched) / float64(report.TotalDocumented) * 100.0
	}

	return report, nil
}

// matchAPIFile matches a single API doc file to code
func (m *APIMatcher) matchAPIFile(apiFile string, report *APIMatchReport) error {
	// Parse API doc
	apiDoc, err := api.Parse(apiFile)
	if err != nil {
		return fmt.Errorf("failed to parse API doc: %w", err)
	}

	// Match each documented function to code
	for _, apiFunc := range apiDoc.Functions {
		if match := m.findImplementation(apiFunc); match != nil {
			// Verify signature matches
			issues := m.verifySignature(apiFunc, match)
			apiMatch := APIMatch{
				APIFunc:  apiFunc,
				CodeItem: match,
				Verified: len(issues) == 0,
				Issues:   issues,
			}
			report.Matches = append(report.Matches, apiMatch)

			// Add signature mismatches
			for _, issue := range issues {
				mismatch := APIMismatch{
					Type:          "signature_diff",
					APIFunc:       apiFunc.Name,
					CodeItem:      match.Name,
					Expected:      apiFunc.Signature,
					Actual:        match.Signature,
					Issue:         issue,
					FixSuggestion: fmt.Sprintf("Update API docs or code signature for '%s'", apiFunc.Name),
					FilePath:      apiFile,
					Line:          0,
				}
				report.Mismatches = append(report.Mismatches, mismatch)
			}
		} else {
			// No implementation found
			mismatch := APIMismatch{
				Type:          "not_implemented",
				APIFunc:       apiFunc.Name,
				Expected:      apiFunc.Signature,
				Issue:         "API function documented but not implemented",
				FixSuggestion: fmt.Sprintf("Implement function '%s' or remove from API docs", apiFunc.Name),
				FilePath:      apiFile,
				Line:          0,
			}
			report.Mismatches = append(report.Mismatches, mismatch)
		}
	}

	return nil
}

// findImplementation finds code implementation for API function
func (m *APIMatcher) findImplementation(apiFunc *api.Function) *CodeItem {
	// First try exact name match on public functions
	for i := range m.codeAnalysis.Functions {
		fn := &m.codeAnalysis.Functions[i]
		if fn.Public && fn.Name == apiFunc.Name {
			return fn
		}
	}

	// Try case-insensitive match
	for i := range m.codeAnalysis.Functions {
		fn := &m.codeAnalysis.Functions[i]
		if fn.Public && strings.EqualFold(fn.Name, apiFunc.Name) {
			return fn
		}
	}

	// Try partial match (API name might include module prefix)
	for i := range m.codeAnalysis.Functions {
		fn := &m.codeAnalysis.Functions[i]
		if fn.Public && strings.Contains(apiFunc.Name, fn.Name) {
			return fn
		}
	}

	return nil
}

// verifySignature compares API signature to code signature
func (m *APIMatcher) verifySignature(apiFunc *api.Function, codeItem *CodeItem) []string {
	issues := []string{}

	// Extract signature details from code
	codeReturns := ""
	if details, ok := codeItem.Details["returns"].(string); ok {
		codeReturns = details
	}

	// Normalize signatures for comparison
	apiSig := normalizeSignature(apiFunc.Signature)
	codeSig := normalizeSignature(codeItem.Signature)

	// Check if signatures are similar (allowing for minor differences)
	if !similarSignatures(apiSig, codeSig) {
		issues = append(issues, fmt.Sprintf("Signature mismatch: API='%s' vs Code='%s'",
			apiFunc.Signature, codeItem.Signature))
	}

	// Check return type if specified
	if apiFunc.Returns != "" && codeReturns != "" {
		apiRet := normalizeSignature(apiFunc.Returns)
		codeRet := normalizeSignature(codeReturns)
		if apiRet != codeRet && !similarTypes(apiRet, codeRet) {
			issues = append(issues, fmt.Sprintf("Return type mismatch: API='%s' vs Code='%s'",
				apiFunc.Returns, codeReturns))
		}
	}

	// Check visibility (API docs should only document public functions)
	if !codeItem.Public {
		issues = append(issues, "Function is private but documented in API")
	}

	return issues
}

// findUndocumented finds public functions not documented in API
func (m *APIMatcher) findUndocumented(report *APIMatchReport) {
	// Build set of documented functions
	documented := make(map[string]bool)
	for _, match := range report.Matches {
		if match.CodeItem != nil {
			documented[match.CodeItem.Name] = true
		}
	}

	// Find undocumented public functions
	for i := range m.codeAnalysis.Functions {
		fn := &m.codeAnalysis.Functions[i]
		if !fn.Public {
			continue
		}

		if !documented[fn.Name] {
			mismatch := APIMismatch{
				Type:          "not_documented",
				CodeItem:      fn.Name,
				Actual:        fn.Signature,
				Issue:         "Public function not documented in API",
				FixSuggestion: fmt.Sprintf("Add API documentation for '%s' in %s:%d", fn.Name, fn.FilePath, fn.Line),
				FilePath:      fn.FilePath,
				Line:          fn.Line,
			}
			report.Mismatches = append(report.Mismatches, mismatch)
		}
	}
}

// Helper functions

// normalizeSignature removes whitespace and standardizes signature format
func normalizeSignature(sig string) string {
	// Remove extra whitespace
	sig = strings.Join(strings.Fields(sig), " ")
	// Remove pub/fn keywords for comparison
	sig = strings.TrimPrefix(sig, "pub ")
	sig = strings.TrimPrefix(sig, "fn ")
	// Lowercase for case-insensitive comparison
	return strings.ToLower(sig)
}

// similarSignatures checks if two signatures are similar (fuzzy match)
func similarSignatures(sig1, sig2 string) bool {
	// Exact match
	if sig1 == sig2 {
		return true
	}

	// Extract function name from both
	name1 := extractFunctionName(sig1)
	name2 := extractFunctionName(sig2)

	// If names don't match, not similar
	if name1 != name2 {
		return false
	}

	// Check if parameter lists are similar (ignoring type details)
	params1 := extractParameterCount(sig1)
	params2 := extractParameterCount(sig2)

	return params1 == params2
}

// similarTypes checks if two type strings are similar
func similarTypes(type1, type2 string) bool {
	// Normalize types
	type1 = strings.ToLower(strings.TrimSpace(type1))
	type2 = strings.ToLower(strings.TrimSpace(type2))

	// Exact match
	if type1 == type2 {
		return true
	}

	// Check for common variations
	// e.g., "String" vs "string", "Vec<T>" vs "vec<T>"
	type1 = strings.ReplaceAll(type1, " ", "")
	type2 = strings.ReplaceAll(type2, " ", "")

	return type1 == type2
}

// extractParameterCount extracts number of parameters from signature
func extractParameterCount(sig string) int {
	// Find parameter list (between first ( and ))
	start := strings.Index(sig, "(")
	end := strings.Index(sig, ")")
	if start == -1 || end == -1 || start >= end {
		return 0
	}

	params := sig[start+1 : end]
	params = strings.TrimSpace(params)
	if params == "" {
		return 0
	}

	// Count commas + 1 (simple heuristic)
	return strings.Count(params, ",") + 1
}

// ToCompactJSON returns compact JSON representation
func (r *APIMatchReport) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"match_cnt":    len(r.Matches),
		"mismatch_cnt": len(r.Mismatches),
		"coverage":     r.Coverage,
		"documented":   r.TotalDocumented,
		"implemented":  r.TotalImplemented,
	}
}
