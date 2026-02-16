package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/parity"
	"github.com/spf13/cobra"
)

func validateAllCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "all",
		Short: "Run all validation systems",
		Long: `Run comprehensive validation across all systems:

1. Database consistency (Phase 4)
2. Parity validation (spec, API, code, tests)
3. Cross-references
4. Test coverage

Returns overall health score (0-100) and results per validator.`,
		RunE: runValidateAll,
	}
}

func runValidateAll(cmd *cobra.Command, args []string) error {
	result := map[string]interface{}{
		"validators": []map[string]interface{}{},
	}

	totalScore := 0.0
	validatorCount := 0
	allPassed := true

	// 1. Database consistency validation
	dbResult, dbErr := validateDatabase()
	dbValidator := map[string]interface{}{
		"name": "database",
		"ok":   dbErr == nil && dbResult != nil && dbResult.OK,
	}
	if dbErr == nil && dbResult != nil {
		dbValidator["issues"] = len(dbResult.Issues)
		if dbResult.OK {
			dbValidator["score"] = 100.0
			totalScore += 100.0
		} else {
			// Partial score based on errors
			score := 100.0 - float64(len(dbResult.Issues))*10.0
			if score < 0 {
				score = 0
			}
			dbValidator["score"] = score
			totalScore += score
			allPassed = false
		}
		validatorCount++
	} else {
		dbValidator["error"] = fmt.Sprintf("%v", dbErr)
		allPassed = false
	}
	result["validators"] = append(result["validators"].([]map[string]interface{}), dbValidator)

	// 2. Parity validation
	parityResult, parityErr := validateParity()
	parityValidator := map[string]interface{}{
		"name": "parity",
		"ok":   parityErr == nil && parityResult != nil && parityResult.OK,
	}
	if parityErr == nil && parityResult != nil {
		parityValidator["score"] = parityResult.HealthScore
		parityValidator["errors"] = len(parityResult.Errors)
		parityValidator["warnings"] = len(parityResult.Warnings)
		totalScore += parityResult.HealthScore
		validatorCount++
		if !parityResult.OK {
			allPassed = false
		}
	} else {
		parityValidator["error"] = fmt.Sprintf("%v", parityErr)
		allPassed = false
	}
	result["validators"] = append(result["validators"].([]map[string]interface{}), parityValidator)

	// Calculate overall health score
	healthScore := 0.0
	if validatorCount > 0 {
		healthScore = totalScore / float64(validatorCount)
	}

	result["ok"] = allPassed
	result["health"] = healthScore
	result["validator_cnt"] = validatorCount

	// Add summary message
	if allPassed {
		result["msg"] = fmt.Sprintf("All validations passed (health: %.1f%%)", healthScore)
	} else {
		result["msg"] = fmt.Sprintf("Some validations failed (health: %.1f%%)", healthScore)
	}

	return output.Success(result)
}

// validateDatabase runs database consistency validation
func validateDatabase() (*db.ValidationReport, error) {
	report, err := database.Validate()
	if err != nil {
		return nil, err
	}
	return report, nil
}

// validateParity runs parity validation
func validateParity() (*parity.ParityReport, error) {
	projectRoot, err := findProjectRoot()
	if err != nil {
		return nil, err
	}

	checker := parity.NewParityChecker(projectRoot)
	return checker.CheckParity()
}
