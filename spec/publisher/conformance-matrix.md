# Publisher Conformance Matrix

This matrix separates the canonical TEA publisher contract from the current Rust
reference implementation profile.

## Scope

- Canonical contract: [`proto/tea/v1/publisher.proto`](../../proto/tea/v1/publisher.proto)
- Reference server: [`tea-server`](../../tea-server)
- Goal: make it obvious which publisher flows are normative, which are optional,
  and which remain intentionally unsupported in the current executable reference

## Transport expectations

- All publisher operations require authentication.
- Unauthorized requests must be rejected explicitly.
- Unsupported optional capabilities must fail explicitly, typically with
  `UNIMPLEMENTED`.
- Referential integrity failures should surface as precondition-style failures,
  not partial writes.

## RPC matrix

| RPC | Contract role | Minimum semantics | Expected explicit failures | Rust reference server |
|-----|---------------|-------------------|----------------------------|-----------------------|
| `CreateProduct` | Core publisher metadata | Create a product; preserve client UUID when supplied | Auth failure, validation failure | Implemented |
| `UpdateProduct` | Core publisher metadata | Apply only `update_mask` fields; preserve unspecified fields | Auth failure, validation failure, not found | Implemented |
| `DeleteProduct` | Core publisher metadata | Delete product; require explicit cascade policy for dependent releases | Auth failure, not found, `FAILED_PRECONDITION` when releases exist and `cascade=false` | Implemented |
| `CreateProductRelease` | Core publisher metadata | Create release linked to an existing product and valid component refs | Auth failure, validation failure, missing parent/dependency | Implemented |
| `UpdateProductRelease` | Core publisher metadata | Apply masked updates; preserve unspecified fields | Auth failure, validation failure, not found | Implemented |
| `DeleteProductRelease` | Core publisher metadata | Delete one release | Auth failure, not found | Implemented |
| `CreateComponent` | Core publisher metadata | Create component; preserve client UUID when supplied | Auth failure, validation failure | Implemented |
| `UpdateComponent` | Core publisher metadata | Apply only `update_mask` fields; preserve unspecified fields | Auth failure, validation failure, not found | Implemented |
| `DeleteComponent` | Core publisher metadata | Delete component; require explicit cascade policy for dependent releases | Auth failure, not found, `FAILED_PRECONDITION` when releases exist and `cascade=false` | Implemented |
| `CreateComponentRelease` | Core publisher metadata | Create release linked to an existing component | Auth failure, validation failure, missing parent | Implemented |
| `UpdateComponentRelease` | Core publisher metadata | Apply masked updates; preserve unspecified fields; do not allow `pre_release` to move from false back to true | Auth failure, validation failure, not found | Implemented |
| `DeleteComponentRelease` | Core publisher metadata | Delete one release | Auth failure, not found | Implemented |
| `UploadArtifact` | Advanced optional publisher capability | Stream metadata first, then content; verify declared checksums before persistence | Auth failure, validation failure, checksum mismatch, storage unavailable, `UNIMPLEMENTED` if storage is absent | Intentionally unimplemented |
| `CreateArtifactFromUrl` | Optional publisher capability | Fetch immutable content, verify declared checksums, register artifact metadata | Auth failure, validation failure, checksum mismatch, unsafe source URL, fetch failure | Implemented |
| `DeleteArtifact` | Optional publisher capability | Delete artifact only when not referenced, unless force-delete semantics are explicitly supported | Auth failure, not found, `FAILED_PRECONDITION` when referenced, `UNIMPLEMENTED` for unsupported force delete | Implemented |
| `CreateCollection` | Optional publisher capability | Create version 1 for a logical collection UUID; referenced artifacts must exist | Auth failure, validation failure, missing subject/artifact | Implemented |
| `UpdateCollection` | Optional publisher capability | Create a new immutable version; prior versions remain addressable | Auth failure, validation failure, missing collection/artifact | Implemented |
| `SignCollection` | Optional publisher capability | Sign one collection version and return updated metadata | Auth failure, validation failure, signing disabled, signer unavailable, `UNIMPLEMENTED` until a real signing/storage flow exists | Intentionally unimplemented |
| `BatchUploadArtifacts` | Advanced optional publisher capability | Accept one session containing multiple artifact uploads with per-item results | Auth failure, validation failure, storage unavailable, `UNIMPLEMENTED` until backed by real storage | Intentionally unimplemented |
| `ImportCollection` | Advanced optional publisher capability | Import collection plus artifact data for migration scenarios | Auth failure, validation failure, storage unavailable, `UNIMPLEMENTED` until backed by complete migration semantics | Intentionally unimplemented |

## Notes for implementers

- “Implemented” in the Rust reference server means the flow exists with real
  validation and persistence behavior, not a placeholder success path.
- “Intentionally unimplemented” means the operation remains part of the TEA
  contract, but the reference server returns an explicit failure until the
  required backing semantics are mature enough to demonstrate safely.
- Current executable coverage for this matrix lives primarily in
  `tea-server/tests/publisher_conformance.rs`, with happy-path transport checks
  in `tea-server/tests/grpc_smoke.rs` and additional capability coverage in
  `tea-server/tests/publisher_capability_coverage.rs`.
- `spec/publisher/conformance-checklist.json` mirrors this matrix in a
  machine-readable form for future automation.
- `spec/publisher/openapi.json` mirrors the publisher HTTP bindings declared in
  `proto/tea/v1/publisher.proto`, including explicit reference-profile status
  per RPC.
- `tools/generate_publisher_openapi.py` is the checked-in source for
  `spec/publisher/openapi.json`, including concrete request/response examples
  for the draft HTTP profile.
- `tools/validate_publisher_conformance.py` validates that checklist entries and
  linked test references stay in sync.
- `tools/validate_publisher_openapi.py` validates that the publisher OpenAPI
  profile stays in parity with the protobuf HTTP annotations and the
  machine-readable checklist.
- New publisher work should update this matrix whenever the contract or
  reference profile changes.
