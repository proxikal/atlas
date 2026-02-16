package feature

import (
	"os"
	"path/filepath"
	"testing"
)

func TestValidate(t *testing.T) {
	tmpDir := t.TempDir()

	// Create test files
	implFile := filepath.Join(tmpDir, "impl.rs")
	implContent := `
pub fn function_one() {}
pub fn function_two() {}
pub fn function_three() {}
`
	os.WriteFile(implFile, []byte(implContent), 0644)

	testFile := filepath.Join(tmpDir, "test.rs")
	testContent := `
#[test]
fn test_one() {}

#[test]
fn test_two() {}
`
	os.WriteFile(testFile, []byte(testContent), 0644)

	tests := []struct {
		name           string
		feature        *Feature
		wantValid      bool
		wantFnCount    int
		wantTestCount  int
		wantImplExists bool
		wantTestExists bool
	}{
		{
			name: "valid feature with matching counts",
			feature: &Feature{
				Name:          "test",
				ImplFile:      "impl.rs",
				TestFile:      "test.rs",
				FunctionCount: 3,
				TestCount:     2,
				Status:        "Implemented",
			},
			wantValid:      true,
			wantFnCount:    3,
			wantTestCount:  2,
			wantImplExists: true,
			wantTestExists: true,
		},
		{
			name: "function count mismatch",
			feature: &Feature{
				Name:          "test",
				ImplFile:      "impl.rs",
				TestFile:      "test.rs",
				FunctionCount: 5, // Wrong count
				TestCount:     2,
				Status:        "Implemented",
			},
			wantValid:      false,
			wantFnCount:    3,
			wantTestCount:  2,
			wantImplExists: true,
			wantTestExists: true,
		},
		{
			name: "missing implementation file",
			feature: &Feature{
				Name:     "test",
				ImplFile: "nonexistent.rs",
				TestFile: "test.rs",
				Status:   "Implemented",
			},
			wantValid:      false,
			wantImplExists: false,
			wantTestExists: true,
			wantTestCount:  2, // The test file exists and has 2 tests
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			report, err := Validate(tt.feature, tmpDir)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if report.Valid != tt.wantValid {
				t.Errorf("Valid = %v, want %v (errors: %v)", report.Valid, tt.wantValid, report.Errors)
			}

			if report.ImplFileExists != tt.wantImplExists {
				t.Errorf("ImplFileExists = %v, want %v", report.ImplFileExists, tt.wantImplExists)
			}

			if report.TestFileExists != tt.wantTestExists {
				t.Errorf("TestFileExists = %v, want %v", report.TestFileExists, tt.wantTestExists)
			}

			if tt.wantImplExists && report.ActualFunctions != tt.wantFnCount {
				t.Errorf("ActualFunctions = %d, want %d", report.ActualFunctions, tt.wantFnCount)
			}

			if tt.wantTestExists && report.ActualTests != tt.wantTestCount {
				t.Errorf("ActualTests = %d, want %d", report.ActualTests, tt.wantTestCount)
			}
		})
	}
}

func TestCountRustFunctions(t *testing.T) {
	tests := []struct {
		name    string
		content string
		want    int
	}{
		{
			name: "multiple functions",
			content: `
pub fn function_one() {}
pub fn function_two() {}
fn private_function() {}
pub fn function_three() {}
`,
			want: 3,
		},
		{
			name:    "no functions",
			content: "struct Foo {}",
			want:    0,
		},
		{
			name: "with attributes",
			content: `
#[inline]
pub fn inline_function() {}
`,
			want: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpFile := filepath.Join(t.TempDir(), "test.rs")
			os.WriteFile(tmpFile, []byte(tt.content), 0644)

			count, err := countRustFunctions(tmpFile)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if count != tt.want {
				t.Errorf("countRustFunctions() = %d, want %d", count, tt.want)
			}
		})
	}
}

func TestCountRustTests(t *testing.T) {
	tests := []struct {
		name    string
		content string
		want    int
	}{
		{
			name: "multiple tests",
			content: `
#[test]
fn test_one() {}

#[test]
fn test_two() {}

#[tokio::test]
async fn async_test() {}
`,
			want: 3,
		},
		{
			name:    "no tests",
			content: "fn regular_function() {}",
			want:    0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tmpFile := filepath.Join(t.TempDir(), "test.rs")
			os.WriteFile(tmpFile, []byte(tt.content), 0644)

			count, err := countRustTests(tmpFile)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if count != tt.want {
				t.Errorf("countRustTests() = %d, want %d", count, tt.want)
			}
		})
	}
}

func TestValidateBatch(t *testing.T) {
	tmpDir := t.TempDir()

	features := []*Feature{
		{Name: "feature1", Status: "Planned"},
		{Name: "feature2", Status: "Planned"},
	}

	reports, err := ValidateBatch(features, tmpDir)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(reports) != 2 {
		t.Errorf("got %d reports, want 2", len(reports))
	}
}

func TestCountErrors(t *testing.T) {
	reports := []*ValidationReport{
		{Errors: []string{"error1", "error2"}},
		{Errors: []string{"error3"}},
		{Errors: []string{}},
	}

	count := CountErrors(reports)
	if count != 3 {
		t.Errorf("CountErrors() = %d, want 3", count)
	}
}

func TestFilterInvalid(t *testing.T) {
	reports := []*ValidationReport{
		{FeatureName: "f1", Valid: true},
		{FeatureName: "f2", Valid: false},
		{FeatureName: "f3", Valid: false},
	}

	invalid := FilterInvalid(reports)
	if len(invalid) != 2 {
		t.Errorf("FilterInvalid() returned %d reports, want 2", len(invalid))
	}
}
