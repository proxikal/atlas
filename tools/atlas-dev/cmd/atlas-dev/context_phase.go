package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/context"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

var contextPhaseStdin bool

func contextPhaseCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "phase <path>",
		Short: "Show context for a specific phase",
		Long: `Display comprehensive context for a specific phase by path.
Includes phase metadata, objectives, deliverables, acceptance criteria,
category progress, related decisions, and navigation hints.`,
		Example: `  # Show context for a phase
  atlas-dev context phase phases/stdlib/phase-01.md

  # Read from stdin
  echo '{"path":"phases/stdlib/phase-01.md"}' | atlas-dev context phase --stdin`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var phasePath string

			// Get path from stdin or args
			if contextPhaseStdin {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				phasePath, err = compose.ExtractFirstPath(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("phase path required")
				}
				phasePath = args[0]
			}

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

	cmd.Flags().BoolVar(&contextPhaseStdin, "stdin", false, "Read path from stdin JSON")

	return cmd
}
