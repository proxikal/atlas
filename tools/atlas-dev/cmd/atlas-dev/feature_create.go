package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureCreateCmd() *cobra.Command {
	var (
		name        string
		displayName string
		category    string
		version     string
		status      string
		description string
		specPath    string
		apiPath     string
	)

	cmd := &cobra.Command{
		Use:   "create",
		Short: "Create a new feature",
		Long:  `Create a new feature with markdown file in docs/features/ and database record.`,
		Example: `  # Create feature
  atlas-dev feature create \
    --name pattern-matching \
    --display "Pattern Matching" \
    --category core \
    --status Planned`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Validate required fields
			if name == "" {
				return fmt.Errorf("--name is required")
			}

			// Default display name to name if not provided
			if displayName == "" {
				displayName = toTitleCase(name)
			}

			// Default version and status
			if version == "" {
				version = "v0.1"
			}
			if status == "" {
				status = "Planned"
			}

			// Create feature in database
			req := db.CreateFeatureRequest{
				Name:        name,
				DisplayName: displayName,
				Version:     version,
				Status:      status,
				Description: description,
				SpecPath:    specPath,
				APIPath:     apiPath,
			}

			feature, err := database.CreateFeature(req)
			if err != nil {
				return err
			}

			// Create markdown file
			markdownPath := fmt.Sprintf("../../docs/features/%s.md", name)
			err = createFeatureMarkdown(markdownPath, feature, category)
			if err != nil {
				return fmt.Errorf("failed to create markdown file: %w", err)
			}

			result := feature.ToCompactJSON()
			result["msg"] = "Feature created"
			result["file"] = markdownPath
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Feature name (slug format) (required)")
	cmd.Flags().StringVar(&displayName, "display", "", "Display name (defaults to name)")
	cmd.Flags().StringVar(&category, "category", "", "Category")
	cmd.Flags().StringVar(&version, "version", "v0.1", "Version")
	cmd.Flags().StringVar(&status, "status", "Planned", "Status (Planned, InProgress, Implemented)")
	cmd.Flags().StringVar(&description, "description", "", "Description")
	cmd.Flags().StringVar(&specPath, "spec", "", "Spec file path")
	cmd.Flags().StringVar(&apiPath, "api", "", "API file path")

	return cmd
}

// createFeatureMarkdown creates a markdown file for the feature
func createFeatureMarkdown(path string, feature *db.Feature, category string) error {
	// Ensure directory exists
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %w", err)
	}

	// Generate markdown content
	content := fmt.Sprintf(`# %s

**Version:** %s
**Status:** %s
`, feature.DisplayName, feature.Version, feature.Status)

	if category != "" {
		content += fmt.Sprintf("**Category:** %s\n", category)
	}

	if feature.SpecPath.Valid && feature.SpecPath.String != "" {
		content += fmt.Sprintf("**Spec:** %s\n", feature.SpecPath.String)
	}

	if feature.APIPath.Valid && feature.APIPath.String != "" {
		content += fmt.Sprintf("**API:** %s\n", feature.APIPath.String)
	}

	content += "\n---\n\n## Overview\n\n"

	if feature.Description.Valid && feature.Description.String != "" {
		content += feature.Description.String + "\n\n"
	} else {
		content += "[Add feature overview here]\n\n"
	}

	content += `## Functions

[List key functions here]

## Implementation

**Implementation:** [Path to implementation file]
**Tests:** [Path to test file]
**Test Count:** 0
**Function Count:** 0
**Parity:** 0%

## Related

[Related features, phases, or decisions]
`

	// Write file
	err := os.WriteFile(path, []byte(content), 0644)
	if err != nil {
		return fmt.Errorf("failed to write markdown file: %w", err)
	}

	return nil
}

// toTitleCase converts "pattern-matching" to "Pattern Matching"
func toTitleCase(s string) string {
	words := strings.Split(s, "-")
	for i, word := range words {
		if len(word) > 0 {
			words[i] = strings.ToUpper(word[:1]) + word[1:]
		}
	}
	return strings.Join(words, " ")
}
