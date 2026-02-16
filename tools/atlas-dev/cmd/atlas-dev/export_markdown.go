package main

import (
	"github.com/atlas-lang/atlas-dev/internal/export"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func exportMarkdownCmd() *cobra.Command {
	var outputDir string

	cmd := &cobra.Command{
		Use:   "markdown",
		Short: "Export database to markdown files (for humans)",
		Long: `Generate STATUS.md and tracker files from database.

IMPORTANT: This is OPTIONAL and for human consumption only.
- AI agents NEVER use these files
- Database is the single source of truth
- Generated on-demand when humans request it`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create markdown exporter
			exporter := export.NewMarkdownExporter(database)

			// Export to markdown
			result, err := exporter.Export(outputDir)
			if err != nil {
				return err
			}

			// Return compact JSON result
			return output.Success(map[string]interface{}{
				"output_dir": result.OutputDir,
				"files":      result.FileCount,
				"created":    result.FilesCreated,
			})
		},
	}

	cmd.Flags().StringVarP(&outputDir, "output", "o", "status", "Output directory for generated files")

	return cmd
}
