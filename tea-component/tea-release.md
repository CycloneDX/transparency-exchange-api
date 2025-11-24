# TEA Component Release

## Overview

A TEA Component Release represents a specific version of a TEA Component lineage. It is the concrete, versioned entity which has collections of security-related artefacts (SBOM, VDR/VEX, attestations, etc.).

Key attributes:
- uuid: Unique identifier of the component release
- component: UUID of the TEA Component this release belongs to
- version: Human-readable version string
- createdDate: Timestamp when the release was created in TEA
- releaseDate: Upstream release timestamp
- preRelease: Indicates pre-release/beta status
- identifiers: Array of identifiers (idType: CPE/TEI/PURL; idValue: string)

Collections for a release contain artefacts relevant to that specific release.

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

Notes:
- Use uppercase idType values exactly as defined by the schema enum: CPE, TEI, PURL.
