package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func componentCreateCmd() *cobra.Command {
	var (
		displayName string
		description string
	)

	cmd := &cobra.Command{
		Use:   "create <name>",
		Short: "Create a new component",
		Long:  `Create a new decision component for organizing decisions.`,
		Example: `  # Create new component
  atlas-dev component create runtime --display "Runtime" --description "Runtime system"`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

			// Check if exists
			exists, err := database.ComponentExists(name)
			if err != nil {
				return err
			}
			if exists {
				return fmt.Errorf("component already exists: %s", name)
			}

			// Use name as display if not provided
			if displayName == "" {
				displayName = name
			}

			comp, err := database.CreateComponent(db.CreateComponentRequest{
				Name:        name,
				DisplayName: displayName,
				Description: description,
			})
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"name": comp.Name,
				"disp": comp.DisplayName,
			})
		},
	}

	cmd.Flags().StringVar(&displayName, "display", "", "Display name")
	cmd.Flags().StringVar(&description, "description", "", "Description")

	return cmd
}
