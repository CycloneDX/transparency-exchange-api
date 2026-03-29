# TEA Protobuf Makefile
# Common commands for working with TEA protobuf definitions

.PHONY: all deps lint breaking generate generate-go generate-rust export-deps validate verify clean help

# Default target
all: deps validate lint generate generate-rust

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

# Generate Buf-managed code
generate:
	@echo "==> Generating code..."
	buf generate
	@echo "==> Buf-managed code generated in gen/go/"

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

# Generate Rust bindings via tea-server's tonic-build pipeline
generate-rust: export-deps
	@echo "==> Generating Rust bindings..."
	cargo check --manifest-path ../tea-server/Cargo.toml --locked
	@echo "==> Rust bindings generated via tea-server/build.rs"

# Format protobuf files
format:
	@echo "==> Formatting protobuf files..."
	buf format -w

# Export dependencies (for IDE support)
export-deps:
	@echo "==> Exporting dependencies..."
	rm -rf .buf-deps
	buf export . --output .buf-deps
	@echo "==> Dependencies exported to .buf-deps/"

# Validate Buf workspace configuration
validate:
	@echo "==> Validating buf workspace..."
	buf config ls-modules >/dev/null
	@echo "==> Buf workspace configuration is valid"

# Build (lint + breaking check)
build: validate lint breaking
	@echo "==> Build checks passed"

# Full local verification for schema + generated artifacts
verify: validate lint generate generate-rust
	@echo "==> Proto verification passed"

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
	@echo "  all           - deps + validate + lint + generate + generate-rust (default)"
	@echo "  deps          - Update Buf dependencies"
	@echo "  validate      - Validate Buf workspace configuration"
	@echo "  lint          - Lint protobuf files"
	@echo "  breaking      - Check for breaking changes (vs main branch)"
	@echo "  breaking-bsr  - Check for breaking changes (vs BSR)"
	@echo "  generate      - Generate Buf-managed code (Go)"
	@echo "  generate-go   - Generate only Go code"
	@echo "  generate-rust - Generate Rust bindings via tea-server/build.rs"
	@echo "  format        - Format protobuf files"
	@echo "  export-deps   - Export dependencies for IDE support"
	@echo "  verify        - Validate + lint + generate + generate-rust"
	@echo "  build         - Run lint + breaking checks"
	@echo "  push          - Push to Buf Schema Registry"
	@echo "  clean         - Remove generated files"
	@echo "  help          - Show this help message"
