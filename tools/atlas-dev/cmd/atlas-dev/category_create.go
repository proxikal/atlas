package main

import (
	"fmt"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func categoryCreateCmd() *cobra.Command {
	var displayName string

	cmd := &cobra.Command{
		Use:   "create <name>",
		Short: "Create a new phase category",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]
			if displayName == "" {
				displayName = name
			}

			_, err := database.Exec(`
				INSERT INTO categories (name, display_name, total)
				VALUES (?, ?, 0)
			`, name, displayName)
			if err != nil {
				return fmt.Errorf("failed to create category: %w", err)
			}

			return output.Success(map[string]interface{}{
				"name": name,
				"disp": displayName,
			})
		},
	}

	cmd.Flags().StringVar(&displayName, "display", "", "Display name")
	return cmd
}
