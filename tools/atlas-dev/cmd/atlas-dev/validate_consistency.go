package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/parity"
	"github.com/spf13/cobra"
)

func validateConsistencyCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "consistency",
		Short: "Detect internal documentation conflicts",
		Long: `Check for inconsistencies across documentation:

- Feature docs vs API docs
- Spec requirements vs code implementations
- Phase requirements vs actual tests
- Cross-document contradictions

Returns conflict report with recommended resolutions.`,
		RunE: runValidateConsistency,
	}
}

func runValidateConsistency(cmd *cobra.Command, args []string) error {
	// Find project root
	projectRoot, err := findProjectRoot()
	if err != nil {
		return fmt.Errorf("failed to find project root: %w", err)
	}

	// Run parity check to get all data
	checker := parity.NewParityChecker(projectRoot)
	parityReport, err := checker.CheckParity()
	if err != nil {
		return err
	}

	// Build consistency report from parity data
	result := map[string]interface{}{
		"ok":            true,
		"conflicts":     []map[string]interface{}{},
		"conflict_cnt":  0,
	}

	conflicts := []map[string]interface{}{}

	// Check for spec-code inconsistencies
	if details, ok := parityReport.Details["spec"].(map[string]interface{}); ok {
		if mismatchCnt, ok := details["mismatch_cnt"].(int); ok && mismatchCnt > 0 {
			conflicts = append(conflicts, map[string]interface{}{
				"type":   "spec_code_conflict",
				"count":  mismatchCnt,
				"issue":  "Spec requirements don't match code implementation",
				"source": "Specification vs Code",
				"fix":    "Run 'atlas-dev validate parity --detailed' for details",
			})
		}
	}

	// Check for API-code inconsistencies
	if details, ok := parityReport.Details["api"].(map[string]interface{}); ok {
		if mismatchCnt, ok := details["mismatch_cnt"].(int); ok && mismatchCnt > 0 {
			conflicts = append(conflicts, map[string]interface{}{
				"type":   "api_code_conflict",
				"count":  mismatchCnt,
				"issue":  "API documentation doesn't match code",
				"source": "API Docs vs Code",
				"fix":    "Run 'atlas-dev api validate' for details",
			})
		}
	}

	// Check for test count inconsistencies
	if details, ok := parityReport.Details["tests"].(map[string]interface{}); ok {
		if deficit, ok := details["deficit"].(int); ok && deficit > 0 {
			conflicts = append(conflicts, map[string]interface{}{
				"type":   "test_count_conflict",
				"count":  deficit,
				"issue":  "Actual test count below phase requirements",
				"source": "Phase Requirements vs Test Files",
				"fix":    "Run 'atlas-dev validate tests' for details",
			})
		}
	}

	// Check for broken references
	if details, ok := parityReport.Details["refs"].(map[string]interface{}); ok {
		if brokenCnt, ok := details["broken_cnt"].(int); ok && brokenCnt > 0 {
			conflicts = append(conflicts, map[string]interface{}{
				"type":   "broken_references",
				"count":  brokenCnt,
				"issue":  "Documentation contains broken cross-references",
				"source": "Cross-document References",
				"fix":    "Check reference validator output",
			})
		}
	}

	// Update result
	result["conflicts"] = conflicts
	result["conflict_cnt"] = len(conflicts)
	result["ok"] = len(conflicts) == 0

	// Add message
	if len(conflicts) == 0 {
		result["msg"] = "No internal inconsistencies detected"
	} else {
		result["msg"] = fmt.Sprintf("Found %d types of inconsistencies", len(conflicts))
	}

	// Add overall health score
	result["health"] = parityReport.HealthScore

	return output.Success(result)
}
