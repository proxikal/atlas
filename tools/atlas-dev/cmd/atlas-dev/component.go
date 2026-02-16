package main

import (
	"github.com/spf13/cobra"
)

func componentCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "component",
		Short: "Component management commands",
		Long:  `Manage decision components - create, list components for organizing decisions.`,
	}

	cmd.AddCommand(componentCreateCmd())
	cmd.AddCommand(componentListCmd())

	return cmd
}
