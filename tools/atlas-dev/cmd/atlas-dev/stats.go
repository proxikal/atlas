package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func statsCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "stats",
		Short: "Show velocity and completion estimates",
		Long:  `Calculate development velocity (phases per day/week) and project estimated completion date.`,
		Example: `  # Show project statistics
  atlas-dev stats`,
		RunE: func(cmd *cobra.Command, args []string) error {
			stats, err := database.GetStats()
			if err != nil {
				return err
			}

			return output.Success(stats.ToCompactJSON())
		},
	}
}
