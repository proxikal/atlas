package main

import (
	"fmt"
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/feature"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureReadCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "read <name>",
		Short: "Read a feature",
		Long:  `Read complete feature details combining database and markdown file data.`,
		Example: `  # Read feature
  atlas-dev feature read pattern-matching

  # Read from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				name, err = compose.ExtractFirstString(input, "name")
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("feature name required")
				}
				name = args[0]
			}

			// Get from database
			dbFeature, err := database.GetFeature(name)
			if err != nil {
				return err
			}

			// Parse markdown file
			markdownPath := filepath.Join("../../docs/features", name+".md")
			parsedFeature, err := feature.Parse(markdownPath)
			if err != nil {
				// If markdown doesn't exist, just use DB data
				result := dbFeature.ToCompactJSON()
				result["msg"] = "Feature found (markdown file missing)"
				return output.Success(result)
			}

			// Combine DB + file data
			result := dbFeature.ToCompactJSON()
			if parsedFeature.Overview != "" {
				result["overview"] = parsedFeature.Overview
			}
			if parsedFeature.ImplFile != "" {
				result["impl"] = parsedFeature.ImplFile
			}
			if parsedFeature.TestFile != "" {
				result["tests"] = parsedFeature.TestFile
			}
			if parsedFeature.FunctionCount > 0 {
				result["fn_cnt"] = parsedFeature.FunctionCount
			}
			if parsedFeature.TestCount > 0 {
				result["test_cnt"] = parsedFeature.TestCount
			}
			if parsedFeature.Parity > 0 {
				result["parity"] = parsedFeature.Parity
			}
			if len(parsedFeature.Functions) > 0 {
				result["functions"] = parsedFeature.Functions
			}

			return output.Success(result)
		},
	}

	return cmd
}
