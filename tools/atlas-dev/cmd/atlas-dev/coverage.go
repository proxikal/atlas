package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func coverageCmd() *cobra.Command {
	var category string

	cmd := &cobra.Command{
		Use:   "test-coverage",
		Short: "Show test coverage statistics",
		Long:  `Display aggregated test counts and coverage percentages.`,
		Example: `  # Show overall coverage
  atlas-dev test-coverage

  # Show coverage for specific category
  atlas-dev test-coverage --category stdlib`,
		RunE: func(cmd *cobra.Command, args []string) error {
			coverage, err := database.GetTestCoverage(category)
			if err != nil {
				return err
			}

			result := coverage.ToCompactJSON()
			if category != "" {
				result["cat"] = category
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")

	return cmd
}
