# TEA Server

Reference implementation of the [CycloneDX Transparency Exchange API (TEA)](../README.md) standard.

[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](../LICENSE)
[![Build](https://github.com/CycloneDX/transparency-exchange-api/actions/workflows/ci.yaml/badge.svg)](https://github.com/CycloneDX/transparency-exchange-api/actions/workflows/ci.yaml)

## Features

- **Full TEA API**: Products, Components, Collections, Artifacts, Deprecation
- **Multiple identifier types**: TEI, PURL, CPE, SWID, GAV, GTIN, GMN, UDI, ASIN, Hash
- **Supply chain security**: Sigstore attestation signing, SLSA provenance, SBOM generation
- **Transport security**: TLS and mTLS with client certificate authentication
- **Observability**: OpenTelemetry tracing, structured logging, Prometheus metrics
- **Database support**: PostgreSQL (production), in-memory (development/testing)

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 16+ (or use Docker)
- protobuf compiler (for gRPC)

### Run with Docker Compose

```bash
cd tea-server
docker compose up -d
```

Server will be available at http://localhost:8734

### Run Locally

```bash
# Set required environment variables
export TEA_JWT_SECRET="$(openssl rand -hex 32)"
export TEA_DATABASE_URL="postgres://tea:tea@localhost:5432/tea"

# Run database migrations
sqlx migrate run

# Start the server
cargo run --release
```

## Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `TEA_PORT` | Server listen port | `8734` | No |
| `TEA_SERVER_URL` | Public server URL | `http://localhost:8734` | No |
| `TEA_JWT_SECRET` | JWT signing key | - | **Yes** (release) |
| `TEA_DATABASE_URL` | PostgreSQL connection string | - | Yes (with DB) |
| `TEA_TLS_CERT_PATH` | TLS certificate path | - | No |
| `TEA_TLS_KEY_PATH` | TLS private key path | - | No |
| `TEA_TLS_CLIENT_CA_PATH` | mTLS client CA path | - | No |
| `TEA_SIGNING_MODE` | Attestation signing mode | `disabled` | No |
| `TEA_RATE_LIMIT_RPM` | Requests per minute per IP | `60` | No |
| `TEA_RATE_LIMIT_BURST` | Burst allowance | `10` | No |

### TLS Configuration

```bash
# Enable TLS
export TEA_TLS_CERT_PATH=/path/to/server.crt
export TEA_TLS_KEY_PATH=/path/to/server.key

# Enable mTLS (client certificates)
export TEA_TLS_CLIENT_CA_PATH=/path/to/client-ca.crt
```

### Attestation Signing

```bash
# Keyless signing via Sigstore (requires OIDC token)
export TEA_SIGNING_MODE=keyless

# Key-based signing
export TEA_SIGNING_MODE=key
export TEA_SIGNING_KEY_PATH=/path/to/private.key
```

## API Endpoints

### Products

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/products` | Create a product |
| `GET` | `/v1/products/{uuid}` | Get a product |
| `PUT` | `/v1/products/{uuid}` | Update a product |
| `DELETE` | `/v1/products/{uuid}` | Delete a product |
| `POST` | `/v1/products/{uuid}/deprecate` | Deprecate a product |

### Components

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/components` | Create a component |
| `GET` | `/v1/components/{uuid}` | Get a component |
| `PUT` | `/v1/components/{uuid}` | Update a component |
| `DELETE` | `/v1/components/{uuid}` | Delete a component |
| `POST` | `/v1/components/{uuid}/deprecate` | Deprecate a component |

### Collections

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/collections` | Create a collection |
| `GET` | `/v1/collections/{uuid}` | Get a collection |
| `GET` | `/v1/collections/{uuid}/versions/{version}` | Get collection version |
| `PUT` | `/v1/collections/{uuid}` | Update a collection |
| `DELETE` | `/v1/collections/{uuid}` | Delete a collection |

### Artifacts

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/artifacts` | Create an artifact |
| `GET` | `/v1/artifacts/{uuid}` | Get an artifact |
| `PUT` | `/v1/artifacts/{uuid}` | Update an artifact |
| `DELETE` | `/v1/artifacts/{uuid}` | Delete an artifact |

## Development

### Run Tests

```bash
# Unit and E2E tests
cargo test

# Integration tests (requires Docker for testcontainers)
cargo test --test integration

# E2E conformance tests
cargo test --test e2e_conformance
```

### Code Quality

```bash
# Format check
cargo fmt --check

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Deployment

### Docker

```bash
# Build image
docker build -t tea-server .

# Run container
docker run -p 8734:8734 \
  -e TEA_JWT_SECRET=your-secret \
  -e TEA_DATABASE_URL=postgres://... \
  tea-server
```

### Kubernetes

See `deploy/k8s/` for Kubernetes manifests.

```bash
kubectl apply -k deploy/k8s/
```

## Architecture

```
tea-server/
├── src/
│   ├── domain/           # Domain entities and business logic
│   │   ├── common/       # Shared value objects (identifiers, checksums, etc.)
│   │   ├── product/      # Product aggregate
│   │   ├── component/    # Component aggregate
│   │   ├── collection/   # Collection aggregate
│   │   └── artifact/     # Artifact entity
│   ├── infrastructure/   # Technical implementations
│   │   ├── http/         # HTTP routes and handlers
│   │   ├── grpc/         # gRPC services (stub)
│   │   ├── persistence/  # Repository implementations
│   │   ├── auth/         # Authentication (JWT, mTLS)
│   │   ├── evidence/     # Attestation signing
│   │   └── middleware/   # Rate limiting, etc.
│   ├── config/           # Configuration loading
│   └── main.rs           # Application entry point
├── migrations/           # Database migrations
└── tests/               # Integration and E2E tests
```

## Security Considerations

- **JWT Secret**: Must be at least 32 bytes. Generate with `openssl rand -hex 32`
- **TLS**: Recommended for all production deployments
- **mTLS**: Required for high-security environments
- **Rate Limiting**: Enabled by default in release builds
- **Input Validation**: All inputs validated against TEA specification
- **Audit Trail**: All mutations tracked with `created_by` and `modified_by`

## Supply Chain Security

This project implements:

- **SLSA Level 3**: Provenance generation via GitHub Actions
- **SBOM**: Generated at build time (SPDX and CycloneDX formats)
- **Sigstore Signing**: All attestations and SBOMs signed
- **Dependency Pinning**: All dependencies locked in `Cargo.lock`

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.

## License

Apache 2.0 - See [LICENSE](../LICENSE) for details.
