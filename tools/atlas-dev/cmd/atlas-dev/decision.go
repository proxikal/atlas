package main

import (
	"github.com/spf13/cobra"
)

func decisionCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "decision",
		Short: "Decision log management commands",
		Long:  `Manage architectural decisions - create, list, search, read, and update decision logs.`,
	}

	cmd.AddCommand(decisionCreateCmd())
	cmd.AddCommand(decisionListCmd())
	cmd.AddCommand(decisionSearchCmd())
	cmd.AddCommand(decisionReadCmd())
	cmd.AddCommand(decisionUpdateCmd())
	cmd.AddCommand(decisionNextIDCmd())
	cmd.AddCommand(decisionExportCmd())

	return cmd
}
