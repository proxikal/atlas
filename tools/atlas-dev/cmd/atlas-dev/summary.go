package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func summaryCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "summary",
		Short: "Show comprehensive project summary",
		Long:  `Display project progress dashboard with category breakdowns, current/next phases, and blocked count.`,
		Example: `  # Show project summary
  atlas-dev summary`,
		RunE: func(cmd *cobra.Command, args []string) error {
			summary, err := database.GetSummary()
			if err != nil {
				return err
			}

			return output.Success(summary.ToCompactJSON())
		},
	}
}
