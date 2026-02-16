package parity

import (
	"fmt"
	"path/filepath"
)

// ParityError represents a parity validation error
type ParityError struct {
	Type     string // "spec_code_mismatch", "api_code_mismatch", "test_count_mismatch", "broken_reference"
	Severity string // "error", "warning"
	Source   string
	Issue    string
	Fix      string
}

// ParityReport contains comprehensive parity validation results
type ParityReport struct {
	OK           bool
	HealthScore  float64
	TotalChecks  int
	PassedChecks int
	FailedChecks int
	Errors       []ParityError
	Warnings     []ParityError
	Details      map[string]interface{}
}

// ParityChecker orchestrates all parity validation subsystems
type ParityChecker struct {
	projectRoot string
	codeDir     string
	specDir     string
	apiDir      string
	phaseDir    string
	testDir     string
	docsDir     string
}

// NewParityChecker creates a new parity checker
func NewParityChecker(projectRoot string) *ParityChecker {
	return &ParityChecker{
		projectRoot: projectRoot,
		codeDir:     filepath.Join(projectRoot, "crates"),
		specDir:     filepath.Join(projectRoot, "docs/specification"),
		apiDir:      filepath.Join(projectRoot, "docs/api"),
		phaseDir:    filepath.Join(projectRoot, "phases"),
		testDir:     filepath.Join(projectRoot, "crates"),
		docsDir:     filepath.Join(projectRoot, "docs"),
	}
}

// WithCodeDir sets custom code directory
func (c *ParityChecker) WithCodeDir(dir string) *ParityChecker {
	c.codeDir = dir
	return c
}

// WithSpecDir sets custom spec directory
func (c *ParityChecker) WithSpecDir(dir string) *ParityChecker {
	c.specDir = dir
	return c
}

// WithAPIDir sets custom API directory
func (c *ParityChecker) WithAPIDir(dir string) *ParityChecker {
	c.apiDir = dir
	return c
}

// WithPhaseDir sets custom phase directory
func (c *ParityChecker) WithPhaseDir(dir string) *ParityChecker {
	c.phaseDir = dir
	return c
}

// WithTestDir sets custom test directory
func (c *ParityChecker) WithTestDir(dir string) *ParityChecker {
	c.testDir = dir
	return c
}

// CheckParity runs comprehensive parity validation
func (c *ParityChecker) CheckParity() (*ParityReport, error) {
	report := &ParityReport{
		OK:      true,
		Details: make(map[string]interface{}),
		Errors:  []ParityError{},
		Warnings: []ParityError{},
	}

	// 1. Analyze code
	codeAnalysis, err := c.analyzeCode()
	if err != nil {
		return nil, fmt.Errorf("code analysis failed: %w", err)
	}
	report.Details["code"] = codeAnalysis.ToCompactJSON()

	// 2. Validate spec-to-code parity
	specReport, err := c.validateSpecParity(codeAnalysis)
	if err != nil {
		// Non-fatal - continue with warning
		c.addWarning(report, "spec_validation_failed", "", fmt.Sprintf("Spec validation failed: %v", err), "Check spec directory path")
	} else {
		report.Details["spec"] = specReport.ToCompactJSON()
		c.processSpecResults(report, specReport)
	}

	// 3. Validate API-to-code parity
	apiReport, err := c.validateAPIParity(codeAnalysis)
	if err != nil {
		// Non-fatal - continue with warning
		c.addWarning(report, "api_validation_failed", "", fmt.Sprintf("API validation failed: %v", err), "Check API directory path")
	} else {
		report.Details["api"] = apiReport.ToCompactJSON()
		c.processAPIResults(report, apiReport)
	}

	// 4. Validate test coverage
	testReport, err := c.validateTestCoverage()
	if err != nil {
		// Non-fatal - continue with warning
		c.addWarning(report, "test_validation_failed", "", fmt.Sprintf("Test validation failed: %v", err), "Check test directory path")
	} else {
		report.Details["tests"] = testReport.ToCompactJSON()
		c.processTestResults(report, testReport)
	}

	// 5. Validate cross-references
	refReport, err := c.validateReferences()
	if err != nil {
		// Non-fatal - continue with warning
		c.addWarning(report, "ref_validation_failed", "", fmt.Sprintf("Reference validation failed: %v", err), "Check docs directory path")
	} else {
		report.Details["refs"] = refReport.ToCompactJSON()
		c.processRefResults(report, refReport)
	}

	// Calculate health score
	c.calculateHealthScore(report)

	// Set OK flag
	report.OK = len(report.Errors) == 0

	return report, nil
}

// analyzeCode analyzes codebase
func (c *ParityChecker) analyzeCode() (*CodeAnalysis, error) {
	analyzer := NewCodeAnalyzer(c.codeDir)
	return analyzer.AnalyzeCodebase()
}

// validateSpecParity validates spec-to-code parity
func (c *ParityChecker) validateSpecParity(codeAnalysis *CodeAnalysis) (*SpecMatchReport, error) {
	matcher := NewSpecMatcher(c.specDir, codeAnalysis)
	return matcher.MatchSpecs()
}

// validateAPIParity validates API-to-code parity
func (c *ParityChecker) validateAPIParity(codeAnalysis *CodeAnalysis) (*APIMatchReport, error) {
	matcher := NewAPIMatcher(c.apiDir, codeAnalysis)
	return matcher.MatchAPIs()
}

// validateTestCoverage validates test coverage
func (c *ParityChecker) validateTestCoverage() (*TestAnalysisReport, error) {
	analyzer := NewTestAnalyzer(c.phaseDir, c.testDir)
	return analyzer.AnalyzeTests()
}

// validateReferences validates cross-references
func (c *ParityChecker) validateReferences() (*ReferenceReport, error) {
	validator := NewReferenceValidator(c.projectRoot, c.docsDir)
	return validator.ValidateReferences()
}

// processSpecResults processes spec validation results
func (c *ParityChecker) processSpecResults(report *ParityReport, specReport *SpecMatchReport) {
	report.TotalChecks += specReport.TotalSpec

	// Add errors for mismatches
	for _, mismatch := range specReport.Mismatches {
		c.addError(report, "spec_code_mismatch",
			fmt.Sprintf("%s:%d", mismatch.FilePath, mismatch.Line),
			fmt.Sprintf("Spec requirement '%s' not implemented: %s", mismatch.SpecItem, mismatch.Issue),
			mismatch.FixSuggestion)
	}

	// Add warnings for unspecified items
	for _, item := range specReport.Unspecified {
		if item.Public {
			c.addWarning(report, "code_not_specified",
				fmt.Sprintf("%s:%d", item.FilePath, item.Line),
				fmt.Sprintf("Public %s '%s' not specified", item.Type, item.Name),
				fmt.Sprintf("Add spec for '%s' or make it private", item.Name))
		}
	}

	report.PassedChecks += len(specReport.Matches)
}

// processAPIResults processes API validation results
func (c *ParityChecker) processAPIResults(report *ParityReport, apiReport *APIMatchReport) {
	report.TotalChecks += apiReport.TotalDocumented

	// Add errors for mismatches
	for _, mismatch := range apiReport.Mismatches {
		severity := "error"
		if mismatch.Type == "not_documented" {
			severity = "warning" // Not documenting is a warning, not error
		}

		if severity == "error" {
			c.addError(report, "api_code_mismatch",
				fmt.Sprintf("%s:%d", mismatch.FilePath, mismatch.Line),
				mismatch.Issue,
				mismatch.FixSuggestion)
		} else {
			c.addWarning(report, "api_code_mismatch",
				fmt.Sprintf("%s:%d", mismatch.FilePath, mismatch.Line),
				mismatch.Issue,
				mismatch.FixSuggestion)
		}
	}

	report.PassedChecks += len(apiReport.Matches)
}

// processTestResults processes test validation results
func (c *ParityChecker) processTestResults(report *ParityReport, testReport *TestAnalysisReport) {
	report.TotalChecks += len(testReport.Requirements)

	// Add errors for deficits
	for _, req := range testReport.Requirements {
		if !req.Met {
			c.addError(report, "test_count_mismatch",
				req.PhasePath,
				fmt.Sprintf("Phase '%s' requires %d tests but has %d (deficit: %d)",
					req.PhaseID, req.Required, req.Actual, req.Deficit),
				fmt.Sprintf("Add %d more tests for phase '%s'", req.Deficit, req.PhaseID))
		} else {
			report.PassedChecks++
		}
	}
}

// processRefResults processes reference validation results
func (c *ParityChecker) processRefResults(report *ParityReport, refReport *ReferenceReport) {
	report.TotalChecks += refReport.TotalRefs

	// Add errors for broken refs
	for _, broken := range refReport.BrokenRefs {
		c.addError(report, "broken_reference",
			fmt.Sprintf("%s:%d", broken.Ref.SourceFile, broken.Ref.SourceLine),
			fmt.Sprintf("Broken %s reference to '%s'", broken.ErrorType, broken.Ref.TargetPath),
			broken.FixSuggestion)
	}

	// Add warnings for orphaned docs
	for _, orphan := range refReport.OrphanedDocs {
		c.addWarning(report, "orphaned_document",
			orphan,
			"Document not referenced anywhere",
			fmt.Sprintf("Add reference to '%s' or remove if obsolete", orphan))
	}

	report.PassedChecks += refReport.ValidRefs
}

// addError adds an error to report
func (c *ParityChecker) addError(report *ParityReport, errType, source, issue, fix string) {
	report.Errors = append(report.Errors, ParityError{
		Type:     errType,
		Severity: "error",
		Source:   source,
		Issue:    issue,
		Fix:      fix,
	})
	report.FailedChecks++
}

// addWarning adds a warning to report
func (c *ParityChecker) addWarning(report *ParityReport, errType, source, issue, fix string) {
	report.Warnings = append(report.Warnings, ParityError{
		Type:     errType,
		Severity: "warning",
		Source:   source,
		Issue:    issue,
		Fix:      fix,
	})
}

// calculateHealthScore calculates overall health score (0-100)
func (c *ParityChecker) calculateHealthScore(report *ParityReport) {
	if report.TotalChecks == 0 {
		report.HealthScore = 100.0
		return
	}

	// Base score from passed checks
	passRate := float64(report.PassedChecks) / float64(report.TotalChecks) * 100.0

	// Penalties for errors and warnings
	errorPenalty := float64(len(report.Errors)) * 2.0  // Each error -2 points
	warningPenalty := float64(len(report.Warnings)) * 0.5 // Each warning -0.5 points

	score := passRate - errorPenalty - warningPenalty

	// Clamp to 0-100
	if score < 0 {
		score = 0
	}
	if score > 100 {
		score = 100
	}

	report.HealthScore = score
}

// ToCompactJSON returns compact JSON representation
func (r *ParityReport) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"ok":          r.OK,
		"health":      r.HealthScore,
		"checks":      r.TotalChecks,
		"passed":      r.PassedChecks,
		"failed":      r.FailedChecks,
		"err_cnt":     len(r.Errors),
		"warn_cnt":    len(r.Warnings),
	}

	// Include errors if any
	if len(r.Errors) > 0 {
		errorList := []map[string]interface{}{}
		for _, e := range r.Errors {
			errorList = append(errorList, map[string]interface{}{
				"type": e.Type,
				"src":  e.Source,
				"issue": e.Issue,
				"fix":  e.Fix,
			})
		}
		result["errors"] = errorList
	}

	// Include warnings if any
	if len(r.Warnings) > 0 {
		warnList := []map[string]interface{}{}
		for _, w := range r.Warnings {
			warnList = append(warnList, map[string]interface{}{
				"type": w.Type,
				"src":  w.Source,
				"issue": w.Issue,
				"fix":  w.Fix,
			})
		}
		result["warnings"] = warnList
	}

	// Include subsystem details
	if len(r.Details) > 0 {
		result["details"] = r.Details
	}

	return result
}
