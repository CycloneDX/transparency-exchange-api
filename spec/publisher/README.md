# TEA Publisher API Profile

The publisher API is a recommended TEA capability for systems that need to
register and manage product transparency metadata, artifacts, and collections.

## Canonical source of truth

For publisher semantics, the canonical contract is:

- [`proto/tea/v1/publisher.proto`](../../proto/tea/v1/publisher.proto) for the
  RPC surface, message shapes, and lifecycle rules
- [`proto/README.md`](../../proto/README.md) for the current reference-server
  coverage and safe-subset guidance

The OpenAPI material in this directory should be treated as a draft HTTP
profile, not the sole normative source for publisher behavior.

## Reference implementation posture

The Rust `tea-server` is an executable reference implementation, not a claim
that every optional publisher capability is production-complete.

- Stable publisher metadata workflows are implemented.
- Advanced publisher capabilities that still need storage, signing, or bulk
  import semantics remain intentionally unsupported.
- Unsupported operations must fail explicitly, typically with
  `UNIMPLEMENTED`, rather than partially succeeding.

## Current documents

- `openapi.json`: draft HTTP publisher profile aligned with the canonical
  `google.api.http` annotations in `proto/tea/v1/publisher.proto`
- `../../tools/generate_publisher_openapi.py`: checked-in generator for
  `openapi.json`, including concrete request/response examples for the
  reference-supported publisher flows
- `../../tools/render_sbom_tools_publisher_examples.py`: generator for
  `../../docs/generated/sbom-tools-publisher-profile-examples.md`, which turns
  the publisher OpenAPI examples into integration-ready `sbom-tools` snippets
- `../../tools/render_sbom_tools_reqwest_snippets.py`: generator for
  `../../docs/generated/sbom-tools-publisher-reqwest-snippets.md`, which turns
  the same publisher-profile examples into Rust/`reqwest` integration helpers
- `../../tools/render_aggregate_openapi_publisher_fragment.py`: generator for
  `../../spec/generated/publisher-profile-fragment.yaml`, which carries the
  publisher-profile-backed fragment synced into the aggregate `spec/openapi.yaml`
- `../../tools/render_publisher_conformance_report.py`: generator for the CI
  publisher conformance/parity report artifacts and GitHub summary snippets
- `../../tools/sync_aggregate_openapi_publisher_block.py`: generator that keeps
  the publisher summary block in `../../spec/openapi.yaml` aligned with the
  canonical publisher profile inputs
- `../../tools/build_publisher_release_doc_bundle.py`: generator for a
  lightweight release-doc bundle that packages the HTML conformance report with
  the generated integration docs, plus a sibling `.tar.gz` archive and checksum
- `../../tools/validate_publisher_release_doc_bundle.py`: validator for the
  release-doc bundle manifest, file checksums, and packaged archive
- `conformance-matrix.md`: current canonical-vs-reference behavior matrix
- `conformance-checklist.json`: machine-readable publisher conformance checklist
- `../../tools/validate_publisher_openapi.py`: validator that keeps
  `openapi.json`, the protobuf HTTP bindings, and checklist reference status in
  sync
- `../../tea-server/tests/publisher_conformance.rs`: executable reference checks
  for explicit publisher behavior and failure semantics
- `../../tools/validate_publisher_conformance.py`: CI validator for checklist
  structure and linked test references

## What to update first

When extending the publisher surface:

1. update the protobuf contract first
2. document lifecycle semantics and failure behavior
3. align the reference implementation
4. then refresh any HTTP/OpenAPI profile material

## Regeneration

When the publisher contract changes, regenerate and validate the HTTP profile
with:

```bash
python3 tools/generate_publisher_openapi.py
python3 tools/render_sbom_tools_publisher_examples.py
python3 tools/render_sbom_tools_reqwest_snippets.py
python3 tools/render_aggregate_openapi_publisher_fragment.py
python3 tools/sync_aggregate_openapi_publisher_block.py
python3 tools/validate_publisher_openapi.py
python3 tools/build_publisher_release_doc_bundle.py
python3 tools/validate_publisher_release_doc_bundle.py --bundle-dir dist/publisher-release-doc-bundle
```
