package main

import (
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateComponentsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "components",
		Short: "Migrate decision components to components table",
		Long:  `Extract unique components from decisions table and populate components table.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Get unique components from decisions
			rows, err := database.Query("SELECT DISTINCT component FROM decisions WHERE component != '' ORDER BY component")
			if err != nil {
				return err
			}
			defer rows.Close()

			var components []string
			for rows.Next() {
				var comp string
				if err := rows.Scan(&comp); err != nil {
					return err
				}
				components = append(components, comp)
			}

			// Insert each component
			migrated := 0
			for _, comp := range components {
				displayName := toDisplayName(comp)

				// Insert or ignore (skip if already exists)
				_, err := database.Exec(`
					INSERT OR IGNORE INTO components (name, display_name, description)
					VALUES (?, ?, ?)
				`, comp, displayName, "Component from decision migration")

				if err != nil {
					return err
				}
				migrated++
			}

			return output.Success(map[string]interface{}{
				"msg":      "Component migration complete",
				"migrated": migrated,
			})
		},
	}

	return cmd
}

func toDisplayName(name string) string {
	// Simple title case for display names
	parts := strings.Split(name, "-")
	for i, part := range parts {
		if len(part) > 0 {
			parts[i] = strings.ToUpper(part[:1]) + part[1:]
		}
	}
	return strings.Join(parts, " ")
}
