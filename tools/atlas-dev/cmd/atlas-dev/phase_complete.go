package main

import (
	"fmt"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/git"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseCompleteCmd() *cobra.Command {
	var (
		description string
		date        string
		commit      bool
		dryRun      bool
		tests       int
	)

	cmd := &cobra.Command{
		Use:   "complete <phase-path>",
		Short: "Mark a phase as complete",
		Long: `Mark a phase as complete and automatically update all tracking.

Updates:
- Phase status to "completed"
- Category progress (via SQL triggers)
- Total progress metadata (via SQL triggers)
- Audit log entry
- Optional git commit

Example:
  atlas-dev phase complete phases/stdlib/phase-07b.md \
    --desc "HashSet with 25 tests, 100% parity" \
    --tests 25 \
    --commit

  # With stdin (auto-detected)
  echo '{"path":"phases/stdlib/phase-07b.md"}' | \
    atlas-dev phase complete --desc "Complete" --tests 25`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var phasePath string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				phasePath, err = compose.ExtractFirstPath(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("phase path required")
				}
				phasePath = args[0]
			}

			// Use current date if not specified
			if date == "" {
				date = time.Now().Format(time.RFC3339)
			}

			// Complete phase
			result, err := database.CompletePhase(db.CompletePhaseRequest{
				PhasePath:   phasePath,
				Description: description,
				Date:        date,
				TestCount:   tests,
				DryRun:      dryRun,
			})
			if err != nil {
				return err
			}

			// Create git commit if requested
			var commitSHA string
			if commit && !dryRun {
				sha, err := git.CommitPhase(result.PhaseName, description)
				if err != nil {
					// Git commit failed, but phase was updated
					// Return warning but don't fail
					return output.Success(map[string]interface{}{
						"phase": result.PhaseName,
						"cat":   result.Category,
						"progress": map[string]interface{}{
							"cat": []int{result.CategoryProgress.Completed, result.CategoryProgress.Total, result.CategoryProgress.Percentage},
							"tot": []int{result.TotalProgress.Completed, result.TotalProgress.Total, result.TotalProgress.Percentage},
						},
						"next":       formatNextPhase(result.NextPhase),
						"git_error":  err.Error(),
						"git_warning": "Phase completed but git commit failed",
					})
				}
				commitSHA = sha
			}

			// Build response
			response := map[string]interface{}{
				"phase": result.PhaseName,
				"cat":   result.Category,
				"progress": map[string]interface{}{
					"cat": []int{result.CategoryProgress.Completed, result.CategoryProgress.Total, result.CategoryProgress.Percentage},
					"tot": []int{result.TotalProgress.Completed, result.TotalProgress.Total, result.TotalProgress.Percentage},
				},
			}

			if result.NextPhase != nil {
				response["next"] = formatNextPhase(result.NextPhase)
			}

			if commitSHA != "" {
				response["commit"] = commitSHA
			}

			if dryRun {
				response["dry_run"] = true
			}

			return output.Success(response)
		},
	}

	cmd.Flags().StringVarP(&description, "desc", "d", "", "Phase completion description (required)")
	cmd.Flags().StringVar(&date, "date", "", "Completion date (default: today, ISO 8601 format)")
	cmd.Flags().BoolVarP(&commit, "commit", "c", false, "Create git commit")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Show what would change without modifying database")
	cmd.Flags().IntVar(&tests, "tests", 0, "Number of tests added")

	_ = cmd.MarkFlagRequired("desc")

	return cmd
}

func formatNextPhase(phase *db.PhaseListItem) string {
	if phase == nil {
		return ""
	}
	return phase.Name
}
