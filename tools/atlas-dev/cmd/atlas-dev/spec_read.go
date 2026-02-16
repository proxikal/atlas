package main

import (
	"encoding/json"
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func specReadCmd() *cobra.Command {
	var (
		section  string
		withCode bool
		raw      bool
	)

	cmd := &cobra.Command{
		Use:   "read <spec-name>",
		Short: "Read specification from database",
		Long:  `Read specification content from database. No MD files required.`,
		Example: `  # Read full spec
  atlas-dev spec read syntax

  # Read specific section
  atlas-dev spec read syntax --section "Keywords"

  # Read from stdin (auto-detected)
  echo '{"name":"syntax"}' | atlas-dev spec read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var specName string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				if len(input.Items) == 0 {
					return fmt.Errorf("empty stdin input")
				}

				item := input.Items[0]
				// Try to extract name or path
				if name, ok := item["name"].(string); ok {
					specName = name
				} else if path, ok := item["path"].(string); ok {
					// Extract name from path
					specName = extractNameFromPath(path)
				} else {
					return fmt.Errorf("name or path required")
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("spec name required")
				}
				specName = args[0]
			}

			// Query database by name
			var (
				title    string
				version  string
				status   string
				outline  string
				sections string
				content  string
			)

			err := database.QueryRow(`
				SELECT title, version, status, outline, sections, content
				FROM specs
				WHERE name = ?
			`, specName).Scan(&title, &version, &status, &outline, &sections, &content)

			if err != nil {
				return fmt.Errorf("spec not found: %s", specName)
			}

			result := map[string]interface{}{
				"ok":    true,
				"title": title,
			}

			if version != "" {
				result["ver"] = version
			}
			if status != "" {
				result["stat"] = status
			}

			// Return raw content if requested
			if raw {
				result["content"] = content
				return output.Success(result)
			}

			// Filter to specific section if requested
			if section != "" {
				// Parse sections JSON
				var sectionsList []interface{}
				if err := json.Unmarshal([]byte(sections), &sectionsList); err != nil {
					return fmt.Errorf("failed to parse sections: %w", err)
				}

				// Find matching section (simplified - would need proper recursive search)
				found := false
				for _, s := range sectionsList {
					if sMap, ok := s.(map[string]interface{}); ok {
						if sMap["title"] == section {
							result["section"] = sMap["title"]
							result["content"] = sMap["content"]
							found = true
							break
						}
					}
				}

				if !found {
					return fmt.Errorf("section not found: %s", section)
				}
			} else {
				// Return outline and full sections
				var outlineList []string
				if err := json.Unmarshal([]byte(outline), &outlineList); err == nil {
					result["outline"] = outlineList
				}

				var sectionsList []interface{}
				if err := json.Unmarshal([]byte(sections), &sectionsList); err == nil {
					result["sections"] = sectionsList
				}
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&section, "section", "", "Read specific section")
	cmd.Flags().BoolVar(&withCode, "with-code", false, "Include code blocks")
	cmd.Flags().BoolVar(&raw, "raw", false, "Return raw markdown content")

	return cmd
}

func extractNameFromPath(path string) string {
	// Extract name from path like "docs/specification/syntax.md" -> "syntax"
	name := path
	if idx := len(path) - 1; idx >= 0 {
		for i := idx; i >= 0; i-- {
			if path[i] == '/' {
				name = path[i+1:]
				break
			}
		}
	}
	// Remove .md extension
	if len(name) > 3 && name[len(name)-3:] == ".md" {
		name = name[:len(name)-3]
	}
	return name
}
