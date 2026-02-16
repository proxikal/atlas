package compose

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"
)

// StdinInput represents parsed stdin input
type StdinInput struct {
	Raw    []byte
	IsArray bool
	Items   []map[string]interface{}
}

// ReadStdin reads all content from stdin until EOF
func ReadStdin() ([]byte, error) {
	// Check if stdin is available
	stat, err := os.Stdin.Stat()
	if err != nil {
		return nil, fmt.Errorf("failed to stat stdin: %w", err)
	}

	// Check if stdin is a pipe or has data
	if (stat.Mode() & os.ModeCharDevice) != 0 {
		return nil, fmt.Errorf("no input from stdin (not a pipe)")
	}

	// Read all from stdin
	data, err := io.ReadAll(os.Stdin)
	if err != nil {
		return nil, fmt.Errorf("failed to read stdin: %w", err)
	}

	if len(data) == 0 {
		return nil, fmt.Errorf("empty stdin")
	}

	return data, nil
}

// ParseJSONFromStdin parses stdin content as JSON (object or array)
func ParseJSONFromStdin(data []byte) (*StdinInput, error) {
	input := &StdinInput{
		Raw:   data,
		Items: []map[string]interface{}{},
	}

	// Try to parse as array first
	var arr []map[string]interface{}
	if err := json.Unmarshal(data, &arr); err == nil {
		input.IsArray = true
		input.Items = arr
		return input, nil
	}

	// Try to parse as single object
	var obj map[string]interface{}
	if err := json.Unmarshal(data, &obj); err == nil {
		input.IsArray = false
		input.Items = []map[string]interface{}{obj}
		return input, nil
	}

	// Try to parse as array of strings (IDs)
	var strArr []string
	if err := json.Unmarshal(data, &strArr); err == nil {
		input.IsArray = true
		for _, s := range strArr {
			input.Items = append(input.Items, map[string]interface{}{"id": s})
		}
		return input, nil
	}

	return nil, fmt.Errorf("invalid JSON: must be object, array of objects, or array of strings")
}

// ExtractIDs extracts ID fields from parsed JSON input
func ExtractIDs(input *StdinInput) []string {
	ids := []string{}

	for _, item := range input.Items {
		// Try "id" field
		if id, ok := item["id"].(string); ok && id != "" {
			ids = append(ids, id)
			continue
		}

		// Try "ID" field (uppercase)
		if id, ok := item["ID"].(string); ok && id != "" {
			ids = append(ids, id)
			continue
		}

		// Try "phase_id" field
		if id, ok := item["phase_id"].(string); ok && id != "" {
			ids = append(ids, id)
			continue
		}

		// Try "decision_id" field
		if id, ok := item["decision_id"].(string); ok && id != "" {
			ids = append(ids, id)
			continue
		}

		// Try "feature_id" field
		if id, ok := item["feature_id"].(string); ok && id != "" {
			ids = append(ids, id)
			continue
		}
	}

	return ids
}

// ExtractPaths extracts path fields from parsed JSON input
func ExtractPaths(input *StdinInput) []string {
	paths := []string{}

	for _, item := range input.Items {
		// Try "path" field
		if path, ok := item["path"].(string); ok && path != "" {
			paths = append(paths, path)
			continue
		}

		// Try "file_path" field
		if path, ok := item["file_path"].(string); ok && path != "" {
			paths = append(paths, path)
			continue
		}

		// Try "phase_path" field
		if path, ok := item["phase_path"].(string); ok && path != "" {
			paths = append(paths, path)
			continue
		}

		// Try "spec_path" field
		if path, ok := item["spec_path"].(string); ok && path != "" {
			paths = append(paths, path)
			continue
		}
	}

	return paths
}

// ExtractField extracts a specific field from all items
func ExtractField(input *StdinInput, fieldName string) []string {
	values := []string{}

	for _, item := range input.Items {
		if val, ok := item[fieldName].(string); ok && val != "" {
			values = append(values, val)
		}
	}

	return values
}

// ExtractFirstID extracts the first ID from stdin (for single-item commands)
func ExtractFirstID(input *StdinInput) (string, error) {
	ids := ExtractIDs(input)
	if len(ids) == 0 {
		return "", fmt.Errorf("no ID found in stdin")
	}
	return ids[0], nil
}

// ExtractFirstPath extracts the first path from stdin (for single-item commands)
func ExtractFirstPath(input *StdinInput) (string, error) {
	paths := ExtractPaths(input)
	if len(paths) == 0 {
		return "", fmt.Errorf("no path found in stdin")
	}
	return paths[0], nil
}

// ReadAndParseStdin reads and parses stdin in one call
func ReadAndParseStdin() (*StdinInput, error) {
	data, err := ReadStdin()
	if err != nil {
		return nil, err
	}

	return ParseJSONFromStdin(data)
}

// HasStdin checks if stdin has data without consuming it
func HasStdin() bool {
	stat, err := os.Stdin.Stat()
	if err != nil {
		return false
	}

	// Check if stdin is a pipe or file (not a terminal)
	return (stat.Mode() & os.ModeCharDevice) == 0
}

// FormatAsLines converts array of values to newline-separated output (for xargs)
func FormatAsLines(values []string) string {
	return strings.Join(values, "\n")
}

// FormatAsJSON converts array of values to JSON array
func FormatAsJSON(values []string) (string, error) {
	data, err := json.Marshal(values)
	if err != nil {
		return "", err
	}
	return string(data), nil
}
