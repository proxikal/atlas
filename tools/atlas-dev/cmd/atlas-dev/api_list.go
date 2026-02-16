package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiListCmd() *cobra.Command {
	var limit int

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List API documentation modules",
		Long:  `List API module names/titles. Token-efficient index for surgical reads.`,
		Example: `  # List all API modules
  atlas-dev api list

  # Surgical workflow
  atlas-dev api count              # Get total
  atlas-dev api list               # Get module names
  atlas-dev api read stdlib        # Read specific module`,
		RunE: func(cmd *cobra.Command, args []string) error {
			rows, err := database.Query(`
				SELECT module, title, functions_count
				FROM api_docs
				ORDER BY module
				LIMIT ?
			`, limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			modules := []map[string]interface{}{}
			for rows.Next() {
				var module, title string
				var fnCount int
				if err := rows.Scan(&module, &title, &fnCount); err != nil {
					return err
				}

				modules = append(modules, map[string]interface{}{
					"mod":   module,
					"ttl":   title,
					"fn_ct": fnCount,
				})
			}

			return output.Success(map[string]interface{}{
				"ok":      true,
				"modules": modules,
				"cnt":     len(modules),
			})
		},
	}

	cmd.Flags().IntVar(&limit, "limit", 10, "Max results (default 10)")

	return cmd
}
