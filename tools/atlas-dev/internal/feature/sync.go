package feature

import (
	"fmt"
	"os"
	"path/filepath"
	"time"
)

// SyncResult represents the result of syncing a feature
type SyncResult struct {
	FeatureName   string                 `json:"feature"`
	Updated       bool                   `json:"updated"`
	Changes       map[string]interface{} `json:"changes,omitempty"`
	FunctionCount int                    `json:"function_count,omitempty"`
	TestCount     int                    `json:"test_count,omitempty"`
	Parity        float64                `json:"parity,omitempty"`
	LastModified  string                 `json:"last_modified,omitempty"`
	Errors        []string               `json:"errors,omitempty"`
}

// Sync syncs a feature from its codebase implementation
func Sync(feature *Feature, projectRoot string, dryRun bool) (*SyncResult, error) {
	result := &SyncResult{
		FeatureName: feature.Name,
		Updated:     false,
		Changes:     make(map[string]interface{}),
		Errors:      []string{},
	}

	// Parse implementation file
	if feature.ImplFile != "" {
		implPath := filepath.Join(projectRoot, feature.ImplFile)
		if stat, err := os.Stat(implPath); err == nil {
			// Get last modified time
			result.LastModified = stat.ModTime().Format(time.RFC3339)

			// Count functions
			functionCount, err := countRustFunctions(implPath)
			if err != nil {
				result.Errors = append(result.Errors, fmt.Sprintf("failed to count functions: %v", err))
			} else {
				result.FunctionCount = functionCount
				if functionCount != feature.FunctionCount {
					result.Updated = true
					result.Changes["function_count"] = map[string]int{
						"old": feature.FunctionCount,
						"new": functionCount,
					}
					feature.FunctionCount = functionCount
				}
			}
		} else {
			result.Errors = append(result.Errors, fmt.Sprintf("implementation file not found: %s", feature.ImplFile))
		}
	}

	// Parse test file
	if feature.TestFile != "" {
		testPath := filepath.Join(projectRoot, feature.TestFile)
		if _, err := os.Stat(testPath); err == nil {
			// Count tests
			testCount, err := countRustTests(testPath)
			if err != nil {
				result.Errors = append(result.Errors, fmt.Sprintf("failed to count tests: %v", err))
			} else {
				result.TestCount = testCount
				if testCount != feature.TestCount {
					result.Updated = true
					result.Changes["test_count"] = map[string]int{
						"old": feature.TestCount,
						"new": testCount,
					}
					feature.TestCount = testCount
				}
			}
		} else {
			result.Errors = append(result.Errors, fmt.Sprintf("test file not found: %s", feature.TestFile))
		}
	}

	// Calculate parity if both counts available
	if feature.FunctionCount > 0 && feature.TestCount > 0 {
		parity := (float64(feature.TestCount) / float64(feature.FunctionCount)) * 100.0
		// Round to 1 decimal place
		parity = float64(int(parity*10)) / 10.0

		if parity != feature.Parity {
			result.Updated = true
			result.Changes["parity"] = map[string]float64{
				"old": feature.Parity,
				"new": parity,
			}
			feature.Parity = parity
		}
		result.Parity = parity
	}

	// If dry-run, don't write changes
	if dryRun {
		return result, nil
	}

	// TODO: Update markdown file with new values
	// This would require more sophisticated markdown manipulation
	// For now, we'll just update the database via the DB layer

	return result, nil
}

// SyncAll syncs all features
func SyncAll(features []*Feature, projectRoot string, dryRun bool) ([]*SyncResult, error) {
	results := make([]*SyncResult, 0, len(features))

	for _, feature := range features {
		result, err := Sync(feature, projectRoot, dryRun)
		if err != nil {
			return nil, fmt.Errorf("failed to sync feature %s: %w", feature.Name, err)
		}
		results = append(results, result)
	}

	return results, nil
}

// CountUpdated counts how many features were updated
func CountUpdated(results []*SyncResult) int {
	count := 0
	for _, result := range results {
		if result.Updated {
			count++
		}
	}
	return count
}

// HasErrors checks if any results have errors
func HasErrors(results []*SyncResult) bool {
	for _, result := range results {
		if len(result.Errors) > 0 {
			return true
		}
	}
	return false
}
