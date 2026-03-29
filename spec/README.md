# TEA OpenAPI Spec

The OpenAPI 3.1 specification for the Transparency Exchange API is available in [openapi.yaml](./openapi.yaml).

- `spec/openapi.yaml` currently represents the draft HTTP profile for TEA.
- The protobuf contracts in [`proto/`](../proto) are the canonical source for
  publisher RPC semantics and current reference-server interoperability rules.
- For the current publisher reference profile, see
  [`spec/publisher/README.md`](./publisher/README.md) and
  [`spec/publisher/conformance-matrix.md`](./publisher/conformance-matrix.md).
- For the publisher-specific OpenAPI profile that mirrors the canonical
  publisher HTTP bindings, see
  [`spec/publisher/openapi.json`](./publisher/openapi.json).
- That publisher profile is generated from
  [`tools/generate_publisher_openapi.py`](../tools/generate_publisher_openapi.py)
  and validated in CI.
- The aggregate `spec/openapi.yaml` also carries a generated publisher summary
  block that is synced by
  [`tools/sync_aggregate_openapi_publisher_block.py`](../tools/sync_aggregate_openapi_publisher_block.py).
- That aggregate block is backed by the generated fragment
  [`spec/generated/publisher-profile-fragment.yaml`](./generated/publisher-profile-fragment.yaml),
  rendered by
  [`tools/render_aggregate_openapi_publisher_fragment.py`](../tools/render_aggregate_openapi_publisher_fragment.py).
- Publisher release review artifacts can also be packaged locally with
  [`tools/build_publisher_release_doc_bundle.py`](../tools/build_publisher_release_doc_bundle.py)
  and validated with
  [`tools/validate_publisher_release_doc_bundle.py`](../tools/validate_publisher_release_doc_bundle.py).

- [Generating API Clients from OpenAPI Spec](#generating-api-clients-from-openapi-spec)

## Generating API Clients from OpenAPI Spec

We use the OpenAPI Generator with configuration per language/framework in the `generators` folder. An example is:

```bash
docker run \
    --rm \
    -v "$(PWD):/local" \
    openapitools/openapi-generator-cli \
    batch --clean /local/spec/generators/typescript.yaml
```

## Preview Specs

Fire up the `swagger-ui` with Docker from the root of the repository:

```bash
docker run \
    -p 87348080 \
    -e SWAGGER_JSON=/koala/spec/openapi.yaml \
    -v $(pwd):/koala swaggerapi/swagger-ui
```

And browse to [http://localhost:8734](http://localhost:8734).
