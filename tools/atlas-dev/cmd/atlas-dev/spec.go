package main

import (
	"github.com/spf13/cobra"
)

func specCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "spec",
		Short: "Specification management commands",
		Long: `Manage specification documents - read, search, validate, and check grammar.

Specifications are tracked in docs/specification/ directory.
Use these commands to parse, validate, and search spec documents.`,
	}

	cmd.AddCommand(specReadCmd())
	cmd.AddCommand(specSearchCmd())
	cmd.AddCommand(specValidateCmd())
	cmd.AddCommand(specGrammarCmd())
	cmd.AddCommand(specSyncCmd())

	return cmd
}
