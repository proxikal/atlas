package parity

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// CodeItem represents a parsed code element with location
type CodeItem struct {
	Name     string
	Type     string // "function", "struct", "enum", "trait", "impl", "test"
	FilePath string
	Line     int
	Signature string
	Public   bool
	Details  map[string]interface{} // Additional type-specific details
}

// CodeAnalysis represents the complete analysis of a codebase
type CodeAnalysis struct {
	Functions []CodeItem
	Structs   []CodeItem
	Enums     []CodeItem
	Traits    []CodeItem
	Impls     []CodeItem
	Tests     []CodeItem
	TotalFiles int
	ParseErrors []string
}

// CodeAnalyzer parses Rust source code
type CodeAnalyzer struct {
	rootPath string
	patterns *rustPatterns
}

// rustPatterns contains regex patterns for Rust syntax
type rustPatterns struct {
	pubFn      *regexp.Regexp
	privateFn  *regexp.Regexp
	pubStruct  *regexp.Regexp
	privateStruct *regexp.Regexp
	pubEnum    *regexp.Regexp
	privateEnum *regexp.Regexp
	trait      *regexp.Regexp
	impl       *regexp.Regexp
	testFn     *regexp.Regexp
	testMod    *regexp.Regexp
}

// NewCodeAnalyzer creates a new code analyzer
func NewCodeAnalyzer(rootPath string) *CodeAnalyzer {
	return &CodeAnalyzer{
		rootPath: rootPath,
		patterns: compileRustPatterns(),
	}
}

// compileRustPatterns compiles all regex patterns
func compileRustPatterns() *rustPatterns {
	return &rustPatterns{
		// pub fn name<T>(args) -> ReturnType
		pubFn: regexp.MustCompile(`^\s*pub\s+fn\s+(\w+)(<[^>]+>)?\s*\(([^)]*)\)(\s*->\s*([^{;]+))?`),

		// fn name<T>(args) -> ReturnType (private)
		privateFn: regexp.MustCompile(`^\s*fn\s+(\w+)(<[^>]+>)?\s*\(([^)]*)\)(\s*->\s*([^{;]+))?`),

		// pub struct Name<T> { ... }
		pubStruct: regexp.MustCompile(`^\s*pub\s+struct\s+(\w+)(<[^>]+>)?`),

		// struct Name<T> { ... }
		privateStruct: regexp.MustCompile(`^\s*struct\s+(\w+)(<[^>]+>)?`),

		// pub enum Name<T> { ... }
		pubEnum: regexp.MustCompile(`^\s*pub\s+enum\s+(\w+)(<[^>]+>)?`),

		// enum Name<T> { ... }
		privateEnum: regexp.MustCompile(`^\s*enum\s+(\w+)(<[^>]+>)?`),

		// pub trait Name<T> { ... }
		trait: regexp.MustCompile(`^\s*pub\s+trait\s+(\w+)(<[^>]+>)?`),

		// impl<T> Name<T> for Type { ... } or impl Name { ... }
		impl: regexp.MustCompile(`^\s*impl(<[^>]+>)?\s+(\w+)(<[^>]+>)?(\s+for\s+(\w+))?`),

		// #[test] fn test_name() { ... }
		testFn: regexp.MustCompile(`^\s*#\[test\]`),

		// #[cfg(test)] mod tests { ... }
		testMod: regexp.MustCompile(`^\s*#\[cfg\(test\)\]\s*mod\s+(\w+)`),
	}
}

// AnalyzeCodebase analyzes all Rust files in the codebase
func (a *CodeAnalyzer) AnalyzeCodebase() (*CodeAnalysis, error) {
	analysis := &CodeAnalysis{
		Functions: []CodeItem{},
		Structs:   []CodeItem{},
		Enums:     []CodeItem{},
		Traits:    []CodeItem{},
		Impls:     []CodeItem{},
		Tests:     []CodeItem{},
		ParseErrors: []string{},
	}

	// Walk all Rust files
	err := filepath.Walk(a.rootPath, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip target directory and hidden directories
		if info.IsDir() {
			if info.Name() == "target" || strings.HasPrefix(info.Name(), ".") {
				return filepath.SkipDir
			}
			return nil
		}

		// Only process .rs files
		if filepath.Ext(path) != ".rs" {
			return nil
		}

		analysis.TotalFiles++

		// Parse the file
		if err := a.analyzeFile(path, analysis); err != nil {
			analysis.ParseErrors = append(analysis.ParseErrors,
				fmt.Sprintf("%s: %v", path, err))
		}

		return nil
	})

	if err != nil {
		return nil, fmt.Errorf("failed to walk codebase: %w", err)
	}

	return analysis, nil
}

// analyzeFile parses a single Rust file
func (a *CodeAnalyzer) analyzeFile(filePath string, analysis *CodeAnalysis) error {
	file, err := os.Open(filePath)
	if err != nil {
		return err
	}
	defer func() { _ = file.Close() }()

	scanner := bufio.NewScanner(file)
	lineNum := 0
	inTestMod := false
	inComment := false
	nextIsTest := false

	for scanner.Scan() {
		lineNum++
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Handle multi-line comments
		if strings.Contains(trimmed, "/*") {
			inComment = true
		}
		if strings.Contains(trimmed, "*/") {
			inComment = false
			continue
		}
		if inComment || strings.HasPrefix(trimmed, "//") {
			continue
		}

		// Check for #[test] attribute
		if a.patterns.testFn.MatchString(trimmed) {
			nextIsTest = true
			continue
		}

		// Check for test module
		if matches := a.patterns.testMod.FindStringSubmatch(trimmed); matches != nil {
			inTestMod = true
			continue
		}

		// Parse public functions
		if matches := a.patterns.pubFn.FindStringSubmatch(trimmed); matches != nil {
			item := CodeItem{
				Name:      matches[1],
				Type:      "function",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    true,
				Details: map[string]interface{}{
					"generics": matches[2],
					"params":   matches[3],
					"returns":  matches[5],
				},
			}

			if nextIsTest || inTestMod {
				item.Type = "test"
				analysis.Tests = append(analysis.Tests, item)
				nextIsTest = false
			} else {
				analysis.Functions = append(analysis.Functions, item)
			}
			continue
		}

		// Parse private functions (might be tests)
		if matches := a.patterns.privateFn.FindStringSubmatch(trimmed); matches != nil {
			item := CodeItem{
				Name:      matches[1],
				Type:      "function",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    false,
				Details: map[string]interface{}{
					"generics": matches[2],
					"params":   matches[3],
					"returns":  matches[5],
				},
			}

			if nextIsTest || inTestMod || strings.HasPrefix(matches[1], "test_") {
				item.Type = "test"
				analysis.Tests = append(analysis.Tests, item)
				nextIsTest = false
			} else {
				analysis.Functions = append(analysis.Functions, item)
			}
			continue
		}

		// Parse public structs
		if matches := a.patterns.pubStruct.FindStringSubmatch(trimmed); matches != nil {
			analysis.Structs = append(analysis.Structs, CodeItem{
				Name:      matches[1],
				Type:      "struct",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    true,
				Details: map[string]interface{}{
					"generics": matches[2],
				},
			})
			continue
		}

		// Parse private structs
		if matches := a.patterns.privateStruct.FindStringSubmatch(trimmed); matches != nil {
			analysis.Structs = append(analysis.Structs, CodeItem{
				Name:      matches[1],
				Type:      "struct",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    false,
				Details: map[string]interface{}{
					"generics": matches[2],
				},
			})
			continue
		}

		// Parse public enums
		if matches := a.patterns.pubEnum.FindStringSubmatch(trimmed); matches != nil {
			analysis.Enums = append(analysis.Enums, CodeItem{
				Name:      matches[1],
				Type:      "enum",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    true,
				Details: map[string]interface{}{
					"generics": matches[2],
				},
			})
			continue
		}

		// Parse private enums
		if matches := a.patterns.privateEnum.FindStringSubmatch(trimmed); matches != nil {
			analysis.Enums = append(analysis.Enums, CodeItem{
				Name:      matches[1],
				Type:      "enum",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    false,
				Details: map[string]interface{}{
					"generics": matches[2],
				},
			})
			continue
		}

		// Parse traits
		if matches := a.patterns.trait.FindStringSubmatch(trimmed); matches != nil {
			analysis.Traits = append(analysis.Traits, CodeItem{
				Name:      matches[1],
				Type:      "trait",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    true,
				Details: map[string]interface{}{
					"generics": matches[2],
				},
			})
			continue
		}

		// Parse impl blocks
		if matches := a.patterns.impl.FindStringSubmatch(trimmed); matches != nil {
			implName := matches[2]
			forType := matches[5]
			if forType != "" {
				implName = fmt.Sprintf("%s for %s", implName, forType)
			}

			analysis.Impls = append(analysis.Impls, CodeItem{
				Name:      implName,
				Type:      "impl",
				FilePath:  filePath,
				Line:      lineNum,
				Signature: trimmed,
				Public:    false, // impl blocks don't have pub keyword
				Details: map[string]interface{}{
					"generics": matches[1],
					"trait":    matches[2],
					"for_type": forType,
				},
			})
			continue
		}
	}

	if err := scanner.Err(); err != nil {
		return err
	}

	return nil
}

// ToCompactJSON returns a compact JSON representation
func (a *CodeAnalysis) ToCompactJSON() map[string]interface{} {
	return map[string]interface{}{
		"fn_cnt":     len(a.Functions),
		"struct_cnt": len(a.Structs),
		"enum_cnt":   len(a.Enums),
		"trait_cnt":  len(a.Traits),
		"impl_cnt":   len(a.Impls),
		"test_cnt":   len(a.Tests),
		"file_cnt":   a.TotalFiles,
		"err_cnt":    len(a.ParseErrors),
	}
}
