package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionExportCmd() *cobra.Command {
	var outputDir string

	cmd := &cobra.Command{
		Use:   "export",
		Short: "Export decisions to markdown",
		Long:  `Export all decisions to markdown files grouped by component.`,
		Example: `  # Export to docs/decisions/
  atlas-dev decision export --output docs/decisions`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Get all decisions
			decisions, err := database.ListDecisions(db.ListDecisionsOptions{
				Limit: 1000, // Get all
			})
			if err != nil {
				return err
			}

			if len(decisions) == 0 {
				return output.Success(map[string]interface{}{
					"msg":   "No decisions to export",
					"files": 0,
				})
			}

			// Group by component
			byComponent := make(map[string][]*db.DecisionListItem)
			for _, d := range decisions {
				byComponent[d.Component] = append(byComponent[d.Component], d)
			}

			// Create output directory
			if err := os.MkdirAll(outputDir, 0755); err != nil {
				return fmt.Errorf("failed to create output directory: %w", err)
			}

			filesCreated := 0

			// Export each component
			for component, componentDecisions := range byComponent {
				filename := filepath.Join(outputDir, fmt.Sprintf("%s-decisions.md", component))

				var content strings.Builder
				content.WriteString(fmt.Sprintf("# %s Decisions\n\n", strings.Title(component)))
				content.WriteString("| ID | Title | Date | Status |\n")
				content.WriteString("|----|----|------|--------|\n")

				for _, d := range componentDecisions {
					content.WriteString(fmt.Sprintf("| %s | %s | %s | %s |\n",
						d.ID, d.Title, d.Date, d.Status))
				}

				if err := os.WriteFile(filename, []byte(content.String()), 0644); err != nil {
					return fmt.Errorf("failed to write file %s: %w", filename, err)
				}

				filesCreated++
			}

			return output.Success(map[string]interface{}{
				"msg":       "Decisions exported",
				"files":     filesCreated,
				"decisions": len(decisions),
				"output":    outputDir,
			})
		},
	}

	cmd.Flags().StringVarP(&outputDir, "output", "o", "docs/decisions", "Output directory")

	return cmd
}
