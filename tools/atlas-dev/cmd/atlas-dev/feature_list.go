package main

import (
	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureListCmd() *cobra.Command {
	var (
		category string
		status   string
	)

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List features",
		Long:  `List all features with optional category and status filters.`,
		Example: `  # List all features
  atlas-dev feature list

  # List by status
  atlas-dev feature list --status Implemented

  # List by category
  atlas-dev feature list --category core

  # Filter from stdin (auto-detected - show only features from input)
  echo '[{"name":"pattern-matching"},{"name":"modules"}]' | atlas-dev feature list`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var filterNames []string

			// Auto-detect stdin for filtering
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				filterNames = compose.ExtractField(input, "name")
			}

			features, err := database.ListFeatures(category, status)
			if err != nil {
				return err
			}

			// Convert to compact JSON
			items := make([]map[string]interface{}, 0, len(features))
			for _, f := range features {
				// Filter by stdin names if provided
				if len(filterNames) > 0 {
					found := false
					for _, fn := range filterNames {
						if fn == f.Name {
							found = true
							break
						}
					}
					if !found {
						continue
					}
				}

				items = append(items, map[string]interface{}{
					"name":    f.Name,
					"display": f.DisplayName,
					"ver":     f.Version,
					"stat":    f.Status,
				})
			}

			result := map[string]interface{}{
				"features": items,
				"cnt":      len(features),
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")

	return cmd
}
