package main

import (
	"fmt"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/feature"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureValidateCmd() *cobra.Command {
	var (
		projectRoot string
	)

	cmd := &cobra.Command{
		Use:   "validate <name>",
		Short: "Validate a feature against codebase",
		Long:  `Validate feature documentation against actual implementation code.`,
		Example: `  # Validate feature
  atlas-dev feature validate pattern-matching

  # Validate with custom project root
  atlas-dev feature validate pattern-matching --root ../..`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

			// Parse markdown file
			markdownPath := filepath.Join("../../docs/features", name+".md")
			parsedFeature, err := feature.Parse(markdownPath)
			if err != nil {
				return fmt.Errorf("failed to parse feature file: %w", err)
			}

			// Default project root
			if projectRoot == "" {
				projectRoot = "../.."
			}

			// Validate
			report, err := feature.Validate(parsedFeature, projectRoot)
			if err != nil {
				return err
			}

			// Convert to compact JSON
			result := map[string]interface{}{
				"feature": name,
				"valid":   report.Valid,
			}

			if report.SpecRefValid {
				result["spec_ok"] = true
			} else {
				result["spec_ok"] = false
			}

			if report.APIRefValid {
				result["api_ok"] = true
			} else {
				result["api_ok"] = false
			}

			if report.ImplFileExists {
				result["impl_exists"] = true
			} else {
				result["impl_exists"] = false
			}

			if report.TestFileExists {
				result["test_exists"] = true
			} else {
				result["test_exists"] = false
			}

			if report.ExpectedFunctions > 0 {
				result["fn_match"] = report.FunctionCountMatch
				result["fn_exp"] = report.ExpectedFunctions
				result["fn_act"] = report.ActualFunctions
			}

			if report.ExpectedTests > 0 {
				result["test_match"] = report.TestCountMatch
				result["test_exp"] = report.ExpectedTests
				result["test_act"] = report.ActualTests
			}

			if len(report.Errors) > 0 {
				result["errors"] = report.Errors
			}

			if len(report.Warnings) > 0 {
				result["warnings"] = report.Warnings
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&projectRoot, "root", "../..", "Project root directory")

	return cmd
}
