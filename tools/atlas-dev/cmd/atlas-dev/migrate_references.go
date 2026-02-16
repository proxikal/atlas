package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateReferencesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "references",
		Short: "Migrate reference files to database",
		Long:  `Parse status/references/*.md files and insert into reference_docs table.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			files, err := filepath.Glob("status/references/*.md")
			if err != nil {
				return err
			}

			if len(files) == 0 {
				return output.Success(map[string]interface{}{
					"msg": "No reference files found",
					"cnt": 0,
				})
			}

			migrated := 0
			for _, file := range files {
				if err := migrateReferenceFile(file); err != nil {
					fmt.Fprintf(os.Stderr, "Warning: failed to migrate %s: %v\n", file, err)
					continue
				}
				migrated++
			}

			return output.Success(map[string]interface{}{
				"msg":      "Reference migration complete",
				"migrated": migrated,
			})
		},
	}

	return cmd
}

func migrateReferenceFile(path string) error {
	content, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	text := string(content)
	lines := strings.Split(text, "\n")

	// Extract name from filename: phase-mapping.md → phase-mapping
	name := strings.TrimSuffix(filepath.Base(path), ".md")

	// Extract title (first # heading)
	var title string
	foundTitle := false

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		if strings.HasPrefix(trimmed, "# ") && !foundTitle {
			title = strings.TrimPrefix(trimmed, "# ")
			foundTitle = true
			break
		}
	}

	if title == "" {
		title = strings.ReplaceAll(name, "-", " ")
		title = strings.Title(title)
	}

	// Determine type from name
	refType := "mapping"
	if strings.Contains(name, "standard") || strings.Contains(name, "quality") {
		refType = "standards"
	} else if strings.Contains(name, "checklist") || strings.Contains(name, "verification") {
		refType = "checklist"
	} else if strings.Contains(name, "documentation") {
		refType = "mapping"
	}

	// Insert into DB (OR IGNORE to skip duplicates)
	_, err = database.Exec(`
		INSERT OR IGNORE INTO reference_docs (name, title, type, content)
		VALUES (?, ?, ?, ?)
	`, name, title, refType, text)

	return err
}
