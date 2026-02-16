package main

import (
	"github.com/spf13/cobra"
)

func apiCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "api",
		Short: "API documentation management commands",
		Long: `Manage API documentation - read, validate, generate, and track coverage.

API documentation is tracked in docs/api/ directory.
Use these commands to parse, validate against code, and generate API docs.`,
	}

	cmd.AddCommand(apiReadCmd())
	cmd.AddCommand(apiValidateCmd())
	cmd.AddCommand(apiGenerateCmd())
	cmd.AddCommand(apiCoverageCmd())
	cmd.AddCommand(apiSyncCmd())

	return cmd
}
