package main

import (
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
  atlas-dev feature list --category core`,
		RunE: func(cmd *cobra.Command, args []string) error {
			features, err := database.ListFeatures(category, status)
			if err != nil {
				return err
			}

			// Convert to compact JSON
			items := make([]map[string]interface{}, 0, len(features))
			for _, f := range features {
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
