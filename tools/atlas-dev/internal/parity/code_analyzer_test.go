package parity

import (
	"os"
	"path/filepath"
	"testing"
)

func TestCodeAnalyzer_AnalyzeCodebase(t *testing.T) {
	// Create temp directory with test Rust files
	tempDir := t.TempDir()

	// Create a test Rust file
	testFile := filepath.Join(tempDir, "test.rs")
	content := `
pub fn public_function(x: i32) -> i32 {
    x + 1
}

fn private_function() {
    println!("private");
}

pub struct PublicStruct {
    field: i32,
}

struct PrivateStruct {
    data: String,
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub trait Display {
    fn display(&self);
}

impl Display for PublicStruct {
    fn display(&self) {
        println!("{}", self.field);
    }
}

#[test]
fn test_addition() {
    assert_eq!(public_function(1), 2);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_private() {
        assert!(true);
    }
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	// Analyze codebase
	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatalf("AnalyzeCodebase failed: %v", err)
	}

	// Verify results
	if analysis.TotalFiles != 1 {
		t.Errorf("Expected 1 file, got %d", analysis.TotalFiles)
	}

	// Check functions (should find public_function and private_function)
	if len(analysis.Functions) < 2 {
		t.Errorf("Expected at least 2 functions, got %d", len(analysis.Functions))
	}

	// Check structs (should find PublicStruct and PrivateStruct)
	if len(analysis.Structs) < 2 {
		t.Errorf("Expected at least 2 structs, got %d", len(analysis.Structs))
	}

	// Check enums (should find Color)
	if len(analysis.Enums) < 1 {
		t.Errorf("Expected at least 1 enum, got %d", len(analysis.Enums))
	}

	// Check traits (should find Display)
	if len(analysis.Traits) < 1 {
		t.Errorf("Expected at least 1 trait, got %d", len(analysis.Traits))
	}

	// Check impl blocks (should find impl Display for PublicStruct)
	if len(analysis.Impls) < 1 {
		t.Errorf("Expected at least 1 impl block, got %d", len(analysis.Impls))
	}

	// Check tests (should find both test functions)
	if len(analysis.Tests) < 2 {
		t.Errorf("Expected at least 2 tests, got %d", len(analysis.Tests))
	}
}

func TestCodeAnalyzer_PublicVisibility(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "visibility.rs")
	content := `
pub fn public_fn() {}
fn private_fn() {}
pub struct PublicStruct {}
struct PrivateStruct {}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Check public function
	publicFnCount := 0
	privateFnCount := 0
	for _, fn := range analysis.Functions {
		if fn.Public {
			publicFnCount++
		} else {
			privateFnCount++
		}
	}

	if publicFnCount < 1 {
		t.Error("Expected at least 1 public function")
	}
	if privateFnCount < 1 {
		t.Error("Expected at least 1 private function")
	}

	// Check public struct
	publicStructCount := 0
	privateStructCount := 0
	for _, s := range analysis.Structs {
		if s.Public {
			publicStructCount++
		} else {
			privateStructCount++
		}
	}

	if publicStructCount < 1 {
		t.Error("Expected at least 1 public struct")
	}
	if privateStructCount < 1 {
		t.Error("Expected at least 1 private struct")
	}
}

func TestCodeAnalyzer_Generics(t *testing.T) {
	tempDir := t.TempDir()

	testFile := filepath.Join(tempDir, "generics.rs")
	content := `
pub fn generic_fn<T>(value: T) -> T {
    value
}

pub struct GenericStruct<T, U> {
    field1: T,
    field2: U,
}

pub enum Option<T> {
    Some(T),
    None,
}
`

	if err := os.WriteFile(testFile, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Verify generic function was parsed
	found := false
	for _, fn := range analysis.Functions {
		if fn.Name == "generic_fn" {
			found = true
			if generics, ok := fn.Details["generics"].(string); ok && generics != "" {
				t.Logf("Found generics: %s", generics)
			}
		}
	}
	if !found {
		t.Error("Expected to find generic_fn")
	}

	// Verify generic struct was parsed
	found = false
	for _, s := range analysis.Structs {
		if s.Name == "GenericStruct" {
			found = true
			break
		}
	}
	if !found {
		t.Error("Expected to find GenericStruct")
	}
}

func TestCodeAnalyzer_SkipTargetDir(t *testing.T) {
	tempDir := t.TempDir()

	// Create a target directory (should be skipped)
	targetDir := filepath.Join(tempDir, "target")
	if err := os.MkdirAll(targetDir, 0755); err != nil {
		t.Fatal(err)
	}

	targetFile := filepath.Join(targetDir, "ignored.rs")
	if err := os.WriteFile(targetFile, []byte("pub fn ignored() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	// Create a normal file
	normalFile := filepath.Join(tempDir, "normal.rs")
	if err := os.WriteFile(normalFile, []byte("pub fn normal() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	analyzer := NewCodeAnalyzer(tempDir)
	analysis, err := analyzer.AnalyzeCodebase()
	if err != nil {
		t.Fatal(err)
	}

	// Should only find 1 file (target dir should be skipped)
	if analysis.TotalFiles != 1 {
		t.Errorf("Expected 1 file (target dir should be skipped), got %d", analysis.TotalFiles)
	}
}

func TestCodeAnalyzer_ToCompactJSON(t *testing.T) {
	analysis := &CodeAnalysis{
		Functions:  make([]CodeItem, 5),
		Structs:    make([]CodeItem, 3),
		Enums:      make([]CodeItem, 2),
		Traits:     make([]CodeItem, 1),
		Tests:      make([]CodeItem, 10),
		TotalFiles: 7,
	}

	result := analysis.ToCompactJSON()

	tests := []struct {
		key      string
		expected int
	}{
		{"fn_cnt", 5},
		{"struct_cnt", 3},
		{"enum_cnt", 2},
		{"trait_cnt", 1},
		{"test_cnt", 10},
		{"file_cnt", 7},
	}

	for _, tt := range tests {
		if val, ok := result[tt.key].(int); !ok || val != tt.expected {
			t.Errorf("Expected %s=%d, got %v", tt.key, tt.expected, result[tt.key])
		}
	}
}
