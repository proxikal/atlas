package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureDeleteCmd() *cobra.Command {
	var (
		deleteFile bool
		dryRun     bool
	)

	cmd := &cobra.Command{
		Use:   "delete <name>",
		Short: "Delete a feature",
		Long:  `Delete a feature from the database and optionally the markdown file.`,
		Example: `  # Delete from DB only
  atlas-dev feature delete pattern-matching

  # Delete from DB and file
  atlas-dev feature delete pattern-matching --file

  # Delete from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature delete`,
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

			// Dry-run: preview deletion
			if dryRun {
				markdownPath := filepath.Join("../../docs/features", name+".md")
				result := map[string]interface{}{
					"dry_run": true,
					"op":      "delete_feature",
					"feature": name,
					"db":      true,
					"file":    deleteFile,
					"msg":     "Preview only - no changes made",
				}

				// Check if file exists
				if _, err := os.Stat(markdownPath); err == nil {
					result["file_exists"] = true
					result["file_path"] = markdownPath
				}

				return output.Success(result)
			}

			// Delete from database
			err := database.DeleteFeature(name)
			if err != nil {
				return err
			}

			result := map[string]interface{}{
				"msg":     "Feature deleted",
				"feature": name,
			}

			// Optionally delete markdown file
			if deleteFile {
				markdownPath := filepath.Join("../../docs/features", name+".md")
				err := os.Remove(markdownPath)
				if err != nil {
					result["file_warning"] = fmt.Sprintf("failed to delete markdown file: %v", err)
				} else {
					result["file_deleted"] = true
				}
			}

			return output.Success(result)
		},
	}

	cmd.Flags().BoolVar(&deleteFile, "file", false, "Also delete markdown file")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview deletion without applying")

	return cmd
}
