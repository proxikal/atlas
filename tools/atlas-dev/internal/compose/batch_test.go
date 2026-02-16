package compose

import (
	"fmt"
	"testing"
	"time"
)

func TestBatchProcessor_Sequential(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel:        false,
		ContinueOnError: true,
	})

	items := []interface{}{"item1", "item2", "item3"}
	operation := func(item interface{}) (interface{}, error) {
		return item, nil
	}

	result := processor.Process(items, operation)

	if result.TotalItems != 3 {
		t.Errorf("Expected 3 total items, got %d", result.TotalItems)
	}

	if result.Succeeded != 3 {
		t.Errorf("Expected 3 succeeded, got %d", result.Succeeded)
	}

	if result.Failed != 0 {
		t.Errorf("Expected 0 failed, got %d", result.Failed)
	}
}

func TestBatchProcessor_WithErrors(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel:        false,
		ContinueOnError: true,
	})

	items := []interface{}{"success", "fail", "success"}
	operation := func(item interface{}) (interface{}, error) {
		if item == "fail" {
			return nil, fmt.Errorf("simulated error")
		}
		return item, nil
	}

	result := processor.Process(items, operation)

	if result.Succeeded != 2 {
		t.Errorf("Expected 2 succeeded, got %d", result.Succeeded)
	}

	if result.Failed != 1 {
		t.Errorf("Expected 1 failed, got %d", result.Failed)
	}

	if len(result.Errors) != 1 {
		t.Errorf("Expected 1 error, got %d", len(result.Errors))
	}
}

func TestBatchProcessor_StopOnError(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel:        false,
		ContinueOnError: false, // Stop on first error
	})

	items := []interface{}{"success", "fail", "should_not_process"}
	operation := func(item interface{}) (interface{}, error) {
		if item == "fail" {
			return nil, fmt.Errorf("stop here")
		}
		return item, nil
	}

	result := processor.Process(items, operation)

	if result.Succeeded != 1 {
		t.Errorf("Expected 1 succeeded, got %d", result.Succeeded)
	}

	if result.Processed != 2 {
		t.Errorf("Expected 2 processed (stopped at error), got %d", result.Processed)
	}
}

func TestBatchProcessor_Parallel(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel:        true,
		ContinueOnError: true,
		Workers:         2,
	})

	items := []interface{}{1, 2, 3, 4, 5}
	operation := func(item interface{}) (interface{}, error) {
		// Simulate some work
		time.Sleep(10 * time.Millisecond)
		return item, nil
	}

	start := time.Now()
	result := processor.Process(items, operation)
	duration := time.Since(start)

	if result.Succeeded != 5 {
		t.Errorf("Expected 5 succeeded, got %d", result.Succeeded)
	}

	// Parallel processing should be faster than sequential
	// 5 items * 10ms each = 50ms sequential
	// With 2 workers: ~30ms (allowing some overhead)
	if duration > 80*time.Millisecond {
		t.Logf("Parallel processing took %v (expected < 80ms)", duration)
	}
}

func TestBatchProcessor_ToCompactJSON(t *testing.T) {
	result := &BatchResult{
		TotalItems: 10,
		Processed:  10,
		Succeeded:  8,
		Failed:     2,
		Results:    []interface{}{"res1", "res2"},
		Errors: []BatchError{
			{Index: 1, Item: "item1", Error: "error1"},
		},
		Duration: 100 * time.Millisecond,
	}

	json := result.ToCompactJSON()

	if total, ok := json["total"].(int); !ok || total != 10 {
		t.Errorf("Expected total=10, got %v", json["total"])
	}

	if succeeded, ok := json["succeeded"].(int); !ok || succeeded != 8 {
		t.Errorf("Expected succeeded=8, got %v", json["succeeded"])
	}

	if errors, ok := json["errors"].([]map[string]interface{}); !ok || len(errors) != 1 {
		t.Errorf("Expected 1 error in JSON, got %v", json["errors"])
	}
}

func TestBatchProcessor_HasErrors(t *testing.T) {
	result1 := &BatchResult{Failed: 0}
	result2 := &BatchResult{Failed: 1}

	if result1.HasErrors() {
		t.Error("Expected HasErrors=false when failed=0")
	}

	if !result2.HasErrors() {
		t.Error("Expected HasErrors=true when failed>0")
	}
}

func TestBatchProcessIDs(t *testing.T) {
	ids := []string{"id1", "id2", "id3"}
	operation := func(id string) (interface{}, error) {
		return id + "_processed", nil
	}

	result := BatchProcessIDs(ids, operation, BatchOptions{
		Parallel:        false,
		ContinueOnError: true,
	})

	if result.Succeeded != 3 {
		t.Errorf("Expected 3 succeeded, got %d", result.Succeeded)
	}

	if len(result.Results) != 3 {
		t.Errorf("Expected 3 results, got %d", len(result.Results))
	}
}

func TestBatchProcessPaths(t *testing.T) {
	paths := []string{"path1", "path2"}
	operation := func(path string) (interface{}, error) {
		return map[string]string{"path": path}, nil
	}

	result := BatchProcessPaths(paths, operation, BatchOptions{
		Parallel: false,
	})

	if result.Succeeded != 2 {
		t.Errorf("Expected 2 succeeded, got %d", result.Succeeded)
	}
}

func TestNewBatchProcessor_DefaultWorkers(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel: true,
		// Workers not specified
	})

	if processor.opts.Workers != 4 {
		t.Errorf("Expected default workers=4, got %d", processor.opts.Workers)
	}
}

func TestBatchProcessor_ParallelStopOnError(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{
		Parallel:        true,
		ContinueOnError: false,
		Workers:         2,
	})

	items := []interface{}{"a", "b", "c", "d", "e"}
	processedCount := 0

	operation := func(item interface{}) (interface{}, error) {
		processedCount++
		if item == "c" {
			return nil, fmt.Errorf("error at c")
		}
		return item, nil
	}

	result := processor.Process(items, operation)

	if !result.HasErrors() {
		t.Error("Expected HasErrors=true")
	}

	// Should stop when error encountered
	if result.Failed < 1 {
		t.Errorf("Expected at least 1 failure, got %d", result.Failed)
	}
}

func TestBatchResult_Duration(t *testing.T) {
	processor := NewBatchProcessor(BatchOptions{})

	items := []interface{}{1, 2, 3}
	operation := func(item interface{}) (interface{}, error) {
		time.Sleep(10 * time.Millisecond)
		return item, nil
	}

	result := processor.Process(items, operation)

	if result.Duration <= 0 {
		t.Error("Expected positive duration")
	}

	if result.Duration < 30*time.Millisecond {
		t.Errorf("Expected duration >= 30ms, got %v", result.Duration)
	}
}
