package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/feature"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureValidateCmd() *cobra.Command {

	cmd := &cobra.Command{
		Use:   "validate <name>",
		Short: "Validate a feature against codebase",
		Long:  `Validate feature documentation against actual implementation code.`,
		Example: `  # Validate feature
  atlas-dev feature validate pattern-matching

  # Validate with custom project root
  atlas-dev feature validate pattern-matching --root ../..

  # Validate from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature validate`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				name, err = compose.ExtractFirstString(input, "name")
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("feature name required")
				}
				name = args[0]
			}

			// Parse markdown file
			markdownPath := filepath.Join("../../docs/features", name+".md")
			parsedFeature, err := feature.Parse(markdownPath)
			if err != nil {
				return fmt.Errorf("failed to parse feature file: %w", err)
			}

			// Use current working directory as project root
			projectRoot, err := os.Getwd()
			if err != nil {
				return fmt.Errorf("failed to get working directory: %w", err)
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

	return cmd
}
