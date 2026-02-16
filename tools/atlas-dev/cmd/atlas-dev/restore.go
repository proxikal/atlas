package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/backup"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func restoreCmd() *cobra.Command {
	var confirm bool

	cmd := &cobra.Command{
		Use:   "restore <backup-file>",
		Short: "Restore database from backup",
		Long: `Restore database from a backup file.

IMPORTANT: Requires --confirm flag to prevent accidental data loss.

A safety backup of the current database is created before restore.
If restore fails, the database is automatically restored from the safety backup.

Database integrity is verified after restore.`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			backupFile := args[0]

			// Validate backup file
			if err := backup.ValidateBackupFile(backupFile); err != nil {
				return fmt.Errorf("invalid backup file: %w", err)
			}

			// Get database path from global flag
			dbPath, _ := cmd.Flags().GetString("db")

			// Create backup manager
			backupMgr := backup.NewBackupManager(dbPath)

			// Perform restore
			result, err := backupMgr.Restore(backupFile, confirm)
			if err != nil {
				return err
			}

			// Return compact JSON result
			return output.Success(map[string]interface{}{
				"restored":  result.RestoredFrom,
				"safety":    result.SafetyBackup,
				"size":      result.SizeBytes,
				"integrity": result.IntegrityOK,
				"ts":        result.Timestamp,
			})
		},
	}

	cmd.Flags().BoolVar(&confirm, "confirm", false, "Confirm restore operation (required)")

	return cmd
}
