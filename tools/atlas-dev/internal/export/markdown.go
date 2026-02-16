package export

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"text/template"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// MarkdownExporter generates markdown files from database
type MarkdownExporter struct {
	db *db.DB
}

// NewMarkdownExporter creates a new markdown exporter
func NewMarkdownExporter(database *db.DB) *MarkdownExporter {
	return &MarkdownExporter{db: database}
}

// ExportResult represents the result of an export operation
type ExportResult struct {
	OutputDir    string   `json:"output_dir"`
	FilesCreated []string `json:"files_created"`
	FileCount    int      `json:"file_count"`
}

// Export generates STATUS.md and tracker files from database
func (e *MarkdownExporter) Export(outputDir string) (*ExportResult, error) {
	// Create output directory
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create output directory: %w", err)
	}

	result := &ExportResult{
		OutputDir:    outputDir,
		FilesCreated: []string{},
	}

	// Generate STATUS.md
	statusPath := filepath.Join(outputDir, "STATUS.md")
	if err := e.generateStatusMD(statusPath); err != nil {
		return nil, fmt.Errorf("failed to generate STATUS.md: %w", err)
	}
	result.FilesCreated = append(result.FilesCreated, statusPath)

	// Generate tracker files
	trackersDir := filepath.Join(outputDir, "trackers")
	if err := os.MkdirAll(trackersDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create trackers directory: %w", err)
	}

	trackerFiles, err := e.generateTrackerFiles(trackersDir)
	if err != nil {
		return nil, fmt.Errorf("failed to generate tracker files: %w", err)
	}
	result.FilesCreated = append(result.FilesCreated, trackerFiles...)
	result.FileCount = len(result.FilesCreated)

	return result, nil
}

// generateStatusMD creates STATUS.md from database
func (e *MarkdownExporter) generateStatusMD(path string) error {
	// Get total progress
	totalProg, err := e.db.GetTotalProgress()
	if err != nil {
		return fmt.Errorf("failed to get total progress: %w", err)
	}

	// Get all categories
	categories, err := e.db.GetAllCategories()
	if err != nil {
		return fmt.Errorf("failed to get categories: %w", err)
	}

	// Get last completed phase
	var lastPhase *db.PhaseDetails
	lastPhase, _ = e.db.GetCurrentPhase()

	// Get next phase
	var nextPhase *db.PhaseListItem
	if lastPhase != nil {
		nextPhase, _ = e.db.GetNextPhaseInCategory(lastPhase.Category)
	}
	if nextPhase == nil {
		// Get first pending overall
		opts := db.ListPhasesOptions{Status: "pending"}
		phases, _ := e.db.ListPhases(opts)
		if len(phases) > 0 {
			nextPhase = phases[0]
		}
	}

	// Prepare template data
	data := map[string]interface{}{
		"Generated":       time.Now().Format("2006-01-02 15:04:05"),
		"TotalCompleted":  totalProg.TotalCompleted,
		"TotalPhases":     totalProg.TotalPhases,
		"TotalPercentage": totalProg.Percentage,
		"Categories":      categories,
		"LastPhase":       lastPhase,
		"NextPhase":       nextPhase,
	}

	// Render template
	tmpl, err := template.New("status").Parse(statusTemplate)
	if err != nil {
		return fmt.Errorf("failed to parse template: %w", err)
	}

	file, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("failed to create file: %w", err)
	}
	defer func() { _ = file.Close() }()

	if err := tmpl.Execute(file, data); err != nil {
		return fmt.Errorf("failed to execute template: %w", err)
	}

	return nil
}

// generateTrackerFiles creates tracker files per category
func (e *MarkdownExporter) generateTrackerFiles(trackersDir string) ([]string, error) {
	// Get all categories
	categories, err := e.db.GetAllCategories()
	if err != nil {
		return nil, fmt.Errorf("failed to get categories: %w", err)
	}

	var files []string

	for _, cat := range categories {
		// Get phases for this category
		opts := db.ListPhasesOptions{Category: cat.Name}
		phases, err := e.db.ListPhases(opts)
		if err != nil {
			return nil, fmt.Errorf("failed to get phases for category %s: %w", cat.Name, err)
		}

		// Generate tracker file
		filename := fmt.Sprintf("%d-%s.md", cat.ID, cat.Name)
		path := filepath.Join(trackersDir, filename)

		if err := e.generateTrackerFile(path, cat, phases); err != nil {
			return nil, fmt.Errorf("failed to generate tracker for %s: %w", cat.Name, err)
		}

		files = append(files, path)
	}

	return files, nil
}

// generateTrackerFile creates a single tracker file
func (e *MarkdownExporter) generateTrackerFile(path string, cat *db.Category, phases []*db.PhaseListItem) error {
	data := map[string]interface{}{
		"Category":   cat,
		"Phases":     phases,
		"Generated":  time.Now().Format("2006-01-02 15:04:05"),
		"Completed":  cat.Completed,
		"Total":      cat.Total,
		"Percentage": cat.Percentage,
	}

	tmpl, err := template.New("tracker").Funcs(template.FuncMap{
		"checkbox": func(status string) string {
			if status == "completed" {
				return "✅"
			}
			return "⬜"
		},
	}).Parse(trackerTemplate)
	if err != nil {
		return fmt.Errorf("failed to parse tracker template: %w", err)
	}

	file, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("failed to create tracker file: %w", err)
	}
	defer func() { _ = file.Close() }()

	if err := tmpl.Execute(file, data); err != nil {
		return fmt.Errorf("failed to execute tracker template: %w", err)
	}

	return nil
}

// statusTemplate is the template for STATUS.md
const statusTemplate = `# Atlas Development Status

**Generated:** {{.Generated}}

## Overall Progress

**{{.TotalCompleted}}/{{.TotalPhases}} phases complete ({{.TotalPercentage}}%)**

{{if .LastPhase}}
### Last Completed
- **Phase:** {{.LastPhase.Name}}
- **Path:** {{.LastPhase.Path}}
- **Category:** {{.LastPhase.Category}}
{{- if .LastPhase.CompletedDate.Valid}}
- **Completed:** {{.LastPhase.CompletedDate.String}}
{{- end}}
{{- if .LastPhase.Description.Valid}}
- **Description:** {{.LastPhase.Description.String}}
{{- end}}
{{end}}

{{if .NextPhase}}
### Next Phase
- **Phase:** {{.NextPhase.Name}}
- **Path:** {{.NextPhase.Path}}
- **Category:** {{.NextPhase.Category}}
{{end}}

## Category Breakdown

| Category | Display Name | Completed | Total | Progress | Status |
|----------|--------------|-----------|-------|----------|--------|
{{- range .Categories}}
| {{.Name}} | {{.DisplayName}} | {{.Completed}} | {{.Total}} | {{.Percentage}}% | {{.Status}} |
{{- end}}

---

*This file is auto-generated from the database. Do not edit manually.*
*To update: run ` + "`atlas-dev export markdown`" + `*
`

// trackerTemplate is the template for tracker files
const trackerTemplate = `# {{.Category.DisplayName}} Tracker

**Category:** {{.Category.Name}}
**Progress:** {{.Completed}}/{{.Total}} ({{.Percentage}}%)
**Status:** {{.Category.Status}}
**Generated:** {{.Generated}}

## Phases

{{range .Phases -}}
{{checkbox .Status}} **{{.Name}}**
   - Path: ` + "`{{.Path}}`" + `
   - Status: {{.Status}}
{{- if .CompletedDate.Valid}}
   - Completed: {{.CompletedDate.String}}
{{- end}}

{{end}}

---

*This file is auto-generated from the database. Do not edit manually.*
*To update: run ` + "`atlas-dev export markdown`" + `*
`

// formatCategoryName converts category name to display name
func formatCategoryName(name string) string {
	// Convert "stdlib" -> "Standard Library"
	parts := strings.Split(name, "-")
	for i, part := range parts {
		parts[i] = strings.Title(part)
	}
	return strings.Join(parts, " ")
}
