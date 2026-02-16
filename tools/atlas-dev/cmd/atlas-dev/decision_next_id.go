package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionNextIDCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "next-id",
		Short: "Preview next decision ID",
		Long:  `Show the next auto-generated decision ID.`,
		Example: `  # Get next ID
  atlas-dev decision next-id`,
		RunE: func(cmd *cobra.Command, args []string) error {
			nextID, err := database.GetNextDecisionID()
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"next_id": nextID,
			})
		},
	}

	return cmd
}
