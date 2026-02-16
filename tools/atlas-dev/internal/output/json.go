package output

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
)

// Success outputs compact JSON with ok=true
func Success(data map[string]interface{}) error {
	// Add ok=true
	data["ok"] = true

	// Remove null/empty fields
	cleaned := removeEmpty(data)

	// Compact encoding (no spaces)
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(cleaned)
}

// Error outputs compact JSON with ok=false and error message
func Error(err error, details map[string]interface{}) error {
	data := map[string]interface{}{
		"ok":  false,
		"err": err.Error(),
	}

	// Add details
	for k, v := range details {
		data[k] = v
	}

	// Remove null/empty fields
	cleaned := removeEmpty(data)

	// Compact encoding
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(cleaned)
}

// removeEmpty removes null/empty values
func removeEmpty(m map[string]interface{}) map[string]interface{} {
	result := make(map[string]interface{})
	for k, v := range m {
		if !isEmpty(v) {
			result[k] = v
		}
	}
	return result
}

// isEmpty checks if value is empty/null
func isEmpty(v interface{}) bool {
	if v == nil {
		return true
	}

	switch val := v.(type) {
	case string:
		return val == ""
	case []interface{}:
		return len(val) == 0
	case map[string]interface{}:
		return len(val) == 0
	default:
		return false
	}
}

// StreamLine outputs a single JSON object per line (for streaming)
func StreamLine(data map[string]interface{}) error {
	cleaned := removeEmpty(data)
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(cleaned)
}

// Lines outputs array of strings as newline-separated values (for xargs)
func Lines(values []string) error {
	_, err := fmt.Fprintln(os.Stdout, strings.Join(values, "\n"))
	return err
}

// LinesFromField extracts field from array of objects and outputs as lines
func LinesFromField(items []map[string]interface{}, field string) error {
	values := []string{}
	for _, item := range items {
		if val, ok := item[field].(string); ok && val != "" {
			values = append(values, val)
		}
	}
	return Lines(values)
}

// SuccessWithFormat outputs data in specified format
func SuccessWithFormat(data map[string]interface{}, format string) error {
	switch format {
	case "lines":
		// Try to extract array field for lines output
		for _, key := range []string{"items", "results", "phases", "decisions", "features"} {
			if arr, ok := data[key].([]map[string]interface{}); ok {
				// Try common ID fields
				for _, field := range []string{"id", "path", "name"} {
					if len(arr) > 0 {
						if _, exists := arr[0][field]; exists {
							return LinesFromField(arr, field)
						}
					}
				}
			}
		}
		// Fallback to JSON if can't extract lines
		return Success(data)
	case "json":
		fallthrough
	default:
		return Success(data)
	}
}

// Array outputs array directly with ok=true wrapper
func Array(items interface{}) error {
	return Success(map[string]interface{}{
		"items": items,
	})
}
