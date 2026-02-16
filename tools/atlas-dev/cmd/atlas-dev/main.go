package main

import (
	"log/slog"
	"os"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

var (
	// Global flags
	dbPath string
	debug  bool

	// Global database handle
	database *db.DB

	// Version
	version = "1.0.0"
)

func main() {
	rootCmd := &cobra.Command{
		Use:     "atlas-dev",
		Short:   "Atlas development automation tool",
		Long:    `Atlas Dev automates development workflows using pure SQLite as single source of truth.`,
		Version: version,
		PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
			// Configure logging
			level := slog.LevelInfo
			if debug {
				level = slog.LevelDebug
			}
			handler := slog.NewJSONHandler(os.Stderr, &slog.HandlerOptions{
				Level: level,
			})
			slog.SetDefault(slog.New(handler))

			// Open database (skip for commands that don't need it)
			if cmd.Name() == "version" {
				return nil
			}

			var err error
			database, err = db.New(dbPath)
			if err != nil {
				return err
			}

			return nil
		},
		PersistentPostRunE: func(cmd *cobra.Command, args []string) error {
			// Close database
			if database != nil {
				return database.Close()
			}
			return nil
		},
	}

	// Persistent flags
	rootCmd.PersistentFlags().StringVar(&dbPath, "db", "atlas-dev.db", "Database path")
	rootCmd.PersistentFlags().BoolVar(&debug, "debug", false, "Enable debug logging")

	// Add commands
	rootCmd.AddCommand(versionCmd())
	rootCmd.AddCommand(migrateCmd())
	rootCmd.AddCommand(phaseCmd())
	rootCmd.AddCommand(decisionCmd())
	rootCmd.AddCommand(validateCmd())

	if err := rootCmd.Execute(); err != nil {
		output.Fatal(err)
	}
}

func versionCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Show version information",
		Run: func(cmd *cobra.Command, args []string) {
			if err := output.Success(map[string]interface{}{
				"version":        version,
				"schema_version": "1",
			}); err != nil {
				output.Fatal(err)
			}
		},
	}
}
