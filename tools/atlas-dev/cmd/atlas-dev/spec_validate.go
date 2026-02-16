package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/spec"
	"github.com/spf13/cobra"
)

func specValidateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "validate <spec-file>",
		Short: "Validate a specification document",
		Long:  `Validate spec consistency: check cross-references, internal links, and code block syntax.`,
		Example: `  # Validate spec
  atlas-dev spec validate ../../docs/specification/syntax.md

  # Validate from stdin (auto-detected)
  echo '{"path":"docs/specification/syntax.md"}' | atlas-dev spec validate`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var specPath string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				specPath, err = compose.ExtractFirstPath(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("spec file path required")
				}
				specPath = args[0]
			}

			// Parse spec
			parsed, err := spec.Parse(specPath)
			if err != nil {
				return err
			}

			errors := []string{}
			warnings := []string{}

			// Validate references
			specDir := filepath.Dir(specPath)
			for _, ref := range parsed.References {
				// Skip external URLs
				if strings.HasPrefix(ref, "http://") || strings.HasPrefix(ref, "https://") {
					continue
				}

				// Check if file exists
				parts := strings.Split(ref, "#")
				refPath := parts[0]

				if refPath != "" {
					fullPath := filepath.Join(specDir, refPath)
					if _, err := os.Stat(fullPath); err != nil {
						errors = append(errors, fmt.Sprintf("broken reference: %s", ref))
					}
				}

				// Check section reference if present
				if len(parts) > 1 {
					sectionName := parts[1]
					if refPath == "" {
						// Internal reference
						if parsed.FindSection(sectionName) == nil {
							errors = append(errors, fmt.Sprintf("broken internal link: #%s", sectionName))
						}
					}
				}
			}

			// Validate code blocks (basic check)
			for _, block := range parsed.CodeBlocks {
				if block.Language == "" {
					warnings = append(warnings, "code block without language tag")
				}
			}

			result := map[string]interface{}{
				"spec":  filepath.Base(specPath),
				"valid": len(errors) == 0,
			}

			if len(errors) > 0 {
				result["errors"] = errors
			}
			if len(warnings) > 0 {
				result["warnings"] = warnings
			}

			result["refs_checked"] = len(parsed.References)
			result["code_blocks"] = len(parsed.CodeBlocks)

			return output.Success(result)
		},
	}

	return cmd
}
