package main

import (
	"fmt"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/spec"
	"github.com/spf13/cobra"
)

func specReadCmd() *cobra.Command {
	var (
		section  string
		withCode bool
	)

	cmd := &cobra.Command{
		Use:   "read <spec-file>",
		Short: "Read a specification document",
		Long:  `Read and parse a specification markdown file. Optionally filter to a specific section.`,
		Example: `  # Read full spec
  atlas-dev spec read ../../docs/specification/syntax.md

  # Read specific section
  atlas-dev spec read ../../docs/specification/syntax.md --section "Keywords"

  # Read from stdin (auto-detected)
  echo '{"path":"docs/specification/syntax.md"}' | atlas-dev spec read`,
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

			// Make path absolute if relative
			if !filepath.IsAbs(specPath) {
				var err error
				specPath, err = filepath.Abs(specPath)
				if err != nil {
					return fmt.Errorf("failed to resolve path: %w", err)
				}
			}

			// Parse spec
			parsed, err := spec.Parse(specPath)
			if err != nil {
				return err
			}

			result := map[string]interface{}{
				"title": parsed.Title,
			}

			if parsed.Version != "" {
				result["ver"] = parsed.Version
			}
			if parsed.Status != "" {
				result["stat"] = parsed.Status
			}

			// Filter to specific section if requested
			if section != "" {
				found := parsed.FindSection(section)
				if found == nil {
					return fmt.Errorf("section not found: %s", section)
				}

				result["section"] = found.Title
				result["content"] = found.Content

				if withCode && len(found.CodeBlocks) > 0 {
					blocks := []map[string]string{}
					for _, block := range found.CodeBlocks {
						blocks = append(blocks, map[string]string{
							"lang": block.Language,
							"code": block.Code,
						})
					}
					result["code"] = blocks
				}
			} else {
				// Return outline
				outline := parsed.GetOutline()
				result["outline"] = outline
				result["sections"] = len(parsed.Sections)
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&section, "section", "", "Read specific section")
	cmd.Flags().BoolVar(&withCode, "with-code", false, "Include code blocks")

	return cmd
}
