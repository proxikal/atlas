package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionUpdateCmd() *cobra.Command {
	var (
		status       string
		supersededBy string
		dryRun       bool
	)

	cmd := &cobra.Command{
		Use:   "update <id>",
		Short: "Update decision status",
		Long:  `Update decision status or mark as superseded.`,
		Example: `  # Update status
  atlas-dev decision update DR-001 --status accepted

  # Mark as superseded
  atlas-dev decision update DR-001 --superseded-by DR-002

  # Update from stdin (auto-detected)
  echo '{"id":"DR-001"}' | atlas-dev decision update --status accepted`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var id string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				id, err = compose.ExtractFirstID(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("decision ID required")
				}
				id = args[0]
			}

			if status == "" && supersededBy == "" {
				return fmt.Errorf("must provide --status or --superseded-by")
			}

			// Dry-run: preview changes
			if dryRun {
				// Get current state (read-only)
				current, err := database.GetDecision(id)
				if err != nil {
					return err
				}

				result := map[string]interface{}{
					"dry_run": true,
					"op":      "update_decision",
					"id":      id,
					"before": map[string]interface{}{
						"stat": current.Status,
					},
					"after": map[string]interface{}{},
					"msg":   "Preview only - no changes made",
				}

				if status != "" {
					result["after"].(map[string]interface{})["stat"] = status
				}
				if supersededBy != "" {
					result["after"].(map[string]interface{})["super"] = supersededBy
				}

				return output.Success(result)
			}

			req := db.UpdateDecisionRequest{
				ID:           id,
				Status:       status,
				SupersededBy: supersededBy,
			}

			decision, err := database.UpdateDecision(req)
			if err != nil {
				return err
			}

			result := decision.ToCompactJSON()
			result["msg"] = "Decision updated"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&status, "status", "", "New status")
	cmd.Flags().StringVar(&supersededBy, "superseded-by", "", "Superseded by decision ID")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview changes without applying")

	return cmd
}
