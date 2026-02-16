package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateHistoryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "history",
		Short: "Migrate history files to database",
		Long:  `Parse status/history/*.md files and insert into history table.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			files, err := filepath.Glob("status/history/*.md")
			if err != nil {
				return err
			}

			if len(files) == 0 {
				return output.Success(map[string]interface{}{
					"msg": "No history files found",
					"cnt": 0,
				})
			}

			migrated := 0
			for _, file := range files {
				if err := migrateHistoryFile(file); err != nil {
					fmt.Fprintf(os.Stderr, "Warning: failed to migrate %s: %v\n", file, err)
					continue
				}
				migrated++
			}

			return output.Success(map[string]interface{}{
				"msg":      "History migration complete",
				"migrated": migrated,
			})
		},
	}

	return cmd
}

func migrateHistoryFile(path string) error {
	content, err := os.ReadFile(path)
	if err != nil {
		return err
	}

	text := string(content)
	lines := strings.Split(text, "\n")

	// Extract name from filename: v0.1-summary.md → v0.1-summary
	name := strings.TrimSuffix(filepath.Base(path), ".md")

	// Extract title (first # heading)
	var title string
	var summaryLines []string
	foundTitle := false

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		if strings.HasPrefix(trimmed, "# ") && !foundTitle {
			title = strings.TrimPrefix(trimmed, "# ")
			foundTitle = true
			continue
		}

		// Collect first few non-empty lines as summary
		if foundTitle && trimmed != "" && !strings.HasPrefix(trimmed, "#") && len(summaryLines) < 3 {
			summaryLines = append(summaryLines, trimmed)
		}
	}

	if title == "" {
		title = name
	}

	summary := strings.Join(summaryLines, " ")
	if len(summary) > 500 {
		summary = summary[:497] + "..."
	}
	if summary == "" {
		summary = "History entry: " + name
	}

	// Determine type from name
	htype := "summary"
	if strings.Contains(name, "restructure") {
		htype = "restructure"
	}

	// Extract date from content or use current
	date := "2026-02-15" // Default
	for _, line := range lines {
		if strings.Contains(line, "2026-") || strings.Contains(line, "2025-") || strings.Contains(line, "2024-") {
			// Try to extract date
			if match := strings.Index(line, "202"); match >= 0 {
				potential := line[match : match+10]
				if len(potential) == 10 && potential[4] == '-' && potential[7] == '-' {
					date = potential
					break
				}
			}
		}
	}

	// Insert into DB (OR IGNORE to skip duplicates)
	_, err = database.Exec(`
		INSERT OR IGNORE INTO history (name, title, date, summary, content, type)
		VALUES (?, ?, ?, ?, ?, ?)
	`, name, title, date, summary, text, htype)

	return err
}
