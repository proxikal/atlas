package main

import (
	"fmt"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func referenceCreateCmd() *cobra.Command {
	var title, reftype, content string

	cmd := &cobra.Command{
		Use:   "create <name>",
		Short: "Create reference doc",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

			_, err := database.Exec(`
				INSERT INTO reference_docs (name, title, type, content)
				VALUES (?, ?, ?, ?)
			`, name, title, reftype, content)
			if err != nil {
				return fmt.Errorf("failed to create reference: %w", err)
			}

			return output.Success(map[string]interface{}{
				"name": name,
				"type": reftype,
			})
		},
	}

	cmd.Flags().StringVar(&title, "title", "", "Title (required)")
	cmd.Flags().StringVar(&reftype, "type", "", "Type (mapping/standard/checklist)")
	cmd.Flags().StringVar(&content, "content", "", "Content (required)")
	cmd.MarkFlagRequired("title")
	cmd.MarkFlagRequired("type")
	cmd.MarkFlagRequired("content")
	return cmd
}
