package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureSearchCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "search <query>",
		Short: "Search features",
		Long:  `Search features by name, display name, or description.`,
		Example: `  # Search features
  atlas-dev feature search pattern`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			query := args[0]

			if query == "" {
				return fmt.Errorf("search query cannot be empty")
			}

			features, err := database.SearchFeatures(query)
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
				"query":    query,
				"features": items,
				"cnt":      len(features),
			}

			return output.Success(result)
		},
	}

	return cmd
}
