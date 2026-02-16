package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/api"
	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiReadCmd() *cobra.Command {
	var (
		function string
		detailed bool
	)

	cmd := &cobra.Command{
		Use:   "read <api-file>",
		Short: "Read API documentation",
		Long:  `Read and parse API documentation markdown file.`,
		Example: `  # Read API doc
  atlas-dev api read ../../docs/api/stdlib.md

  # Read specific function
  atlas-dev api read ../../docs/api/stdlib.md --function print

  # Read from stdin (auto-detected)
  echo '{"path":"docs/api/stdlib.md"}' | atlas-dev api read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var apiPath string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				apiPath, err = compose.ExtractFirstPath(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("API file path required")
				}
				apiPath = args[0]
			}

			// Parse API doc
			doc, err := api.Parse(apiPath)
			if err != nil {
				return err
			}

			result := map[string]interface{}{
				"title": doc.Title,
			}

			if function != "" {
				fn := doc.FindFunction(function)
				if fn == nil {
					return fmt.Errorf("function not found: %s", function)
				}

				result["func"] = fn.Name
				result["sig"] = fn.Signature
				if fn.Description != "" {
					result["desc"] = fn.Description
				}
			} else {
				result["functions"] = len(doc.Functions)

				if detailed {
					fns := []map[string]string{}
					for _, fn := range doc.Functions {
						fns = append(fns, map[string]string{
							"name": fn.Name,
							"sig":  fn.Signature,
						})
					}
					result["list"] = fns
				}
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&function, "function", "", "Read specific function")
	cmd.Flags().BoolVar(&detailed, "detailed", false, "Include full details")

	return cmd
}
