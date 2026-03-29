# Generated sbom-tools Publisher Profile Examples

This document is generated from `spec/publisher/openapi.json` by
`tools/render_sbom_tools_publisher_examples.py`.

These examples target the draft canonical publisher HTTP profile under
`/v1/publisher/...`. They are useful when `sbom-tools` integrates through a
transcoding gateway or another HTTP layer that follows the publisher protobuf
contract.

Important: these examples are not the same thing as the current
`tea-server`-specific HTTP write surface described in `docs/sbom-tools-integration.md`.
Use this generated document when you want the canonical publisher payload
shapes, not the reference server's bespoke HTTP handlers.

## Environment

```bash
export TEA_PROFILE_BASE_URL=http://localhost:8080
export TEA_TOKEN=replace-with-a-real-writer-token
export COLLECTION_UUID=44444444-4444-4444-8444-444444444444
```

## `CreateProduct`

- HTTP: `POST /v1/publisher/products`
- Example request: `log4jProduct`
- Example response: `createdProduct`

Request payload:

```json
{
  "uuid": "11111111-1111-4111-8111-111111111111",
  "name": "Apache Log4j 2",
  "description": "Java logging framework product line maintained by Apache.",
  "identifiers": [
    {
      "idType": "IDENTIFIER_TYPE_PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    }
  ],
  "vendor": {
    "name": "Apache Software Foundation",
    "url": "https://logging.apache.org/log4j/2.x/"
  },
  "homepageUrl": "https://logging.apache.org/log4j/2.x/",
  "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/",
  "vcsUrl": "https://github.com/apache/logging-log4j2"
}
```

Example `curl` call:

```bash
curl -sS \
  -H "Authorization: Bearer ${TEA_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST ${TEA_PROFILE_BASE_URL}/v1/publisher/products \
  -d @<(cat <<'JSON'
{
  "uuid": "11111111-1111-4111-8111-111111111111",
  "name": "Apache Log4j 2",
  "description": "Java logging framework product line maintained by Apache.",
  "identifiers": [
    {
      "idType": "IDENTIFIER_TYPE_PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    }
  ],
  "vendor": {
    "name": "Apache Software Foundation",
    "url": "https://logging.apache.org/log4j/2.x/"
  },
  "homepageUrl": "https://logging.apache.org/log4j/2.x/",
  "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/",
  "vcsUrl": "https://github.com/apache/logging-log4j2"
}
JSON
)
```

Created product response:

```json
{
  "uuid": "11111111-1111-4111-8111-111111111111",
  "name": "Apache Log4j 2",
  "description": "Java logging framework product line maintained by Apache.",
  "identifiers": [
    {
      "idType": "IDENTIFIER_TYPE_PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    }
  ],
  "vendor": {
    "name": "Apache Software Foundation",
    "url": "https://logging.apache.org/log4j/2.x/"
  },
  "createdDate": "2026-03-28T15:00:00Z",
  "modifiedDate": "2026-03-28T15:00:00Z",
  "homepageUrl": "https://logging.apache.org/log4j/2.x/",
  "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/",
  "vcsUrl": "https://github.com/apache/logging-log4j2"
}
```

## `CreateArtifactFromUrl`

- HTTP: `POST /v1/publisher/artifacts/from-url`
- Example request: `artifactFromUrl`
- Example response: `createdArtifact`

Request payload:

```json
{
  "metadata": {
    "uuid": "55555555-5555-4555-8555-555555555555",
    "name": "Log4j SBOM",
    "type": "ARTIFACT_TYPE_BOM",
    "mimeType": "application/vnd.cyclonedx+json",
    "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
    "subject": {
      "type": "SUBJECT_TYPE_COMPONENT",
      "name": "Apache Log4j Core",
      "version": "2.24.3",
      "identifiers": [
        {
          "idType": "IDENTIFIER_TYPE_PURL",
          "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3"
        }
      ]
    },
    "componentDistributions": [
      "jar"
    ],
    "specVersion": "1.6",
    "expectedChecksums": [
      {
        "algType": "CHECKSUM_ALGORITHM_SHA256",
        "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
      }
    ]
  },
  "sourceUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
  "signatureUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json.sig",
  "expectedChecksums": [
    {
      "algType": "CHECKSUM_ALGORITHM_SHA256",
      "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }
  ]
}
```

Example `curl` call:

```bash
curl -sS \
  -H "Authorization: Bearer ${TEA_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST ${TEA_PROFILE_BASE_URL}/v1/publisher/artifacts/from-url \
  -d @<(cat <<'JSON'
{
  "metadata": {
    "uuid": "55555555-5555-4555-8555-555555555555",
    "name": "Log4j SBOM",
    "type": "ARTIFACT_TYPE_BOM",
    "mimeType": "application/vnd.cyclonedx+json",
    "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
    "subject": {
      "type": "SUBJECT_TYPE_COMPONENT",
      "name": "Apache Log4j Core",
      "version": "2.24.3",
      "identifiers": [
        {
          "idType": "IDENTIFIER_TYPE_PURL",
          "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3"
        }
      ]
    },
    "componentDistributions": [
      "jar"
    ],
    "specVersion": "1.6",
    "expectedChecksums": [
      {
        "algType": "CHECKSUM_ALGORITHM_SHA256",
        "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
      }
    ]
  },
  "sourceUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
  "signatureUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json.sig",
  "expectedChecksums": [
    {
      "algType": "CHECKSUM_ALGORITHM_SHA256",
      "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }
  ]
}
JSON
)
```

Registered artifact response:

```json
{
  "uuid": "55555555-5555-4555-8555-555555555555",
  "name": "Log4j SBOM",
  "type": "ARTIFACT_TYPE_BOM",
  "componentDistributions": [
    "jar"
  ],
  "formats": [
    {
      "mimeType": "application/vnd.cyclonedx+json",
      "description": "CycloneDX SBOM (JSON)",
      "url": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
      "signatureUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json.sig",
      "checksums": [
        {
          "algType": "CHECKSUM_ALGORITHM_SHA256",
          "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }
      ],
      "sizeBytes": 20480,
      "specVersion": "1.6"
    }
  ],
  "createdDate": "2026-03-28T15:05:00Z",
  "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
  "subject": {
    "type": "SUBJECT_TYPE_COMPONENT",
    "name": "Apache Log4j Core",
    "version": "2.24.3",
    "identifiers": [
      {
        "idType": "IDENTIFIER_TYPE_PURL",
        "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3"
      }
    ]
  }
}
```

## `CreateCollection`

- HTTP: `POST /v1/publisher/collections`
- Example request: `collectionV1`
- Example response: `collection`

Request payload:

```json
{
  "uuid": "44444444-4444-4444-8444-444444444444",
  "belongsTo": "COLLECTION_SCOPE_RELEASE",
  "artifactUuids": [
    "55555555-5555-4555-8555-555555555555"
  ],
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_INITIAL_RELEASE",
    "comment": "Initial transparency artifact publication.",
    "affectedArtifactUuids": [
      "55555555-5555-4555-8555-555555555555"
    ]
  }
}
```

Example `curl` call:

```bash
curl -sS \
  -H "Authorization: Bearer ${TEA_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST ${TEA_PROFILE_BASE_URL}/v1/publisher/collections \
  -d @<(cat <<'JSON'
{
  "uuid": "44444444-4444-4444-8444-444444444444",
  "belongsTo": "COLLECTION_SCOPE_RELEASE",
  "artifactUuids": [
    "55555555-5555-4555-8555-555555555555"
  ],
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_INITIAL_RELEASE",
    "comment": "Initial transparency artifact publication.",
    "affectedArtifactUuids": [
      "55555555-5555-4555-8555-555555555555"
    ]
  }
}
JSON
)
```

Created collection response:

```json
{
  "uuid": "44444444-4444-4444-8444-444444444444",
  "version": 2,
  "date": "2026-03-28T15:06:00Z",
  "belongsTo": "COLLECTION_SCOPE_RELEASE",
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
    "comment": "Add VEX alongside the original SBOM.",
    "affectedArtifactUuids": [
      "66666666-6666-4666-8666-666666666666"
    ]
  },
  "artifacts": [
    {
      "uuid": "55555555-5555-4555-8555-555555555555",
      "name": "Log4j SBOM",
      "type": "ARTIFACT_TYPE_BOM",
      "formats": [
        {
          "mimeType": "application/vnd.cyclonedx+json",
          "url": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
          "checksums": [
            {
              "algType": "CHECKSUM_ALGORITHM_SHA256",
              "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            }
          ]
        }
      ]
    }
  ],
  "createdDate": "2026-03-28T15:06:00Z",
  "conformanceVectors": [
    "OWASP SCVS",
    "SLSA"
  ]
}
```

## `UpdateCollection`

- HTTP: `POST /v1/publisher/collections/{uuid}/versions`
- Example request: `nextVersion`
- Example response: `collectionV2`
- Note: Replace `${COLLECTION_UUID}` with the logical collection UUID.

Request payload:

```json
{
  "artifactUuids": [
    "55555555-5555-4555-8555-555555555555",
    "66666666-6666-4666-8666-666666666666"
  ],
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
    "comment": "Add VEX alongside the original SBOM.",
    "affectedArtifactUuids": [
      "66666666-6666-4666-8666-666666666666"
    ]
  }
}
```

Example `curl` call:

```bash
curl -sS \
  -H "Authorization: Bearer ${TEA_TOKEN}" \
  -H "Content-Type: application/json" \
  -X POST ${TEA_PROFILE_BASE_URL}/v1/publisher/collections/${COLLECTION_UUID}/versions \
  -d @<(cat <<'JSON'
{
  "artifactUuids": [
    "55555555-5555-4555-8555-555555555555",
    "66666666-6666-4666-8666-666666666666"
  ],
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
    "comment": "Add VEX alongside the original SBOM.",
    "affectedArtifactUuids": [
      "66666666-6666-4666-8666-666666666666"
    ]
  }
}
JSON
)
```

Next collection version response:

```json
{
  "uuid": "44444444-4444-4444-8444-444444444444",
  "version": 2,
  "date": "2026-03-28T15:06:00Z",
  "belongsTo": "COLLECTION_SCOPE_RELEASE",
  "updateReason": {
    "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
    "comment": "Add VEX alongside the original SBOM.",
    "affectedArtifactUuids": [
      "66666666-6666-4666-8666-666666666666"
    ]
  },
  "artifacts": [
    {
      "uuid": "55555555-5555-4555-8555-555555555555",
      "name": "Log4j SBOM",
      "type": "ARTIFACT_TYPE_BOM",
      "formats": [
        {
          "mimeType": "application/vnd.cyclonedx+json",
          "url": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
          "checksums": [
            {
              "algType": "CHECKSUM_ALGORITHM_SHA256",
              "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            }
          ]
        }
      ]
    }
  ],
  "createdDate": "2026-03-28T15:06:00Z",
  "conformanceVectors": [
    "OWASP SCVS",
    "SLSA"
  ]
}
```
