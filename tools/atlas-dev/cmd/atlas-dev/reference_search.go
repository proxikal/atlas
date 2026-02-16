package main

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func referenceSearchCmd() *cobra.Command {
	var refType string

	cmd := &cobra.Command{
		Use:   "search <pattern>",
		Short: "Search reference docs",
		Long:  `Grep through all reference doc content. Returns matching lines with doc name and line number.`,
		Example: `  # Search for pattern
  atlas-dev reference search "phase"

  # Search in specific type
  atlas-dev reference search "mapping" --type mapping

  # Case-sensitive search
  atlas-dev reference search "Foundation"`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			pattern := args[0]

			// Compile regex
			re, err := regexp.Compile("(?i)" + pattern)
			if err != nil {
				return fmt.Errorf("invalid pattern: %w", err)
			}

			// Build query
			query := `SELECT name, title, type, content FROM reference_docs`
			var queryArgs []interface{}

			if refType != "" {
				query += ` WHERE type = ?`
				queryArgs = append(queryArgs, refType)
			}

			query += ` ORDER BY type, name`

			rows, err := database.Query(query, queryArgs...)
			if err != nil {
				return err
			}
			defer rows.Close()

			type match struct {
				Doc  string `json:"doc"`
				Type string `json:"type"`
				Line int    `json:"line"`
				Text string `json:"text"`
			}

			matches := []match{}
			totalDocs := 0
			matchedDocs := 0

			for rows.Next() {
				var name, title, docType, content string
				if err := rows.Scan(&name, &title, &docType, &content); err != nil {
					return err
				}

				totalDocs++
				lines := strings.Split(content, "\n")
				docMatched := false

				for i, line := range lines {
					if re.MatchString(line) {
						matches = append(matches, match{
							Doc:  name,
							Type: docType,
							Line: i + 1,
							Text: strings.TrimSpace(line),
						})
						docMatched = true
					}
				}

				if docMatched {
					matchedDocs++
				}
			}

			return output.Success(map[string]interface{}{
				"pattern": pattern,
				"matches": len(matches),
				"docs":    matchedDocs,
				"total":   totalDocs,
				"results": matches,
			})
		},
	}

	cmd.Flags().StringVar(&refType, "type", "", "Filter by reference type (mapping, standards, checklist)")

	return cmd
}
