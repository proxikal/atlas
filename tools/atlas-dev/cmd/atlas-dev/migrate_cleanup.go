package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateCleanupCmd() *cobra.Command {
	var dryRun bool

	cmd := &cobra.Command{
		Use:   "cleanup",
		Short: "Remove test entries from migration",
		Long:  `Remove test-* entries that were created during development.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			deleted := 0

			if dryRun {
				// Count what would be deleted
				var count int
				database.QueryRow("SELECT COUNT(*) FROM components WHERE name LIKE 'test-%'").Scan(&count)
				deleted += count
				database.QueryRow("SELECT COUNT(*) FROM history WHERE name LIKE 'test-%' OR name LIKE 'v0.2-%'").Scan(&count)
				deleted += count
				database.QueryRow("SELECT COUNT(*) FROM reference_docs WHERE name LIKE 'test-%'").Scan(&count)
				deleted += count

				return output.Success(map[string]interface{}{
					"msg":     "Dry run - would delete",
					"deleted": deleted,
				})
			}

			// Delete test components
			result, _ := database.Exec("DELETE FROM components WHERE name LIKE 'test-%'")
			if rows, err := result.RowsAffected(); err == nil {
				deleted += int(rows)
			}

			// Delete test history (including v0.2-summary which doesn't exist as file)
			result, _ = database.Exec("DELETE FROM history WHERE name LIKE 'test-%' OR name LIKE 'v0.2-%'")
			if rows, err := result.RowsAffected(); err == nil {
				deleted += int(rows)
			}

			// Delete test references
			result, _ = database.Exec("DELETE FROM reference_docs WHERE name LIKE 'test-%'")
			if rows, err := result.RowsAffected(); err == nil {
				deleted += int(rows)
			}

			return output.Success(map[string]interface{}{
				"msg":     "Cleanup complete",
				"deleted": deleted,
			})
		},
	}

	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Show what would be deleted")
	return cmd
}
