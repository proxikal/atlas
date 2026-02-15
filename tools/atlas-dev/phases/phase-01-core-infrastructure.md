# Phase 01: Core Infrastructure

**Objective:** Set up Go project structure, CLI framework, configuration system, and testing infrastructure.

**Priority:** CRITICAL (blocks all other phases)

---

## Deliverables

1. ✅ Go module initialized
2. ✅ Cobra CLI framework integrated
3. ✅ Config system (TOML-based)
4. ✅ Error handling framework
5. ✅ Logging system
6. ✅ Test infrastructure
7. ✅ `atlas-dev version` command works

---

## File Structure

```
tools/atlas-dev/
├── go.mod                              # Go module
├── go.sum                              # Dependencies
├── Makefile                            # Build system
├── cmd/
│   └── atlas-dev/
│       └── main.go                     # CLI entry point
├── internal/
│   ├── config/
│   │   ├── config.go                  # Config struct + loader
│   │   └── defaults.go                # Default config
│   ├── errors/
│   │   ├── errors.go                  # Error types
│   │   └── codes.go                   # Exit codes
│   ├── output/
│   │   ├── json.go                    # JSON output formatter
│   │   ├── human.go                   # Human-readable output
│   │   └── writer.go                  # Output writer
│   └── version/
│       └── version.go                  # Version info
├── testdata/                           # Test fixtures
│   ├── config/
│   │   └── test-config.toml
│   └── empty/
└── README.md
```

---

## Implementation Steps

### Step 1: Initialize Go Module

**Already done!** (`go.mod` exists)

Verify:
```bash
cd tools/atlas-dev
go mod tidy
```

---

### Step 2: Update Dependencies

Add required packages:
```bash
go get github.com/spf13/cobra@latest
go get github.com/spf13/viper@latest  # Config management
go get github.com/BurntSushi/toml@latest
```

Update `go.mod`:
```go
module github.com/atlas-lang/atlas-dev

go 1.22

require (
    github.com/spf13/cobra v1.8.0
    github.com/spf13/viper v1.18.2
    github.com/BurntSushi/toml v1.3.2
)
```

---

### Step 3: Implement Config System

**File:** `internal/config/config.go`

```go
package config

import (
    "os"
    "path/filepath"

    "github.com/BurntSushi/toml"
)

// Config represents the atlas-dev configuration
type Config struct {
    Output  OutputConfig  `toml:"output"`
    Cache   CacheConfig   `toml:"cache"`
    Git     GitConfig     `toml:"git"`
    AI      AIConfig      `toml:"ai"`
    Paths   PathsConfig   `toml:"paths"`
}

type OutputConfig struct {
    Format     string `toml:"format"`      // "json" | "human"
    Compact    bool   `toml:"compact"`     // Minified JSON
    Color      bool   `toml:"color"`       // ANSI colors
    Emoji      bool   `toml:"emoji"`       // Emoji in output
}

type CacheConfig struct {
    Enabled    bool   `toml:"enabled"`
    TTLHours   int    `toml:"ttl_hours"`
    MaxSizeMB  int    `toml:"max_size_mb"`
}

type GitConfig struct {
    AutoCommit      bool   `toml:"auto_commit"`
    CommitTemplate  string `toml:"commit_template"`
}

type AIConfig struct {
    Optimize    bool `toml:"optimize"`    // AI-optimized output
    Abbreviate  bool `toml:"abbreviate"`  // Short field names
    OmitEmpty   bool `toml:"omit_empty"`  // Skip null/empty
}

type PathsConfig struct {
    AtlasRoot string `toml:"atlas_root"`
}

// Load loads config from file, falling back to defaults
func Load(path string) (*Config, error) {
    cfg := Defaults()

    if path == "" {
        path = DefaultConfigPath()
    }

    // If config file doesn't exist, return defaults
    if _, err := os.Stat(path); os.IsNotExist(err) {
        return cfg, nil
    }

    // Load and merge with defaults
    if _, err := toml.DecodeFile(path, cfg); err != nil {
        return nil, err
    }

    return cfg, nil
}

// DefaultConfigPath returns the default config file path
func DefaultConfigPath() string {
    home, _ := os.UserHomeDir()
    return filepath.Join(home, ".config", "atlas-atlas-dev", "config.toml")
}
```

**File:** `internal/config/defaults.go`

```go
package config

// Defaults returns AI-optimized default configuration
func Defaults() *Config {
    return &Config{
        Output: OutputConfig{
            Format:  "json",   // JSON by default
            Compact: true,     // Minified
            Color:   false,    // No ANSI colors
            Emoji:   false,    // No emoji
        },
        Cache: CacheConfig{
            Enabled:   true,
            TTLHours:  24,
            MaxSizeMB: 100,
        },
        Git: GitConfig{
            AutoCommit:     false,
            CommitTemplate: "Mark {phase} complete ({progress})",
        },
        AI: AIConfig{
            Optimize:   true,
            Abbreviate: true,
            OmitEmpty:  true,
        },
        Paths: PathsConfig{
            AtlasRoot: detectAtlasRoot(),
        },
    }
}

func detectAtlasRoot() string {
    // Try to find Atlas root from current directory
    cwd, _ := os.Getwd()

    // Walk up directory tree looking for STATUS.md
    dir := cwd
    for {
        if _, err := os.Stat(filepath.Join(dir, "STATUS.md")); err == nil {
            return dir
        }

        parent := filepath.Dir(dir)
        if parent == dir {
            break  // Reached root
        }
        dir = parent
    }

    return cwd
}
```

---

### Step 4: Implement Error Handling

**File:** `internal/errors/errors.go`

```go
package errors

import "fmt"

// AppError represents an application error with exit code
type AppError struct {
    Code    int
    Message string
    Details map[string]interface{}
}

func (e *AppError) Error() string {
    return e.Message
}

// New creates a new AppError
func New(code int, message string) *AppError {
    return &AppError{
        Code:    code,
        Message: message,
        Details: make(map[string]interface{}),
    }
}

// WithDetails adds details to the error
func (e *AppError) WithDetails(key string, value interface{}) *AppError {
    e.Details[key] = value
    return e
}

// Common errors
var (
    ErrInvalidArgs   = New(ExitInvalidArgs, "Invalid arguments")
    ErrFileNotFound  = New(ExitFileNotFound, "File not found")
    ErrValidation    = New(ExitValidation, "Validation failed")
    ErrGit           = New(ExitGit, "Git operation failed")
    ErrCache         = New(ExitCache, "Cache error")
    ErrPermission    = New(ExitPermission, "Permission denied")
)
```

**File:** `internal/errors/codes.go`

```go
package errors

// Exit codes
const (
    ExitSuccess      = 0
    ExitInvalidArgs  = 1
    ExitFileNotFound = 2
    ExitValidation   = 3
    ExitGit          = 4
    ExitCache        = 5
    ExitPermission   = 6
)
```

---

### Step 5: Implement Output System

**File:** `internal/output/writer.go`

```go
package output

import (
    "encoding/json"
    "fmt"
    "io"
    "os"

    "github.com/atlas-lang/atlas-dev/internal/config"
)

// Writer handles output formatting
type Writer struct {
    cfg    *config.Config
    writer io.Writer
}

// New creates a new output writer
func New(cfg *config.Config) *Writer {
    return &Writer{
        cfg:    cfg,
        writer: os.Stdout,
    }
}

// Success writes a successful result
func (w *Writer) Success(data interface{}) error {
    if w.cfg.Output.Format == "human" {
        return w.writeHuman(data)
    }
    return w.writeJSON(data, true)
}

// Error writes an error result
func (w *Writer) Error(err error, details map[string]interface{}) error {
    result := map[string]interface{}{
        "ok":  false,
        "err": err.Error(),
    }

    if details != nil && len(details) > 0 {
        result["details"] = details
    }

    if w.cfg.Output.Format == "human" {
        return w.writeHumanError(err, details)
    }
    return w.writeJSON(result, false)
}

func (w *Writer) writeJSON(data interface{}, success bool) error {
    // Add ok field if not present
    if m, ok := data.(map[string]interface{}); ok {
        if _, hasOk := m["ok"]; !hasOk {
            m["ok"] = success
        }
    }

    var output []byte
    var err error

    if w.cfg.Output.Compact {
        output, err = json.Marshal(data)
    } else {
        output, err = json.MarshalIndent(data, "", "  ")
    }

    if err != nil {
        return err
    }

    fmt.Fprintln(w.writer, string(output))
    return nil
}

func (w *Writer) writeHuman(data interface{}) error {
    // TODO: Implement human-readable formatting
    // For now, fall back to pretty JSON
    output, err := json.MarshalIndent(data, "", "  ")
    if err != nil {
        return err
    }
    fmt.Fprintln(w.writer, string(output))
    return nil
}

func (w *Writer) writeHumanError(err error, details map[string]interface{}) error {
    fmt.Fprintf(w.writer, "❌ Error: %s\n", err.Error())
    if details != nil && len(details) > 0 {
        fmt.Fprintln(w.writer, "\nDetails:")
        for k, v := range details {
            fmt.Fprintf(w.writer, "  %s: %v\n", k, v)
        }
    }
    return nil
}
```

---

### Step 6: Implement Version Info

**File:** `internal/version/version.go`

```go
package version

import "fmt"

var (
    Version   = "1.0.0"
    GitCommit = "dev"
    BuildDate = "unknown"
)

// Info returns version information
func Info() map[string]string {
    return map[string]string{
        "version":    Version,
        "git_commit": GitCommit,
        "build_date": BuildDate,
    }
}

// String returns version as string
func String() string {
    return fmt.Sprintf("atlas-dev v%s (%s)", Version, GitCommit)
}
```

---

### Step 7: Update Main CLI

**File:** `cmd/atlas-dev/main.go` (replace existing)

```go
package main

import (
    "fmt"
    "os"

    "github.com/spf13/cobra"

    "github.com/atlas-lang/atlas-dev/internal/config"
    "github.com/atlas-lang/atlas-dev/internal/errors"
    "github.com/atlas-lang/atlas-dev/internal/output"
    "github.com/atlas-lang/atlas-dev/internal/version"
)

var (
    cfg        *config.Config
    out        *output.Writer
    configFile string
)

func main() {
    rootCmd := &cobra.Command{
        Use:           "atlas-dev",
        Short:         "Atlas phase completion tracking automation",
        Long:          `AI-optimized CLI tool for Atlas development workflow automation.`,
        Version:       version.String(),
        SilenceUsage:  true,
        SilenceErrors: true,
        PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
            // Load config
            var err error
            cfg, err = config.Load(configFile)
            if err != nil {
                return fmt.Errorf("failed to load config: %w", err)
            }

            // Initialize output writer
            out = output.New(cfg)
            return nil
        },
    }

    // Global flags
    rootCmd.PersistentFlags().StringVar(&configFile, "config", "", "config file (default: ~/.config/atlas-atlas-dev/config.toml)")
    rootCmd.PersistentFlags().Bool("human", false, "human-readable output (overrides config)")
    rootCmd.PersistentFlags().Bool("no-cache", false, "disable cache")

    // Version command
    versionCmd := &cobra.Command{
        Use:   "version",
        Short: "Show version information",
        Run: func(cmd *cobra.Command, args []string) {
            humanFlag, _ := cmd.Flags().GetBool("human")
            if humanFlag {
                fmt.Println(version.String())
            } else {
                out.Success(version.Info())
            }
        },
    }

    rootCmd.AddCommand(versionCmd)

    // Execute
    if err := rootCmd.Execute(); err != nil {
        if appErr, ok := err.(*errors.AppError); ok {
            out.Error(appErr, appErr.Details)
            os.Exit(appErr.Code)
        }
        fmt.Fprintf(os.Stderr, "Error: %v\n", err)
        os.Exit(1)
    }
}
```

---

### Step 8: Update Makefile

**File:** `Makefile`

```makefile
.PHONY: build install test clean fmt lint

# Build binary
build:
	go build -o bin/atlas-dev \
		-ldflags "-X github.com/atlas-lang/atlas-dev/internal/version.GitCommit=$(shell git rev-parse --short HEAD) \
		          -X github.com/atlas-lang/atlas-dev/internal/version.BuildDate=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)" \
		./cmd/atlas-dev

# Install to PATH
install:
	go install \
		-ldflags "-X github.com/atlas-lang/atlas-dev/internal/version.GitCommit=$(shell git rev-parse --short HEAD) \
		          -X github.com/atlas-lang/atlas-dev/internal/version.BuildDate=$(shell date -u +%Y-%m-%dT%H:%M:%SZ)" \
		./cmd/atlas-dev

# Run tests
test:
	go test -v ./...

# Clean build artifacts
clean:
	rm -rf bin/

# Format code
fmt:
	go fmt ./...

# Lint
lint:
	golangci-lint run || true

# Development: build and run version
dev: build
	./bin/atlas-dev version

# Show version
version:
	@echo "atlas-dev v$(shell git describe --tags --always)"
```

---

### Step 9: Add Tests

**File:** `internal/config/config_test.go`

```go
package config

import (
    "testing"
)

func TestDefaults(t *testing.T) {
    cfg := Defaults()

    if cfg.Output.Format != "json" {
        t.Errorf("Expected format=json, got %s", cfg.Output.Format)
    }

    if !cfg.Output.Compact {
        t.Error("Expected compact=true")
    }

    if !cfg.AI.Optimize {
        t.Error("Expected AI.Optimize=true")
    }
}

func TestLoad(t *testing.T) {
    // Test loading non-existent config (should return defaults)
    cfg, err := Load("/nonexistent/config.toml")
    if err != nil {
        t.Fatalf("Expected no error, got %v", err)
    }

    if cfg.Output.Format != "json" {
        t.Errorf("Expected default format=json, got %s", cfg.Output.Format)
    }
}
```

---

## Testing

### Manual Tests

```bash
# Build
cd tools/atlas-dev
make build

# Test version command (JSON output)
./bin/atlas-dev version
# Expected: {"version":"1.0.0","git_commit":"...","build_date":"..."}

# Test version command (human output)
./bin/atlas-dev version --human
# Expected: atlas-dev v1.0.0 (...)

# Test with non-existent config (should use defaults)
./bin/atlas-dev version --config /tmp/nonexistent.toml
# Expected: Works fine, uses default config

# Run tests
make test
```

---

## Acceptance Criteria

### Functional Requirements
- [ ] `go build` succeeds without errors
- [ ] `make build` creates `bin/atlas-dev`
- [ ] `atlas-dev version` outputs JSON (default)
- [ ] `atlas-dev version --human` outputs human-readable text
- [ ] Config system loads defaults when config file doesn't exist
- [ ] All tests pass (`make test`)
- [ ] Code is formatted (`make fmt`)

### Token Efficiency Requirements
- [ ] `atlas-dev --help` output < 100 tokens
- [ ] `atlas-dev version --help` output < 60 tokens
- [ ] Default output is JSON (not human-readable)
- [ ] JSON uses compact notation (no null/empty fields)
- [ ] No emoji in default output (JSON mode)
- [ ] Config defaults: `format=json`, `compact=true`, `ai.optimize=true`

---

## Next Phase

**Phase 2:** Phase Management System
- Implement `phase complete` command
- Tracker file parser/writer
- STATUS.md parser/writer
- Percentage calculator
- Git automation

---

## Notes

- Keep it simple - focus on structure, not features
- All output is JSON by default (AI-optimized)
- Config system supports overrides via flags
- Error handling uses typed errors with exit codes
- Version info embedded at build time
