package context

import (
	"bufio"
	"fmt"
	"os"
	"regexp"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/parser"
)

// PhaseFile represents parsed phase file data
type PhaseFile struct {
	Objective          string
	Priority           string
	Dependencies       []string
	Deliverables       []string
	AcceptanceCriteria []string
	EstimatedTime      string
	Files              []string
}

// blockerRegex matches blocker warning lines
var blockerRegex = regexp.MustCompile(`^\*\*REQUIRED:\*\*\s*(.+)$`)

// ParsePhaseFile parses a phase markdown file
func ParsePhaseFile(path string) (*PhaseFile, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open phase file: %w", err)
	}
	defer func() { _ = file.Close() }()

	// Parse sections
	sections, err := parser.SplitByHeadings(file)
	if err != nil {
		return nil, fmt.Errorf("failed to parse sections: %w", err)
	}

	phaseFile := &PhaseFile{
		Deliverables:       []string{},
		AcceptanceCriteria: []string{},
		Dependencies:       []string{},
		Files:              []string{},
	}

	// Extract objective
	if objSection := parser.FindSection(sections, "Objective"); objSection != nil {
		phaseFile.Objective = extractFirstParagraph(objSection.Content)
	}

	// Extract priority, dependencies, estimate from blockers section
	if blockSection := parser.FindSection(sections, "BLOCKERS - CHECK BEFORE STARTING"); blockSection != nil {
		extractBlockerInfo(blockSection.Content, phaseFile)
	}

	// Extract files section
	if filesSection := parser.FindSection(sections, "Files"); filesSection != nil {
		phaseFile.Files = extractFiles(filesSection.Content)
	}

	// Extract dependencies
	if depSection := parser.FindSection(sections, "Dependencies"); depSection != nil {
		phaseFile.Dependencies = extractDependencies(depSection.Content)
	}

	// Extract acceptance criteria
	if acceptSection := parser.FindSection(sections, "Acceptance"); acceptSection != nil {
		phaseFile.AcceptanceCriteria = extractAcceptanceCriteria(acceptSection.Content)
	}

	// Extract implementation details as deliverables
	// Find all level 3 sections that come after the Implementation section
	phaseFile.Deliverables = extractDeliverablesFromSections(sections)

	return phaseFile, nil
}

// extractFirstParagraph gets the first non-empty paragraph
func extractFirstParagraph(content string) string {
	lines := strings.Split(content, "\n")
	var paragraph []string

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if trimmed == "" {
			if len(paragraph) > 0 {
				break
			}
			continue
		}
		paragraph = append(paragraph, trimmed)
	}

	return strings.Join(paragraph, " ")
}

// extractBlockerInfo extracts priority, dependencies, and estimate from blocker section
func extractBlockerInfo(content string, phaseFile *PhaseFile) {
	scanner := bufio.NewScanner(strings.NewReader(content))

	for scanner.Scan() {
		line := scanner.Text()

		// Check for blocker required line
		if matches := blockerRegex.FindStringSubmatch(line); matches != nil {
			req := strings.TrimSpace(matches[1])
			// Extract phase number if it mentions a phase
			if strings.Contains(strings.ToLower(req), "phase") {
				phaseFile.Dependencies = append(phaseFile.Dependencies, req)
			}
		}

		// Extract priority if mentioned
		if strings.Contains(strings.ToLower(line), "priority:") {
			parts := strings.SplitN(line, ":", 2)
			if len(parts) == 2 {
				phaseFile.Priority = strings.TrimSpace(parts[1])
			}
		}

		// Extract estimate if mentioned
		if strings.Contains(strings.ToLower(line), "estimate") || strings.Contains(strings.ToLower(line), "time") {
			if strings.Contains(line, "hour") || strings.Contains(line, "min") {
				phaseFile.EstimatedTime = extractTimeEstimate(line)
			}
		}
	}
}

// extractTimeEstimate extracts time estimate from text
func extractTimeEstimate(text string) string {
	// Look for patterns like "4-6 hours", "2 hours", "30 minutes"
	timeRegex := regexp.MustCompile(`(\d+(?:-\d+)?)\s*(hour|hr|minute|min)s?`)
	if matches := timeRegex.FindStringSubmatch(text); matches != nil {
		return strings.Join(matches[1:], " ")
	}
	return ""
}

// extractFiles extracts file paths from Files section
func extractFiles(content string) []string {
	var files []string
	scanner := bufio.NewScanner(strings.NewReader(content))

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Match patterns like:
		// **Create:** `file/path.go`
		// **Update:** `file/path.go`
		createRegex := regexp.MustCompile(`^\*\*(?:Create|Update):\*\*\s*` + "`" + `([^` + "`" + `]+)` + "`")
		if matches := createRegex.FindStringSubmatch(line); matches != nil {
			files = append(files, matches[1])
		}
	}

	return files
}

// extractDependencies extracts dependencies from content
func extractDependencies(content string) []string {
	var deps []string
	scanner := bufio.NewScanner(strings.NewReader(content))

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Skip empty lines and section headers
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Extract bullet points
		if strings.HasPrefix(line, "-") || strings.HasPrefix(line, "*") {
			dep := strings.TrimSpace(strings.TrimPrefix(strings.TrimPrefix(line, "-"), "*"))
			if dep != "" {
				deps = append(deps, dep)
			}
		}
	}

	return deps
}

// extractAcceptanceCriteria extracts acceptance criteria checkboxes
func extractAcceptanceCriteria(content string) []string {
	var criteria []string
	scanner := bufio.NewScanner(strings.NewReader(content))

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Match checkbox patterns: - [ ] criterion or - criterion
		if strings.HasPrefix(line, "- [ ]") {
			criterion := strings.TrimSpace(strings.TrimPrefix(line, "- [ ]"))
			if criterion != "" {
				criteria = append(criteria, criterion)
			}
		} else if strings.HasPrefix(line, "- ") && !strings.Contains(line, "[") {
			criterion := strings.TrimSpace(strings.TrimPrefix(line, "- "))
			if criterion != "" && !strings.HasPrefix(criterion, "#") {
				criteria = append(criteria, criterion)
			}
		}
	}

	return criteria
}

// extractDeliverablesFromSections extracts deliverables from level 3 sections after Implementation
func extractDeliverablesFromSections(sections []parser.Section) []string {
	var deliverables []string

	// Find the Implementation section index
	implIndex := -1
	for i, section := range sections {
		if strings.ToLower(section.Heading) == "implementation" && section.Level == 2 {
			implIndex = i
			break
		}
	}

	// No Implementation section found
	if implIndex == -1 {
		return deliverables
	}

	// Collect all level 3 sections after Implementation until next level 2 section
	for i := implIndex + 1; i < len(sections); i++ {
		section := sections[i]

		// Stop at next level 2 section (e.g., Tests, Acceptance)
		if section.Level <= 2 {
			break
		}

		// Collect level 3 sections as deliverables
		if section.Level == 3 {
			deliverables = append(deliverables, section.Heading)
		}
	}

	return deliverables
}
