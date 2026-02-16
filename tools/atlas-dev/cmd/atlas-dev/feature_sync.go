package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/feature"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureSyncCmd() *cobra.Command {
	var dryRun bool

	cmd := &cobra.Command{
		Use:   "sync <name>",
		Short: "Sync feature from codebase",
		Long:  `Auto-update feature documentation from actual implementation code.`,
		Example: `  # Sync feature
  atlas-dev feature sync pattern-matching

  # Dry-run (preview changes)
  atlas-dev feature sync pattern-matching --dry-run

  # Sync from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature sync`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				name, err = compose.ExtractFirstString(input, "name")
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("feature name required")
				}
				name = args[0]
			}

			// Parse markdown file
			markdownPath := filepath.Join("../../docs/features", name+".md")
			parsedFeature, err := feature.Parse(markdownPath)
			if err != nil {
				return fmt.Errorf("failed to parse feature file: %w", err)
			}

			// Use current working directory as project root
			projectRoot, err := os.Getwd()
			if err != nil {
				return fmt.Errorf("failed to get working directory: %w", err)
			}

			// Sync from codebase
			syncResult, err := feature.Sync(parsedFeature, projectRoot, dryRun)
			if err != nil {
				return err
			}

			// If not dry-run and changes detected, update database
			if !dryRun && syncResult.Updated {
				req := db.UpdateFeatureRequest{
					Name: name,
				}

				// Update implementation notes with sync info
				notes := fmt.Sprintf("Synced from code - %d functions, %d tests, %.1f%% parity",
					syncResult.FunctionCount, syncResult.TestCount, syncResult.Parity)
				req.ImplementationNotes = notes

				_, err := database.UpdateFeature(req)
				if err != nil {
					return fmt.Errorf("failed to update database: %w", err)
				}
			}

			// Convert to compact JSON
			result := map[string]interface{}{
				"feature": name,
				"updated": syncResult.Updated,
				"dry_run": dryRun,
			}

			if syncResult.FunctionCount > 0 {
				result["fn_cnt"] = syncResult.FunctionCount
			}
			if syncResult.TestCount > 0 {
				result["test_cnt"] = syncResult.TestCount
			}
			if syncResult.Parity > 0 {
				result["parity"] = syncResult.Parity
			}

			if len(syncResult.Changes) > 0 {
				result["changes"] = syncResult.Changes
			}

			if len(syncResult.Errors) > 0 {
				result["errors"] = syncResult.Errors
			}

			if syncResult.LastModified != "" {
				result["modified"] = syncResult.LastModified
			}

			return output.Success(result)
		},
	}

	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview changes without applying")

	return cmd
}
