package compose

import (
	"fmt"
	"os"
	"sync"
	"time"
)

// BatchResult represents the result of a batch operation
type BatchResult struct {
	TotalItems    int
	Processed     int
	Succeeded     int
	Failed        int
	Results       []interface{}
	Errors        []BatchError
	Duration      time.Duration
}

// BatchError represents an error for a specific item
type BatchError struct {
	Index int
	Item  interface{}
	Error string
}

// BatchOptions configures batch processing behavior
type BatchOptions struct {
	Parallel        bool   // Process items in parallel
	ContinueOnError bool   // Continue processing even if items fail
	ShowProgress    bool   // Show progress to stderr
	Workers         int    // Number of parallel workers (default: 4)
}

// BatchProcessor processes multiple items with a given operation
type BatchProcessor struct {
	opts BatchOptions
}

// NewBatchProcessor creates a new batch processor
func NewBatchProcessor(opts BatchOptions) *BatchProcessor {
	// Set default workers if not specified
	if opts.Workers == 0 {
		opts.Workers = 4
	}
	return &BatchProcessor{opts: opts}
}

// Process executes operation for each item and collects results
func (bp *BatchProcessor) Process(items []interface{}, operation func(interface{}) (interface{}, error)) *BatchResult {
	start := time.Now()

	result := &BatchResult{
		TotalItems: len(items),
		Results:    make([]interface{}, 0, len(items)),
		Errors:     []BatchError{},
	}

	if bp.opts.Parallel {
		bp.processParallel(items, operation, result)
	} else {
		bp.processSequential(items, operation, result)
	}

	result.Duration = time.Since(start)
	result.Processed = result.Succeeded + result.Failed

	return result
}

// processSequential processes items one by one
func (bp *BatchProcessor) processSequential(items []interface{}, operation func(interface{}) (interface{}, error), result *BatchResult) {
	for i, item := range items {
		if bp.opts.ShowProgress {
			bp.showProgress(i+1, len(items), item)
		}

		res, err := operation(item)
		if err != nil {
			result.Failed++
			result.Errors = append(result.Errors, BatchError{
				Index: i,
				Item:  item,
				Error: err.Error(),
			})

			if !bp.opts.ContinueOnError {
				return
			}
		} else {
			result.Succeeded++
			result.Results = append(result.Results, res)
		}
	}
}

// processParallel processes items concurrently using worker pool
func (bp *BatchProcessor) processParallel(items []interface{}, operation func(interface{}) (interface{}, error), result *BatchResult) {
	type workItem struct {
		index int
		item  interface{}
	}

	type workResult struct {
		index  int
		result interface{}
		err    error
	}

	// Create channels
	jobs := make(chan workItem, len(items))
	results := make(chan workResult, len(items))

	// Start workers
	var wg sync.WaitGroup
	for w := 0; w < bp.opts.Workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for job := range jobs {
				res, err := operation(job.item)
				results <- workResult{
					index:  job.index,
					result: res,
					err:    err,
				}
			}
		}()
	}

	// Send jobs
	go func() {
		for i, item := range items {
			jobs <- workItem{index: i, item: item}
		}
		close(jobs)
	}()

	// Wait for workers to finish
	go func() {
		wg.Wait()
		close(results)
	}()

	// Collect results
	processed := 0
	for res := range results {
		processed++

		if bp.opts.ShowProgress {
			bp.showProgress(processed, len(items), items[res.index])
		}

		if res.err != nil {
			result.Failed++
			result.Errors = append(result.Errors, BatchError{
				Index: res.index,
				Item:  items[res.index],
				Error: res.err.Error(),
			})

			if !bp.opts.ContinueOnError {
				// Drain remaining results
				for range results {
				}
				return
			}
		} else {
			result.Succeeded++
			result.Results = append(result.Results, res.result)
		}
	}
}

// showProgress outputs progress to stderr
func (bp *BatchProcessor) showProgress(current, total int, item interface{}) {
	percentage := float64(current) / float64(total) * 100.0
	fmt.Fprintf(os.Stderr, "[%d/%d] (%.1f%%) Processing: %v\n", current, total, percentage, item)
}

// ToCompactJSON converts BatchResult to compact JSON
func (br *BatchResult) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"total":     br.TotalItems,
		"processed": br.Processed,
		"succeeded": br.Succeeded,
		"failed":    br.Failed,
		"duration":  br.Duration.Milliseconds(),
	}

	if len(br.Results) > 0 {
		result["results"] = br.Results
	}

	if len(br.Errors) > 0 {
		errors := []map[string]interface{}{}
		for _, e := range br.Errors {
			errors = append(errors, map[string]interface{}{
				"index": e.Index,
				"item":  e.Item,
				"error": e.Error,
			})
		}
		result["errors"] = errors
	}

	return result
}

// HasErrors returns true if any items failed
func (br *BatchResult) HasErrors() bool {
	return br.Failed > 0
}

// BatchProcessIDs processes a list of IDs with given operation
func BatchProcessIDs(ids []string, operation func(string) (interface{}, error), opts BatchOptions) *BatchResult {
	// Convert IDs to interface{} array
	items := make([]interface{}, len(ids))
	for i, id := range ids {
		items[i] = id
	}

	// Create processor
	processor := NewBatchProcessor(opts)

	// Wrap operation to handle string type
	wrappedOp := func(item interface{}) (interface{}, error) {
		id, ok := item.(string)
		if !ok {
			return nil, fmt.Errorf("item is not a string")
		}
		return operation(id)
	}

	return processor.Process(items, wrappedOp)
}

// BatchProcessPaths processes a list of paths with given operation
func BatchProcessPaths(paths []string, operation func(string) (interface{}, error), opts BatchOptions) *BatchResult {
	return BatchProcessIDs(paths, operation, opts)
}
