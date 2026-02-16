package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func categoryListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List all categories",
		RunE: func(cmd *cobra.Command, args []string) error {
			rows, err := database.Query(`
				SELECT name, display_name, completed, total, percentage, status
				FROM categories ORDER BY id
			`)
			if err != nil {
				return err
			}
			defer rows.Close()

			items := []map[string]interface{}{}
			for rows.Next() {
				var name, disp, stat string
				var comp, tot, pct int
				rows.Scan(&name, &disp, &comp, &tot, &pct, &stat)
				items = append(items, map[string]interface{}{
					"name": name,
					"disp": disp,
					"prog": []int{comp, tot, pct},
					"stat": stat,
				})
			}

			return output.Success(map[string]interface{}{
				"categories": items,
				"cnt":        len(items),
			})
		},
	}
}
