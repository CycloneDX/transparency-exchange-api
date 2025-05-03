# The TEA product API

After TEA discovery, a user has a URL to the TEA product API where
the TEI is used to query for data. The TEI marks the product sold,
which can be a single unit or multiple units in a bundle.

- For a single product, the output will be metadata about the
  product and a TEA COMPONENT object.
- For a composed product consisting of a bundle, the response
  will be multiple TEA COMPONENT objects.

In addition, all known TEIs for the product will be returned,
in order for a TEA client to avoid duplication.

## Authorization

Authorization can be done on multiple levels, including
which products and versions are supported for a specific user.

## TEA Product object

A TEA Product object has the following parts:

- __uuid__: A unique identifier for the TEA product
- __name__: Product name
- __identifiers__: List of identifiers for the product
   - __idType__: Type of identifier, e.g. `tei`, `purl`, `cpe`
   - __idValue__: Identifier value
- __components__: List of TEA components for the product
   - __uuid__: Unique identifier of the TEA component

The TEA Component UUID is used in the Component API to find out which versions
of the Component that exists.

The goal of the TEA Product API is to provide a selection of product
versions to assist the user software in finding a match for the
owned version.

### Example

An example of a product consisting of an OSS project and all its Maven artifacts:

```json
{
  "uuid": "09e8c73b-ac45-4475-acac-33e6a7314e6d",
  "name": "Apache Log4j 2",
  "identifiers": [
    {
      "idType": "cpe",
      "idValue": "cpe:2.3:a:apache:log4j"
    },
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-api"
    },
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    },
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-layout-template-json"
    }
  ],
  "components": [
    "3910e0fd-aff4-48d6-b75f-8bf6b84687f0",
    "b844c9bd-55d6-478c-af59-954a932b6ad3",
    "d6d3f754-d4f4-4672-b096-b994b064ca2d"
  ]
}
```

### API usage

The user will find this API end point using TEA discovery.

A user will approach the API just to discover data before purchase,
or with a specific product and product version in scope.
The format of the version may follow many syntaxes, so maybe
the API needs to be able to provide some sort of format
for the version string.

An automated system may want to provide the user with a GUI,
listing versions and being able to scroll to the next page
until the user selects a version.

### Tea API operation

* Recommendation: Support HTTP compression
* Recommendation: HTTP content negotiation
  * Like "I prefer JSON, but can accept XML"
* Pagination support
  * max per page
  * start page
  * Default value defined


