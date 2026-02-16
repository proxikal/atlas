package main

import (
	"github.com/spf13/cobra"
)

func exportCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "export",
		Short: "Export database to various formats",
		Long: `Export database to markdown (for humans) or JSON (for backups).

Markdown exports are optional and generated on-demand for human consumption.
AI agents always query the database directly - never use exported markdown files.`,
	}

	cmd.AddCommand(exportMarkdownCmd())
	cmd.AddCommand(exportJSONCmd())

	return cmd
}
