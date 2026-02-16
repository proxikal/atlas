package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func referenceListCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List reference docs",
		RunE: func(cmd *cobra.Command, args []string) error {
			rows, err := database.Query(`SELECT name, title, type FROM reference_docs ORDER BY type, name`)
			if err != nil {
				return err
			}
			defer rows.Close()

			items := []map[string]interface{}{}
			for rows.Next() {
				var name, title, reftype string
				rows.Scan(&name, &title, &reftype)
				items = append(items, map[string]interface{}{
					"name": name,
					"ttl":  title,
					"type": reftype,
				})
			}

			return output.Success(map[string]interface{}{
				"references": items,
				"cnt":        len(items),
			})
		},
	}
}
