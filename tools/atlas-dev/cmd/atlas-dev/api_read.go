package main

import (
	"encoding/json"
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
		raw      bool
	)

	cmd := &cobra.Command{
		Use:   "read <module-name>",
		Short: "Read API documentation from database",
		Long:  `Read API documentation content from database. No MD files required.`,
		Example: `  # Read full API doc
  atlas-dev api read stdlib

  # Read specific function
  atlas-dev api read stdlib --function print

  # Read from stdin (auto-detected)
  echo '{"name":"stdlib"}' | atlas-dev api read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var moduleName string

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
				// Try to extract name or module
				if name, ok := item["name"].(string); ok {
					moduleName = name
				} else if module, ok := item["module"].(string); ok {
					moduleName = module
				} else {
					return fmt.Errorf("name or module required")
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("module name required")
				}
				moduleName = args[0]
			}

			// Query database by module name
			var (
				title          string
				functionsJSON  string
				functionsCount int
				content        string
			)

			err := database.QueryRow(`
				SELECT title, functions, functions_count, content
				FROM api_docs
				WHERE module = ? OR name = ?
			`, moduleName, moduleName).Scan(&title, &functionsJSON, &functionsCount, &content)

			if err != nil {
				return fmt.Errorf("API doc not found: %s", moduleName)
			}

			result := map[string]interface{}{
				"ok":    true,
				"title": title,
			}

			// Return raw content if requested
			if raw {
				result["content"] = content
				return output.Success(result)
			}

			// Parse functions JSON
			var functions []*api.Function
			if err := json.Unmarshal([]byte(functionsJSON), &functions); err != nil {
				return fmt.Errorf("failed to parse functions: %w", err)
			}

			if function != "" {
				// Find specific function
				var found *api.Function
				for _, fn := range functions {
					if fn.Name == function {
						found = fn
						break
					}
				}

				if found == nil {
					return fmt.Errorf("function not found: %s", function)
				}

				result["func"] = found.Name
				result["sig"] = found.Signature
				if found.Description != "" {
					result["desc"] = found.Description
				}
			} else {
				result["functions"] = functionsCount

				if detailed {
					fns := []map[string]string{}
					for _, fn := range functions {
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
	cmd.Flags().BoolVar(&raw, "raw", false, "Return raw markdown content")

	return cmd
}
