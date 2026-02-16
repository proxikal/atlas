package db

import (
	"fmt"
)

// ValidationIssue represents a validation problem
type ValidationIssue struct {
	Check       string
	Severity    string // "error" | "warning"
	Message     string
	Suggestion  string
	ActualValue interface{}
	ExpectedValue interface{}
}

// ValidationReport represents the result of database validation
type ValidationReport struct {
	OK          bool
	ChecksRun   int
	ErrorsFound int
	Issues      []ValidationIssue
}

// Validate performs comprehensive database consistency checks
func (db *DB) Validate() (*ValidationReport, error) {
	report := &ValidationReport{
		OK:     true,
		Issues: []ValidationIssue{},
	}

	// Run all validation checks
	checks := []func(*ValidationReport) error{
		db.validateCategoryCounts,
		db.validateCategoryPercentages,
		db.validateTotalPhasesMetadata,
		db.validateCompletedPhasesMetadata,
		db.validateOrphanedPhases,
		db.validateInvalidStatuses,
		db.validateTriggers,
	}

	for _, check := range checks {
		report.ChecksRun++
		if err := check(report); err != nil {
			// Non-fatal error - continue with other checks
			report.addIssue("system", "error",
				fmt.Sprintf("Check failed: %v", err),
				"Review database integrity")
		}
	}

	report.ErrorsFound = len(report.Issues)
	report.OK = report.ErrorsFound == 0

	return report, nil
}

// validateCategoryCounts checks if category completed counts match actual
func (db *DB) validateCategoryCounts(report *ValidationReport) error {
	rows, err := db.conn.Query(`
		SELECT c.name, c.completed,
		       (SELECT COUNT(*) FROM phases WHERE category = c.name AND status = 'completed') as actual
		FROM categories c
	`)
	if err != nil {
		return err
	}
	defer func() { _ = rows.Close() }()

	for rows.Next() {
		var category string
		var reported, actual int

		if err := rows.Scan(&category, &reported, &actual); err != nil {
			return err
		}

		if reported != actual {
			report.addIssue(
				fmt.Sprintf("category_count_%s", category),
				"error",
				fmt.Sprintf("Category %s has incorrect completed count", category),
				fmt.Sprintf("UPDATE categories SET completed = %d WHERE name = '%s'",
					actual, category),
			)
		}
	}

	return rows.Err()
}

// validateCategoryPercentages checks if percentages are calculated correctly
func (db *DB) validateCategoryPercentages(report *ValidationReport) error {
	rows, err := db.conn.Query(`
		SELECT c.name, c.completed, c.total, c.percentage,
		       ROUND(CAST(c.completed AS REAL) / c.total * 100) as expected
		FROM categories c
		WHERE c.total > 0
	`)
	if err != nil {
		return err
	}
	defer func() { _ = rows.Close() }()

	for rows.Next() {
		var category string
		var completed, total, reported, expected int

		if err := rows.Scan(&category, &completed, &total, &reported, &expected); err != nil {
			return err
		}

		if reported != expected {
			report.addIssue(
				fmt.Sprintf("category_percentage_%s", category),
				"error",
				fmt.Sprintf("Category %s has incorrect percentage (%d%%, expected %d%%)",
					category, reported, expected),
				fmt.Sprintf("UPDATE categories SET percentage = %d WHERE name = '%s'",
					expected, category),
			)
		}
	}

	return rows.Err()
}

// validateTotalPhasesMetadata checks if total_phases metadata is correct
func (db *DB) validateTotalPhasesMetadata(report *ValidationReport) error {
	var reported string
	err := db.conn.QueryRow("SELECT value FROM metadata WHERE key = 'total_phases'").Scan(&reported)
	if err != nil {
		return err
	}

	var actual int
	err = db.conn.QueryRow("SELECT SUM(total) FROM categories").Scan(&actual)
	if err != nil {
		return err
	}

	if reported != fmt.Sprintf("%d", actual) {
		report.addIssue(
			"metadata_total_phases",
			"error",
			fmt.Sprintf("total_phases metadata incorrect (reported: %s, actual: %d)",
				reported, actual),
			fmt.Sprintf("UPDATE metadata SET value = '%d' WHERE key = 'total_phases'", actual),
		)
	}

	return nil
}

// validateCompletedPhasesMetadata checks if completed_phases metadata is correct
func (db *DB) validateCompletedPhasesMetadata(report *ValidationReport) error {
	var reported string
	err := db.conn.QueryRow("SELECT value FROM metadata WHERE key = 'completed_phases'").Scan(&reported)
	if err != nil {
		return err
	}

	var actual int
	err = db.conn.QueryRow("SELECT COUNT(*) FROM phases WHERE status = 'completed'").Scan(&actual)
	if err != nil {
		return err
	}

	if reported != fmt.Sprintf("%d", actual) {
		report.addIssue(
			"metadata_completed_phases",
			"error",
			fmt.Sprintf("completed_phases metadata incorrect (reported: %s, actual: %d)",
				reported, actual),
			fmt.Sprintf("UPDATE metadata SET value = '%d' WHERE key = 'completed_phases'", actual),
		)
	}

	return nil
}

// validateOrphanedPhases checks for phases with invalid categories
func (db *DB) validateOrphanedPhases(report *ValidationReport) error {
	rows, err := db.conn.Query(`
		SELECT p.id, p.path, p.category
		FROM phases p
		WHERE NOT EXISTS (
			SELECT 1 FROM categories c WHERE c.name = p.category
		)
	`)
	if err != nil {
		return err
	}
	defer func() { _ = rows.Close() }()

	for rows.Next() {
		var id int
		var path, category string

		if err := rows.Scan(&id, &path, &category); err != nil {
			return err
		}

		report.addIssue(
			fmt.Sprintf("orphaned_phase_%d", id),
			"error",
			fmt.Sprintf("Phase %s has invalid category: %s", path, category),
			"Either add category to categories table or fix phase category",
		)
	}

	return rows.Err()
}

// validateInvalidStatuses checks for phases with invalid status values
func (db *DB) validateInvalidStatuses(report *ValidationReport) error {
	rows, err := db.conn.Query(`
		SELECT id, path, status
		FROM phases
		WHERE status NOT IN ('pending', 'in_progress', 'completed', 'blocked')
	`)
	if err != nil {
		return err
	}
	defer func() { _ = rows.Close() }()

	for rows.Next() {
		var id int
		var path, status string

		if err := rows.Scan(&id, &path, &status); err != nil {
			return err
		}

		report.addIssue(
			fmt.Sprintf("invalid_status_%d", id),
			"error",
			fmt.Sprintf("Phase %s has invalid status: %s", path, status),
			fmt.Sprintf("UPDATE phases SET status = 'pending' WHERE id = %d", id),
		)
	}

	return rows.Err()
}

// validateTriggers checks that all required triggers exist
func (db *DB) validateTriggers(report *ValidationReport) error {
	requiredTriggers := []string{
		"update_category_progress",
		"update_phases_timestamp",
		"update_decisions_timestamp",
		"update_features_timestamp",
	}

	for _, trigger := range requiredTriggers {
		var count int
		err := db.conn.QueryRow(`
			SELECT COUNT(*)
			FROM sqlite_master
			WHERE type = 'trigger' AND name = ?
		`, trigger).Scan(&count)

		if err != nil {
			return err
		}

		if count == 0 {
			report.addIssue(
				fmt.Sprintf("missing_trigger_%s", trigger),
				"error",
				fmt.Sprintf("Required trigger missing: %s", trigger),
				"Re-run migration schema to recreate triggers",
			)
		}
	}

	return nil
}

// addIssue adds an issue to the validation report
func (report *ValidationReport) addIssue(check, severity, message, suggestion string) {
	report.Issues = append(report.Issues, ValidationIssue{
		Check:      check,
		Severity:   severity,
		Message:    message,
		Suggestion: suggestion,
	})
	report.OK = false
}

// ToCompactJSON converts ValidationReport to compact JSON map
func (report *ValidationReport) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"valid": report.OK,
		"chk":   report.ChecksRun,
		"err":   report.ErrorsFound,
	}

	if len(report.Issues) > 0 {
		issues := make([]map[string]interface{}, len(report.Issues))
		for i, issue := range report.Issues {
			issues[i] = map[string]interface{}{
				"chk":  issue.Check,
				"sev":  issue.Severity,
				"msg":  issue.Message,
				"fix":  issue.Suggestion,
			}
		}
		result["issues"] = issues
	}

	return result
}
