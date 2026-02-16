package parity

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
)

// TestRequirement represents test count requirement from phase
type TestRequirement struct {
	PhasePath    string
	PhaseID      string
	Category     string
	Required     int
	Actual       int
	Deficit      int
	Met          bool
	TestFiles    []string
}

// TestAnalysisReport contains test validation results
type TestAnalysisReport struct {
	Requirements  []TestRequirement
	TotalRequired int
	TotalActual   int
	TotalMet      int
	TotalDeficit  int
	Coverage      float64
}

// TestAnalyzer validates test coverage against requirements
type TestAnalyzer struct {
	phaseDir string
	testDir  string
}

// NewTestAnalyzer creates a new test analyzer
func NewTestAnalyzer(phaseDir, testDir string) *TestAnalyzer {
	return &TestAnalyzer{
		phaseDir: phaseDir,
		testDir:  testDir,
	}
}

// AnalyzeTests analyzes test coverage vs requirements
func (a *TestAnalyzer) AnalyzeTests() (*TestAnalysisReport, error) {
	report := &TestAnalysisReport{
		Requirements: []TestRequirement{},
	}

	// Find all phase files
	phaseFiles, err := a.findPhaseFiles()
	if err != nil {
		return nil, fmt.Errorf("failed to find phase files: %w", err)
	}

	// Extract test requirements from each phase
	for _, phaseFile := range phaseFiles {
		req, err := a.extractTestRequirement(phaseFile)
		if err != nil {
			// Skip phases without test requirements
			continue
		}

		// Count actual tests for this phase
		actualTests, testFiles := a.countTests(req.PhaseID, req.Category)
		req.Actual = actualTests
		req.TestFiles = testFiles
		req.Deficit = req.Required - req.Actual
		req.Met = req.Actual >= req.Required

		report.Requirements = append(report.Requirements, *req)
		report.TotalRequired += req.Required
		report.TotalActual += req.Actual
		if req.Met {
			report.TotalMet++
		}
	}

	// Calculate deficit
	if report.TotalActual < report.TotalRequired {
		report.TotalDeficit = report.TotalRequired - report.TotalActual
	}

	// Calculate coverage
	if report.TotalRequired > 0 {
		report.Coverage = float64(report.TotalActual) / float64(report.TotalRequired) * 100.0
		if report.Coverage > 100.0 {
			report.Coverage = 100.0
		}
	}

	return report, nil
}

// findPhaseFiles finds all phase markdown files
func (a *TestAnalyzer) findPhaseFiles() ([]string, error) {
	var files []string

	err := filepath.Walk(a.phaseDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() && strings.HasSuffix(path, ".md") {
			files = append(files, path)
		}

		return nil
	})

	return files, err
}

// extractTestRequirement extracts test requirement from phase file
func (a *TestAnalyzer) extractTestRequirement(phasePath string) (*TestRequirement, error) {
	file, err := os.Open(phasePath)
	if err != nil {
		return nil, err
	}
	defer func() { _ = file.Close() }()

	req := &TestRequirement{
		PhasePath: phasePath,
	}

	// Extract phase ID and category from path
	// e.g., phases/stdlib/phase-07b.md -> phase-07b, stdlib
	parts := strings.Split(filepath.ToSlash(phasePath), "/")
	if len(parts) >= 2 {
		req.Category = parts[len(parts)-2]
		fileName := parts[len(parts)-1]
		req.PhaseID = strings.TrimSuffix(fileName, ".md")
	}

	scanner := bufio.NewScanner(file)

	// Patterns to match test requirements
	// "Minimum test count: 35 tests"
	// "**Minimum test count:** 40 tests"
	// "Tests required: 25"
	patterns := []*regexp.Regexp{
		regexp.MustCompile(`(?i)minimum\s+test\s+count:\s*(\d+)`),
		regexp.MustCompile(`(?i)tests?\s+required:\s*(\d+)`),
		regexp.MustCompile(`(?i)\*\*minimum\s+test\s+count:\*\*\s*(\d+)`),
		regexp.MustCompile(`(?i)target:\s*(\d+)\+?\s+tests?`),
	}

	found := false
	for scanner.Scan() {
		line := scanner.Text()

		for _, pattern := range patterns {
			matches := pattern.FindStringSubmatch(line)
			if len(matches) > 1 {
				count, err := strconv.Atoi(matches[1])
				if err == nil {
					req.Required = count
					found = true
					break
				}
			}
		}

		if found {
			break
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, err
	}

	if !found {
		return nil, fmt.Errorf("no test requirement found")
	}

	return req, nil
}

// countTests counts actual tests for a phase
func (a *TestAnalyzer) countTests(phaseID, category string) (int, []string) {
	if a.testDir == "" {
		return 0, []string{}
	}

	totalTests := 0
	testFiles := []string{}

	// Pattern to match test functions
	testPattern := regexp.MustCompile(`^\s*#\[test\]`)
	fnPattern := regexp.MustCompile(`^\s*fn\s+test_`)

	// Walk test directory
	_ = filepath.Walk(a.testDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}

		// Only process .rs test files
		if info.IsDir() || !strings.HasSuffix(path, ".rs") {
			return nil
		}

		// Check if this file is related to the phase/category
		// This is a simple heuristic - might need refinement
		relPath := strings.ToLower(filepath.ToSlash(path))
		categoryMatch := strings.Contains(relPath, strings.ToLower(category))
		phaseMatch := strings.Contains(relPath, strings.ToLower(phaseID))

		// Count tests in file
		count := a.countTestsInFile(path, testPattern, fnPattern)
		if count > 0 && (categoryMatch || phaseMatch) {
			totalTests += count
			testFiles = append(testFiles, path)
		}

		return nil
	})

	return totalTests, testFiles
}

// countTestsInFile counts test functions in a single file
func (a *TestAnalyzer) countTestsInFile(path string, testPattern, fnPattern *regexp.Regexp) int {
	file, err := os.Open(path)
	if err != nil {
		return 0
	}
	defer func() { _ = file.Close() }()

	count := 0
	scanner := bufio.NewScanner(file)
	nextIsTest := false

	for scanner.Scan() {
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Check for #[test] attribute
		if testPattern.MatchString(trimmed) {
			nextIsTest = true
			continue
		}

		// Check for test function (either after #[test] or fn test_*)
		if nextIsTest || fnPattern.MatchString(trimmed) {
			if strings.Contains(trimmed, "fn ") {
				count++
				nextIsTest = false
			}
		}
	}

	return count
}

// GetDeficits returns requirements with deficits
func (r *TestAnalysisReport) GetDeficits() []TestRequirement {
	deficits := []TestRequirement{}
	for _, req := range r.Requirements {
		if !req.Met {
			deficits = append(deficits, req)
		}
	}
	return deficits
}

// ToCompactJSON returns compact JSON representation
func (r *TestAnalysisReport) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"required":   r.TotalRequired,
		"actual":     r.TotalActual,
		"coverage":   r.Coverage,
		"met_cnt":    r.TotalMet,
		"deficit":    r.TotalDeficit,
		"total_reqs": len(r.Requirements),
	}

	// Include deficits if any
	deficits := r.GetDeficits()
	if len(deficits) > 0 {
		deficitList := []map[string]interface{}{}
		for _, d := range deficits {
			deficitList = append(deficitList, map[string]interface{}{
				"phase":   d.PhaseID,
				"cat":     d.Category,
				"req":     d.Required,
				"actual":  d.Actual,
				"deficit": d.Deficit,
			})
		}
		result["deficits"] = deficitList
	}

	return result
}
