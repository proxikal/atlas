package feature

import (
	"bufio"
	"fmt"
	"os"
	"regexp"
	"strings"
)

// Feature represents parsed feature documentation
type Feature struct {
	Name            string   `json:"name"`
	DisplayName     string   `json:"display_name"`
	Version         string   `json:"version"`
	Status          string   `json:"status"`
	Category        string   `json:"category,omitempty"`
	Overview        string   `json:"overview,omitempty"`
	SpecPath        string   `json:"spec_path,omitempty"`
	APIPath         string   `json:"api_path,omitempty"`
	ImplFile        string   `json:"impl_file,omitempty"`
	TestFile        string   `json:"test_file,omitempty"`
	FunctionCount   int      `json:"function_count,omitempty"`
	TestCount       int      `json:"test_count,omitempty"`
	Parity          float64  `json:"parity,omitempty"`
	Functions       []string `json:"functions,omitempty"`
	RelatedPhases   []string `json:"related_phases,omitempty"`
	RelatedFeatures []string `json:"related_features,omitempty"`
}

// Parse parses a feature markdown file
func Parse(path string) (*Feature, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open feature file: %w", err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	feature := &Feature{
		Functions:       []string{},
		RelatedPhases:   []string{},
		RelatedFeatures: []string{},
	}

	var (
		inMetadata    bool
		inOverview    bool
		overviewLines []string
	)

	for scanner.Scan() {
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Extract display name from title (# Feature Name)
		if strings.HasPrefix(trimmed, "# ") && feature.DisplayName == "" {
			feature.DisplayName = strings.TrimPrefix(trimmed, "# ")
			// Generate name from display name (lowercase, replace spaces with hyphens)
			feature.Name = generateFeatureName(feature.DisplayName)
			continue
		}

		// Detect metadata section start
		if strings.HasPrefix(trimmed, "**Version:**") {
			inMetadata = true
			feature.Version = extractValue(trimmed, "**Version:**")
			continue
		}

		if inMetadata {
			// Extract metadata fields
			if strings.HasPrefix(trimmed, "**Status:**") {
				feature.Status = extractValue(trimmed, "**Status:**")
			} else if strings.HasPrefix(trimmed, "**Category:**") {
				feature.Category = extractValue(trimmed, "**Category:**")
			} else if strings.HasPrefix(trimmed, "**Spec:**") {
				feature.SpecPath = extractValue(trimmed, "**Spec:**")
			} else if strings.HasPrefix(trimmed, "**API:**") {
				feature.APIPath = extractValue(trimmed, "**API:**")
			} else if strings.HasPrefix(trimmed, "**Related:**") {
				relatedText := extractValue(trimmed, "**Related:**")
				feature.RelatedFeatures = parseCommaSeparated(relatedText)
			} else if trimmed == "---" {
				inMetadata = false
			}
			continue
		}

		// Detect overview section
		if strings.HasPrefix(trimmed, "## Overview") {
			inOverview = true
			continue
		}

		// End overview on next section
		if inOverview && strings.HasPrefix(trimmed, "## ") {
			inOverview = false
			feature.Overview = strings.TrimSpace(strings.Join(overviewLines, " "))
			overviewLines = []string{}
		}

		// Collect overview text
		if inOverview && trimmed != "" && !strings.HasPrefix(trimmed, "#") {
			overviewLines = append(overviewLines, trimmed)
		}

		// Parse implementation section
		if strings.Contains(trimmed, "**Implementation:**") || strings.Contains(trimmed, "**Impl:**") {
			feature.ImplFile = extractValue(trimmed, "**Implementation:**")
			if feature.ImplFile == "" {
				feature.ImplFile = extractValue(trimmed, "**Impl:**")
			}
		}
		if strings.Contains(trimmed, "**Tests:**") || strings.Contains(trimmed, "**Test:**") {
			feature.TestFile = extractValue(trimmed, "**Tests:**")
			if feature.TestFile == "" {
				feature.TestFile = extractValue(trimmed, "**Test:**")
			}
		}
		if strings.Contains(trimmed, "**Test Count:**") {
			feature.TestCount = extractInt(trimmed, "**Test Count:**")
		}
		if strings.Contains(trimmed, "**Function Count:**") {
			feature.FunctionCount = extractInt(trimmed, "**Function Count:**")
		}
		if strings.Contains(trimmed, "**Parity:**") {
			feature.Parity = extractFloat(trimmed, "**Parity:**")
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("failed to scan feature file: %w", err)
	}

	// Final overview collection if file ended
	if len(overviewLines) > 0 {
		feature.Overview = strings.TrimSpace(strings.Join(overviewLines, " "))
	}

	// Validate required fields
	if feature.DisplayName == "" {
		return nil, fmt.Errorf("missing display name (title)")
	}
	if feature.Version == "" {
		return nil, fmt.Errorf("missing version")
	}
	if feature.Status == "" {
		return nil, fmt.Errorf("missing status")
	}

	return feature, nil
}

// generateFeatureName creates a name from display name
func generateFeatureName(displayName string) string {
	// Remove "in Atlas" suffix if present
	name := strings.TrimSuffix(displayName, " in Atlas")
	name = strings.TrimSpace(name)

	// Convert to lowercase and replace spaces with hyphens
	name = strings.ToLower(name)
	name = regexp.MustCompile(`\s+`).ReplaceAllString(name, "-")

	// Remove special characters except hyphens
	name = regexp.MustCompile(`[^a-z0-9-]`).ReplaceAllString(name, "")

	return name
}

// extractValue extracts value after a label
func extractValue(line, label string) string {
	parts := strings.SplitN(line, label, 2)
	if len(parts) < 2 {
		return ""
	}
	value := strings.TrimSpace(parts[1])
	// Remove markdown formatting
	value = strings.Trim(value, "`")
	return value
}

// extractInt extracts integer value
func extractInt(line, label string) int {
	valueStr := extractValue(line, label)
	var value int
	fmt.Sscanf(valueStr, "%d", &value)
	return value
}

// extractFloat extracts float value (for parity percentage)
func extractFloat(line, label string) float64 {
	valueStr := extractValue(line, label)
	// Remove % if present
	valueStr = strings.TrimSuffix(valueStr, "%")
	var value float64
	fmt.Sscanf(valueStr, "%f", &value)
	return value
}

// parseCommaSeparated parses comma-separated values
func parseCommaSeparated(text string) []string {
	if text == "" {
		return []string{}
	}
	parts := strings.Split(text, ",")
	result := []string{}
	for _, part := range parts {
		trimmed := strings.TrimSpace(part)
		if trimmed != "" {
			result = append(result, trimmed)
		}
	}
	return result
}
