# TEA Product Release

## Overview

A TEA Product Release represents a specific versioned release of a TEA Product. It is the primary resolvable entity via TEI and the entry point for discovery of included components and related collections of security artefacts.

Key attributes:
- uuid: Unique identifier of the product release
- product: UUID of the TEA Product this release belongs to
- version: Human-readable version string of the product release
- createdDate: Timestamp when the product release was created in TEA
- releaseDate: Upstream product release timestamp
- preRelease: Indicates pre-release/beta status
- identifiers: Array of identifiers (idType: CPE/TEI/PURL; idValue: string)
- components: Array of component references included in this product release
  - uuid: UUID of the TEA Component
  - release: Optional UUID of a specific component release to pin an exact version

Collections for a product release contain artefacts relevant to that product release.

## JSON examples

The following example is reused from the OpenAPI schema (`components/schemas/productRelease.examples`), ensuring exact field names and casing.

```json
{
  "uuid": "123e4567-e89b-12d3-a456-426614174000",
  "version": "2.24.3",
  "createdDate": "2025-04-01T15:43:00Z",
  "releaseDate": "2025-04-01T15:43:00Z",
  "identifiers": [
    {
      "idType": "TEI",
      "idValue": "tei:vendor:product@2.24.3"
    }
  ],
  "components": [
    {
      "uuid": "3910e0fd-aff4-48d6-b75f-8bf6b84687f0"
    },
    {
      "uuid": "b844c9bd-55d6-478c-af59-954a932b6ad3",
      "release": "da89e38e-95e7-44ca-aa7d-f3b6b34c7fab"
    }
  ]
}
```

Notes:
- Property `product` exists in the schema and links a product release to its parent product; it may not be present in all examples.
- Use uppercase idType values exactly as defined by the schema enum: CPE, TEI, PURL.
