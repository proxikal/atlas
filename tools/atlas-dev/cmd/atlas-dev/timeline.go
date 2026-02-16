package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func timelineCmd() *cobra.Command {
	var days int

	cmd := &cobra.Command{
		Use:   "timeline",
		Short: "Show completion timeline",
		Long:  `Display phases completed grouped by date.`,
		Example: `  # Show full timeline
  atlas-dev timeline

  # Show last 30 days
  atlas-dev timeline --days 30`,
		RunE: func(cmd *cobra.Command, args []string) error {
			timeline, err := database.GetTimeline(days)
			if err != nil {
				return err
			}

			// Convert to compact JSON
			items := make([]map[string]interface{}, len(timeline))
			for i, entry := range timeline {
				items[i] = entry.ToCompactJSON()
			}

			result := map[string]interface{}{
				"timeline": items,
				"cnt":      len(items),
			}

			if days > 0 {
				result["days"] = days
			}

			return output.Success(result)
		},
	}

	cmd.Flags().IntVarP(&days, "days", "d", 0, "Limit to recent N days")

	return cmd
}
