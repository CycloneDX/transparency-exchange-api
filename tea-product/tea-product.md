# The TEA Product object

After TEA discovery, the [Transparency Exchange Identifier (TEI)](/discovery/readme.md) resolves to one or more TEA Product Releases, each a concrete, versioned offering; a TEI normally resolves to exactly one. A TEA Product is a higher-level object that groups multiple TEA Product Releases for a product line or family and can be browsed via `/product/{uuid}/releases`.

A TEA Product Release contains component references. Each reference identifies a TEA Component and should also identify a particular TEA Component Release.

In addition, all known TEIs for the product will be returned,
in order for a TEA client to avoid duplication. This list can
also include known Package URLs (PURL) and CPEs for the product.

## Authorization

Authorization can be done on multiple levels, including
which products and versions are supported for a specific user.

## Composite products

A TEA Product Release is the starting point of discovery. It contains
component references: each identifies a TEA Component by UUID, and should
also identify a particular TEA Component Release by UUID. A reference without a
release UUID does not select a TEA Component Release.

The __TEA Product__ object groups multiple releases together.

## Structure

Key attributes:

- __uuid__: A unique identifier for the TEA product
- __name__: Product name
- __identifiers__: List of identifiers for the product
   - __idType__: Type of identifier, e.g. `TEI`, `PURL`, `CPE`
   - __idValue__: Identifier value

Required fields:

- uuid, name, identifiers

### Example

An example of a product consisting of an OSS project and all its Maven artefacts:

```json
{
  "uuid": "09e8c73b-ac45-4475-acac-33e6a7314e6d",
  "name": "Apache Log4j 2",
  "identifiers": [
    {
      "idType": "CPE",
      "idValue": "cpe:2.3:a:apache:log4j"
    },
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-api"
    },
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    },
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-layout-template-json"
    }
  ]
}
```

Releases for this Product can be browsed via the API endpoint `/product/{uuid}/releases`.

### API usage

The user will find this API end point using TEA discovery.

A user will approach the API just to discover data before purchase,
or with a specific product and product version in scope.
The format of the version string can follow many syntaxes. This specification
does not constrain that format.

An automated system may want to provide the user with a GUI,
listing versions and being able to scroll to the next page
until the user selects a version.
