package main

import (
	"github.com/atlas-lang/atlas-dev/internal/compose"
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
  atlas-dev decision list --limit 10 --offset 20

  # Filter from stdin (auto-detected - show only decisions from input)
  echo '[{"id":"DR-001"},{"id":"DR-002"}]' | atlas-dev decision list`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var filterIDs []string

			// Auto-detect stdin for filtering
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				filterIDs = compose.ExtractIDs(input)
			}

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
			items := make([]map[string]interface{}, 0, len(decisions))
			for _, d := range decisions {
				// Filter by stdin IDs if provided
				if len(filterIDs) > 0 {
					found := false
					for _, fid := range filterIDs {
						if fid == d.ID {
							found = true
							break
						}
					}
					if !found {
						continue
					}
				}

				items = append(items, d.ToCompactJSON())
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
