package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionReadCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "read <id>",
		Short: "Read decision details",
		Long:  `Display full details for a decision by ID.`,
		Example: `  # Read decision
  atlas-dev decision read DR-001

  # Read from stdin (auto-detected)
  echo '{"id":"DR-001"}' | atlas-dev decision read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var id string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				id, err = compose.ExtractFirstID(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("decision ID required")
				}
				id = args[0]
			}

			decision, err := database.GetDecision(id)
			if err != nil {
				return err
			}

			return output.Success(decision.ToCompactJSON())
		},
	}

	return cmd
}
