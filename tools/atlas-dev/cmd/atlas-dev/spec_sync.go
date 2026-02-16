package main

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func specSyncCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sync",
		Short: "Sync specification files to database",
		Long:  `Scan docs/specification/*.md files and populate specs table. Migration-only command.`,
		Example: `  # Sync all spec files
  atlas-dev spec sync`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Find spec files
			specDir := "docs/specification"
			pattern := filepath.Join(specDir, "*.md")
			files, err := filepath.Glob(pattern)
			if err != nil {
				return fmt.Errorf("failed to glob spec files: %w", err)
			}

			synced := 0
			for _, path := range files {
				// Extract name from filename
				name := strings.TrimSuffix(filepath.Base(path), ".md")

				// Simple section extraction from filename pattern
				section := "specification"
				if strings.Contains(name, "grammar") {
					section = "grammar"
				} else if strings.Contains(name, "bytecode") {
					section = "bytecode"
				} else if strings.Contains(name, "diagnostic") {
					section = "diagnostics"
				}

				// Title = capitalize name
				title := strings.ReplaceAll(name, "-", " ")
				title = strings.Title(title)

				// Insert or update
				_, err := database.Exec(`
					INSERT INTO specs (path, name, section, title)
					VALUES (?, ?, ?, ?)
					ON CONFLICT(path) DO UPDATE SET
						name = excluded.name,
						section = excluded.section,
						title = excluded.title,
						updated_at = datetime('now')
				`, path, name, section, title)

				if err != nil {
					return fmt.Errorf("failed to insert spec %s: %w", path, err)
				}
				synced++
			}

			return output.Success(map[string]interface{}{
				"synced": synced,
				"dir":    specDir,
			})
		},
	}

	return cmd
}
