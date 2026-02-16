package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func validateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "validate",
		Short: "Validation commands for database, parity, tests, and consistency",
		Long: `Run various validation checks:

- validate          - Check database consistency (default)
- validate parity   - Validate spec/API/code/test parity
- validate all      - Run all validators
- validate tests    - Validate test coverage
- validate consistency - Detect documentation conflicts

Use 'validate [command] --help' for more information.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Default: database validation
			report, err := database.Validate()
			if err != nil {
				return err
			}

			result := report.ToCompactJSON()
			if report.OK {
				result["msg"] = "Database is consistent"
			}

			return output.Success(result)
		},
	}

	// Add subcommands
	cmd.AddCommand(validateParityCmd())
	cmd.AddCommand(validateAllCmd())
	cmd.AddCommand(validateTestsCmd())
	cmd.AddCommand(validateConsistencyCmd())

	return cmd
}
