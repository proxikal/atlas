package main

import (
	"fmt"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func referenceReadCmd() *cobra.Command {
	var (
		grep     string
		lines    string
		maxLines int
	)

	cmd := &cobra.Command{
		Use:   "read <name>",
		Short: "Read reference doc",
		Long:  `Read reference doc with surgical output options. Default: summary only (title + first 5 lines).`,
		Example: `  # Summary (title + first 5 lines)
  atlas-dev reference read phase-mapping

  # Grep for pattern
  atlas-dev reference read phase-mapping --grep "foundation"

  # Specific line range
  atlas-dev reference read phase-mapping --lines 10-20

  # All lines (limited to 100 max)
  atlas-dev reference read phase-mapping --lines all`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

			// Get reference doc
			var title, refType, content string
			err := database.QueryRow(`
				SELECT title, type, content FROM reference_docs WHERE name = ?
			`, name).Scan(&title, &refType, &content)
			if err != nil {
				return fmt.Errorf("reference not found: %s", name)
			}

			result := map[string]interface{}{
				"name": name,
				"ttl":  title,
				"type": refType,
			}

			// Process content based on flags
			contentLines := strings.Split(content, "\n")

			if grep != "" {
				// Grep mode: show matching lines only
				matched := []string{}
				pattern, err := regexp.Compile("(?i)" + grep)
				if err != nil {
					return fmt.Errorf("invalid grep pattern: %w", err)
				}

				for i, line := range contentLines {
					if pattern.MatchString(line) {
						matched = append(matched, fmt.Sprintf("%d: %s", i+1, line))
					}
				}

				result["matches"] = len(matched)
				result["lines"] = matched
				return output.Success(result)
			}

			if lines != "" {
				// Line range mode
				if lines == "all" {
					maxLines = 100 // Hard limit
					if len(contentLines) > maxLines {
						result["truncated"] = true
						result["total"] = len(contentLines)
						contentLines = contentLines[:maxLines]
					}
					result["lines"] = contentLines
					return output.Success(result)
				}

				// Parse range: "10-20"
				var start, end int
				_, err := fmt.Sscanf(lines, "%d-%d", &start, &end)
				if err != nil {
					return fmt.Errorf("invalid line range (use N-M format): %s", lines)
				}

				if start < 1 || end > len(contentLines) || start > end {
					return fmt.Errorf("invalid range: %s (file has %d lines)", lines, len(contentLines))
				}

				result["lines"] = contentLines[start-1:end]
				return output.Success(result)
			}

			// Default: summary (first 5 lines)
			summaryLines := 5
			if len(contentLines) > summaryLines {
				result["truncated"] = true
				result["total"] = len(contentLines)
				contentLines = contentLines[:summaryLines]
			}
			result["lines"] = contentLines

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&grep, "grep", "", "Show only lines matching pattern")
	cmd.Flags().StringVar(&lines, "lines", "", "Line range (N-M) or 'all' (max 100)")

	return cmd
}
