package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
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
		dryRun      bool
	)

	cmd := &cobra.Command{
		Use:   "update <name>",
		Short: "Update a feature",
		Long:  `Update feature metadata in the database.`,
		Example: `  # Update status
  atlas-dev feature update pattern-matching --status Implemented

  # Update version
  atlas-dev feature update pattern-matching --version v0.2

  # Update from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature update --status Implemented`,
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

			// Dry-run: preview changes
			if dryRun {
				// Get current state (read-only)
				current, err := database.GetFeature(name)
				if err != nil {
					return err
				}

				result := map[string]interface{}{
					"dry_run": true,
					"op":      "update_feature",
					"name":    name,
					"before":  current.ToCompactJSON(),
					"changes": map[string]interface{}{},
					"msg":     "Preview only - no changes made",
				}

				if version != "" {
					result["changes"].(map[string]interface{})["ver"] = version
				}
				if status != "" {
					result["changes"].(map[string]interface{})["stat"] = status
				}
				if description != "" {
					result["changes"].(map[string]interface{})["desc"] = description
				}

				return output.Success(result)
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
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview changes without applying")

	return cmd
}
