# TEA Component Release Object

## Overview

A __TEA Component Release__ represents a specific version of a TEA Component lineage. It is the concrete, versioned entity which has collections of security-related artifacts (SBOM, VDR/VEX, attestations, etc.).

A TEA Component Release object represents a specific version of a component,
identified by a unique version number and associated metadata.
Each release may include multiple distributions,
which capture variations such as architecture, packaging, or localization.

- For software components,
  each distribution typically corresponds to a different digital file delivered to users
  (e.g., by platform or packaging type).
- For hardware components, distributions may reflect differences in packaging, language, or other physical attributes.

Each distribution is assigned a unique `distributionType`, defined by the producer,
which is used to associate relevant TEA Artifacts with that distribution.
Since TEA Artifacts can be associated with multiple release objects,
the taxonomy for `distributionType` values should be defined on a TEA service level
and consistently applied to all TEA Artifacts published by that producer.
This ensures global uniqueness and reliable association across releases.

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
    - __algType__: Checksum algorithm used.
    - __algValue__: Checksum value.

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

### Examples

#### Single distribution

This example shows a TEA Component Release for Apache Log4j Core, which is distributed as a single JAR file.
Even though there is only one distribution,
the `distributions` attribute is included to enable searching for the release by the SHA-256 checksum of the JAR.
This structure also allows for future extensibility if additional distributions are introduced.

<details>
  <summary>Example of simple release</summary>

```json
{
  "uuid": "b1e2c3d4-5678-49ab-9cde-123456789abc",
  "version": "2.24.3",
  "createdDate": "2024-12-10T10:51:00Z",
  "releaseDate": "2024-12-13T12:52:29Z",
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3"
    }
  ],
  "distributions": [
    {
      "distributionType": "jar",
      "description": "Binary distribution",
      "identifiers": [
        {
          "idType": "PURL",
          "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3?type=jar"
        }
      ],
      "checksums": [
        {
          "algType": "SHA-256",
          "algValue": "b1e2c3d4f5a67890b1e2c3d4f5a67890b1e2c3d4f5a67890b1e2c3d4f5a67890"
        }
      ],
      "url": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3.jar",
      "signatureUrl": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3.jar.asc"
    }
  ]
}
```
</details>

#### Multiple distributions

This is an example of a TEA Component Release for Apache Tomcat 11.0.7 binary distributions.
The example defines four distinct `distributionType`s,
which is essential not only for associating the correct SBOMs with each distribution,
but also for accurately tracking and reporting vulnerabilities that may affect only specific distributions.
For instance:

- The `zip` and `tar.gz` distributions contain only Java JARs.
- The `windows-x64.zip` distribution additionally includes the
  [Apache Procrun](https://commons.apache.org/proper/commons-daemon/procrun.html) binary,
  which is specific to Windows and may introduce unique vulnerabilities.
- The `windows-x64.exe` distribution contains the same data as `windows-x64.zip`,
  but is packaged as a self-extracting installer
  created by the [Nullsoft Scriptable Install System](https://nsis.sourceforge.io/Main_Page).

By defining separate `distributionType`s,
it becomes possible to precisely associate artefacts and vulnerability disclosures with the affected distributions,
ensuring accurate risk assessment and remediation.

<details>
  <summary>Example of four different binary distributions in the same release</summary>

```json
{
  "uuid": "605d0ecb-1057-40e4-9abf-c400b10f0345",
  "version": "11.0.7",
  "createdDate": "2025-05-07T18:08:00Z",
  "releaseDate": "2025-05-12T18:08:00Z",
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.7"
    }
  ],
  "distributions": [
    {
      "distributionType": "zip",
      "description": "Core binary distribution, zip archive",
      "identifiers": [
        {
          "idType": "PURL",
          "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.7?type=zip"
        }
      ],
      "checksums": [
        {
          "algType": "SHA_256",
          "algValue": "9da736a1cdd27231e70187cbc67398d29ca0b714f885e7032da9f1fb247693c1"
        }
      ],
      "url": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7.zip",
      "signatureUrl": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7.zip.asc"
    },
    {
      "distributionType": "tar.gz",
      "description": "Core binary distribution, tar.gz archive",
      "identifiers": [
        {
          "idType": "PURL",
          "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.7?type=tar.gz"
        }
      ],
      "checksums": [
        {
          "algType": "SHA_256",
          "algValue": "2fcece641c62ba1f28e1d7b257493151fc44f161fb391015ee6a95fa71632fb9"
        }
      ],
      "url": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7.tar.gz",
      "signatureUrl": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7.tar.gz.asc"
    },
    {
      "distributionType": "windows-x64.zip",
      "description": "Core binary distribution, Windows x64 zip archive",
      "identifiers": [
        {
          "idType": "PURL",
          "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.7?classifier=windows-x64&type=zip"
        }
      ],
      "checksums": [
        {
          "algType": "SHA_256",
          "algValue": "62a5c358d87a8ef21d7ec1b3b63c9bbb577453dda9c00cbb522b16cee6c23fc4"
        }
      ],
      "url": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7-windows-x64.zip",
      "signatureUrl": "https://repo.maven.apache.org/maven2/org/apache/tomcat/tomcat/11.0.7/tomcat-11.0.7.zip.asc"
    },
    {
      "distributionType": "windows-x64.exe",
      "description": "Core binary distribution, Windows Service Installer (MSI)",
      "checksums": [
        {
          "algType": "SHA_512",
          "algValue": "1d3824e7643c8aba455ab0bd9e67b14a60f2aaa6aa7775116bce40eb0579e8ced162a4f828051d3b867e96ee2858ec5da0cc654e83a83ba30823cbea0df4ff96"
        }
      ],
      "url": "https://dlcdn.apache.org/tomcat/tomcat-11/v11.0.7/bin/apache-tomcat-11.0.7.exe",
      "signatureUrl": "https://downloads.apache.org/tomcat/tomcat-11/v11.0.7/bin/apache-tomcat-11.0.7.exe.asc"
    }
  ]
}
```
</details>


## Handling the Pre-Release flag

#### Pre-release flag usage

The `preRelease` flag is used to indicate that a release is not production ready,
regardless of the version naming scheme.
This helps consumers identify non-production releases without relying on conventions like `-beta`,
`-rc`, or `-M` in the version string.

There are two main scenarios for using the `preRelease` flag:

- **Pending release:** The distribution is still undergoing quality assurance or review and is not yet officially released.
  These typically lack a `releaseDate` attribute.
  Once the release is approved, the `preRelease` flag is set to `false` and the `releaseDate` is added.
- **Permanent pre-release:** The distribution is intentionally marked as a pre-release
 (e.g., beta, milestone, or release candidate) and will never be considered production ready,
  even after all checks are complete.
  These may have a `releaseDate`, but `preRelease` remains `true`.

<details>
  <summary>Examples of non-production ready distributions</summary>

- **Pending release (no `releaseDate`):**
  ```json
  {
    "uuid": "e2a1c7b4-3f2d-4e8a-9c1a-7b2e4d5f6a8b",
    "version": "11.0.0",
    "createdDate": "2025-09-01T00:00:00Z",
    "preRelease": true,
    "identifiers": [
      {
        "idType": "PURL",
        "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.0?repository_url=https:%2F%2Frepository.apache.org%2Fcontent%2Fgroups%2Fstaging%2F"
      }
    ]
  }
  ```
- **Transition to production-ready (`preRelease` flag turned off, `releaseDate` added)**:
  ```json
  {
    "uuid": "e2a1c7b4-3f2d-4e8a-9c1a-7b2e4d5f6a8b",
    "version": "11.0.0",
    "createdDate": "2025-09-01T00:00:00Z",
    "releaseDate": "2025-09-10T12:00:00Z",
    "preRelease": false,
    "identifiers": [
      {
        "idType": "PURL",
        "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.0"
      }
    ]
  }
  ```
- **Beta version (has `releaseDate`, but not production ready)**:
  ```json
  {
    "uuid": "95f481df-f760-47f4-b2f2-f8b76d858450",
    "version": "11.0.0-M26",
    "createdDate": "2024-09-13T17:49:00Z",
    "releaseDate": "2024-09-16T17:49:00Z",
    "preRelease": true,
    "identifiers": [
      {
        "idType": "PURL",
        "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.0-M26"
      }
    ]
  }
  ```
</details>


Notes:
- Use uppercase idType values exactly as defined by the schema enum: CPE, TEI, PURL.
