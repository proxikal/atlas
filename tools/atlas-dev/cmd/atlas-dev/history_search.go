package main

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func historySearchCmd() *cobra.Command {
	var contextLines int

	cmd := &cobra.Command{
		Use:   "search <pattern>",
		Short: "Search history entries",
		Long:  `Grep through all history content. Returns matching lines with entry name and line number.`,
		Example: `  # Search for pattern
  atlas-dev history search "complete"

  # Case-sensitive search
  atlas-dev history search "Phase"

  # With context lines
  atlas-dev history search "migration" --context 2`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			pattern := args[0]

			// Compile regex
			re, err := regexp.Compile("(?i)" + pattern)
			if err != nil {
				return fmt.Errorf("invalid pattern: %w", err)
			}

			// Get all history entries
			rows, err := database.Query(`SELECT name, title, content FROM history ORDER BY date DESC`)
			if err != nil {
				return err
			}
			defer rows.Close()

			type match struct {
				Entry string `json:"entry"`
				Line  int    `json:"line"`
				Text  string `json:"text"`
			}

			matches := []match{}
			totalEntries := 0
			matchedEntries := 0

			for rows.Next() {
				var name, title, content string
				if err := rows.Scan(&name, &title, &content); err != nil {
					return err
				}

				totalEntries++
				lines := strings.Split(content, "\n")
				entryMatched := false

				for i, line := range lines {
					if re.MatchString(line) {
						matches = append(matches, match{
							Entry: name,
							Line:  i + 1,
							Text:  strings.TrimSpace(line),
						})
						entryMatched = true
					}
				}

				if entryMatched {
					matchedEntries++
				}
			}

			return output.Success(map[string]interface{}{
				"pattern":  pattern,
				"matches":  len(matches),
				"entries":  matchedEntries,
				"total":    totalEntries,
				"results":  matches,
			})
		},
	}

	cmd.Flags().IntVar(&contextLines, "context", 0, "Show N lines of context around matches")

	return cmd
}
