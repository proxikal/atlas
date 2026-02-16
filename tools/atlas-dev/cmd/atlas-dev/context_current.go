package main

import (
	"github.com/atlas-lang/atlas-dev/internal/context"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func contextCurrentCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "current",
		Short: "Show context for next phase to work on",
		Long: `Display comprehensive context for the next phase to work on.
Includes phase metadata, objectives, deliverables, acceptance criteria,
category progress, related decisions, and navigation hints.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create aggregator
			aggregator := context.NewAggregator(database)

			// Get current phase context
			ctx, err := aggregator.GetCurrentPhaseContext()
			if err != nil {
				return err
			}

			// Output compact JSON
			return output.Success(ctx.ToCompactJSON())
		},
	}
}
