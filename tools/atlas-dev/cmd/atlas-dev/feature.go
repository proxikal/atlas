package main

import (
	"github.com/spf13/cobra"
)

func featureCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "feature",
		Aliases: []string{"f", "feat"},
		Short:   "Feature management commands",
		Long: `Manage feature documentation - create, list, read, update, validate, sync, delete, and search features.

Features are tracked in both the database and docs/features/ directory.
The database is the source of truth for metadata, while markdown files contain full documentation.`,
	}

	cmd.AddCommand(featureCreateCmd())
	cmd.AddCommand(featureListCmd())
	cmd.AddCommand(featureReadCmd())
	cmd.AddCommand(featureUpdateCmd())
	cmd.AddCommand(featureValidateCmd())
	cmd.AddCommand(featureSyncCmd())
	cmd.AddCommand(featureDeleteCmd())
	cmd.AddCommand(featureSearchCmd())

	return cmd
}
