package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseInfoCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "info <phase-path>",
		Short: "Show phase details",
		Long:  `Display detailed information about a specific phase.`,
		Example: `  # Show phase info
  atlas-dev phase info phases/stdlib/phase-01.md

  # Read from stdin (auto-detected)
  echo '{"path":"phases/stdlib/phase-01.md"}' | atlas-dev phase info`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var phasePath string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
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

			phase, err := database.GetPhaseInfo(phasePath)
			if err != nil {
				return err
			}

			return output.Success(phase.ToCompactJSON())
		},
	}

	return cmd
}
