package main

import (
	"fmt"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/export"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func exportJSONCmd() *cobra.Command {
	var outputPath string

	cmd := &cobra.Command{
		Use:   "json",
		Short: "Export database to JSON backup file",
		Long: `Create complete JSON backup of entire database.

Exports all tables (categories, phases, decisions, metadata, audit_log)
to a timestamped JSON file for disaster recovery and migration.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Generate filename if not specified
			if outputPath == "" {
				outputPath = export.GenerateBackupFilename("backup")
			}

			// Create JSON exporter
			exporter := export.NewJSONExporter(database)

			// Export to JSON
			result, err := exporter.Export(outputPath)
			if err != nil {
				return err
			}

			// Return compact JSON result
			return output.Success(map[string]interface{}{
				"file":   result.FilePath,
				"size":   result.SizeBytes,
				"tables": result.Tables,
				"ts":     result.Timestamp,
			})
		},
	}

	cmd.Flags().StringVarP(&outputPath, "output", "o", "", fmt.Sprintf("Output file path (default: backup-%s.json)", time.Now().Format("20060102-150405")))

	return cmd
}
