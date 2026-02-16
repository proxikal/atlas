package main

import (
	"github.com/spf13/cobra"
)

func phaseCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "phase",
		Aliases: []string{"p"},
		Short:   "Phase management commands",
		Long:    `Manage development phases - complete, query current/next, list, and view details.`,
	}

	cmd.AddCommand(phaseCompleteCmd())
	cmd.AddCommand(phaseCurrentCmd())
	cmd.AddCommand(phaseNextCmd())
	cmd.AddCommand(phaseInfoCmd())
	cmd.AddCommand(phaseListCmd())

	return cmd
}
