package main

import (
	"github.com/atlas-lang/atlas-dev/internal/backup"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func backupCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "backup",
		Short: "Create timestamped database backup",
		Long: `Create a backup of the database in .backups/ directory.

Backups are automatically timestamped and old backups are cleaned up
(keeps last 10 by default).

Backup files are verified for integrity after creation.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Get database path from global flag
			dbPath, _ := cmd.Flags().GetString("db")

			// Create backup manager
			backupMgr := backup.NewBackupManager(dbPath)

			// Create backup
			result, err := backupMgr.Create()
			if err != nil {
				return err
			}

			// Return compact JSON result
			return output.Success(map[string]interface{}{
				"backup": result.BackupPath,
				"size":   result.SizeBytes,
				"ts":     result.Timestamp,
			})
		},
	}
}
