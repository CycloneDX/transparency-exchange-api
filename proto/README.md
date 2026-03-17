# TEA Protobuf Definitions

This directory contains the Protocol Buffer (protobuf) definitions for the Transparency Exchange API (TEA) specification.

## Overview

The TEA specification defines a standard API for exchanging product transparency artifacts (SBOMs, VEX, attestations, etc.) between systems. This protobuf implementation provides:

- **Type-safe contracts** - Machine-readable API definitions
- **gRPC services** - High-performance RPC with streaming support
- **REST mapping** - gRPC-Gateway annotations for HTTP/JSON APIs
- **Multi-language SDKs** - Generated clients for Go, Rust, TypeScript, Python

## Directory Structure

```
proto/
├── buf.yaml              # Buf V2 module configuration
├── buf.gen.yaml          # Code generation configuration
├── buf.lock              # Dependency lock file (generated)
├── README.md             # This file
└── tea/
    └── v1/
        ├── common.proto      # Common types (identifiers, checksums, pagination)
        ├── product.proto     # Product and ProductRelease entities
        ├── component.proto   # Component and ComponentRelease entities
        ├── collection.proto  # Collection and artifact grouping
        ├── artifact.proto    # Artifact types and formats
        ├── discovery.proto   # TEI resolution and discovery service
        ├── consumer.proto    # Read-only consumer API
        ├── publisher.proto   # Write operations for publishers
        └── insights.proto    # Query and analytics API
```

## Services

### DiscoveryService (`discovery.proto`)
TEI (Transparency Exchange Identifier) resolution and API discovery.

| RPC | HTTP | Description |
|-----|------|-------------|
| `Discover` | `GET /v1/discovery?tei=...` | Resolve TEI to ProductRelease |
| `GetWellKnown` | `GET /.well-known/tea` | Discovery document |
| `HealthCheck` | `GET /v1/health` | Service health |
| `GetServerInfo` | `GET /v1/info` | Server metadata |

### ConsumerService (`consumer.proto`)
Read-only access to transparency artifacts.

| RPC | HTTP | Description |
|-----|------|-------------|
| `ListProducts` | `GET /v1/products` | List products |
| `GetProduct` | `GET /v1/products/{uuid}` | Get product |
| `ListProductReleases` | `GET /v1/products/{uuid}/releases` | List releases |
| `GetProductRelease` | `GET /v1/product-releases/{uuid}` | Get release |
| `ListComponents` | `GET /v1/components` | List components |
| `GetComponent` | `GET /v1/components/{uuid}` | Get component |
| `GetCollection` | `GET /v1/collections/{uuid}` | Get collection |
| `GetArtifact` | `GET /v1/artifacts/{uuid}` | Get artifact |
| `StreamArtifactContent` | `GET /v1/artifacts/{uuid}/stream` | Download artifact |
| `SearchByIdentifier` | `GET /v1/search/identifier` | Search by ID |
| `SearchByChecksum` | `GET /v1/search/checksum` | Search by hash |

### PublisherService (`publisher.proto`)
Write operations for artifact publishers.

| RPC | HTTP | Description |
|-----|------|-------------|
| `CreateProduct` | `POST /v1/publisher/products` | Create product |
| `UpdateProduct` | `PUT /v1/publisher/products/{uuid}` | Update product |
| `DeleteProduct` | `DELETE /v1/publisher/products/{uuid}` | Delete product |
| `UploadArtifact` | `POST /v1/publisher/artifacts` | Upload artifact |
| `CreateCollection` | `POST /v1/publisher/collections` | Create collection |
| `SignCollection` | `POST /v1/publisher/collections/{uuid}/sign` | Sign collection |

### InsightsService (`insights.proto`)
Query and analytics capabilities.

| RPC | HTTP | Description |
|-----|------|-------------|
| `Query` | `POST /v1/insights/query` | CEL expression query |
| `GetVulnerabilitySummary` | `GET /v1/insights/.../vulnerabilities` | Vulnerability info |
| `GetComponentDependencies` | `GET /v1/insights/.../dependencies` | Dependency tree |
| `CompareSBOMs` | `POST /v1/insights/compare/sbom` | SBOM diff |

## Prerequisites

- [Buf CLI](https://buf.build/docs/installation) v1.28.0+
- For Go: Go 1.21+
- For Rust: Rust 1.70+
- For TypeScript: Node.js 18+

## Quick Start

### Install Buf CLI

```bash
# macOS
brew install bufbuild/buf/buf

# Linux/Windows
# See: https://buf.build/docs/installation
```

### Update Dependencies

```bash
cd proto
buf dep update
```

### Lint Protobuf Files

```bash
buf lint
```

### Check Breaking Changes

```bash
# Against published version
buf breaking --against buf.build/cyclonedx/tea

# Against local git ref
buf breaking --against '.git#branch=main'
```

### Generate Code

```bash
# Generate all targets
buf generate

# Generate specific target
buf generate --template buf.gen.yaml --path tea/v1/consumer.proto
```

## Generated Code

After running `buf generate`, code is output to `gen/`:

```
gen/
├── go/          # Go (protobuf + gRPC + gateway)
├── rust/src/    # Rust (prost + tonic)
├── ts/          # TypeScript (Connect-ES)
├── python/      # Python (protobuf + gRPC)
├── openapi/     # OpenAPI v2 specification
└── docs/        # Markdown documentation
```

## Usage Examples

### Go Client

```go
package main

import (
    "context"
    teav1 "github.com/CycloneDX/transparency-exchange-api/gen/go/tea/v1"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
)

func main() {
    conn, _ := grpc.Dial("localhost:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
    defer conn.Close()

    client := teav1.NewConsumerServiceClient(conn)
    
    // Resolve a TEI
    discovery := teav1.NewDiscoveryServiceClient(conn)
    resp, _ := discovery.Discover(context.Background(), &teav1.DiscoverRequest{
        Tei: "urn:tei:uuid:example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1",
    })
    
    // Get the product release
    release, _ := client.GetProductRelease(context.Background(), &teav1.GetProductReleaseRequest{
        Uuid: resp.ProductReleaseUuid,
    })
    
    fmt.Printf("Found release: %s v%s\n", release.Uuid, release.Version)
}
```

### Rust Client

```rust
use tea::v1::{
    consumer_service_client::ConsumerServiceClient,
    discovery_service_client::DiscoveryServiceClient,
    DiscoverRequest, GetProductReleaseRequest,
};
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await?;

    let mut discovery = DiscoveryServiceClient::new(channel.clone());
    let mut consumer = ConsumerServiceClient::new(channel);

    // Resolve a TEI
    let resp = discovery
        .discover(DiscoverRequest {
            tei: "urn:tei:uuid:example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1".into(),
        })
        .await?
        .into_inner();

    // Get the product release
    let release = consumer
        .get_product_release(GetProductReleaseRequest {
            uuid: resp.product_release_uuid,
        })
        .await?
        .into_inner();

    println!("Found release: {} v{}", release.uuid, release.version);
    Ok(())
}
```

### TypeScript Client

```typescript
import { createPromiseClient } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { ConsumerService, DiscoveryService } from "./gen/ts/tea/v1/consumer_connect";

const transport = createGrpcTransport({
  baseUrl: "http://localhost:50051",
  httpVersion: "2",
});

const discovery = createPromiseClient(DiscoveryService, transport);
const consumer = createPromiseClient(ConsumerService, transport);

async function main() {
  // Resolve a TEI
  const discoverResp = await discovery.discover({
    tei: "urn:tei:uuid:example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1",
  });

  // Get the product release
  const release = await consumer.getProductRelease({
    uuid: discoverResp.productReleaseUuid,
  });

  console.log(`Found release: ${release.uuid} v${release.version}`);
}

main();
```

## Versioning

This protobuf package follows [Semantic Versioning 2.0](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality
- **PATCH** version for backwards-compatible bug fixes

Breaking changes are detected automatically using `buf breaking`.

## Publishing

To publish to Buf Schema Registry:

```bash
# Login to BSR
buf registry login

# Push module
buf push
```

## Contributing

1. Make changes to `.proto` files
2. Run `buf lint` to check style
3. Run `buf breaking` to check compatibility
4. Run `buf generate` to regenerate code
5. Submit a pull request

## License

Apache License 2.0 - See [LICENSE](../LICENSE) for details.

## References

- [TEA Specification](https://github.com/CycloneDX/transparency-exchange-api)
- [Buf Documentation](https://buf.build/docs/)
- [gRPC Documentation](https://grpc.io/docs/)
- [Protocol Buffers](https://protobuf.dev/)
- [CycloneDX](https://cyclonedx.org/)