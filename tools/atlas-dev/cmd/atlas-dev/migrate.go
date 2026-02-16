package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate",
		Short: "Database migration commands",
		Long:  `Migrate existing markdown files to SQLite or initialize fresh schema.`,
	}

	cmd.AddCommand(migrateSchemaCmd())
	cmd.AddCommand(migrateBootstrapCmd())

	return cmd
}

func migrateSchemaCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "schema",
		Short: "Initialize database schema",
		Long:  `Create all tables, indexes, triggers, views and seed initial data.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Initialize schema
			if err := database.InitSchema(); err != nil {
				return fmt.Errorf("failed to initialize schema: %w", err)
			}

			return output.Success(map[string]interface{}{
				"msg":    "Schema initialized successfully",
				"tables": 10,
				"views":  4,
			})
		},
	}
}

func migrateBootstrapCmd() *cobra.Command {
	var force bool

	cmd := &cobra.Command{
		Use:   "bootstrap",
		Short: "Bootstrap database from existing markdown files",
		Long: `One-time migration: parse STATUS.md and trackers/*.md to populate database.
Backs up markdown files to .migration-backup/ directory.

CRITICAL: This should only be run ONCE. After migration, markdown files will be deleted.
Re-running this command after deletion would be catastrophic (no source data).`,
		Example: `  # First-time migration
  atlas-dev migrate bootstrap

  # Force re-migration (WARNING: destructive)
  atlas-dev migrate bootstrap --force`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Check if already migrated
			migrated, err := database.IsMigrated()
			if err != nil {
				return fmt.Errorf("failed to check migration status: %w", err)
			}

			if migrated && !force {
				return fmt.Errorf("database already migrated - markdown files may have been deleted\n\nIf you really want to re-run migration, use --force flag (WARNING: may cause data loss)")
			}

			if force {
				return output.Success(map[string]interface{}{
					"msg":     "Force re-migration not implemented yet",
					"warning": "This would be a destructive operation",
				})
			}

			// TODO: Implement bootstrap migration (Phase 2)
			// This will parse existing STATUS.md and trackers/*.md
			// and populate the database

			// After successful migration:
			// if err := database.MarkAsMigrated(); err != nil {
			//     return err
			// }

			return fmt.Errorf("bootstrap not yet implemented - use 'migrate schema' for fresh database")
		},
	}

	cmd.Flags().BoolVar(&force, "force", false, "Force re-migration (WARNING: may cause data loss)")

	return cmd
}
