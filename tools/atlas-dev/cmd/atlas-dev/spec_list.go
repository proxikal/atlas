package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func specListCmd() *cobra.Command {
	var limit int

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List specifications",
		Long:  `List spec names/titles. Token-efficient index for surgical reads.`,
		Example: `  # List all specs (default 10)
  atlas-dev spec list

  # List 20 specs
  atlas-dev spec list --limit 20

  # Surgical workflow
  atlas-dev spec count          # Get total
  atlas-dev spec list           # Get names
  atlas-dev spec read syntax    # Read specific spec`,
		RunE: func(cmd *cobra.Command, args []string) error {
			rows, err := database.Query(`
				SELECT name, title, section
				FROM specs
				ORDER BY section, name
				LIMIT ?
			`, limit)
			if err != nil {
				return err
			}
			defer rows.Close()

			specs := []map[string]interface{}{}
			for rows.Next() {
				var name, title, section string
				if err := rows.Scan(&name, &title, &section); err != nil {
					return err
				}

				specs = append(specs, map[string]interface{}{
					"name": name,
					"ttl":  title,
					"sec":  section,
				})
			}

			return output.Success(map[string]interface{}{
				"ok":    true,
				"specs": specs,
				"cnt":   len(specs),
			})
		},
	}

	cmd.Flags().IntVar(&limit, "limit", 10, "Max results (default 10)")

	return cmd
}
