# TEA Component Release Object

## Overview

A __TEA Component Release__ represents a specific version of a TEA Component lineage. It is the concrete, versioned entity which has collections of security-related artifacts (SBOM, VDR/VEX, attestations, etc.).

Key attributes:

- __uuid__: Unique identifier of the TEA Component Release (uuid)
- __component__: UUID of the TEA Component this release belongs to
- __componentName__: Name of the TEA Component this release belongs to
- __version__: Human-readable version string
- __createdDate__: Timestamp when this Release was created in TEA (for sorting purposes)
- __releaseDate__: Timestamp of the release
- __preRelease__: A flag indicating pre-release (or beta) status. May be disabled after the creation of the release object, but can't be enabled after creation of an object.
- __identifiers__: Array of identifiers for the component
- __distributions__: List of different formats of this component release

Collections for a release contain artifacts relevant to that specific release.

Required fields:

- uuid, version, createdDate

## TEA component release distribution object

Distribution are object to declare different distribution formats of a component,
like source code or a package for a specific Linux distribution or a CPU type

Key attributes:

- __distributionId__: A unique identifier for the TEA Distribution object (uuid)
- __description__: Free-text description of the distribution.
- __identifiers__: Array of identifiers for the distribution of the release
- __url__: Direct download URL for the distribution
- __signatureUrl__: Direct download URL for the distribution's detached signature
- __checksums__: Array of checksums for the distribution

Required fields:

- distributionId

## JSON examples

The following examples are reused from the OpenAPI schema (`components/schemas/release.examples`), ensuring exact field names and casing.

```json
{
  "uuid": "605d0ecb-1057-40e4-9abf-c400b10f0345",
  "version": "11.0.6",
  "createdDate": "2025-04-01T15:43:00Z",
  "releaseDate": "2025-04-01T15:43:00Z",
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.6"
    }
  ]
}
```

```json
{
  "uuid": "da89e38e-95e7-44ca-aa7d-f3b6b34c7fab",
  "version": "10.1.40",
  "createdDate": "2025-04-01T18:20:00Z",
  "releaseDate": "2025-04-01T18:20:00Z",
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@10.1.40"
    }
  ]
}
```

```json
{
  "uuid": "95f481df-f760-47f4-b2f2-f8b76d858450",
  "version": "11.0.0-M26",
  "createdDate": "2024-09-13T17:49:00Z",
  "preRelease": true,
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.0-M26"
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


Notes:
- Use uppercase idType values exactly as defined by the schema enum: CPE, TEI, PURL.
