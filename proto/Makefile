# TEA Protobuf Makefile
# Common commands for working with TEA protobuf definitions

.PHONY: all deps lint breaking generate clean help

# Default target
all: deps lint generate

# Install dependencies
deps:
	@echo "==> Updating Buf dependencies..."
	buf dep update

# Lint protobuf files
lint:
	@echo "==> Linting protobuf files..."
	buf lint

# Check for breaking changes against main branch
breaking:
	@echo "==> Checking for breaking changes..."
	buf breaking --against '.git#branch=main'

# Check for breaking changes against published BSR module
breaking-bsr:
	@echo "==> Checking for breaking changes against BSR..."
	buf breaking --against buf.build/cyclonedx/tea

# Generate code for all targets
generate:
	@echo "==> Generating code..."
	buf generate
	@echo "==> Code generated in gen/"

# Generate only Go code
generate-go:
	@echo "==> Generating Go code..."
	buf generate --template buf.gen.yaml --include-imports \
		--path tea/v1/common.proto \
		--path tea/v1/product.proto \
		--path tea/v1/component.proto \
		--path tea/v1/collection.proto \
		--path tea/v1/artifact.proto \
		--path tea/v1/discovery.proto \
		--path tea/v1/consumer.proto \
		--path tea/v1/publisher.proto \
		--path tea/v1/insights.proto

# Format protobuf files
format:
	@echo "==> Formatting protobuf files..."
	buf format -w

# Export dependencies (for IDE support)
export-deps:
	@echo "==> Exporting dependencies..."
	buf export --output .buf-deps
	@echo "==> Dependencies exported to .buf-deps/"

# Validate configuration
validate:
	@echo "==> Validating buf configuration..."
	buf config validate

# Build (lint + breaking check)
build: lint breaking
	@echo "==> Build checks passed"

# Push to Buf Schema Registry
push:
	@echo "==> Pushing to Buf Schema Registry..."
	buf push

# Clean generated files
clean:
	@echo "==> Cleaning generated files..."
	rm -rf gen/
	rm -rf .buf-deps/

# Show help
help:
	@echo "TEA Protobuf Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  all           - deps + lint + generate (default)"
	@echo "  deps          - Update Buf dependencies"
	@echo "  lint          - Lint protobuf files"
	@echo "  breaking      - Check for breaking changes (vs main branch)"
	@echo "  breaking-bsr  - Check for breaking changes (vs BSR)"
	@echo "  generate      - Generate code for all languages"
	@echo "  generate-go   - Generate only Go code"
	@echo "  format        - Format protobuf files"
	@echo "  export-deps   - Export dependencies for IDE support"
	@echo "  validate      - Validate buf configuration"
	@echo "  build         - Run lint + breaking checks"
	@echo "  push          - Push to Buf Schema Registry"
	@echo "  clean         - Remove generated files"
	@echo "  help          - Show this help message"