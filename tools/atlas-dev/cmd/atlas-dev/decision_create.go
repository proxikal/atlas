package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionCreateCmd() *cobra.Command {
	var (
		component     string
		title         string
		decisionText  string
		rationale     string
		alternatives  string
		consequences  string
		status        string
		relatedPhases string
		tags          string
	)

	cmd := &cobra.Command{
		Use:   "create",
		Short: "Create a new decision log",
		Long:  `Create a new architectural decision with auto-generated ID (DR-001, DR-002, etc).`,
		Example: `  # Create decision
  atlas-dev decision create \
    --component stdlib \
    --title "Hash function design" \
    --decision "Use FNV-1a for HashMap" \
    --rationale "Fast, simple, good distribution"`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Validate required fields
			if component == "" {
				return fmt.Errorf("--component is required")
			}
			if title == "" {
				return fmt.Errorf("--title is required")
			}
			if decisionText == "" {
				return fmt.Errorf("--decision is required")
			}
			if rationale == "" {
				return fmt.Errorf("--rationale is required")
			}

			// Create decision
			req := db.CreateDecisionRequest{
				Component:     component,
				Title:         title,
				DecisionText:  decisionText,
				Rationale:     rationale,
				Alternatives:  alternatives,
				Consequences:  consequences,
				Status:        status,
				RelatedPhases: relatedPhases,
				Tags:          tags,
			}

			decision, err := database.CreateDecision(req)
			if err != nil {
				return err
			}

			result := decision.ToCompactJSON()
			result["msg"] = "Decision created"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&component, "component", "", "Component/category (required)")
	cmd.Flags().StringVar(&title, "title", "", "Decision title (required)")
	cmd.Flags().StringVar(&decisionText, "decision", "", "Decision text (required)")
	cmd.Flags().StringVar(&rationale, "rationale", "", "Rationale (required)")
	cmd.Flags().StringVar(&alternatives, "alternatives", "", "Alternatives considered")
	cmd.Flags().StringVar(&consequences, "consequences", "", "Consequences")
	cmd.Flags().StringVar(&status, "status", "accepted", "Status (proposed, accepted, rejected)")
	cmd.Flags().StringVar(&relatedPhases, "related-phases", "", "Related phase IDs")
	cmd.Flags().StringVar(&tags, "tags", "", "Tags (comma-separated)")

	return cmd
}
