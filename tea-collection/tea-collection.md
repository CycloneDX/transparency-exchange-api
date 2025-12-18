# TEA Releases and Collections

## The TEA Component Release object (TRO)

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

The `uuid` of the TEA Component Release object is identical to the `uuid` of its associated
[TEA Collection object (TCO)](#the-tea-collection-object-tco).

### Structure

A TEA Component Release object contains the following fields:

- __uuid__: Unique identifier for the TEA Component Release.
- __version__: Version number of the release.
- __createdDate__: Timestamp when the release object was created.
- __releaseDate__: Timestamp of the actual release.
- __preRelease__: Boolean flag indicating if this is a pre-release (e.g., beta).
  This flag can be disabled after creation, but not enabled.
- __identifiers__: List of identifiers for the component.
- __idType__: Type of identifier (e.g., `TEI`, `PURL`, `CPE`).
- __idValue__: Value of the identifier.
- __distributions__: List of release distributions, each with:
  - __distributionType__: Unique identifier for the distribution type.
  - __description__: Free-text description of the distribution.
  - __identifiers__: List of identifiers specific to this distribution.
  - __idType__: Type of identifier (e.g., `TEI`, `PURL`, `CPE`).
  - __idValue__: Value of the identifier.
  - __url__: Direct download URL for the distribution.
  - __signatureUrl__: Direct download URL for the distribution's external signature.
  - __checksums__: List of checksums for the distribution.
    - __algType__: Checksum algorithm used.
    - __algValue__: Checksum value.

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

## TEA Collection object (TCO)

For each product and version there is a Tea Collection object, which is a list
of available artefacts for this specific version. The TEA Index is a list of
TEA collections.

The TEA collection is normally created by the TEA application server at
publication time of artefacts. The publisher may sign the collection
object as a JSON file at time of publication.

If there are any updates of artefacts within a collection for the same
version of a product, then a new TEA Collection object is created and signed.
This update will have the same UUID, but a new version number. A reason
for the update will have to be provided. This shall be used to
correct mistakes, spelling errors as well as to provide new information
on dynamic artefact types such as LCE or VEX. If the product
is modified, that is a new product version and that should generate
a new collection object with a new UUID and updated metadata.

The API allows for retrieving the latest version of the collection,
or a specific version.

### Dynamic or static Collection objects

The TCO is produced by the TEA software platform. There are two ways
to implement this:

* __Dynamic__: The TCO is built for each API request and created
  dynamically.
* __Static__: The TCO is built at publication time as a static
  object by the publisher. This object can be digitally signed at
  publication time and version controlled.

### Collection object

  The TEA Collection object has the following parts:

  - Preamble
  - __uuid__: UUID of the TEA Collection object.
    Note that this is equal to the UUID of the associated TEA Component Release object.
    When updating a collection, only the `version` is changed.
  - __version__: TEA Collection version, incremented each time its content changes.
    Versions start with 1.
  - __date__: TEA Collection version release date.
  - __belongsTo__: Scope of the collection; enum values `RELEASE` or `PRODUCT_RELEASE`.
  - __updateReason__: Reason for the update/release of the TEA Collection object.
    - __type__: Type of update reason.
      See [reasons for TEA Collection update](#the-reason-for-tco-update-enum) below.
    - __comment__: Free text description.
  -
  - __artifacts__: List of TEA Artifact objects.
    See [below](#tea-artifact-object).

## TEA Artifact object

A TEA Artifact object represents a security-related document or file linked to a component release,
such as an SBOM, VEX, attestation, or license.
TEA Artifacts are strictly **immutable**: if the underlying document changes, a new TEA Artifact object must be created.
URLs referenced in this object must always resolve to the same resource to ensure that published checksums remain valid and verifiable.

TEA Artifacts can be reused across multiple TEA Collections,
allowing the same document to be referenced by different component or product releases.
This promotes consistency and reduces duplication.

Optionally, each TEA Artifact can specify the `distributionType` identifiers of the distributions it applies to.
If this field is absent, the TEA Artifact is considered applicable to all distributions of the release.

### Structure

A TEA Artifact object contains the following fields:

- __uuid__: The UUID of the TEA Artefact object. This uniquely identifies the TEA Artifact.
- __name__: A human-readable name for the artefact.
- __type__: The type of artefact. See [TEA Artifact types](#tea-artefact-types) for allowed values (e.g., `BOM`, `VULNERABILITIES`, `LICENSE`).
- __componentDistributions__ (optional):  
  An array of `distributionType` identifiers indicating which distributions this TEA Artifact applies to.
  If omitted, the TEA Artifact applies to all distributions.
- __formats__:  
  An array of objects, each representing the same artefact content in a different format.
  The order of the list is not significant.
  Each format object includes:
  - __mimeType__: The MIME type of the document (e.g., `application/vnd.cyclonedx+xml`).
  - __description__: A free-text description of the artefact format.
  - __url__: A direct download URL for the artefact. This must point to an immutable resource.
  - __signatureUrl__ (optional): A direct download URL for a detached digital signature of the artefact, if available.
  - __checksums__:  
    An array of checksum objects for the artefact, each containing:
    - __algType__: The checksum algorithm used (e.g., `SHA_256`, `SHA3_512`).
    - __algValue__: The checksum value as a string.

### Notes

- The `formats` array allows the same artefact to be provided in multiple encodings or serializations (e.g., JSON, XML).
- The `checksums` field provides integrity verification for each artefact format.
- The `signatureUrl` enables consumers to verify the authenticity of the artefact using detached signatures.
- Artefacts should be published to stable, versioned URLs to ensure immutability and traceability.

## The reason for TCO update enum

| ENUM             | Description                            |
|------------------|----------------------------------------|
| INITIAL_RELEASE  | Initial release of the collection      |
| VEX_UPDATED      | Updated the VEX artefact(s)            |
| ARTIFACT_UPDATED | Updated the artefact(s) other than VEX |
| ARTIFACT_REMOVED | Removal of artefact                    |
| ARTIFACT_ADDED   | Addition of an artefact                |

Updates of VEX (CSAF) files may be handled in a different way by a TEA client,
producing different alerts than other changes of a collection.

## TEA Artifact types

| ENUM            | Description                                                                         |
|-----------------|-------------------------------------------------------------------------------------|
| ATTESTATION     | Machine-readable statements containing facts, evidence, or testimony.               |
| BOM             | Bill of Materials: SBOM, OBOM, HBOM, SaaSBOM, etc.                                  |
| BUILD_META      | Build-system specific metadata file: `pom.xml`, `package.json`, `.nuspec`, etc.     |
| CERTIFICATION   | Industry, regulatory, or other certification from an accredited certification body. |
| FORMULATION     | Describes how a component or service was manufactured or deployed.                  |
| LICENSE         | License file                                                                        |
| RELEASE_NOTES   | Release notes document                                                              |
| SECURITY_TXT    | A `security.txt` file                                                               |
| THREAT_MODEL    | A threat model                                                                      |
| VULNERABILITIES | A list of vulnerabilities: VDR/VEX                                                  |
| OTHER           | Document that does not fall into any of the above categories                        |

### Examples

```json
{
  "uuid": "4c72fe22-9d83-4c2f-8eba-d6db484f32c8",
  "version": 1,
  "date": "2024-12-13T00:00:00Z",
  "updateReason": {
    "type": "ARTIFACT_UPDATED",
    "comment": "VDR file updated"
  },
  "artifacts": [
    {
      "uuid": "1cb47b95-8bf8-3bad-a5a4-0d54d86e10ce",
      "name": "Build SBOM",
      "type": "BOM",
      "formats": [
        {
          "mimeType": "application/vnd.cyclonedx+xml",
          "description": "CycloneDX SBOM (XML)",
          "url": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3-cyclonedx.xml",
          "signatureUrl": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3-cyclonedx.xml.asc",
          "checksums": [
            {
              "algType": "MD5",
              "algValue": "2e1a525afc81b0a8ecff114b8b743de9"
            },
            {
              "algType": "SHA-1",
              "algValue": "5a7d4caef63c5c5ccdf07c39337323529eb5a770"
            }
          ]
        }
      ]
    },
    {
      "uuid": "dfa35519-9734-4259-bba1-3e825cf4be06",
      "name": "Vulnerability Disclosure Report",
      "type": "VULNERABILITIES",
      "formats": [
        {
          "mimeType": "application/vnd.cyclonedx+xml",
          "description": "CycloneDX VDR (XML)",
          "url": "https://logging.apache.org/cyclonedx/vdr.xml",
          "checksums": [
            {
              "algType": "SHA-256",
              "algValue": "75b81020b3917cb682b1a7605ade431e062f7a4c01a412f0b87543b6e995ad2a"
            }
          ]
        }
      ]
    }
  ]
}
```
