package context

import (
	"database/sql"
	"fmt"
	"log/slog"
	"time"

	"github.com/atlas-lang/atlas-dev/internal/db"
)

// PhaseContext represents complete context for a phase
type PhaseContext struct {
	// Phase metadata from DB
	ID       int    `json:"id,omitempty"`
	Path     string `json:"path"`
	Name     string `json:"name"`
	Category string `json:"cat"`
	Status   string `json:"stat"`

	// Phase file data
	Objective          string   `json:"obj,omitempty"`
	Priority           string   `json:"pri,omitempty"`
	Deliverables       []string `json:"dlv,omitempty"`
	AcceptanceCriteria []string `json:"acc,omitempty"`
	Files              []string `json:"files,omitempty"`
	Dependencies       []string `json:"deps,omitempty"`
	EstimatedTime      string   `json:"est,omitempty"`

	// Category progress
	CategoryProgress *ProgressData `json:"catProg,omitempty"`

	// Related data
	RelatedDecisions []*DecisionSummary `json:"decisions,omitempty"`
	TestCoverage     *TestCoverageData  `json:"tests,omitempty"`

	// Navigation
	Navigation *NavigationData `json:"nav,omitempty"`
}

// ProgressData represents progress information
type ProgressData struct {
	Completed  int `json:"cmp"`
	Total      int `json:"tot"`
	Percentage int `json:"pct"`
}

// DecisionSummary represents a related decision
type DecisionSummary struct {
	ID        string `json:"id"`
	Title     string `json:"title"`
	Component string `json:"comp,omitempty"`
	Date      string `json:"date"`
}

// TestCoverageData represents test coverage info
type TestCoverageData struct {
	Count  int `json:"cnt"`
	Target int `json:"tgt,omitempty"`
}

// NavigationData provides prev/next phase hints
type NavigationData struct {
	Previous *PhaseSummary `json:"prev,omitempty"`
	Next     *PhaseSummary `json:"next,omitempty"`
}

// PhaseSummary represents a phase reference
type PhaseSummary struct {
	Path string `json:"path"`
	Name string `json:"name"`
}

// Aggregator aggregates phase context from multiple sources
type Aggregator struct {
	db *db.DB
}

// NewAggregator creates a new context aggregator
func NewAggregator(database *db.DB) *Aggregator {
	return &Aggregator{db: database}
}

// GetPhaseContext returns complete context for a phase
func (a *Aggregator) GetPhaseContext(phasePath string) (*PhaseContext, error) {
	start := time.Now()
	slog.Debug("aggregating phase context", "path", phasePath)

	// 1. Get phase metadata from DB
	phase, err := a.db.GetPhaseByPath(phasePath)
	if err != nil {
		return nil, fmt.Errorf("failed to get phase from DB: %w", err)
	}

	// 2. Parse phase file
	phaseFile, err := ParsePhaseFile(phasePath)
	if err != nil {
		slog.Warn("failed to parse phase file", "path", phasePath, "error", err)
		// Continue without phase file data
		phaseFile = &PhaseFile{}
	}

	// 3. Build context
	ctx := &PhaseContext{
		ID:                 phase.ID,
		Path:               phase.Path,
		Name:               phase.Name,
		Category:           phase.Category,
		Status:             phase.Status,
		Objective:          phaseFile.Objective,
		Priority:           phaseFile.Priority,
		Deliverables:       phaseFile.Deliverables,
		AcceptanceCriteria: phaseFile.AcceptanceCriteria,
		Files:              phaseFile.Files,
		Dependencies:       phaseFile.Dependencies,
		EstimatedTime:      phaseFile.EstimatedTime,
	}

	// 4. Get category progress
	if catProg, err := a.getCategoryProgress(phase.Category); err == nil {
		ctx.CategoryProgress = catProg
	}

	// 5. Get related decisions
	if decisions, err := a.getRelatedDecisions(phase.Category, 5); err == nil {
		ctx.RelatedDecisions = decisions
	}

	// 6. Get test coverage
	if phase.TestCount > 0 {
		ctx.TestCoverage = &TestCoverageData{
			Count: phase.TestCount,
		}
	}

	// 7. Get navigation
	if nav, err := a.getNavigation(phase); err == nil {
		ctx.Navigation = nav
	}

	duration := time.Since(start)
	slog.Debug("context aggregated", "path", phasePath, "duration_ms", duration.Milliseconds())

	return ctx, nil
}

// getCategoryProgress gets category progress from DB
func (a *Aggregator) getCategoryProgress(category string) (*ProgressData, error) {
	cat, err := a.db.GetCategory(category)
	if err != nil {
		return nil, err
	}

	return &ProgressData{
		Completed:  cat.Completed,
		Total:      cat.Total,
		Percentage: cat.Percentage,
	}, nil
}

// getRelatedDecisions gets recent decisions for the category
func (a *Aggregator) getRelatedDecisions(category string, limit int) ([]*DecisionSummary, error) {
	opts := db.ListDecisionsOptions{
		Component: category,
		Limit:     limit,
	}

	decisions, err := a.db.ListDecisions(opts)
	if err != nil {
		return nil, err
	}

	var summaries []*DecisionSummary
	for _, d := range decisions {
		summaries = append(summaries, &DecisionSummary{
			ID:        d.ID,
			Title:     d.Title,
			Component: d.Component,
			Date:      d.Date,
		})
	}

	return summaries, nil
}

// getNavigation gets previous and next phases
func (a *Aggregator) getNavigation(phase *db.Phase) (*NavigationData, error) {
	nav := &NavigationData{}

	// Get previous phase (last completed in same category before this one)
	if prev := a.getPreviousPhase(phase); prev != nil {
		nav.Previous = prev
	}

	// Get next phase (first pending in same category after this one)
	if next := a.getNextPhase(phase); next != nil {
		nav.Next = next
	}

	return nav, nil
}

// getPreviousPhase finds the previous phase in the same category
func (a *Aggregator) getPreviousPhase(phase *db.Phase) *PhaseSummary {
	// Query for completed phases in same category before this one
	query := `
		SELECT path, name
		FROM phases
		WHERE category = ?
		  AND id < ?
		  AND status = 'completed'
		ORDER BY id DESC
		LIMIT 1
	`

	var path, name string
	err := a.db.QueryRow(query, phase.Category, phase.ID).Scan(&path, &name)
	if err != nil {
		if err != sql.ErrNoRows {
			slog.Warn("failed to get previous phase", "error", err)
		}
		return nil
	}

	return &PhaseSummary{Path: path, Name: name}
}

// getNextPhase finds the next phase in the same category
func (a *Aggregator) getNextPhase(phase *db.Phase) *PhaseSummary {
	// Query for pending phases in same category after this one
	query := `
		SELECT path, name
		FROM phases
		WHERE category = ?
		  AND id > ?
		  AND status = 'pending'
		ORDER BY id ASC
		LIMIT 1
	`

	var path, name string
	err := a.db.QueryRow(query, phase.Category, phase.ID).Scan(&path, &name)
	if err != nil {
		if err != sql.ErrNoRows {
			slog.Warn("failed to get next phase", "error", err)
		}
		return nil
	}

	return &PhaseSummary{Path: path, Name: name}
}

// GetCurrentPhaseContext returns context for the next phase to work on
func (a *Aggregator) GetCurrentPhaseContext() (*PhaseContext, error) {
	// Get last completed phase
	lastCompleted, err := a.db.GetCurrentPhase()
	if err != nil || lastCompleted == nil {
		// No completed phases yet, get first pending overall
		return a.getFirstPendingPhase()
	}

	// Get next pending phase in same category
	nextPhase, err := a.db.GetNextPhaseInCategory(lastCompleted.Category)
	if err != nil || nextPhase == nil {
		// No more in this category, get first pending overall
		return a.getFirstPendingPhase()
	}

	// Get full context for next phase
	return a.GetPhaseContext(nextPhase.Path)
}

// getFirstPendingPhase gets the first pending phase overall
func (a *Aggregator) getFirstPendingPhase() (*PhaseContext, error) {
	query := `
		SELECT path
		FROM phases
		WHERE status = 'pending'
		ORDER BY category, id
		LIMIT 1
	`

	var path string
	err := a.db.QueryRow(query).Scan(&path)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, fmt.Errorf("no pending phases found")
		}
		return nil, fmt.Errorf("failed to query pending phases: %w", err)
	}

	return a.GetPhaseContext(path)
}

// ToCompactJSON converts context to compact JSON map
func (ctx *PhaseContext) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"ok":   true,
		"path": ctx.Path,
		"name": ctx.Name,
		"cat":  ctx.Category,
		"stat": ctx.Status,
	}

	// Add optional fields only if present
	if ctx.ID > 0 {
		result["id"] = ctx.ID
	}
	if ctx.Objective != "" {
		result["obj"] = ctx.Objective
	}
	if ctx.Priority != "" {
		result["pri"] = ctx.Priority
	}
	if len(ctx.Deliverables) > 0 {
		result["dlv"] = ctx.Deliverables
	}
	if len(ctx.AcceptanceCriteria) > 0 {
		result["acc"] = ctx.AcceptanceCriteria
	}
	if len(ctx.Files) > 0 {
		result["files"] = ctx.Files
	}
	if len(ctx.Dependencies) > 0 {
		result["deps"] = ctx.Dependencies
	}
	if ctx.EstimatedTime != "" {
		result["est"] = ctx.EstimatedTime
	}
	if ctx.CategoryProgress != nil {
		result["catProg"] = []int{
			ctx.CategoryProgress.Completed,
			ctx.CategoryProgress.Total,
			ctx.CategoryProgress.Percentage,
		}
	}
	if len(ctx.RelatedDecisions) > 0 {
		result["decisions"] = ctx.RelatedDecisions
	}
	if ctx.TestCoverage != nil {
		result["tests"] = map[string]int{
			"cnt": ctx.TestCoverage.Count,
		}
		if ctx.TestCoverage.Target > 0 {
			result["tests"].(map[string]int)["tgt"] = ctx.TestCoverage.Target
		}
	}
	if ctx.Navigation != nil {
		nav := make(map[string]interface{})
		if ctx.Navigation.Previous != nil {
			nav["prev"] = ctx.Navigation.Previous
		}
		if ctx.Navigation.Next != nil {
			nav["next"] = ctx.Navigation.Next
		}
		if len(nav) > 0 {
			result["nav"] = nav
		}
	}

	return result
}
