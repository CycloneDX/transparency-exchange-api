# TEA OpenAPI Spec

The OpenAPI 3.1 specification for the Transparency Exchange API is available in [openapi.json](./openapi.json).

- [Generating API Clients from OpenAPI Spec](#generating-api-clients-from-openapi-spec)

## Generating API Clients from OpenAPI Spec

We use the OpenAPI Generator with configuration per language/framework in the `generators` folder. An example is:

```
docker run --rm -v "$(PWD):/local" openapitools/openapi-generator-cli batch --clean /local/spec/generators/typescript.yaml
```