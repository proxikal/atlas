.PHONY: build install test clean

# Build binary
build:
	CGO_ENABLED=0 go build -ldflags="-w -s" -o bin/atlas-dev ./cmd/atlas-dev

# Install to PATH
install:
	CGO_ENABLED=0 go install -ldflags="-w -s" ./cmd/atlas-dev

# Run tests
test:
	go test -v ./...

# Clean build artifacts
clean:
	rm -rf bin/

# Development: build and run
dev: build
	./bin/atlas-dev

# Format code
fmt:
	go fmt ./...

# Lint
lint:
	golangci-lint run

# Show version
version:
	@echo "atlas-dev v1.0.0"
