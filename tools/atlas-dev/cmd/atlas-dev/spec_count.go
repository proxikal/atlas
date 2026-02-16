package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func specCountCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "count",
		Short: "Count specifications",
		Long:  `Count total specifications in database. Ultra-compact for AI agents.`,
		Example: `  # Get count
  atlas-dev spec count`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var count int
			err := database.QueryRow(`SELECT COUNT(*) FROM specs`).Scan(&count)
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"ok":  true,
				"cnt": count,
			})
		},
	}

	return cmd
}
