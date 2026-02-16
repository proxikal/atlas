package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func validateCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "validate",
		Short: "Validate database consistency",
		Long: `Check database for consistency issues:
- Category counts match actual phase counts
- Percentages calculated correctly
- Metadata is accurate
- No orphaned phases
- No invalid statuses
- All triggers exist`,
		RunE: func(cmd *cobra.Command, args []string) error {
			report, err := database.Validate()
			if err != nil {
				return err
			}

			result := report.ToCompactJSON()
			if report.OK {
				result["msg"] = "Database is consistent"
			}

			return output.Success(result)
		},
	}
}
