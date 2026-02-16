package main

import (
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionListCmd() *cobra.Command {
	var (
		component string
		status    string
		limit     int
		offset    int
	)

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List decision logs",
		Long:  `List all decisions with optional filtering by component or status.`,
		Example: `  # List all decisions
  atlas-dev decision list

  # Filter by component
  atlas-dev decision list --component stdlib

  # Filter by status
  atlas-dev decision list --status accepted

  # Pagination
  atlas-dev decision list --limit 10 --offset 20`,
		RunE: func(cmd *cobra.Command, args []string) error {
			opts := db.ListDecisionsOptions{
				Component: component,
				Status:    status,
				Limit:     limit,
				Offset:    offset,
			}

			decisions, err := database.ListDecisions(opts)
			if err != nil {
				return err
			}

			// Convert to compact JSON
			items := make([]map[string]interface{}, len(decisions))
			for i, d := range decisions {
				items[i] = d.ToCompactJSON()
			}

			return output.Success(map[string]interface{}{
				"decisions": items,
				"cnt":       len(decisions),
			})
		},
	}

	cmd.Flags().StringVarP(&component, "component", "c", "", "Filter by component")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")
	cmd.Flags().IntVarP(&limit, "limit", "l", 20, "Limit results")
	cmd.Flags().IntVar(&offset, "offset", 0, "Offset for pagination")

	return cmd
}
