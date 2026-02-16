package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureUpdateCmd() *cobra.Command {
	var (
		version     string
		status      string
		description string
		specPath    string
		apiPath     string
	)

	cmd := &cobra.Command{
		Use:   "update <name>",
		Short: "Update a feature",
		Long:  `Update feature metadata in the database.`,
		Example: `  # Update status
  atlas-dev feature update pattern-matching --status Implemented

  # Update version
  atlas-dev feature update pattern-matching --version v0.2`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

			// Build update request
			req := db.UpdateFeatureRequest{
				Name:        name,
				Version:     version,
				Status:      status,
				Description: description,
				SpecPath:    specPath,
				APIPath:     apiPath,
			}

			// Check if any fields to update
			if version == "" && status == "" && description == "" && specPath == "" && apiPath == "" {
				return fmt.Errorf("no fields to update (use --version, --status, etc.)")
			}

			feature, err := database.UpdateFeature(req)
			if err != nil {
				return err
			}

			result := feature.ToCompactJSON()
			result["msg"] = "Feature updated"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&version, "version", "", "Update version")
	cmd.Flags().StringVar(&status, "status", "", "Update status")
	cmd.Flags().StringVar(&description, "description", "", "Update description")
	cmd.Flags().StringVar(&specPath, "spec", "", "Update spec path")
	cmd.Flags().StringVar(&apiPath, "api", "", "Update API path")

	return cmd
}
