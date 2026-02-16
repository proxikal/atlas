package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureDeleteCmd() *cobra.Command {
	var (
		deleteFile bool
	)

	cmd := &cobra.Command{
		Use:   "delete <name>",
		Short: "Delete a feature",
		Long:  `Delete a feature from the database and optionally the markdown file.`,
		Example: `  # Delete from DB only
  atlas-dev feature delete pattern-matching

  # Delete from DB and file
  atlas-dev feature delete pattern-matching --file`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

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

	return cmd
}
