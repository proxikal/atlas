package main

import "github.com/spf13/cobra"

func categoryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "category",
		Short: "Category management commands",
	}
	cmd.AddCommand(categoryCreateCmd())
	cmd.AddCommand(categoryListCmd())
	return cmd
}
