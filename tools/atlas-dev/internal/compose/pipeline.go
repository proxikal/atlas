package compose

import (
	"fmt"
	"time"
)

// PipelineStep represents a single step in a pipeline
type PipelineStep struct {
	Name      string
	Operation func() error
	Rollback  func() error // Optional rollback function
}

// PipelineResult represents the result of pipeline execution
type PipelineResult struct {
	TotalSteps    int
	CompletedSteps int
	FailedStep    string
	Success       bool
	Errors        []string
	Duration      time.Duration
}

// Pipeline represents a multi-step pipeline with error handling
type Pipeline struct {
	steps          []PipelineStep
	dryRun         bool
	stopOnError    bool
	completedSteps []string
}

// NewPipeline creates a new pipeline
func NewPipeline() *Pipeline {
	return &Pipeline{
		steps:          []PipelineStep{},
		stopOnError:    true,
		completedSteps: []string{},
	}
}

// AddStep adds a step to the pipeline
func (p *Pipeline) AddStep(name string, operation func() error, rollback func() error) *Pipeline {
	p.steps = append(p.steps, PipelineStep{
		Name:      name,
		Operation: operation,
		Rollback:  rollback,
	})
	return p
}

// WithDryRun enables dry-run mode
func (p *Pipeline) WithDryRun(enabled bool) *Pipeline {
	p.dryRun = enabled
	return p
}

// WithStopOnError sets whether to stop on first error
func (p *Pipeline) WithStopOnError(stop bool) *Pipeline {
	p.stopOnError = stop
	return p
}

// Execute runs the pipeline
func (p *Pipeline) Execute() *PipelineResult {
	start := time.Now()

	result := &PipelineResult{
		TotalSteps: len(p.steps),
		Success:    true,
		Errors:     []string{},
	}

	if p.dryRun {
		result.Success = true
		result.CompletedSteps = len(p.steps)
		result.Duration = time.Since(start)
		return result
	}

	for i, step := range p.steps {
		err := step.Operation()
		if err != nil {
			result.Success = false
			result.FailedStep = step.Name
			result.Errors = append(result.Errors, fmt.Sprintf("Step %d (%s): %v", i+1, step.Name, err))

			if p.stopOnError {
				// Rollback completed steps in reverse order
				p.rollback(i - 1)
				result.Duration = time.Since(start)
				return result
			}
		} else {
			result.CompletedSteps++
			p.completedSteps = append(p.completedSteps, step.Name)
		}
	}

	result.Duration = time.Since(start)
	return result
}

// rollback rolls back completed steps in reverse order
func (p *Pipeline) rollback(upToIndex int) {
	for i := upToIndex; i >= 0; i-- {
		if p.steps[i].Rollback != nil {
			_ = p.steps[i].Rollback()
		}
	}
}

// ToCompactJSON converts PipelineResult to compact JSON
func (pr *PipelineResult) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"ok":        pr.Success,
		"total":     pr.TotalSteps,
		"completed": pr.CompletedSteps,
		"duration":  pr.Duration.Milliseconds(),
	}

	if !pr.Success {
		result["failed_step"] = pr.FailedStep
	}

	if len(pr.Errors) > 0 {
		result["errors"] = pr.Errors
	}

	return result
}

// DryRunChanges represents changes that would be made in a dry-run
type DryRunChanges struct {
	Operation string
	Before    interface{}
	After     interface{}
	WillChange bool
}

// NewDryRunChanges creates a new dry-run changes report
func NewDryRunChanges(operation string, before, after interface{}) *DryRunChanges {
	return &DryRunChanges{
		Operation:  operation,
		Before:     before,
		After:      after,
		WillChange: before != after,
	}
}

// ToCompactJSON converts DryRunChanges to compact JSON
func (d *DryRunChanges) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"op":     d.Operation,
		"before": d.Before,
		"after":  d.After,
		"change": d.WillChange,
	}
}

// ExitCodeFromError returns appropriate exit code based on error type
func ExitCodeFromError(err error) int {
	if err == nil {
		return 0
	}

	// Could add more sophisticated error type detection here
	// For now, return 1 for any error
	return 1
}

// PropagateExitCode propagates exit code through pipeline
// Returns non-zero on first failure
func PropagateExitCode(codes ...int) int {
	for _, code := range codes {
		if code != 0 {
			return code
		}
	}
	return 0
}
