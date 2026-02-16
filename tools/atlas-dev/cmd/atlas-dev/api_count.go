package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiCountCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "count",
		Short: "Count API docs",
		Long:  `Count total API documentation modules. Ultra-compact for AI agents.`,
		Example: `  # Get count
  atlas-dev api count`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var count int
			err := database.QueryRow(`SELECT COUNT(*) FROM api_docs`).Scan(&count)
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
