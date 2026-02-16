package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func componentListCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List all components",
		Long:  `List all decision components.`,
		Example: `  # List components
  atlas-dev component list`,
		RunE: func(cmd *cobra.Command, args []string) error{
			components, err := database.ListComponents()
			if err != nil {
				return err
			}

			items := make([]map[string]interface{}, 0, len(components))
			for _, c := range components {
				items = append(items, map[string]interface{}{
					"name": c.Name,
					"disp": c.DisplayName,
				})
			}

			return output.Success(map[string]interface{}{
				"components": items,
				"cnt":        len(items),
			})
		},
	}

	return cmd
}
