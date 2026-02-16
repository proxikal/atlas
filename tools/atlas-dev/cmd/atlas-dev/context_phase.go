package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/context"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func contextPhaseCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "phase <path>",
		Short: "Show context for a specific phase",
		Long: `Display comprehensive context for a specific phase by path.
Includes phase metadata, objectives, deliverables, acceptance criteria,
category progress, related decisions, and navigation hints.`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			phasePath := args[0]

			// Validate path is not empty
			if phasePath == "" {
				return fmt.Errorf("phase path cannot be empty")
			}

			// Create aggregator
			aggregator := context.NewAggregator(database)

			// Get phase context
			ctx, err := aggregator.GetPhaseContext(phasePath)
			if err != nil {
				return err
			}

			// Output compact JSON
			return output.Success(ctx.ToCompactJSON())
		},
	}
}
