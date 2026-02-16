package parser

import (
	"bufio"
	"fmt"
	"io"
	"regexp"
	"strings"
)

// Metadata extracts key:value pairs from markdown metadata
type Metadata map[string]string

// NumberedItem represents an item in a numbered list
type NumberedItem struct {
	Number int
	Text   string
}

// CheckboxItem represents an item in a checkbox list
type CheckboxItem struct {
	Checked bool
	Text    string
}

// CodeBlock represents a code block with optional language
type CodeBlock struct {
	Language string
	Content  string
}

// Section represents a markdown section with heading and content
type Section struct {
	Level   int
	Heading string
	Content string
}

// Regular expressions for parsing
var (
	metadataRegex  = regexp.MustCompile(`^([A-Za-z][A-Za-z0-9 ]*?):\s*(.+)$`)
	numberedRegex  = regexp.MustCompile(`^(\d+)\.\s+(.+)$`)
	checkboxRegex  = regexp.MustCompile(`^-\s+\[([ xX])\]\s+(.+)$`)
	headingRegex   = regexp.MustCompile(`^(#{1,6})\s+(.+)$`)
	codeBlockRegex = regexp.MustCompile("^```([a-z]*)$")
)

// ExtractMetadata parses key:value metadata lines
func ExtractMetadata(r io.Reader) (Metadata, error) {
	metadata := make(Metadata)
	scanner := bufio.NewScanner(r)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Skip empty lines
		if line == "" {
			continue
		}

		// Match metadata pattern
		matches := metadataRegex.FindStringSubmatch(line)
		if matches != nil {
			key := strings.ToLower(strings.TrimSpace(matches[1]))
			value := strings.TrimSpace(matches[2])
			metadata[key] = value
		}
	}

	return metadata, scanner.Err()
}

// ParseNumberedList extracts numbered list items
func ParseNumberedList(r io.Reader) ([]NumberedItem, error) {
	var items []NumberedItem
	scanner := bufio.NewScanner(r)

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Skip empty lines
		if line == "" {
			continue
		}

		// Match numbered pattern
		matches := numberedRegex.FindStringSubmatch(line)
		if matches != nil {
			var num int
			// Parse number (ignore errors, default to 0)
			_, _ = fmt.Sscanf(matches[1], "%d", &num)

			items = append(items, NumberedItem{
				Number: num,
				Text:   strings.TrimSpace(matches[2]),
			})
		}
	}

	return items, scanner.Err()
}

// ParseCheckboxList extracts checkbox list items
func ParseCheckboxList(r io.Reader) ([]CheckboxItem, error) {
	var items []CheckboxItem
	scanner := bufio.NewScanner(r)

	for scanner.Scan() {
		line := scanner.Text()

		// Match checkbox pattern
		matches := checkboxRegex.FindStringSubmatch(line)
		if matches != nil {
			checked := strings.ToLower(matches[1]) == "x"
			items = append(items, CheckboxItem{
				Checked: checked,
				Text:    strings.TrimSpace(matches[2]),
			})
		}
	}

	return items, scanner.Err()
}

// ExtractCodeBlocks finds all code blocks in markdown
func ExtractCodeBlocks(r io.Reader) ([]CodeBlock, error) {
	var blocks []CodeBlock
	scanner := bufio.NewScanner(r)

	var inBlock bool
	var currentBlock CodeBlock
	var contentLines []string

	for scanner.Scan() {
		line := scanner.Text()

		// Check for code block start/end
		matches := codeBlockRegex.FindStringSubmatch(strings.TrimSpace(line))
		if matches != nil {
			if !inBlock {
				// Start of code block
				inBlock = true
				currentBlock = CodeBlock{Language: matches[1]}
				contentLines = []string{}
			} else {
				// End of code block
				currentBlock.Content = strings.Join(contentLines, "\n")
				blocks = append(blocks, currentBlock)
				inBlock = false
			}
			continue
		}

		// Collect content if in block
		if inBlock {
			contentLines = append(contentLines, line)
		}
	}

	return blocks, scanner.Err()
}

// SplitByHeadings splits document into sections by headings
func SplitByHeadings(r io.Reader) ([]Section, error) {
	var sections []Section
	scanner := bufio.NewScanner(r)

	var currentSection *Section
	var contentLines []string

	for scanner.Scan() {
		line := scanner.Text()

		// Check for heading
		matches := headingRegex.FindStringSubmatch(strings.TrimSpace(line))
		if matches != nil {
			// Save previous section if exists
			if currentSection != nil {
				currentSection.Content = strings.TrimSpace(strings.Join(contentLines, "\n"))
				sections = append(sections, *currentSection)
			}

			// Start new section
			level := len(matches[1])
			heading := strings.TrimSpace(matches[2])
			currentSection = &Section{
				Level:   level,
				Heading: heading,
			}
			contentLines = []string{}
			continue
		}

		// Collect content
		if currentSection != nil {
			contentLines = append(contentLines, line)
		}
	}

	// Save last section
	if currentSection != nil {
		currentSection.Content = strings.TrimSpace(strings.Join(contentLines, "\n"))
		sections = append(sections, *currentSection)
	}

	return sections, scanner.Err()
}

// FindSection finds a section by heading (case-insensitive)
func FindSection(sections []Section, heading string) *Section {
	headingLower := strings.ToLower(heading)
	for i := range sections {
		if strings.ToLower(sections[i].Heading) == headingLower {
			return &sections[i]
		}
	}
	return nil
}

// ExtractMetadataFromSection extracts metadata from a specific section
func ExtractMetadataFromSection(sections []Section, heading string) (Metadata, error) {
	section := FindSection(sections, heading)
	if section == nil {
		return make(Metadata), nil
	}

	return ExtractMetadata(strings.NewReader(section.Content))
}
