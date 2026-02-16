package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/parity"
	"github.com/spf13/cobra"
)

var (
	parityDetailed      bool
	parityFixSuggestions bool
	parityCodeDir       string
	paritySpecDir       string
	parityAPIDir        string
	parityPhaseDir      string
	parityTestDir       string
)

func validateParityCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "parity",
		Short: "Validate parity between code, specs, API docs, and tests",
		Long: `Run comprehensive parity validation across the entire project:

- Spec-to-code parity: verify all spec requirements are implemented
- API-to-code parity: verify API docs match implementations
- Test coverage: verify test counts meet phase requirements
- Cross-references: verify all documentation links are valid

Returns health score (0-100) and detailed mismatch report.`,
		Example: `  # Run parity validation
  atlas-dev validate parity

  # With custom directories
  atlas-dev validate parity --code-dir ../crates --spec-dir ../docs/spec

  # Override directories from stdin
  echo '{"code_dir":"../crates","spec_dir":"../docs/spec"}' | atlas-dev validate parity --stdin`,
		RunE: runValidateParity,
	}

	cmd.Flags().BoolVar(&parityDetailed, "detailed", false, "Include detailed subsystem reports")
	cmd.Flags().BoolVar(&parityFixSuggestions, "fix-suggestions", true, "Include actionable fix suggestions")
	cmd.Flags().StringVar(&parityCodeDir, "code-dir", "", "Code directory (default: crates/)")
	cmd.Flags().StringVar(&paritySpecDir, "spec-dir", "", "Spec directory (default: docs/specification/)")
	cmd.Flags().StringVar(&parityAPIDir, "api-dir", "", "API directory (default: docs/api/)")
	cmd.Flags().StringVar(&parityPhaseDir, "phase-dir", "", "Phase directory (default: phases/)")
	cmd.Flags().StringVar(&parityTestDir, "test-dir", "", "Test directory (default: crates/)")

	return cmd
}

func runValidateParity(cmd *cobra.Command, args []string) error {
	// Auto-detect stdin for directory overrides
	if compose.HasStdin() {
		input, err := compose.ReadAndParseStdin()
		if err != nil {
			return err
		}

		// Extract directory fields from stdin
		if len(input.Items) > 0 {
			item := input.Items[0]
			if codeDir, ok := item["code_dir"].(string); ok && codeDir != "" {
				parityCodeDir = codeDir
			}
			if specDir, ok := item["spec_dir"].(string); ok && specDir != "" {
				paritySpecDir = specDir
			}
			if apiDir, ok := item["api_dir"].(string); ok && apiDir != "" {
				parityAPIDir = apiDir
			}
			if phaseDir, ok := item["phase_dir"].(string); ok && phaseDir != "" {
				parityPhaseDir = phaseDir
			}
			if testDir, ok := item["test_dir"].(string); ok && testDir != "" {
				parityTestDir = testDir
			}
		}
	}

	// Determine project root
	projectRoot, err := findProjectRoot()
	if err != nil {
		return fmt.Errorf("failed to find project root: %w", err)
	}

	// Create parity checker
	checker := parity.NewParityChecker(projectRoot)

	// Apply custom directories if specified
	if parityCodeDir != "" {
		checker = checker.WithCodeDir(parityCodeDir)
	}
	if paritySpecDir != "" {
		checker = checker.WithSpecDir(paritySpecDir)
	}
	if parityAPIDir != "" {
		checker = checker.WithAPIDir(parityAPIDir)
	}
	if parityPhaseDir != "" {
		checker = checker.WithPhaseDir(parityPhaseDir)
	}
	if parityTestDir != "" {
		checker = checker.WithTestDir(parityTestDir)
	}

	// Run parity check
	report, err := checker.CheckParity()
	if err != nil {
		return err
	}

	// Build output
	result := report.ToCompactJSON()

	// Add message based on health
	if report.OK {
		result["msg"] = fmt.Sprintf("Parity validation passed (health: %.1f%%)", report.HealthScore)
	} else {
		result["msg"] = fmt.Sprintf("Parity validation failed (%d errors, health: %.1f%%)",
			len(report.Errors), report.HealthScore)
	}

	// Remove fix suggestions if disabled
	if !parityFixSuggestions {
		// Remove fix field from errors and warnings
		if errors, ok := result["errors"].([]map[string]interface{}); ok {
			for i := range errors {
				delete(errors[i], "fix")
			}
		}
		if warnings, ok := result["warnings"].([]map[string]interface{}); ok {
			for i := range warnings {
				delete(warnings[i], "fix")
			}
		}
	}

	// Remove details if not requested
	if !parityDetailed {
		delete(result, "details")
	}

	// Output result
	if err := output.Success(result); err != nil {
		return err
	}

	// Exit with appropriate code
	if !report.OK {
		os.Exit(3) // Validation failed
	}

	return nil
}

// findProjectRoot finds the project root directory
func findProjectRoot() (string, error) {
	// Start from current directory
	dir, err := os.Getwd()
	if err != nil {
		return "", err
	}

	// Look for indicator files (e.g., .git, Cargo.toml, go.mod)
	for {
		// Check for .git directory
		if _, err := os.Stat(filepath.Join(dir, ".git")); err == nil {
			return dir, nil
		}

		// Check for Cargo.toml (Rust project)
		if _, err := os.Stat(filepath.Join(dir, "Cargo.toml")); err == nil {
			return dir, nil
		}

		// Check for crates directory
		if _, err := os.Stat(filepath.Join(dir, "crates")); err == nil {
			return dir, nil
		}

		// Move up one directory
		parent := filepath.Dir(dir)
		if parent == dir {
			// Reached root without finding project root
			break
		}
		dir = parent
	}

	// Default to current directory
	return os.Getwd()
}
