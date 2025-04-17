# TEA OpenAPI Spec

The OpenAPI 3.1 specification for the Transparency Exchange API is available in
[openapi.json](./openapi.json).

- [Generating API Clients from OpenAPI Spec](#generating-api-clients-from-openapi-spec)

## Generating API Clients from OpenAPI Spec

We use the OpenAPI Generator with configuration per language/framework in the
`generators` folder. An example is:

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
    -p 8080:8080 \
    -e SWAGGER_JSON=/koala/spec/openapi.json \
    -v $(pwd):/koala swaggerapi/swagger-ui
```

And browse to [http://localhost:8080](http://localhost:8080).
