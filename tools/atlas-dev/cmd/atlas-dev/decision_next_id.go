package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionNextIDCmd() *cobra.Command {
	var component string

	cmd := &cobra.Command{
		Use:   "next-id",
		Short: "Preview next decision ID",
		Long:  `Show the next auto-generated decision ID for a component.`,
		Example: `  # Get next ID for stdlib
  atlas-dev decision next-id -c stdlib`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if component == "" {
				return fmt.Errorf("component required (-c flag)")
			}

			nextID, err := database.GetNextDecisionID(component)
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"next_id": nextID,
			})
		},
	}

	cmd.Flags().StringVarP(&component, "component", "c", "", "Component name (required)")

	return cmd
}
