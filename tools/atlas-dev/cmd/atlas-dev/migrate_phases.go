package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migratePhasesCmd() *cobra.Command {
	var dryRun bool

	cmd := &cobra.Command{
		Use:   "phases",
		Short: "Migrate remaining phase files to database",
		Long:  `Parse all phase markdown files and insert missing ones into database.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Get list of phase files not in DB
			missing, err := getMissingPhases()
			if err != nil {
				return err
			}

			if len(missing) == 0 {
				return output.Success(map[string]interface{}{
					"msg": "All phases already migrated",
					"cnt": 0,
				})
			}

			if dryRun {
				fmt.Printf("Would migrate %d phases:\n", len(missing))
				for _, path := range missing {
					fmt.Printf("  - %s\n", path)
				}
				return nil
			}

			// Migrate each phase
			migrated := 0
			for _, path := range missing {
				if err := migratePhaseFile(path); err != nil {
					fmt.Fprintf(os.Stderr, "Warning: failed to migrate %s: %v\n", path, err)
					continue
				}
				migrated++
			}

			return output.Success(map[string]interface{}{
				"msg":      "Phase migration complete",
				"migrated": migrated,
				"total":    len(missing),
			})
		},
	}

	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Show what would be migrated")
	return cmd
}

func getMissingPhases() ([]string, error) {
	// Get all current phase files
	allPhases, err := filepath.Glob("phases/**/phase-*.md")
	if err != nil {
		return nil, err
	}

	// Filter out archived phases
	var current []string
	for _, path := range allPhases {
		if !strings.Contains(path, "/archive/") && !strings.Contains(path, "/blockers/") {
			current = append(current, path)
		}
	}

	// Get phases already in DB
	rows, err := database.Query("SELECT path FROM phases")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	inDB := make(map[string]bool)
	for rows.Next() {
		var path string
		if err := rows.Scan(&path); err != nil {
			return nil, err
		}
		inDB[path] = true
	}

	// Find missing
	var missing []string
	for _, path := range current {
		if !inDB[path] {
			missing = append(missing, path)
		}
	}

	return missing, nil
}

func migratePhaseFile(path string) error {
	// Parse phase file
	phase, err := parsePhaseFile(path)
	if err != nil {
		return err
	}

	// Insert into DB
	_, err = database.Exec(`
		INSERT INTO phases (path, name, category, status, description, blockers, dependencies, acceptance_criteria)
		VALUES (?, ?, ?, 'pending', ?, ?, ?, ?)
	`, phase.Path, phase.Name, phase.Category, phase.Description, phase.Blockers, phase.Dependencies, phase.Acceptance)

	return err
}

type phaseMetadata struct {
	Path         string
	Name         string
	Category     string
	Description  string
	Blockers     string
	Dependencies string
	Acceptance   string
}

func parsePhaseFile(path string) (*phaseMetadata, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	text := string(content)
	lines := strings.Split(text, "\n")

	phase := &phaseMetadata{
		Path: path,
	}

	// Extract category from path: phases/foundation/phase-01.md → foundation
	parts := strings.Split(path, "/")
	if len(parts) >= 2 {
		phase.Category = parts[1]
	}

	// Extract name from path: phase-01-runtime-api.md → phase-01-runtime-api
	basename := filepath.Base(path)
	phase.Name = strings.TrimSuffix(basename, ".md")

	// Extract title/objective as description
	titleRe := regexp.MustCompile(`^#\s+(.+)`)
	objectiveRe := regexp.MustCompile(`(?i)^##\s+Objective\s*$`)
	blockerRe := regexp.MustCompile(`(?i)^##\s+.*(?:BLOCKER|DEPENDENCIES)`)
	acceptanceRe := regexp.MustCompile(`(?i)^##\s+Acceptance`)

	var currentSection string
	var objectiveLines []string
	var blockerLines []string
	var acceptanceLines []string

	for i, line := range lines {
		// Get title as default description
		if match := titleRe.FindStringSubmatch(line); match != nil && phase.Description == "" {
			phase.Description = strings.TrimPrefix(match[1], "Phase ")
			phase.Description = strings.TrimPrefix(phase.Description, fmt.Sprintf("%s: ", strings.ToUpper(phase.Category)))
			continue
		}

		// Section headers
		if objectiveRe.MatchString(line) {
			currentSection = "objective"
			continue
		}
		if blockerRe.MatchString(line) {
			currentSection = "blocker"
			continue
		}
		if acceptanceRe.MatchString(line) {
			currentSection = "acceptance"
			continue
		}
		if strings.HasPrefix(line, "## ") {
			currentSection = ""
			continue
		}

		// Collect section content
		trimmed := strings.TrimSpace(line)
		if trimmed == "" || strings.HasPrefix(trimmed, "```") || strings.HasPrefix(trimmed, "---") {
			continue
		}

		switch currentSection {
		case "objective":
			if i+1 < len(lines) && !strings.HasPrefix(lines[i+1], "##") {
				objectiveLines = append(objectiveLines, trimmed)
			}
		case "blocker":
			if strings.HasPrefix(trimmed, "**") || strings.HasPrefix(trimmed, "-") {
				blockerLines = append(blockerLines, trimmed)
			}
		case "acceptance":
			if strings.HasPrefix(trimmed, "-") {
				acceptanceLines = append(acceptanceLines, trimmed)
			}
		}
	}

	// Use objective as description if available
	if len(objectiveLines) > 0 {
		phase.Description = strings.Join(objectiveLines, " ")
		// Truncate to reasonable length
		if len(phase.Description) > 500 {
			phase.Description = phase.Description[:497] + "..."
		}
	}

	phase.Blockers = strings.Join(blockerLines, "\n")
	phase.Dependencies = phase.Blockers // Same for now
	phase.Acceptance = strings.Join(acceptanceLines, "\n")

	return phase, nil
}
