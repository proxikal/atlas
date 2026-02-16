package main

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate",
		Short: "Database migration commands",
		Long:  `Migrate existing markdown files to SQLite or initialize fresh schema.`,
	}

	cmd.AddCommand(migrateSchemaCmd())
	cmd.AddCommand(migrateBootstrapCmd())
	cmd.AddCommand(migratePhasesCmd())
	cmd.AddCommand(migrateHistoryCmd())
	cmd.AddCommand(migrateReferencesCmd())
	cmd.AddCommand(migrateComponentsCmd())
	cmd.AddCommand(migrateCleanupCmd())

	return cmd
}

func migrateSchemaCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "schema",
		Short: "Initialize database schema",
		Long:  `Create all tables, indexes, triggers, views and seed initial data.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Initialize schema
			if err := database.InitSchema(); err != nil {
				return fmt.Errorf("failed to initialize schema: %w", err)
			}

			return output.Success(map[string]interface{}{
				"msg":    "Schema initialized successfully",
				"tables": 10,
				"views":  4,
			})
		},
	}
}

func migrateBootstrapCmd() *cobra.Command {
	var force bool

	cmd := &cobra.Command{
		Use:   "bootstrap",
		Short: "Bootstrap database from existing markdown files",
		Long: `One-time migration: parse STATUS.md and trackers/*.md to populate database.
Backs up markdown files to .migration-backup/ directory.

CRITICAL: This should only be run ONCE. After migration, markdown files will be deleted.
Re-running this command after deletion would be catastrophic (no source data).`,
		Example: `  # First-time migration
  atlas-dev migrate bootstrap

  # Force re-migration (WARNING: destructive)
  atlas-dev migrate bootstrap --force`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Check if already migrated
			migrated, err := database.IsMigrated()
			if err != nil {
				return fmt.Errorf("failed to check migration status: %w", err)
			}

			if migrated && !force {
				return fmt.Errorf("database already migrated - markdown files may have been deleted\n\nIf you really want to re-run migration, use --force flag (WARNING: may cause data loss)")
			}

			if force {
				return fmt.Errorf("force re-migration not safe - would require manual intervention")
			}

			// Run bootstrap migration
			result, err := runBootstrapMigration()
			if err != nil {
				return err
			}

			// Mark as migrated
			if err := database.MarkAsMigrated(); err != nil {
				return fmt.Errorf("failed to mark as migrated: %w", err)
			}

			return output.Success(result)
		},
	}

	cmd.Flags().BoolVar(&force, "force", false, "Force re-migration (WARNING: may cause data loss)")

	return cmd
}

// Bootstrap migration types
type statusData struct {
	Version         string
	LastUpdated     string
	LastCompleted   string
	NextPhase       string
	CompletedPhases int
	TotalPhases     int
}

type phaseData struct {
	Name          string
	Description   string
	CompletedDate string
	Status        string
}

type trackerData struct {
	TrackerNum      int
	Category        string
	CompletedPhases []phaseData
	PendingPhases   []phaseData
	Completed       int
	Total           int
	Percentage      int
}

func runBootstrapMigration() (map[string]interface{}, error) {
	// 1. Check STATUS.md exists
	statusPath := "../../STATUS.md"
	if _, err := os.Stat(statusPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("STATUS.md not found at %s - are you running from tools/atlas-dev/?", statusPath)
	}

	// 2. Parse STATUS.md
	status, err := parseStatusMd(statusPath)
	if err != nil {
		return nil, fmt.Errorf("failed to parse STATUS.md: %w", err)
	}

	// 3. Parse all tracker files
	trackers, err := parseTrackers("../../status/trackers")
	if err != nil {
		return nil, fmt.Errorf("failed to parse trackers: %w", err)
	}

	// 4. Insert data into database
	phasesInserted := 0
	categoriesInserted := 0

	// Track seen paths to skip duplicates
	seenPaths := make(map[string]bool)

	err = database.WithTransaction(func(tx *db.Transaction) error {
		// Insert categories and phases
		phaseID := 1
		for _, tracker := range trackers {
			// Insert category
			_, err := tx.Exec(`
				INSERT OR REPLACE INTO categories (id, name, display_name, completed, total, percentage, status, updated_at)
				VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
			`, tracker.TrackerNum, tracker.Category, categoryDisplayName(tracker.Category),
				tracker.Completed, tracker.Total, tracker.Percentage, categoryStatus(tracker.Percentage))
			if err != nil {
				return fmt.Errorf("failed to insert category %s: %w", tracker.Category, err)
			}
			categoriesInserted++

			// Insert completed phases
			for _, p := range tracker.CompletedPhases {
				path := fmt.Sprintf("phases/%s/%s", tracker.Category, p.Name)
				if seenPaths[path] {
					continue // Skip duplicate
				}
				seenPaths[path] = true

				_, err := tx.Exec(`
					INSERT INTO phases (id, path, name, category, status, completed_date, description, created_at, updated_at)
					VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
				`, phaseID, path, strings.TrimSuffix(p.Name, ".md"), tracker.Category, "completed", p.CompletedDate, p.Description)
				if err != nil {
					return fmt.Errorf("failed to insert phase %s: %w", p.Name, err)
				}
				phaseID++
				phasesInserted++
			}

			// Insert pending/blocked phases
			for _, p := range tracker.PendingPhases {
				path := fmt.Sprintf("phases/%s/%s", tracker.Category, p.Name)
				if seenPaths[path] {
					continue // Skip duplicate
				}
				seenPaths[path] = true

				_, err := tx.Exec(`
					INSERT INTO phases (id, path, name, category, status, description, created_at, updated_at)
					VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))
				`, phaseID, path, strings.TrimSuffix(p.Name, ".md"), tracker.Category, p.Status, p.Description)
				if err != nil {
					return fmt.Errorf("failed to insert phase %s: %w", p.Name, err)
				}
				phaseID++
				phasesInserted++
			}
		}

		// Insert metadata
		_, err := tx.Exec(`INSERT OR REPLACE INTO metadata (key, value, updated_at) VALUES ('atlas_version', ?, datetime('now'))`, status.Version)
		if err != nil {
			return fmt.Errorf("failed to insert metadata: %w", err)
		}

		// Insert audit log
		_, err = tx.Exec(`
			INSERT INTO audit_log (action, entity_type, entity_id, changes, created_at)
			VALUES ('migration_bootstrap', 'system', 'initial', ?, datetime('now'))
		`, fmt.Sprintf(`{"phases_migrated": %d, "categories_migrated": %d}`, phasesInserted, categoriesInserted))
		if err != nil {
			return fmt.Errorf("failed to insert audit log: %w", err)
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	// 5. Backup markdown files
	if err := backupMarkdownFiles(); err != nil {
		return nil, fmt.Errorf("failed to backup files: %w", err)
	}

	return map[string]interface{}{
		"ok":         true,
		"msg":        "Bootstrap migration complete",
		"phases":     phasesInserted,
		"categories": categoriesInserted,
		"completed":  status.CompletedPhases,
		"backup":     ".migration-backup/",
	}, nil
}

func parseStatusMd(path string) (*statusData, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	data := &statusData{}
	text := string(content)

	// Extract metadata
	data.Version = extractPattern(text, `\*\*Version:\*\* (.+)`)
	data.LastUpdated = extractPattern(text, `\*\*Last Updated:\*\* (.+)`)
	data.LastCompleted = extractPattern(text, `\*\*Last Completed:\*\* (.+)`)
	data.NextPhase = extractPattern(text, `\*\*Next Phase:\*\* (.+)`)

	// Extract progress
	progressStr := extractPattern(text, `\*\*Real Progress:\*\* (\d+)/(\d+)`)
	fmt.Sscanf(progressStr, "%d/%d", &data.CompletedPhases, &data.TotalPhases)

	return data, nil
}

func parseTrackers(dir string) ([]trackerData, error) {
	files, err := filepath.Glob(filepath.Join(dir, "*.md"))
	if err != nil {
		return nil, err
	}

	if len(files) == 0 {
		return nil, fmt.Errorf("no tracker files found in %s", dir)
	}

	var trackers []trackerData
	for _, file := range files {
		tracker, err := parseTrackerFile(file)
		if err != nil {
			return nil, fmt.Errorf("failed to parse %s: %w", file, err)
		}
		trackers = append(trackers, tracker)
	}

	return trackers, nil
}

func parseTrackerFile(path string) (trackerData, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return trackerData{}, err
	}

	data := trackerData{}

	// Extract category from filename: "1-stdlib.md" → stdlib, tracker_num=1
	filename := filepath.Base(path)
	parts := strings.SplitN(strings.TrimSuffix(filename, ".md"), "-", 2)
	if len(parts) == 2 {
		fmt.Sscanf(parts[0], "%d", &data.TrackerNum)
		data.Category = parts[1]
	}

	text := string(content)

	// Parse completed phases: - ✅ phase-name.md **[Description, 2024-01-15]**
	completedRe := regexp.MustCompile(`(?m)^- ✅ (.+?)\.md \*\*\[(.+?), (\d{4}-\d{2}-\d{2})\]\*\*`)
	matches := completedRe.FindAllStringSubmatch(text, -1)
	for _, match := range matches {
		if len(match) >= 4 {
			data.CompletedPhases = append(data.CompletedPhases, phaseData{
				Name:          match[1] + ".md",
				Description:   match[2],
				CompletedDate: match[3],
				Status:        "completed",
			})
		}
	}

	// Parse pending phases: - ⬜ phase-name.md **[Description]**
	pendingRe := regexp.MustCompile(`(?m)^- ⬜ (.+?)\.md \*\*\[(.+?)\]\*\*`)
	matches = pendingRe.FindAllStringSubmatch(text, -1)
	for _, match := range matches {
		if len(match) >= 3 {
			data.PendingPhases = append(data.PendingPhases, phaseData{
				Name:        match[1] + ".md",
				Description: match[2],
				Status:      "pending",
			})
		}
	}

	// Parse blocked phases: - 🚨 phase-name.md **[BLOCKED: reason]**
	blockedRe := regexp.MustCompile(`(?m)^- 🚨 (.+?)\.md \*\*\[BLOCKED: (.+?)\]\*\*`)
	matches = blockedRe.FindAllStringSubmatch(text, -1)
	for _, match := range matches {
		if len(match) >= 3 {
			data.PendingPhases = append(data.PendingPhases, phaseData{
				Name:        match[1] + ".md",
				Description: "BLOCKED: " + match[2],
				Status:      "blocked",
			})
		}
	}

	data.Completed = len(data.CompletedPhases)
	data.Total = len(data.CompletedPhases) + len(data.PendingPhases)
	if data.Total > 0 {
		data.Percentage = int(float64(data.Completed) / float64(data.Total) * 100)
	}

	return data, nil
}

func extractPattern(text, pattern string) string {
	re := regexp.MustCompile(pattern)
	matches := re.FindStringSubmatch(text)
	if len(matches) > 1 {
		return strings.TrimSpace(matches[1])
	}
	return ""
}

func categoryDisplayName(name string) string {
	displayNames := map[string]string{
		"foundation":  "Foundation",
		"stdlib":      "Standard Library",
		"bytecode-vm": "Bytecode & VM",
		"frontend":    "Frontend",
		"typing":      "Type System",
		"interpreter": "Interpreter",
		"cli":         "CLI",
		"lsp":         "LSP",
		"polish":      "Polish & Finalization",
	}
	if display, ok := displayNames[name]; ok {
		return display
	}
	return strings.Title(name)
}

func categoryStatus(percentage int) string {
	if percentage == 0 {
		return "pending"
	} else if percentage == 100 {
		return "complete"
	}
	return "active"
}

func backupMarkdownFiles() error {
	backupDir := "../../.migration-backup"
	if err := os.MkdirAll(backupDir, 0755); err != nil {
		return err
	}

	// Backup STATUS.md
	if err := copyFile("../../STATUS.md", filepath.Join(backupDir, "STATUS.md")); err != nil {
		return err
	}

	// Backup trackers
	trackerBackupDir := filepath.Join(backupDir, "status", "trackers")
	if err := os.MkdirAll(trackerBackupDir, 0755); err != nil {
		return err
	}

	files, err := filepath.Glob("../../status/trackers/*.md")
	if err != nil {
		return err
	}

	for _, file := range files {
		dest := filepath.Join(trackerBackupDir, filepath.Base(file))
		if err := copyFile(file, dest); err != nil {
			return err
		}
	}

	return nil
}

func copyFile(src, dst string) error {
	content, err := os.ReadFile(src)
	if err != nil {
		return err
	}
	return os.WriteFile(dst, content, 0644)
}
