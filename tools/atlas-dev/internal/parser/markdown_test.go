package parser

import (
	"strings"
	"testing"
)

func TestExtractMetadata(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  map[string]string
	}{
		{
			name:  "single metadata",
			input: "Priority: HIGH",
			want:  map[string]string{"priority": "HIGH"},
		},
		{
			name:  "multiple metadata",
			input: "Priority: HIGH\nEstimate: 4 hours\n",
			want: map[string]string{
				"priority": "HIGH",
				"estimate": "4 hours",
			},
		},
		{
			name:  "with spaces",
			input: "Depends On: Phase 1, Phase 2",
			want:  map[string]string{"depends on": "Phase 1, Phase 2"},
		},
		{
			name:  "empty input",
			input: "",
			want:  map[string]string{},
		},
		{
			name:  "no metadata",
			input: "Just some text\nNo metadata here",
			want:  map[string]string{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			metadata, err := ExtractMetadata(strings.NewReader(tt.input))
			if err != nil {
				t.Fatalf("ExtractMetadata() error = %v", err)
			}

			if len(metadata) != len(tt.want) {
				t.Errorf("got %d items, want %d", len(metadata), len(tt.want))
			}

			for k, v := range tt.want {
				if metadata[k] != v {
					t.Errorf("key %q: got %q, want %q", k, metadata[k], v)
				}
			}
		})
	}
}

func TestParseNumberedList(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  []NumberedItem
	}{
		{
			name:  "simple list",
			input: "1. First item\n2. Second item\n3. Third item",
			want: []NumberedItem{
				{Number: 1, Text: "First item"},
				{Number: 2, Text: "Second item"},
				{Number: 3, Text: "Third item"},
			},
		},
		{
			name:  "with extra spaces",
			input: "1.  Item with spaces  \n2.   Another item   ",
			want: []NumberedItem{
				{Number: 1, Text: "Item with spaces"},
				{Number: 2, Text: "Another item"},
			},
		},
		{
			name:  "empty input",
			input: "",
			want:  []NumberedItem{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			items, err := ParseNumberedList(strings.NewReader(tt.input))
			if err != nil {
				t.Fatalf("ParseNumberedList() error = %v", err)
			}

			if len(items) != len(tt.want) {
				t.Errorf("got %d items, want %d", len(items), len(tt.want))
			}

			for i, item := range items {
				if i >= len(tt.want) {
					break
				}
				if item.Number != tt.want[i].Number {
					t.Errorf("item %d: got number %d, want %d", i, item.Number, tt.want[i].Number)
				}
				if item.Text != tt.want[i].Text {
					t.Errorf("item %d: got text %q, want %q", i, item.Text, tt.want[i].Text)
				}
			}
		})
	}
}

func TestParseCheckboxList(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  []CheckboxItem
	}{
		{
			name:  "mixed checkboxes",
			input: "- [ ] Unchecked item\n- [x] Checked item\n- [X] Also checked",
			want: []CheckboxItem{
				{Checked: false, Text: "Unchecked item"},
				{Checked: true, Text: "Checked item"},
				{Checked: true, Text: "Also checked"},
			},
		},
		{
			name:  "all unchecked",
			input: "- [ ] First\n- [ ] Second",
			want: []CheckboxItem{
				{Checked: false, Text: "First"},
				{Checked: false, Text: "Second"},
			},
		},
		{
			name:  "empty input",
			input: "",
			want:  []CheckboxItem{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			items, err := ParseCheckboxList(strings.NewReader(tt.input))
			if err != nil {
				t.Fatalf("ParseCheckboxList() error = %v", err)
			}

			if len(items) != len(tt.want) {
				t.Errorf("got %d items, want %d", len(items), len(tt.want))
			}

			for i, item := range items {
				if i >= len(tt.want) {
					break
				}
				if item.Checked != tt.want[i].Checked {
					t.Errorf("item %d: got checked %v, want %v", i, item.Checked, tt.want[i].Checked)
				}
				if item.Text != tt.want[i].Text {
					t.Errorf("item %d: got text %q, want %q", i, item.Text, tt.want[i].Text)
				}
			}
		})
	}
}

func TestExtractCodeBlocks(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  []CodeBlock
	}{
		{
			name:  "single code block",
			input: "```go\nfunc main() {\n}\n```",
			want: []CodeBlock{
				{Language: "go", Content: "func main() {\n}"},
			},
		},
		{
			name:  "multiple code blocks",
			input: "```bash\necho test\n```\n\nSome text\n\n```python\nprint('hello')\n```",
			want: []CodeBlock{
				{Language: "bash", Content: "echo test"},
				{Language: "python", Content: "print('hello')"},
			},
		},
		{
			name:  "no language specified",
			input: "```\ncode here\n```",
			want: []CodeBlock{
				{Language: "", Content: "code here"},
			},
		},
		{
			name:  "empty input",
			input: "",
			want:  []CodeBlock{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			blocks, err := ExtractCodeBlocks(strings.NewReader(tt.input))
			if err != nil {
				t.Fatalf("ExtractCodeBlocks() error = %v", err)
			}

			if len(blocks) != len(tt.want) {
				t.Errorf("got %d blocks, want %d", len(blocks), len(tt.want))
			}

			for i, block := range blocks {
				if i >= len(tt.want) {
					break
				}
				if block.Language != tt.want[i].Language {
					t.Errorf("block %d: got language %q, want %q", i, block.Language, tt.want[i].Language)
				}
				if block.Content != tt.want[i].Content {
					t.Errorf("block %d: got content %q, want %q", i, block.Content, tt.want[i].Content)
				}
			}
		})
	}
}

func TestSplitByHeadings(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  []Section
	}{
		{
			name:  "simple sections",
			input: "# Title\nContent here\n## Subtitle\nMore content",
			want: []Section{
				{Level: 1, Heading: "Title", Content: "Content here"},
				{Level: 2, Heading: "Subtitle", Content: "More content"},
			},
		},
		{
			name:  "nested headings",
			input: "## Section 1\nText\n### Subsection\nSubtext\n## Section 2\nMore text",
			want: []Section{
				{Level: 2, Heading: "Section 1", Content: "Text"},
				{Level: 3, Heading: "Subsection", Content: "Subtext"},
				{Level: 2, Heading: "Section 2", Content: "More text"},
			},
		},
		{
			name:  "empty input",
			input: "",
			want:  []Section{},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sections, err := SplitByHeadings(strings.NewReader(tt.input))
			if err != nil {
				t.Fatalf("SplitByHeadings() error = %v", err)
			}

			if len(sections) != len(tt.want) {
				t.Errorf("got %d sections, want %d", len(sections), len(tt.want))
			}

			for i, section := range sections {
				if i >= len(tt.want) {
					break
				}
				if section.Level != tt.want[i].Level {
					t.Errorf("section %d: got level %d, want %d", i, section.Level, tt.want[i].Level)
				}
				if section.Heading != tt.want[i].Heading {
					t.Errorf("section %d: got heading %q, want %q", i, section.Heading, tt.want[i].Heading)
				}
				if section.Content != tt.want[i].Content {
					t.Errorf("section %d: got content %q, want %q", i, section.Content, tt.want[i].Content)
				}
			}
		})
	}
}

func TestFindSection(t *testing.T) {
	sections := []Section{
		{Level: 1, Heading: "Introduction", Content: "Intro text"},
		{Level: 2, Heading: "Getting Started", Content: "Start text"},
		{Level: 2, Heading: "Advanced", Content: "Advanced text"},
	}

	tests := []struct {
		name    string
		heading string
		wantNil bool
	}{
		{
			name:    "exact match",
			heading: "Introduction",
			wantNil: false,
		},
		{
			name:    "case insensitive",
			heading: "GETTING STARTED",
			wantNil: false,
		},
		{
			name:    "not found",
			heading: "Missing",
			wantNil: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			section := FindSection(sections, tt.heading)
			if tt.wantNil && section != nil {
				t.Errorf("expected nil, got section")
			}
			if !tt.wantNil && section == nil {
				t.Errorf("expected section, got nil")
			}
		})
	}
}
