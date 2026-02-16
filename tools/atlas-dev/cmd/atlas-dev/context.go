package main

import (
	"github.com/spf13/cobra"
)

func contextCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "context",
		Short: "Phase context aggregation commands",
		Long: `Aggregate comprehensive phase context from database and phase files.
Combines structured DB data with phase instructions, dependencies, related
decisions, and navigation - providing AI agents with everything needed to
start work in a single query.`,
	}

	cmd.AddCommand(contextCurrentCmd())
	cmd.AddCommand(contextPhaseCmd())

	return cmd
}
