# TEA Server

Reference implementation of the [CycloneDX Transparency Exchange API (TEA)](../README.md) standard.

[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](../LICENSE)
[![Build](https://github.com/CycloneDX/transparency-exchange-api/actions/workflows/ci.yaml/badge.svg)](https://github.com/CycloneDX/transparency-exchange-api/actions/workflows/ci.yaml)

## Features

- **TEA HTTP + gRPC runtime**: Public reads, authenticated writes, discovery, and consumer APIs
- **Multiple identifier types**: TEI, PURL, CPE, SWID, GAV, GTIN, GMN, UDI, ASIN, Hash
- **Supply chain security**: Sigstore attestation signing, SLSA provenance, SBOM generation
- **Transport security**: TLS and mTLS with client certificate authentication
- **Observability**: OpenTelemetry tracing, structured logging, Prometheus metrics
- **Database support**: in-memory runtime in the current reference binary, with PostgreSQL migrations and repository adapters available

## Positioning

This server is the Rust reference implementation of the TEA contracts, not the
only intended production deployment shape.

- `proto/` remains the canonical TEA specification surface
- `tea-server` demonstrates the stable, normative flows end to end
- optional publisher capabilities that are not ready yet fail closed with
  `UNIMPLEMENTED` instead of pretending to work

For the explicit spec-vs-reference-server contract, see
[`docs/reference-implementation-profile.md`](../docs/reference-implementation-profile.md).

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
and gRPC will be available at `localhost:50051` when enabled.

### Run Locally

```bash
# Set required environment variables
export TEA_JWT_SECRET="$(openssl rand -hex 32)"
export TEA_DATABASE_URL="postgres://tea:tea@localhost:5432/tea"

# Run database migrations
DATABASE_URL="$TEA_DATABASE_URL" sqlx migrate run

# Start the server
cargo run --release
```

## Configuration

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `TEA_PORT` | Server listen port | `8734` | No |
| `TEA_SERVER_URL` | Public server URL | `http://localhost:8734` | No |
| `TEA_PERSISTENCE_BACKEND` | Persistence backend: `memory` or `postgres` | `memory` | No |
| `TEA_JWT_SECRET` | JWT signing key | - | **Yes** (release) |
| `TEA_DATABASE_URL` | Canonical database DSN for runtime config and migration handoff | - | No |
| `TEA_ALLOWED_ORIGINS` | Comma-separated allowed CORS origins | empty | No |
| `TEA_REQUEST_TIMEOUT_SECS` | Per-request timeout | `30` | No |
| `TEA_RATE_LIMIT_CLEANUP_SECS` | Rate limiter cleanup interval | `300` | No |
| `TEA_GRPC_ENABLED` | Enable discovery/consumer gRPC listener | `false` | No |
| `TEA_GRPC_PORT` | gRPC listen port | `50051` | No |
| `TEA_GRPC_PUBLISHER_ENABLED` | Expose supported publisher gRPC handlers (product/product-release/component/component-release lifecycle plus safe artifact delete) | `false` | No |
| `TEA_RATE_LIMIT_ENABLED` | Enable or disable rate limiting | `true` | No |
| `TEA_RATE_LIMIT_RPM` | Requests per minute per IP | `60` | No |
| `TEA_RATE_LIMIT_BURST` | Burst allowance | `10` | No |
| `TEA_TLS_CERT_PATH` | TLS certificate path | - | No |
| `TEA_TLS_KEY_PATH` | TLS private key path | - | No |
| `TEA_TLS_CLIENT_CA_PATH` | mTLS client CA path | - | No |
| `TEA_SIGNING_MODE` | Attestation signing mode | `disabled` | No |
| `TEA_SIGNING_KEY_PATH` | Private key path for `TEA_SIGNING_MODE=key` | - | No |
| `TEA_REKOR_UPLOAD` | Upload signatures to Rekor when signing is enabled | release-dependent | No |
| `TEA_JWT_ISSUER` | Expected JWT issuer claim | unset | No |
| `TEA_JWT_AUDIENCE` | Expected JWT audience claim | `tea-api` | No |
| `TEA_JWT_WRITE_SCOPE` | Required JWT write scope | `tea:write` | No |
| `TEA_JWT_WRITE_ROLE` | Required JWT write role | `tea-writer` | No |
| `SEAWEEDFS_ENDPOINT` | S3-compatible object storage endpoint | - | No |
| `SEAWEEDFS_BUCKET` | Object storage bucket name | - | No |

Notes:

- `TEA_DATABASE_URL` is the canonical application variable. `DATABASE_URL` is still accepted as a deprecated runtime fallback and remains the variable used by the `sqlx` CLI.
- Set `TEA_PERSISTENCE_BACKEND=postgres` to run the live server against PostgreSQL. Leave it unset to keep the in-memory reference mode.
- Set `TEA_GRPC_ENABLED=true` to serve the TEA discovery/consumer gRPC APIs on `TEA_GRPC_PORT`.
- Set `TEA_GRPC_PUBLISHER_ENABLED=true` only when you want the currently supported publisher gRPC subset; unsupported RPCs still fail closed with `UNIMPLEMENTED`.

## Transport Support Matrix

### HTTP/JSON

- Public reads: `GET /v1/products`, `GET /v1/components`, `GET /v1/artifacts`, `GET /v1/collections`
- Authenticated writes: `POST /v1/products`, `PUT /v1/products/{uuid}`, `DELETE /v1/products/{uuid}?cascade=true`
- Authenticated writes: `POST /v1/components`, `PUT /v1/components/{uuid}`, `DELETE /v1/components/{uuid}?cascade=true`
- Authenticated writes: `POST /v1/artifacts`, `DELETE /v1/artifacts/{uuid}`
- Public collection version reads: `GET /v1/collections/{uuid}/versions`, `GET /v1/collections/{uuid}/versions/{version}`, `GET /v1/collections/{uuid}/compare`
- Authenticated writes: `POST /v1/collections`, `POST /v1/collections/{uuid}/versions`, `DELETE /v1/collections/{uuid}`
- Authenticated deprecation: `POST /v1/*/{uuid}/deprecate`

### gRPC

- Discovery and consumer services are live when `TEA_GRPC_ENABLED=true`
- Publisher gRPC currently supports product, product-release, component, component-release, `CreateArtifactFromUrl`, collection create/update operations, and safe artifact delete when `TEA_GRPC_PUBLISHER_ENABLED=true`
- Collection updates are immutable version bumps: `UpdateCollection` creates the next collection version for the same logical UUID
- Collection signing/import and streaming artifact upload RPCs intentionally fail closed with `UNIMPLEMENTED`

For a practical producer-facing integration guide, including a path tailored for the Rust `sbom-tools` project, see [`docs/sbom-tools-integration.md`](../docs/sbom-tools-integration.md).

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
# Disabled by default unless a real signing backend is configured
export TEA_SIGNING_MODE=disabled

# Key-based signing
export TEA_SIGNING_MODE=key
export TEA_SIGNING_KEY_PATH=/path/to/private.key
```

Keyless signing is not production-ready in this reference server yet; leave
`TEA_SIGNING_MODE=disabled` unless you have configured a real key-based signer.

## API Endpoints

### Public HTTP reads

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/v1/products` | List products |
| `GET` | `/v1/products/{uuid}` | Get a product |
| `GET` | `/v1/components` | List components |
| `GET` | `/v1/components/{uuid}` | Get a component |
| `GET` | `/v1/artifacts` | List artifacts |
| `GET` | `/v1/artifacts/{uuid}` | Get an artifact |
| `GET` | `/v1/collections` | List collections |
| `GET` | `/v1/collections/{uuid}` | Get a collection |
| `GET` | `/v1/collections/{uuid}/versions` | List collection versions |
| `GET` | `/v1/collections/{uuid}/versions/{version}` | Get a specific collection version |
| `GET` | `/v1/collections/{uuid}/compare?base_version=1&target_version=2` | Compare two collection versions |

### Authenticated HTTP writes

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/products` | Create a product |
| `PUT` | `/v1/products/{uuid}` | Replace mutable product fields |
| `DELETE` | `/v1/products/{uuid}` | Delete a product; use `?cascade=true` to remove product releases first |
| `POST` | `/v1/products/{uuid}/deprecate` | Deprecate a product |
| `POST` | `/v1/components` | Create a component |
| `PUT` | `/v1/components/{uuid}` | Replace mutable component fields |
| `DELETE` | `/v1/components/{uuid}` | Delete a component; use `?cascade=true` to remove component releases first |
| `POST` | `/v1/components/{uuid}/deprecate` | Deprecate a component |
| `POST` | `/v1/artifacts` | Register an artifact and its distribution URLs |
| `DELETE` | `/v1/artifacts/{uuid}` | Delete an artifact when no collection still references it |
| `POST` | `/v1/artifacts/{uuid}/deprecate` | Deprecate an artifact |
| `POST` | `/v1/collections` | Create a collection; referenced artifacts must already exist |
| `POST` | `/v1/collections/{uuid}/versions` | Create the next immutable collection version |
| `DELETE` | `/v1/collections/{uuid}` | Delete a collection |
| `POST` | `/v1/collections/{uuid}/deprecate` | Deprecate a collection |

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
DATABASE_URL="$TEA_DATABASE_URL" sqlx migrate run

# Revert last migration
DATABASE_URL="$TEA_DATABASE_URL" sqlx migrate revert
```

## Deployment

### Docker

```bash
# Build image
docker build -t tea-server .

# Run container
docker run -p 8734:8734 \
  -p 50051:50051 \
  -e TEA_JWT_SECRET=your-secret \
  -e TEA_DATABASE_URL=postgres://... \
  -e TEA_GRPC_ENABLED=true \
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
│   │   ├── grpc/         # Discovery/consumer gRPC + supported publisher writes
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
