package main

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiSyncCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sync",
		Short: "Sync API documentation files to database",
		Long:  `Scan docs/api/*.md files and populate api_docs table. Migration-only command.`,
		Example: `  # Sync all API doc files
  atlas-dev api sync`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Find API doc files
			apiDir := "docs/api"
			pattern := filepath.Join(apiDir, "*.md")
			files, err := filepath.Glob(pattern)
			if err != nil {
				return fmt.Errorf("failed to glob API files: %w", err)
			}

			synced := 0
			for _, path := range files {
				// Extract module from filename (e.g., "stdlib.md" → "stdlib")
				name := strings.TrimSuffix(filepath.Base(path), ".md")
				module := name

				// Title = capitalize module name
				title := strings.ReplaceAll(name, "-", " ")
				title = strings.Title(title)

				// Insert or update
				_, err := database.Exec(`
					INSERT INTO api_docs (path, module, name, title)
					VALUES (?, ?, ?, ?)
					ON CONFLICT(path) DO UPDATE SET
						module = excluded.module,
						name = excluded.name,
						title = excluded.title,
						updated_at = datetime('now')
				`, path, module, name, title)

				if err != nil {
					return fmt.Errorf("failed to insert API doc %s: %w", path, err)
				}
				synced++
			}

			return output.Success(map[string]interface{}{
				"synced": synced,
				"dir":    apiDir,
			})
		},
	}

	return cmd
}
