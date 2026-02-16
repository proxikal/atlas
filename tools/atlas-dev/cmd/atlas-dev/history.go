package main

import "github.com/spf13/cobra"

func historyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "history",
		Short: "History tracking commands",
	}
	cmd.AddCommand(historyCreateCmd())
	cmd.AddCommand(historyListCmd())
	cmd.AddCommand(historyReadCmd())
	cmd.AddCommand(historySearchCmd())
	return cmd
}
