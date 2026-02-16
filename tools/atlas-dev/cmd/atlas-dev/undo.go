package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/undo"
	"github.com/spf13/cobra"
)

func undoCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "undo",
		Short: "Undo last operation",
		Long: `Rollback the last operation using audit log.

Supported operations:
- Phase complete (restores to pending status)
- Decision create (deletes the decision)
- Decision update (restores old values)

The operation is atomic and the audit log entry is deleted after successful undo.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Create undo manager
			undoMgr := undo.NewUndoManager(database)

			// Perform undo
			result, err := undoMgr.Undo()
			if err != nil {
				return err
			}

			// Return compact JSON result
			response := map[string]interface{}{
				"action": result.Action,
				"entity": result.EntityType,
				"id":     result.EntityID,
			}

			if len(result.Restored) > 0 {
				response["restored"] = result.Restored
			}

			return output.Success(response)
		},
	}
}
