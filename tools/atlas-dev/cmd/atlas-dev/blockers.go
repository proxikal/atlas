package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func blockersCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "blockers",
		Short: "List blocked phases",
		Long:  `Show all blocked phases with their blocking dependencies.`,
		Example: `  # List all blocked phases
  atlas-dev blockers`,
		RunE: func(cmd *cobra.Command, args []string) error {
			blocked, err := database.GetBlockedPhases()
			if err != nil {
				return err
			}

			// Convert to compact JSON
			items := make([]map[string]interface{}, len(blocked))
			for i, b := range blocked {
				items[i] = b.ToCompactJSON()
			}

			return output.Success(map[string]interface{}{
				"blocked": items,
				"cnt":     len(items),
			})
		},
	}
}
