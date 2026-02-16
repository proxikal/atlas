package main

import (
	"fmt"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
	"time"
)

func historyCreateCmd() *cobra.Command {
	var title, summary, content, htype string

	cmd := &cobra.Command{
		Use:   "create <name>",
		Short: "Create history entry",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			name := args[0]
			date := time.Now().Format("2006-01-02")
			if htype == "" {
				htype = "summary"
			}

			_, err := database.Exec(`
				INSERT INTO history (name, title, date, summary, content, type)
				VALUES (?, ?, ?, ?, ?, ?)
			`, name, title, date, summary, content, htype)
			if err != nil {
				return fmt.Errorf("failed to create history: %w", err)
			}

			return output.Success(map[string]interface{}{
				"name": name,
				"date": date,
			})
		},
	}

	cmd.Flags().StringVar(&title, "title", "", "Title (required)")
	cmd.Flags().StringVar(&summary, "summary", "", "Summary (required)")
	cmd.Flags().StringVar(&content, "content", "", "Full content")
	cmd.Flags().StringVar(&htype, "type", "summary", "Type (summary/restructure)")
	cmd.MarkFlagRequired("title")
	cmd.MarkFlagRequired("summary")
	return cmd
}
