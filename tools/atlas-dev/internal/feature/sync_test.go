package feature

import (
	"os"
	"path/filepath"
	"testing"
)

func TestSync(t *testing.T) {
	tmpDir := t.TempDir()

	// Create impl and test files
	implFile := filepath.Join(tmpDir, "impl.rs")
	implContent := `
pub fn new_function() {}
pub fn another_function() {}
`
	os.WriteFile(implFile, []byte(implContent), 0644)

	testFile := filepath.Join(tmpDir, "test.rs")
	testContent := `
#[test]
fn test_new() {}

#[test]
fn test_another() {}
`
	os.WriteFile(testFile, []byte(testContent), 0644)

	tests := []struct {
		name        string
		feature     *Feature
		dryRun      bool
		wantUpdated bool
		wantFnCount int
		wantTestCnt int
		wantParity  float64
	}{
		{
			name: "sync with changes",
			feature: &Feature{
				Name:          "test",
				ImplFile:      "impl.rs",
				TestFile:      "test.rs",
				FunctionCount: 0,
				TestCount:     0,
			},
			dryRun:      false,
			wantUpdated: true,
			wantFnCount: 2,
			wantTestCnt: 2,
			wantParity:  100.0,
		},
		{
			name: "dry run",
			feature: &Feature{
				Name:          "test",
				ImplFile:      "impl.rs",
				TestFile:      "test.rs",
				FunctionCount: 0,
				TestCount:     0,
			},
			dryRun:      true,
			wantUpdated: true,
			wantFnCount: 2,
			wantTestCnt: 2,
			wantParity:  100.0,
		},
		{
			name: "no changes",
			feature: &Feature{
				Name:          "test",
				ImplFile:      "impl.rs",
				TestFile:      "test.rs",
				FunctionCount: 2,
				TestCount:     2,
				Parity:        100.0,
			},
			dryRun:      false,
			wantUpdated: false,
			wantFnCount: 2,
			wantTestCnt: 2,
			wantParity:  100.0,
		},
		{
			name: "missing files",
			feature: &Feature{
				Name:     "test",
				ImplFile: "nonexistent.rs",
				TestFile: "also-nonexistent.rs",
			},
			dryRun:      false,
			wantUpdated: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := Sync(tt.feature, tmpDir, tt.dryRun)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if result.Updated != tt.wantUpdated {
				t.Errorf("Updated = %v, want %v", result.Updated, tt.wantUpdated)
			}

			if tt.wantFnCount > 0 && result.FunctionCount != tt.wantFnCount {
				t.Errorf("FunctionCount = %d, want %d", result.FunctionCount, tt.wantFnCount)
			}

			if tt.wantTestCnt > 0 && result.TestCount != tt.wantTestCnt {
				t.Errorf("TestCount = %d, want %d", result.TestCount, tt.wantTestCnt)
			}

			if tt.wantParity > 0 && result.Parity != tt.wantParity {
				t.Errorf("Parity = %.1f, want %.1f", result.Parity, tt.wantParity)
			}
		})
	}
}

func TestSyncAll(t *testing.T) {
	tmpDir := t.TempDir()

	features := []*Feature{
		{Name: "f1", ImplFile: "nonexistent.rs"},
		{Name: "f2", ImplFile: "also-nonexistent.rs"},
	}

	results, err := SyncAll(features, tmpDir, true)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(results) != 2 {
		t.Errorf("got %d results, want 2", len(results))
	}
}

func TestCountUpdated(t *testing.T) {
	results := []*SyncResult{
		{Updated: true},
		{Updated: false},
		{Updated: true},
	}

	count := CountUpdated(results)
	if count != 2 {
		t.Errorf("CountUpdated() = %d, want 2", count)
	}
}

func TestHasErrors(t *testing.T) {
	tests := []struct {
		name    string
		results []*SyncResult
		want    bool
	}{
		{
			name: "has errors",
			results: []*SyncResult{
				{Errors: []string{"error1"}},
				{Errors: []string{}},
			},
			want: true,
		},
		{
			name: "no errors",
			results: []*SyncResult{
				{Errors: []string{}},
				{Errors: []string{}},
			},
			want: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := HasErrors(tt.results)
			if got != tt.want {
				t.Errorf("HasErrors() = %v, want %v", got, tt.want)
			}
		})
	}
}
