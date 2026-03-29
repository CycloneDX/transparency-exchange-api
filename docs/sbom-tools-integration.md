# sbom-tools Integration Guide

## Why this guide exists

`sbom-tools` is already a strong fit for TEA:

- it parses and normalizes CycloneDX / SPDX inputs locally
- it already carries optional `reqwest` support for outbound HTTP calls
- it can generate the SBOM, VEX, and report metadata that a TEA publisher wants to register

This guide shows the safest integration path with the current TEA reference server.

## Recommended integration path

For `sbom-tools`, the most practical path today is:

1. generate or normalize the SBOM locally
2. upload the SBOM file to durable object storage that your TEA clients can fetch
3. register TEA metadata over HTTP/JSON
4. use TEA discovery / consumer APIs for downstream reads

Why HTTP first?

- `sbom-tools` already depends on `reqwest`
- the live TEA HTTP surface supports the main product/component lifecycle plus artifact and collection registration
- the gRPC publisher surface is useful, while artifact upload/import still remains intentionally incomplete

## Canonical publisher profile examples

If `sbom-tools` integrates through a gRPC gateway or any HTTP layer that
follows the canonical publisher profile, use the generated examples in
[`docs/generated/sbom-tools-publisher-profile-examples.md`](generated/sbom-tools-publisher-profile-examples.md).

That generated document is sourced from
[`spec/publisher/openapi.json`](../spec/publisher/openapi.json), so its
payloads stay aligned with the canonical publisher contract rather than drifting
from hand-maintained snippets.

If you want Rust-ready `reqwest` scaffolding for the same canonical publisher
profile, use
[`docs/generated/sbom-tools-publisher-reqwest-snippets.md`](generated/sbom-tools-publisher-reqwest-snippets.md).

For release review or handoff, CI also builds a lightweight publisher release
doc bundle that packages the HTML conformance report together with these
generated sbom-tools docs. The bundle now includes a sibling
`publisher-release-doc-bundle.tar.gz` plus a `.sha256` checksum file, so it can
be uploaded directly to a release page, artifact bucket, or static handoff
channel outside CI.

## Current support matrix

### HTTP/JSON in the live reference server

- `POST /v1/products`
- `PUT /v1/products/{uuid}`
- `DELETE /v1/products/{uuid}?cascade=true`
- `POST /v1/components`
- `PUT /v1/components/{uuid}`
- `DELETE /v1/components/{uuid}?cascade=true`
- `POST /v1/artifacts`
- `DELETE /v1/artifacts/{uuid}`
- `POST /v1/collections`
- `POST /v1/collections/{uuid}/versions`
- `DELETE /v1/collections/{uuid}`
- `POST /v1/*/:uuid/deprecate`
- `GET /v1/collections/{uuid}/versions`
- `GET /v1/collections/{uuid}/versions/{version}`
- `GET /v1/collections/{uuid}/compare`
- public `GET /v1/*` read endpoints

### gRPC in the live reference server

- discovery and consumer read services
- publisher `Create/Update/DeleteProduct`
- publisher `Create/Update/DeleteProductRelease`
- publisher `Create/Update/DeleteComponent`
- publisher `Create/Update/DeleteComponentRelease`
- publisher `CreateArtifactFromUrl`
- publisher `CreateCollection` and versioned `UpdateCollection`
- publisher `DeleteArtifact` with collection safety checks

### Still intentionally unavailable

- publisher collection sign / import RPCs
- artifact upload streaming / bulk import RPCs

Those RPCs currently fail closed with `UNIMPLEMENTED` rather than pretending to work.

## Authentication

Publisher writes require a bearer token accepted by the TEA server.

Minimum server-side config:

```bash
export TEA_JWT_SECRET="replace-with-a-real-32-byte-secret"
export TEA_JWT_AUDIENCE="tea-api"
export TEA_JWT_WRITE_SCOPE="tea:write"
export TEA_JWT_WRITE_ROLE="tea-writer"
```

Your client token must satisfy the configured audience and at least one write privilege path:

- `scope` contains the configured write scope
- `permissions` contains an accepted write permission
- `role` matches the configured write role

## Publishing flow for sbom-tools

### 1. Create or locate the product

Register the producer-visible product first.

```bash
curl -sS \
  -H "Authorization: Bearer $TEA_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST http://localhost:8734/v1/products \
  -d '{
    "name": "Example Appliance",
    "description": "Main software appliance tracked by sbom-tools",
    "identifiers": [
      { "id_type": "TEI", "id_value": "urn:tei:uuid:tea.example.com:example-appliance" }
    ],
    "vendor": {
      "name": "Example Vendor",
      "uuid": null,
      "url": "https://example.com",
      "contacts": []
    },
    "homepage_url": "https://example.com/appliance",
    "documentation_url": "https://docs.example.com/appliance",
    "vcs_url": "https://github.com/example/appliance"
  }'
```

### 2. Publish the SBOM file to storage

The current HTTP create-artifact flow registers artifact metadata plus fetch URLs.
It does not upload file bytes into the TEA server.

Recommended storage targets:

- internal S3 / SeaweedFS bucket fronted by signed URLs
- release artifact bucket
- immutable CDN/object path per build

### 3. Register the SBOM as a TEA artifact

```bash
curl -sS \
  -H "Authorization: Bearer $TEA_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST http://localhost:8734/v1/artifacts \
  -d '{
    "name": "example-appliance-1.2.3.cdx.json",
    "type": "BOM",
    "componentDistributions": [],
    "formats": [
      {
        "mimeType": "application/vnd.cyclonedx+json",
        "description": "CycloneDX SBOM generated by sbom-tools",
        "url": "https://artifacts.example.com/sboms/example-appliance/1.2.3.cdx.json",
        "signatureUrl": null,
        "checksums": [
          { "alg_type": "SHA256", "alg_value": "replace-with-real-sha256" }
        ],
        "sizeBytes": null,
        "encoding": null,
        "specVersion": "1.6"
      }
    ],
    "description": "Release SBOM for Example Appliance 1.2.3",
    "subject": {
      "type": "PRODUCT",
      "identifiers": [
        { "id_type": "TEI", "id_value": "urn:tei:uuid:tea.example.com:example-appliance" }
      ],
      "name": "Example Appliance",
      "version": "1.2.3"
    },
    "deprecation": null,
    "dependencies": []
  }'
```

If you prefer gRPC for this step and the SBOM is already available at an
immutable URL, `CreateArtifactFromUrl` is now a good fit: the server fetches
the URL once, verifies the expected checksums, and only then registers the
artifact metadata.

### 4. Register a TEA collection that points at the artifact

Collections are the durable grouping TEA clients read later.

```bash
curl -sS \
  -H "Authorization: Bearer $TEA_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST http://localhost:8734/v1/collections \
  -d '{
    "name": "Example Appliance 1.2.3 release collection",
    "version": 1,
    "belongs_to": "RELEASE",
    "update_reason": "INITIAL_RELEASE",
    "artifacts": ["<artifact-uuid-from-step-3>"],
    "deprecation": null,
    "dependencies": []
  }'
```

### 5. Publish the next collection version when the release metadata changes

If the same logical release collection needs another immutable version, create
the next version instead of mutating v1 in place.

```bash
curl -sS \
  -H "Authorization: Bearer $TEA_TOKEN" \
  -H "Content-Type: application/json" \
  -X POST http://localhost:8734/v1/collections/<collection-uuid>/versions \
  -d '{
    "artifacts": [
      "<artifact-uuid-from-step-3>",
      "<new-artifact-uuid>"
    ],
    "update_reason": "ARTIFACT_ADDED"
  }'
```

## Minimal Rust client example for the live reference server HTTP surface

Because `sbom-tools` already uses `reqwest::blocking`, a small integration helper can stay simple:

```rust
use reqwest::blocking::Client;
use serde_json::json;

fn create_product(base_url: &str, token: &str) -> anyhow::Result<serde_json::Value> {
    let client = Client::builder().build()?;
    let response = client
        .post(format!("{base_url}/v1/products"))
        .bearer_auth(token)
        .json(&json!({
            "name": "Example Appliance",
            "description": "Published from sbom-tools",
            "identifiers": [],
            "vendor": {
                "name": "Example Vendor",
                "uuid": null,
                "url": "https://example.com",
                "contacts": []
            },
            "homepage_url": "https://example.com/appliance",
            "documentation_url": "",
            "vcs_url": ""
        }))
        .send()?
        .error_for_status()?;

    Ok(response.json()?)
}
```

## Where gRPC fits for sbom-tools

gRPC is the better fit when `sbom-tools` wants:

- discovery lookups
- high-volume read access
- controlled UUIDs on publisher product / product-release / component / component-release writes
- versioned collection updates while keeping the same logical collection UUID

Today, gRPC is a good fit when the artifact already lives at an immutable URL.
It is still not the best fit for raw byte uploads because streaming upload and
bulk import RPCs remain intentionally disabled.

## Practical integration recommendation

If `sbom-tools` wants to integrate now, use this split:

- HTTP writes for artifact / collection registration
- gRPC `CreateArtifactFromUrl` when the SBOM already exists at a stable object URL
- HTTP versioning for collection lifecycle updates
- gRPC reads for discovery and consumer lookups
- optional gRPC publisher calls for product / product-release / component / component-release metadata workflows
- optional gRPC `UpdateCollection` calls when a new release collection version should be minted instead of creating a brand-new logical collection

That gives `sbom-tools` a production-usable path today without waiting for the remaining publisher RPCs to land.

## Operational notes

- component deletion is guarded when component releases exist; add `?cascade=true` only when you intend TEA to remove the release records first
- product deletion is guarded when product releases exist; add `?cascade=true` only when you intend TEA to remove the release records first
- artifact deletion is blocked while a collection still references the artifact
- collection writes are still best treated as immutable release records; on gRPC, `UpdateCollection` creates the next version instead of mutating the existing collection in place
