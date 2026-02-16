package main

import (
	"fmt"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/parity"
	"github.com/spf13/cobra"
)

func validateTestsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tests",
		Short: "Validate test coverage against phase requirements",
		Long: `Check test coverage requirements from phase files:

- Parse phase files for test count requirements
- Count actual tests per phase/module
- Compare required vs actual
- Report deficits with file locations

Returns phases meeting requirements vs those with deficits.`,
		RunE: runValidateTests,
	}

	return cmd
}

func runValidateTests(cmd *cobra.Command, args []string) error {
	// Find project root
	projectRoot, err := findProjectRoot()
	if err != nil {
		return fmt.Errorf("failed to find project root: %w", err)
	}

	// Setup paths
	phaseDir := filepath.Join(projectRoot, "phases")
	testDir := filepath.Join(projectRoot, "crates")

	// Create test analyzer
	analyzer := parity.NewTestAnalyzer(phaseDir, testDir)

	// Analyze tests
	report, err := analyzer.AnalyzeTests()
	if err != nil {
		return err
	}

	// Build output
	result := report.ToCompactJSON()

	// Add summary message
	deficits := report.GetDeficits()
	if len(deficits) == 0 {
		result["msg"] = fmt.Sprintf("All test requirements met (%.1f%% coverage)", report.Coverage)
	} else {
		result["msg"] = fmt.Sprintf("%d phases have insufficient tests (%.1f%% coverage)",
			len(deficits), report.Coverage)
	}

	// Include deficit details
	if len(deficits) > 0 {
		deficitDetails := []map[string]interface{}{}
		for _, d := range deficits {
			detail := map[string]interface{}{
				"phase":   d.PhaseID,
				"cat":     d.Category,
				"path":    d.PhasePath,
				"req":     d.Required,
				"actual":  d.Actual,
				"deficit": d.Deficit,
			}
			if len(d.TestFiles) > 0 {
				detail["test_files"] = d.TestFiles
			}
			deficitDetails = append(deficitDetails, detail)
		}
		result["deficit_details"] = deficitDetails
	}

	return output.Success(result)
}
