package feature

import (
	"fmt"
	"os"
	"regexp"
)

// ValidationReport contains validation results for a feature
type ValidationReport struct {
	FeatureName        string   `json:"feature"`
	Valid              bool     `json:"valid"`
	SpecRefValid       bool     `json:"spec_ref_valid"`
	APIRefValid        bool     `json:"api_ref_valid"`
	ImplFileExists     bool     `json:"impl_file_exists"`
	TestFileExists     bool     `json:"test_file_exists"`
	FunctionCountMatch bool     `json:"function_count_match,omitempty"`
	TestCountMatch     bool     `json:"test_count_match,omitempty"`
	ParityAccurate     bool     `json:"parity_accurate,omitempty"`
	ExpectedFunctions  int      `json:"expected_functions,omitempty"`
	ActualFunctions    int      `json:"actual_functions,omitempty"`
	ExpectedTests      int      `json:"expected_tests,omitempty"`
	ActualTests        int      `json:"actual_tests,omitempty"`
	Errors             []string `json:"errors,omitempty"`
	Warnings           []string `json:"warnings,omitempty"`
}

// Validate validates a feature against its implementation
func Validate(feature *Feature, projectRoot string) (*ValidationReport, error) {
	report := &ValidationReport{
		FeatureName: feature.Name,
		Valid:       true,
		Errors:      []string{},
		Warnings:    []string{},
	}

	// Validate spec reference
	if feature.SpecPath != "" {
		specPath := projectRoot + "/" + feature.SpecPath
		if _, err := os.Stat(specPath); err == nil {
			report.SpecRefValid = true
		} else {
			report.SpecRefValid = false
			report.Valid = false
			report.Errors = append(report.Errors, fmt.Sprintf("spec file not found: %s", feature.SpecPath))
		}
	} else {
		report.SpecRefValid = true // No spec to validate
	}

	// Validate API reference
	if feature.APIPath != "" {
		apiPath := projectRoot + "/" + feature.APIPath
		if _, err := os.Stat(apiPath); err == nil {
			report.APIRefValid = true
		} else {
			report.APIRefValid = false
			report.Valid = false
			report.Errors = append(report.Errors, fmt.Sprintf("API file not found: %s", feature.APIPath))
		}
	} else {
		report.APIRefValid = true // No API to validate
	}

	// Validate implementation file
	if feature.ImplFile != "" {
		implPath := projectRoot + "/" + feature.ImplFile
		if _, err := os.Stat(implPath); err == nil {
			report.ImplFileExists = true

			// Count functions if impl file exists
			actualCount, err := countRustFunctions(implPath)
			if err == nil {
				report.ActualFunctions = actualCount
				report.ExpectedFunctions = feature.FunctionCount

				if feature.FunctionCount > 0 {
					if actualCount == feature.FunctionCount {
						report.FunctionCountMatch = true
					} else {
						report.FunctionCountMatch = false
						report.Valid = false
						report.Errors = append(report.Errors,
							fmt.Sprintf("function count mismatch: expected %d, found %d", feature.FunctionCount, actualCount))
					}
				}
			} else {
				report.Warnings = append(report.Warnings, fmt.Sprintf("failed to count functions: %v", err))
			}
		} else {
			report.ImplFileExists = false
			if feature.Status == "Implemented" {
				report.Valid = false
				report.Errors = append(report.Errors, fmt.Sprintf("implementation file not found: %s", feature.ImplFile))
			} else {
				report.Warnings = append(report.Warnings, fmt.Sprintf("implementation file not found: %s (expected for status: %s)", feature.ImplFile, feature.Status))
			}
		}
	} else {
		report.ImplFileExists = true // No impl file specified
	}

	// Validate test file
	if feature.TestFile != "" {
		testPath := projectRoot + "/" + feature.TestFile
		if _, err := os.Stat(testPath); err == nil {
			report.TestFileExists = true

			// Count tests if test file exists
			actualCount, err := countRustTests(testPath)
			if err == nil {
				report.ActualTests = actualCount
				report.ExpectedTests = feature.TestCount

				if feature.TestCount > 0 {
					if actualCount == feature.TestCount {
						report.TestCountMatch = true
					} else {
						report.TestCountMatch = false
						report.Warnings = append(report.Warnings,
							fmt.Sprintf("test count mismatch: expected %d, found %d", feature.TestCount, actualCount))
					}
				}
			} else {
				report.Warnings = append(report.Warnings, fmt.Sprintf("failed to count tests: %v", err))
			}
		} else {
			report.TestFileExists = false
			if feature.Status == "Implemented" {
				report.Warnings = append(report.Warnings, fmt.Sprintf("test file not found: %s", feature.TestFile))
			}
		}
	} else {
		report.TestFileExists = true // No test file specified
	}

	// Validate parity calculation
	if feature.Parity > 0 && report.ImplFileExists && report.TestFileExists {
		// Simplified parity check - could be enhanced
		if feature.Parity > 100 {
			report.ParityAccurate = false
			report.Warnings = append(report.Warnings, fmt.Sprintf("parity > 100%%: %.1f%%", feature.Parity))
		} else {
			report.ParityAccurate = true
		}
	}

	return report, nil
}

// countRustFunctions counts public functions in a Rust file
func countRustFunctions(path string) (int, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}

	text := string(content)

	// Match: pub fn function_name
	// This is a simple regex-based approach
	// For production, consider using tree-sitter or rust-analyzer
	pattern := regexp.MustCompile(`(?m)^\s*pub\s+fn\s+\w+`)
	matches := pattern.FindAllString(text, -1)

	return len(matches), nil
}

// countRustTests counts test functions in a Rust file
func countRustTests(path string) (int, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}

	text := string(content)

	// Match: #[test] or #[tokio::test]
	pattern := regexp.MustCompile(`(?m)^\s*#\[(test|tokio::test)\]`)
	matches := pattern.FindAllString(text, -1)

	return len(matches), nil
}

// ValidateBatch validates multiple features
func ValidateBatch(features []*Feature, projectRoot string) ([]*ValidationReport, error) {
	reports := make([]*ValidationReport, 0, len(features))

	for _, feature := range features {
		report, err := Validate(feature, projectRoot)
		if err != nil {
			return nil, fmt.Errorf("failed to validate feature %s: %w", feature.Name, err)
		}
		reports = append(reports, report)
	}

	return reports, nil
}

// CountErrors counts total errors across reports
func CountErrors(reports []*ValidationReport) int {
	total := 0
	for _, report := range reports {
		total += len(report.Errors)
	}
	return total
}

// CountWarnings counts total warnings across reports
func CountWarnings(reports []*ValidationReport) int {
	total := 0
	for _, report := range reports {
		total += len(report.Warnings)
	}
	return total
}

// FilterInvalid returns only invalid reports
func FilterInvalid(reports []*ValidationReport) []*ValidationReport {
	invalid := []*ValidationReport{}
	for _, report := range reports {
		if !report.Valid {
			invalid = append(invalid, report)
		}
	}
	return invalid
}
