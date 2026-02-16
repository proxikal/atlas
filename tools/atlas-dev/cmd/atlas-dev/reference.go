package main

import "github.com/spf13/cobra"

func referenceCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "reference",
		Short: "Reference doc commands",
	}
	cmd.AddCommand(referenceCreateCmd())
	cmd.AddCommand(referenceListCmd())
	cmd.AddCommand(referenceReadCmd())
	cmd.AddCommand(referenceSearchCmd())
	return cmd
}
