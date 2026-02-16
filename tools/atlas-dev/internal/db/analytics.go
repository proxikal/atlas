package db

import (
	"database/sql"
	"fmt"
	"log/slog"
	"time"
)

// Summary represents comprehensive project summary
type Summary struct {
	Categories    []*CategorySummary
	TotalProgress ProgressSummary
	CurrentPhase  *PhaseListItem
	NextPhase     *PhaseListItem
	BlockedCount  int
}

// CategorySummary represents category progress
type CategorySummary struct {
	Name        string
	DisplayName string
	Completed   int
	Total       int
	Percentage  int
	Status      string
}

// ProgressSummary represents overall progress
type ProgressSummary struct {
	Completed  int
	Total      int
	Percentage int
}

// Stats represents velocity and completion estimates
type Stats struct {
	TotalPhases     int
	CompletedPhases int
	RemainingPhases int
	VelocityPerDay  float64
	VelocityPerWeek float64
	EstimatedDays   int
	CompletionDate  string
	FirstCompletion string
	LastCompletion  string
	DaysElapsed     int
}

// BlockedPhase represents a blocked phase with dependencies
type BlockedPhase struct {
	ID        int
	Path      string
	Name      string
	Category  string
	Blockers  []string
	BlockedBy sql.NullString
}

// TimelineEntry represents phases completed on a specific date
type TimelineEntry struct {
	Date  string
	Count int
}

// CoverageSummary represents test coverage statistics
type CoverageSummary struct {
	TotalTests     int
	PassingTests   int
	FailingTests   int
	CoveragePercent float64
	PhasesWithTests int
}

// GetSummary returns comprehensive project summary
func (db *DB) GetSummary() (*Summary, error) {
	start := time.Now()

	summary := &Summary{}

	// Get all categories with progress
	categories, err := db.GetAllCategories()
	if err != nil {
		return nil, fmt.Errorf("failed to get categories: %w", err)
	}

	summary.Categories = make([]*CategorySummary, len(categories))
	for i, cat := range categories {
		summary.Categories[i] = &CategorySummary{
			Name:        cat.Name,
			DisplayName: cat.DisplayName,
			Completed:   cat.Completed,
			Total:       cat.Total,
			Percentage:  cat.Percentage,
			Status:      cat.Status,
		}
	}

	// Get total progress
	total, err := db.GetTotalProgress()
	if err != nil {
		return nil, fmt.Errorf("failed to get total progress: %w", err)
	}

	summary.TotalProgress = ProgressSummary{
		Completed:  total.TotalCompleted,
		Total:      total.TotalPhases,
		Percentage: total.Percentage,
	}

	// Get current phase
	current, err := db.GetCurrentPhase()
	if err != nil {
		return nil, fmt.Errorf("failed to get current phase: %w", err)
	}
	if current != nil {
		summary.CurrentPhase = &PhaseListItem{
			Path:          current.Path,
			Name:          current.Name,
			Category:      current.Category,
			Status:        current.Status,
			CompletedDate: current.CompletedDate,
		}
	}

	// Get next phase
	next, err := db.GetNextPhase("")
	if err != nil {
		return nil, fmt.Errorf("failed to get next phase: %w", err)
	}
	if next != nil {
		summary.NextPhase = &PhaseListItem{
			Path:          next.Path,
			Name:          next.Name,
			Category:      next.Category,
			Status:        next.Status,
			CompletedDate: next.CompletedDate,
		}
	}

	// Get blocked count
	var blockedCount int
	err = db.conn.QueryRow("SELECT COUNT(*) FROM phases WHERE status = 'blocked'").Scan(&blockedCount)
	if err != nil {
		return nil, fmt.Errorf("failed to count blocked phases: %w", err)
	}
	summary.BlockedCount = blockedCount

	duration := time.Since(start)
	slog.Debug("summary generated",
		"categories", len(summary.Categories),
		"blocked", blockedCount,
		"duration_ms", duration.Milliseconds(),
	)

	return summary, nil
}

// GetStats calculates velocity and completion estimates
func (db *DB) GetStats() (*Stats, error) {
	start := time.Now()

	stats := &Stats{}

	// Get phase counts
	total, err := db.GetTotalProgress()
	if err != nil {
		return nil, fmt.Errorf("failed to get total progress: %w", err)
	}

	stats.TotalPhases = total.TotalPhases
	stats.CompletedPhases = total.TotalCompleted
	stats.RemainingPhases = total.TotalPhases - total.TotalCompleted

	// Get completion date range
	query := `
		SELECT
			MIN(completed_date) as first_completion,
			MAX(completed_date) as last_completion,
			COUNT(*) as completed_count
		FROM phases
		WHERE status = 'completed' AND completed_date IS NOT NULL
	`

	var firstCompletion, lastCompletion sql.NullString
	var completedCount int

	err = db.conn.QueryRow(query).Scan(&firstCompletion, &lastCompletion, &completedCount)
	if err != nil && err != sql.ErrNoRows {
		return nil, fmt.Errorf("failed to query completion dates: %w", err)
	}

	// Handle no completions yet
	if !firstCompletion.Valid || completedCount == 0 {
		stats.VelocityPerDay = 0
		stats.VelocityPerWeek = 0
		stats.EstimatedDays = 0
		stats.CompletionDate = "unknown"
		duration := time.Since(start)
		slog.Debug("stats generated (no completions)", "duration_ms", duration.Milliseconds())
		return stats, nil
	}

	stats.FirstCompletion = firstCompletion.String
	stats.LastCompletion = lastCompletion.String

	// Calculate days elapsed
	firstDate, err := time.Parse(time.RFC3339, firstCompletion.String)
	if err != nil {
		return nil, fmt.Errorf("failed to parse first completion date: %w", err)
	}

	lastDate, err := time.Parse(time.RFC3339, lastCompletion.String)
	if err != nil {
		return nil, fmt.Errorf("failed to parse last completion date: %w", err)
	}

	daysElapsed := int(lastDate.Sub(firstDate).Hours() / 24)
	if daysElapsed == 0 {
		daysElapsed = 1 // Avoid division by zero for same-day completions
	}
	stats.DaysElapsed = daysElapsed

	// Calculate velocity
	stats.VelocityPerDay = float64(completedCount) / float64(daysElapsed)
	stats.VelocityPerWeek = stats.VelocityPerDay * 7

	// Estimate completion
	if stats.VelocityPerDay > 0 {
		stats.EstimatedDays = int(float64(stats.RemainingPhases) / stats.VelocityPerDay)
		completionDate := lastDate.AddDate(0, 0, stats.EstimatedDays)
		stats.CompletionDate = completionDate.Format("2006-01-02")
	} else {
		stats.EstimatedDays = 0
		stats.CompletionDate = "unknown"
	}

	duration := time.Since(start)
	slog.Debug("stats generated",
		"velocity_per_day", stats.VelocityPerDay,
		"estimated_days", stats.EstimatedDays,
		"duration_ms", duration.Milliseconds(),
	)

	return stats, nil
}

// GetBlockedPhases returns all blocked phases with their dependencies
func (db *DB) GetBlockedPhases() ([]*BlockedPhase, error) {
	start := time.Now()

	query := `
		SELECT id, path, name, category, blockers
		FROM phases
		WHERE status = 'blocked'
		ORDER BY category, id
	`

	rows, err := db.conn.Query(query)
	if err != nil {
		return nil, fmt.Errorf("failed to query blocked phases: %w", err)
	}
	defer func() { _ = rows.Close() }()

	var blocked []*BlockedPhase
	for rows.Next() {
		var b BlockedPhase
		if err := rows.Scan(&b.ID, &b.Path, &b.Name, &b.Category, &b.BlockedBy); err != nil {
			return nil, fmt.Errorf("failed to scan blocked phase: %w", err)
		}

		// Parse blockers from JSON array string
		if b.BlockedBy.Valid && b.BlockedBy.String != "" {
			// Simple parsing: ["phase-01", "phase-02"] -> ["phase-01", "phase-02"]
			// For now, store as single string, can enhance to parse JSON later
			b.Blockers = []string{b.BlockedBy.String}
		}

		blocked = append(blocked, &b)
	}

	duration := time.Since(start)
	slog.Debug("blocked phases queried",
		"count", len(blocked),
		"duration_ms", duration.Milliseconds(),
	)

	return blocked, rows.Err()
}

// GetTimeline returns completion timeline grouped by date
func (db *DB) GetTimeline(days int) ([]*TimelineEntry, error) {
	start := time.Now()

	query := `
		SELECT DATE(completed_date) as date, COUNT(*) as count
		FROM phases
		WHERE status = 'completed' AND completed_date IS NOT NULL
	`

	args := []interface{}{}

	// Filter to recent N days if specified
	if days > 0 {
		query += " AND DATE(completed_date) >= DATE('now', '-' || ? || ' days')"
		args = append(args, days)
	}

	query += " GROUP BY DATE(completed_date) ORDER BY date ASC"

	rows, err := db.conn.Query(query, args...)
	if err != nil {
		return nil, fmt.Errorf("failed to query timeline: %w", err)
	}
	defer func() { _ = rows.Close() }()

	var timeline []*TimelineEntry
	for rows.Next() {
		var entry TimelineEntry
		if err := rows.Scan(&entry.Date, &entry.Count); err != nil {
			return nil, fmt.Errorf("failed to scan timeline entry: %w", err)
		}
		timeline = append(timeline, &entry)
	}

	duration := time.Since(start)
	slog.Debug("timeline generated",
		"entries", len(timeline),
		"days_filter", days,
		"duration_ms", duration.Milliseconds(),
	)

	return timeline, rows.Err()
}

// GetTestCoverage returns aggregated test coverage statistics
func (db *DB) GetTestCoverage(category string) (*CoverageSummary, error) {
	start := time.Now()

	query := `
		SELECT
			COALESCE(SUM(test_count), 0) as total_tests,
			COUNT(CASE WHEN test_count > 0 THEN 1 END) as phases_with_tests
		FROM phases
		WHERE status = 'completed'
	`

	args := []interface{}{}
	if category != "" {
		query += " AND category = ?"
		args = append(args, category)
	}

	coverage := &CoverageSummary{}

	var totalTests int
	var phasesWithTests int

	err := db.conn.QueryRow(query, args...).Scan(&totalTests, &phasesWithTests)
	if err != nil {
		return nil, fmt.Errorf("failed to query test coverage: %w", err)
	}

	coverage.TotalTests = totalTests
	coverage.PhasesWithTests = phasesWithTests

	// For now, set passing = total (no failing test tracking yet)
	coverage.PassingTests = totalTests
	coverage.FailingTests = 0

	// Calculate coverage percentage (simplified)
	var totalPhasesWithTests int
	countQuery := "SELECT COUNT(*) FROM phases WHERE status = 'completed'"
	if category != "" {
		countQuery += " AND category = ?"
		err = db.conn.QueryRow(countQuery, category).Scan(&totalPhasesWithTests)
	} else {
		err = db.conn.QueryRow(countQuery).Scan(&totalPhasesWithTests)
	}

	if err != nil {
		return nil, fmt.Errorf("failed to count total phases: %w", err)
	}

	if totalPhasesWithTests > 0 {
		coverage.CoveragePercent = float64(phasesWithTests) / float64(totalPhasesWithTests) * 100
	}

	duration := time.Since(start)
	slog.Debug("test coverage calculated",
		"total_tests", coverage.TotalTests,
		"phases_with_tests", coverage.PhasesWithTests,
		"duration_ms", duration.Milliseconds(),
	)

	return coverage, nil
}

// ToCompactJSON converts Summary to compact JSON map
func (s *Summary) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"cats": make([]map[string]interface{}, len(s.Categories)),
		"tot":  []int{s.TotalProgress.Completed, s.TotalProgress.Total, s.TotalProgress.Percentage},
		"blk":  s.BlockedCount,
	}

	for i, cat := range s.Categories {
		result["cats"].([]map[string]interface{})[i] = map[string]interface{}{
			"name": cat.Name,
			"disp": cat.DisplayName,
			"prog": []int{cat.Completed, cat.Total, cat.Percentage},
			"stat": cat.Status,
		}
	}

	if s.CurrentPhase != nil {
		result["cur"] = map[string]interface{}{
			"path": s.CurrentPhase.Path,
			"name": s.CurrentPhase.Name,
			"cat":  s.CurrentPhase.Category,
		}
	}

	if s.NextPhase != nil {
		result["next"] = map[string]interface{}{
			"path": s.NextPhase.Path,
			"name": s.NextPhase.Name,
			"cat":  s.NextPhase.Category,
		}
	}

	return result
}

// ToCompactJSON converts Stats to compact JSON map
func (s *Stats) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"tot":  s.TotalPhases,
		"cmp":  s.CompletedPhases,
		"rem":  s.RemainingPhases,
		"vpd":  fmt.Sprintf("%.2f", s.VelocityPerDay),
		"vpw":  fmt.Sprintf("%.2f", s.VelocityPerWeek),
		"est":  s.EstimatedDays,
		"comp": s.CompletionDate,
	}

	if s.FirstCompletion != "" {
		result["first"] = s.FirstCompletion[:10] // Date only
	}

	if s.LastCompletion != "" {
		result["last"] = s.LastCompletion[:10] // Date only
	}

	if s.DaysElapsed > 0 {
		result["days"] = s.DaysElapsed
	}

	return result
}

// ToCompactJSON converts BlockedPhase to compact JSON map
func (b *BlockedPhase) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"id":   b.ID,
		"path": b.Path,
		"name": b.Name,
		"cat":  b.Category,
	}

	if len(b.Blockers) > 0 {
		result["blk"] = b.Blockers
	}

	return result
}

// ToCompactJSON converts TimelineEntry to compact JSON map
func (t *TimelineEntry) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"date": t.Date,
		"cnt":  t.Count,
	}
}

// ToCompactJSON converts CoverageSummary to compact JSON map
func (c *CoverageSummary) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"tot":  c.TotalTests,
		"pass": c.PassingTests,
		"fail": c.FailingTests,
		"cov":  fmt.Sprintf("%.1f", c.CoveragePercent),
		"cnt":  c.PhasesWithTests,
	}
}
