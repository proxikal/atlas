package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionReadCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "read <id>",
		Short: "Read decision details",
		Long:  `Display full details for a decision by ID.`,
		Example: `  # Read decision
  atlas-dev decision read DR-001`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			id := args[0]

			decision, err := database.GetDecision(id)
			if err != nil {
				return err
			}

			return output.Success(decision.ToCompactJSON())
		},
	}

	return cmd
}
