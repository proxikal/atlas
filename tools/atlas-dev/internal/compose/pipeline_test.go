package compose

import (
	"fmt"
	"testing"
)

func TestPipeline_Success(t *testing.T) {
	pipeline := NewPipeline()

	step1Called := false
	step2Called := false

	pipeline.AddStep("step1", func() error {
		step1Called = true
		return nil
	}, nil)

	pipeline.AddStep("step2", func() error {
		step2Called = true
		return nil
	}, nil)

	result := pipeline.Execute()

	if !result.Success {
		t.Error("Expected success=true")
	}

	if result.CompletedSteps != 2 {
		t.Errorf("Expected 2 completed steps, got %d", result.CompletedSteps)
	}

	if !step1Called || !step2Called {
		t.Error("Expected both steps to be called")
	}
}

func TestPipeline_Failure(t *testing.T) {
	pipeline := NewPipeline()

	step1Called := false
	step2Called := false

	pipeline.AddStep("step1", func() error {
		step1Called = true
		return nil
	}, nil)

	pipeline.AddStep("step2", func() error {
		step2Called = true
		return fmt.Errorf("step2 failed")
	}, nil)

	step3Called := false
	pipeline.AddStep("step3", func() error {
		step3Called = true
		return nil
	}, nil)

	result := pipeline.Execute()

	if result.Success {
		t.Error("Expected success=false")
	}

	if result.FailedStep != "step2" {
		t.Errorf("Expected failed_step='step2', got '%s'", result.FailedStep)
	}

	if result.CompletedSteps != 1 {
		t.Errorf("Expected 1 completed step, got %d", result.CompletedSteps)
	}

	if !step1Called {
		t.Error("Expected step1 to be called")
	}

	if !step2Called {
		t.Error("Expected step2 to be called")
	}

	if step3Called {
		t.Error("Expected step3 NOT to be called")
	}
}

func TestPipeline_Rollback(t *testing.T) {
	pipeline := NewPipeline()

	rollback1Called := false
	rollback2Called := false

	pipeline.AddStep("step1", func() error {
		return nil
	}, func() error {
		rollback1Called = true
		return nil
	})

	pipeline.AddStep("step2", func() error {
		return nil
	}, func() error {
		rollback2Called = true
		return nil
	})

	pipeline.AddStep("step3", func() error {
		return fmt.Errorf("fail at step3")
	}, nil)

	result := pipeline.Execute()

	if result.Success {
		t.Error("Expected success=false")
	}

	// Rollback should be called for step1 and step2 (in reverse order)
	if !rollback1Called || !rollback2Called {
		t.Error("Expected rollback to be called for step1 and step2")
	}
}

func TestPipeline_DryRun(t *testing.T) {
	pipeline := NewPipeline().WithDryRun(true)

	operationCalled := false
	pipeline.AddStep("step1", func() error {
		operationCalled = true
		return nil
	}, nil)

	result := pipeline.Execute()

	if !result.Success {
		t.Error("Expected success=true in dry-run")
	}

	if operationCalled {
		t.Error("Expected operation NOT to be called in dry-run")
	}

	if result.CompletedSteps != 1 {
		t.Errorf("Expected completed_steps=1 in dry-run, got %d", result.CompletedSteps)
	}
}

func TestPipeline_StopOnError(t *testing.T) {
	pipeline := NewPipeline().WithStopOnError(false)

	step1Called := false
	step2Called := false
	step3Called := false

	pipeline.AddStep("step1", func() error {
		step1Called = true
		return nil
	}, nil)

	pipeline.AddStep("step2", func() error {
		step2Called = true
		return fmt.Errorf("step2 error")
	}, nil)

	pipeline.AddStep("step3", func() error {
		step3Called = true
		return nil
	}, nil)

	result := pipeline.Execute()

	if result.Success {
		t.Error("Expected success=false")
	}

	// With stopOnError=false, all steps should be called
	if !step1Called || !step2Called || !step3Called {
		t.Error("Expected all steps to be called")
	}

	if len(result.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(result.Errors))
	}
}

func TestPipeline_ToCompactJSON(t *testing.T) {
	result := &PipelineResult{
		TotalSteps:     3,
		CompletedSteps: 2,
		FailedStep:     "step3",
		Success:        false,
		Errors:         []string{"error1"},
	}

	json := result.ToCompactJSON()

	if ok, exists := json["ok"].(bool); !exists || ok {
		t.Error("Expected ok=false")
	}

	if total, ok := json["total"].(int); !ok || total != 3 {
		t.Errorf("Expected total=3, got %v", json["total"])
	}

	if failedStep, ok := json["failed_step"].(string); !ok || failedStep != "step3" {
		t.Errorf("Expected failed_step='step3', got %v", json["failed_step"])
	}
}

func TestDryRunChanges_ToCompactJSON(t *testing.T) {
	changes := NewDryRunChanges("update", "before_value", "after_value")

	json := changes.ToCompactJSON()

	if op, ok := json["op"].(string); !ok || op != "update" {
		t.Errorf("Expected op='update', got %v", json["op"])
	}

	if before, ok := json["before"].(string); !ok || before != "before_value" {
		t.Errorf("Expected before='before_value', got %v", json["before"])
	}

	if change, ok := json["change"].(bool); !ok || !change {
		t.Error("Expected change=true")
	}
}

func TestDryRunChanges_NoChange(t *testing.T) {
	changes := NewDryRunChanges("noop", "same", "same")

	if changes.WillChange {
		t.Error("Expected WillChange=false when before==after")
	}
}

func TestExitCodeFromError(t *testing.T) {
	code1 := ExitCodeFromError(nil)
	if code1 != 0 {
		t.Errorf("Expected 0 for nil error, got %d", code1)
	}

	code2 := ExitCodeFromError(fmt.Errorf("some error"))
	if code2 != 1 {
		t.Errorf("Expected 1 for error, got %d", code2)
	}
}

func TestPropagateExitCode(t *testing.T) {
	tests := []struct {
		name     string
		codes    []int
		expected int
	}{
		{"all success", []int{0, 0, 0}, 0},
		{"first fails", []int{1, 0, 0}, 1},
		{"middle fails", []int{0, 2, 0}, 2},
		{"last fails", []int{0, 0, 3}, 3},
		{"empty", []int{}, 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			code := PropagateExitCode(tt.codes...)
			if code != tt.expected {
				t.Errorf("Expected %d, got %d", tt.expected, code)
			}
		})
	}
}

func TestPipeline_Duration(t *testing.T) {
	pipeline := NewPipeline()

	pipeline.AddStep("step1", func() error {
		return nil
	}, nil)

	result := pipeline.Execute()

	if result.Duration <= 0 {
		t.Error("Expected positive duration")
	}
}

func TestNewPipeline(t *testing.T) {
	pipeline := NewPipeline()

	if pipeline == nil {
		t.Fatal("Expected non-nil pipeline")
	}

	if !pipeline.stopOnError {
		t.Error("Expected stopOnError=true by default")
	}

	if len(pipeline.steps) != 0 {
		t.Errorf("Expected 0 steps, got %d", len(pipeline.steps))
	}
}
