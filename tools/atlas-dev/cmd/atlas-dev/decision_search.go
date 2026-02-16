package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionSearchCmd() *cobra.Command {
	var limit int

	cmd := &cobra.Command{
		Use:   "search <query>",
		Short: "Search decision logs",
		Long:  `Search decisions by title, decision text, or rationale.`,
		Example: `  # Search for decisions about "hash"
  atlas-dev decision search hash

  # Search with limit
  atlas-dev decision search "type system" --limit 10`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			query := args[0]

			if query == "" {
				return fmt.Errorf("search query cannot be empty")
			}

			decisions, err := database.SearchDecisions(query, limit)
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
				"query":     query,
			})
		},
	}

	cmd.Flags().IntVarP(&limit, "limit", "l", 20, "Limit results")

	return cmd
}
