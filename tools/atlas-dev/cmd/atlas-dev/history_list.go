package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func historyListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List history entries",
		RunE: func(cmd *cobra.Command, args []string) error {
			rows, err := database.Query(`SELECT name, title, date, type FROM history ORDER BY date DESC`)
			if err != nil {
				return err
			}
			defer rows.Close()

			items := []map[string]interface{}{}
			for rows.Next() {
				var name, title, date, htype string
				rows.Scan(&name, &title, &date, &htype)
				items = append(items, map[string]interface{}{
					"name": name,
					"ttl":  title,
					"date": date,
					"type": htype,
				})
			}

			return output.Success(map[string]interface{}{
				"history": items,
				"cnt":     len(items),
			})
		},
	}
}
