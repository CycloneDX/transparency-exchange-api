# TEA Product Release object

## Overview

A TEA Product Release represents a specific versioned release of a TEA Product. It is the primary resolvable entity via TEI and the entry point for discovery of included components and related collections of security artefacts.

Key attributes:

- __uuid__: A unique identifier for the TEA Product Release
- __product__: UUID of the TEA Product this release belongs to
- __version__: Human-readable version string of the product release
- __createdDate__: Timestamp when the product release was created in TEA (for sorting purposes)
- __releaseDate__: Timestamp of the product release
- __preRelease__: A flag indicating pre-release (or beta) status. May be disabled after the creation of the release object, but can't be enabled after creation of an object. (boolean)
- __identifiers__: Array of identifiers for the product release (idType: CPE/TEI/PURL; idValue: string)
- __components__: Array of component references that compose this product release. A component reference can optionally include the UUID of a specific component release to pin the exact version.

Required fields:

- uuid, version, createdDate, components

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

## Handling the Pre-Release flag

The "Pre-release" flag is used to indicate that this is not a final release.
For a given Component with a UUID, the flag can be set to indicate a "test", "beta", "alpha"
or similar non-deployed release. It can only be set when creating the Component.
The TEA implementation may allow it to be unset (False) once. This is to support
situations where a object is promoted as is after testing to production version. The flag can not
be set after initial creation and publication of the Component.

If the final version is different from the pre-release (bugs fixed, code changed, different binary)
a new Component with a new UUID and version needs to be created.


## Notes

- Property `product` exists in the schema and links a product release to its parent product; it may not be present in all examples.
- Use uppercase idType values exactly as defined by the schema enum: CPE, TEI, PURL.
