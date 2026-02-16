package parity

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// Reference represents a cross-reference in documentation
type Reference struct {
	SourceFile   string
	SourceLine   int
	TargetPath   string
	TargetAnchor string // section/anchor if specified
	Text         string
	Type         string // "markdown", "spec", "api", "phase"
}

// BrokenReference represents a broken cross-reference
type BrokenReference struct {
	Ref           Reference
	ErrorType     string // "file_missing", "section_missing", "invalid_format"
	FixSuggestion string
}

// ReferenceReport contains cross-reference validation results
type ReferenceReport struct {
	TotalRefs    int
	ValidRefs    int
	BrokenRefs   []BrokenReference
	OrphanedDocs []string // Documents not referenced anywhere
}

// ReferenceValidator validates cross-references in documentation
type ReferenceValidator struct {
	rootDir string
	docsDir string
}

// NewReferenceValidator creates a new reference validator
func NewReferenceValidator(rootDir, docsDir string) *ReferenceValidator {
	return &ReferenceValidator{
		rootDir: rootDir,
		docsDir: docsDir,
	}
}

// ValidateReferences validates all cross-references
func (v *ReferenceValidator) ValidateReferences() (*ReferenceReport, error) {
	report := &ReferenceReport{
		BrokenRefs:   []BrokenReference{},
		OrphanedDocs: []string{},
	}

	// Collect all references
	refs, err := v.collectReferences()
	if err != nil {
		return nil, fmt.Errorf("failed to collect references: %w", err)
	}

	report.TotalRefs = len(refs)

	// Validate each reference
	for _, ref := range refs {
		if err := v.validateReference(&ref); err != nil {
			broken := BrokenReference{
				Ref:           ref,
				ErrorType:     v.classifyError(err),
				FixSuggestion: v.generateFixSuggestion(&ref, err),
			}
			report.BrokenRefs = append(report.BrokenRefs, broken)
		} else {
			report.ValidRefs++
		}
	}

	// Find orphaned documents
	report.OrphanedDocs = v.findOrphanedDocs(refs)

	return report, nil
}

// collectReferences collects all references from markdown files
func (v *ReferenceValidator) collectReferences() ([]Reference, error) {
	refs := []Reference{}

	// Walk all markdown files
	err := filepath.Walk(v.rootDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip hidden directories and target directories
		if info.IsDir() {
			if strings.HasPrefix(info.Name(), ".") || info.Name() == "target" {
				return filepath.SkipDir
			}
			return nil
		}

		// Only process markdown files
		if filepath.Ext(path) != ".md" {
			return nil
		}

		// Extract references from file
		fileRefs, err := v.extractReferences(path)
		if err != nil {
			return nil // Skip files with errors
		}

		refs = append(refs, fileRefs...)
		return nil
	})

	return refs, err
}

// extractReferences extracts references from a markdown file
func (v *ReferenceValidator) extractReferences(filePath string) ([]Reference, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, err
	}
	defer func() { _ = file.Close() }()

	refs := []Reference{}
	scanner := bufio.NewScanner(file)
	lineNum := 0

	// Pattern to match markdown links: [text](path) or [text](path#anchor)
	linkPattern := regexp.MustCompile(`\[([^\]]+)\]\(([^)]+)\)`)

	for scanner.Scan() {
		line := scanner.Text()
		lineNum++

		// Find all markdown links
		matches := linkPattern.FindAllStringSubmatch(line, -1)
		for _, match := range matches {
			if len(match) >= 3 {
				text := match[1]
				target := match[2]

				// Skip external URLs
				if strings.HasPrefix(target, "http://") || strings.HasPrefix(target, "https://") {
					continue
				}

				// Parse target (path and anchor)
				targetPath := target
				targetAnchor := ""
				if idx := strings.Index(target, "#"); idx != -1 {
					targetPath = target[:idx]
					targetAnchor = target[idx+1:]
				}

				// Classify reference type
				refType := "markdown"
				if strings.Contains(targetPath, "specification") {
					refType = "spec"
				} else if strings.Contains(targetPath, "api") {
					refType = "api"
				} else if strings.Contains(targetPath, "phase") {
					refType = "phase"
				}

				ref := Reference{
					SourceFile:   filePath,
					SourceLine:   lineNum,
					TargetPath:   targetPath,
					TargetAnchor: targetAnchor,
					Text:         text,
					Type:         refType,
				}
				refs = append(refs, ref)
			}
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, err
	}

	return refs, nil
}

// validateReference validates a single reference
func (v *ReferenceValidator) validateReference(ref *Reference) error {
	// Resolve target path relative to source file
	targetPath := ref.TargetPath

	// If relative path, resolve from source directory
	if !filepath.IsAbs(targetPath) {
		sourceDir := filepath.Dir(ref.SourceFile)
		targetPath = filepath.Join(sourceDir, targetPath)
	}

	// Clean path
	targetPath = filepath.Clean(targetPath)

	// Check if target file exists
	if _, err := os.Stat(targetPath); os.IsNotExist(err) {
		return fmt.Errorf("target file not found: %s", targetPath)
	}

	// If anchor specified, verify it exists in target
	if ref.TargetAnchor != "" {
		if err := v.validateAnchor(targetPath, ref.TargetAnchor); err != nil {
			return fmt.Errorf("anchor not found: %s#%s", targetPath, ref.TargetAnchor)
		}
	}

	return nil
}

// validateAnchor validates that an anchor exists in target file
func (v *ReferenceValidator) validateAnchor(filePath, anchor string) error {
	file, err := os.Open(filePath)
	if err != nil {
		return err
	}
	defer func() { _ = file.Close() }()

	// Convert anchor to heading format
	// e.g., "my-section" -> "My Section" or "## My Section"
	expectedHeading := strings.ReplaceAll(anchor, "-", " ")
	expectedHeading = strings.ToLower(expectedHeading)

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Check for markdown heading
		if strings.HasPrefix(trimmed, "#") {
			// Extract heading text
			heading := strings.TrimSpace(strings.TrimLeft(trimmed, "#"))
			heading = strings.ToLower(heading)

			if heading == expectedHeading {
				return nil
			}

			// Also check if heading converts to anchor
			headingAnchor := strings.ReplaceAll(heading, " ", "-")
			if headingAnchor == anchor {
				return nil
			}
		}
	}

	return fmt.Errorf("anchor not found: %s", anchor)
}

// classifyError classifies validation error type
func (v *ReferenceValidator) classifyError(err error) string {
	errStr := err.Error()
	if strings.Contains(errStr, "not found") {
		if strings.Contains(errStr, "anchor") {
			return "section_missing"
		}
		return "file_missing"
	}
	return "invalid_format"
}

// generateFixSuggestion generates fix suggestion for broken reference
func (v *ReferenceValidator) generateFixSuggestion(ref *Reference, err error) string {
	errType := v.classifyError(err)

	switch errType {
	case "file_missing":
		return fmt.Sprintf("Create missing file '%s' or update reference in %s:%d",
			ref.TargetPath, ref.SourceFile, ref.SourceLine)

	case "section_missing":
		return fmt.Sprintf("Add section '#%s' to '%s' or update reference in %s:%d",
			ref.TargetAnchor, ref.TargetPath, ref.SourceFile, ref.SourceLine)

	default:
		return fmt.Sprintf("Fix reference in %s:%d", ref.SourceFile, ref.SourceLine)
	}
}

// findOrphanedDocs finds documents not referenced anywhere
func (v *ReferenceValidator) findOrphanedDocs(refs []Reference) []string {
	if v.docsDir == "" {
		return []string{}
	}

	// Build set of referenced files
	referenced := make(map[string]bool)
	for _, ref := range refs {
		// Resolve path
		targetPath := ref.TargetPath
		if !filepath.IsAbs(targetPath) {
			sourceDir := filepath.Dir(ref.SourceFile)
			targetPath = filepath.Join(sourceDir, targetPath)
		}
		targetPath = filepath.Clean(targetPath)
		referenced[targetPath] = true
	}

	orphaned := []string{}

	// Walk docs directory
	_ = filepath.Walk(v.docsDir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}

		// Only check markdown files
		if !info.IsDir() && filepath.Ext(path) == ".md" {
			// Skip README files (usually entry points)
			if strings.ToUpper(filepath.Base(path)) == "README.MD" {
				return nil
			}

			if !referenced[path] {
				orphaned = append(orphaned, path)
			}
		}

		return nil
	})

	return orphaned
}

// ToCompactJSON returns compact JSON representation
func (r *ReferenceReport) ToCompactJSON() map[string]interface{} {
	result := map[string]interface{}{
		"total":        r.TotalRefs,
		"valid":        r.ValidRefs,
		"broken_cnt":   len(r.BrokenRefs),
		"orphaned_cnt": len(r.OrphanedDocs),
	}

	// Include broken refs if any
	if len(r.BrokenRefs) > 0 {
		brokenList := []map[string]interface{}{}
		for _, b := range r.BrokenRefs {
			brokenList = append(brokenList, map[string]interface{}{
				"src":  fmt.Sprintf("%s:%d", b.Ref.SourceFile, b.Ref.SourceLine),
				"tgt":  b.Ref.TargetPath,
				"type": b.ErrorType,
				"fix":  b.FixSuggestion,
			})
		}
		result["broken"] = brokenList
	}

	// Include orphaned docs if any
	if len(r.OrphanedDocs) > 0 {
		result["orphaned"] = r.OrphanedDocs
	}

	return result
}
