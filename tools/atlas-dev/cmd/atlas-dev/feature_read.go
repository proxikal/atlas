package main

import (
	"path/filepath"

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
  atlas-dev feature read pattern-matching`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]

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
