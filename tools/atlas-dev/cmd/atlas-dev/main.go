package main

import (
	"fmt"
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
	rootCmd.AddCommand(featureCmd())
	rootCmd.AddCommand(componentCmd())
	rootCmd.AddCommand(categoryCmd())
	rootCmd.AddCommand(historyCmd())
	rootCmd.AddCommand(referenceCmd())
	rootCmd.AddCommand(specCmd())
	rootCmd.AddCommand(apiCmd())
	rootCmd.AddCommand(contextCmd())
	rootCmd.AddCommand(exportCmd())
	rootCmd.AddCommand(undoCmd())
	rootCmd.AddCommand(backupCmd())
	rootCmd.AddCommand(restoreCmd())
	rootCmd.AddCommand(summaryCmd())
	rootCmd.AddCommand(statsCmd())
	rootCmd.AddCommand(blockersCmd())
	rootCmd.AddCommand(timelineCmd())
	rootCmd.AddCommand(coverageCmd())
	rootCmd.AddCommand(validateCmd())
	rootCmd.AddCommand(completionCmd())

	if err := rootCmd.Execute(); err != nil {
		output.Fatal(err)
	}
}

func completionCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "completion [bash|zsh|fish|powershell]",
		Short: "Generate shell completion script",
		Long: `Generate shell completion script for atlas-dev.

To load completions:

Bash:
  $ source <(atlas-dev completion bash)
  # To load for every session:
  $ atlas-dev completion bash > /etc/bash_completion.d/atlas-dev

Zsh:
  $ atlas-dev completion zsh > "${fpath[1]}/_atlas-dev"
  # Then restart your shell

Fish:
  $ atlas-dev completion fish | source
  # To load for every session:
  $ atlas-dev completion fish > ~/.config/fish/completions/atlas-dev.fish

PowerShell:
  PS> atlas-dev completion powershell | Out-String | Invoke-Expression
`,
		DisableFlagsInUseLine: true,
		ValidArgs:             []string{"bash", "zsh", "fish", "powershell"},
		Args:                  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			switch args[0] {
			case "bash":
				return cmd.Root().GenBashCompletion(os.Stdout)
			case "zsh":
				return cmd.Root().GenZshCompletion(os.Stdout)
			case "fish":
				return cmd.Root().GenFishCompletion(os.Stdout, true)
			case "powershell":
				return cmd.Root().GenPowerShellCompletionWithDesc(os.Stdout)
			default:
				return fmt.Errorf("unsupported shell: %s (supported: bash, zsh, fish, powershell)", args[0])
			}
		},
	}

	return cmd
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
