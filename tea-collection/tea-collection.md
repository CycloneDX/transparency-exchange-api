# TEA Releases and Collections

## The TEA Component Release object (TRO)

A TEA Component Release object represents a specific version of a component,
identified by a unique release identifier and associated metadata.
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
  - __idType__: Type of identifier (e.g., `tei`, `purl`, `cpe`).
  - __idValue__: Value of the identifier.
- __distributions__: List of release distributions, each with:
  - __distributionType__: Unique identifier for the distribution type.
  - __description__: Free-text description of the distribution.
  - __identifiers__: List of identifiers specific to this distribution.
    - __idType__: Type of identifier (e.g., `tei`, `purl`, `cpe`).
    - __idValue__: Value of the identifier.
  - __url__: Direct download URL for the distribution.
  - __signatureUrl__: Direct download URL for the distribution's external signature.
  - __checksums__: List of checksums for the distribution.
    - __algType__: Checksum algorithm used.
    - __algValue__: Checksum value.

### Examples

A TEA Component Release object of the binary distribution of Apache Tomcat 11.0.6 will look like:

```json
{
  "uuid": "605d0ecb-1057-40e4-9abf-c400b10f0345",
  "version": "11.0.6",
  "releaseDate": "2025-04-01T15:43:00Z",
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.6"
    }
  ]
}
```

Different versions of Apache Tomcat should have separate TEA Component Release objects:

```json
{
  "uuid": "da89e38e-95e7-44ca-aa7d-f3b6b34c7fab",
  "version": "10.1.4",
  "releaseDate": "2025-04-01T18:20:00Z",
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@10.1.4"
    }
  ]
}
```

The pre-release flag is used to mark versions not production ready
and does not require users to know the version naming scheme adopted by the project.

```json
{
  "uuid": "95f481df-f760-47f4-b2f2-f8b76d858450",
  "version": "11.0.0-M26",
  "releaseDate": "2024-09-13T17:49:00Z",
  "preRelease": true,
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.tomcat/tomcat@11.0.0-M26"
    }
  ]
}
```

## The TEA Collection object (TCO)

For each product and version there is a Tea Collection object, which is a list
of available artifacts for this specific version. The TEA Index is a list of
TEA collections.

The TEA collection is normally created by the TEA application server at
publication time of artifacts. The publisher may sign the collection
object as a JSON file at time of publication.

If there are any updates of artifacts within a collection for the same
version of a product, then a new TEA Collection object is created and signed.
This update will have the same UUID, but a new version number. A reason
for the update will have to be provided. This shall be used to
correct mistakes, spelling errors as well as to provide new information
on dynamic artifact types such as LCE or VEX. If the product
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
  - __releaseDate__: TEA Collection version release date.
  - __updateReason__: Reason for the update/release of the TEA Collection object.
    - __type__: Type of update reason.
      See [reasons for TEA Collection update](#the-reason-for-tco-update-enum) below.
    - __comment__: Free text description.
  -
  - __artifacts__: List of TEA artifact objects.
    See [below](#artifact-object).

## The TEA Artifact object

A TEA Artifact object represents a security-related document or file linked to a component release,
such as an SBOM, VEX, attestation, or license.
Artifacts are strictly **immutable**: if the underlying document changes, a new TEA Artifact object must be created.
URLs referenced in this object must always resolve to the same resource to ensure that published checksums remain valid and verifiable.

TEA Artifacts can be reused across multiple TEA Collections,
allowing the same document to be referenced by different component releases or even different components.
This promotes consistency and reduces duplication.

Optionally, each artifact can specify the `distributionType` identifiers of the distributions it applies to.
If this field is absent, the artifact is considered applicable to all distributions of the release.

### Structure

A TEA Artifact object contains the following fields:

- __uuid__: The UUID of the TEA Artifact object. This uniquely identifies the artifact.
- __name__: A human-readable name for the artifact.
- __type__: The type of artifact. See [TEA Artifact types](#tea-artifact-types) for allowed values (e.g., `BOM`, `VULNERABILITIES`, `LICENSE`).
- __componentDistributions__ (optional):  
  An array of `distributionType` identifiers indicating which distributions this artifact applies to.
  If omitted, the artifact applies to all distributions.
- __formats__:  
  An array of objects, each representing the same artifact content in a different format.
  The order of the list is not significant.
  Each format object includes:
  - __mime_type__: The MIME type of the document (e.g., `application/vnd.cyclonedx+xml`).
  - __description__: A free-text description of the artifact format.
  - __url__: A direct download URL for the artifact. This must point to an immutable resource.
  - __signature_url__ (optional): A direct download URL for a detached digital signature of the artifact, if available.
  - __checksums__:  
    An array of checksum objects for the artifact, each containing:
    - __algType__: The checksum algorithm used (e.g., `SHA_256`, `SHA3_512`).
    - __algValue__: The checksum value as a string.

### Notes

- The `formats` array allows the same artifact to be provided in multiple encodings or serializations (e.g., JSON, XML).
- The `checksums` field provides integrity verification for each artifact format.
- The `signature_url` enables consumers to verify the authenticity of the artifact using detached signatures.
- Artifacts should be published to stable, versioned URLs to ensure immutability and traceability.

## The reason for TCO update enum

| ENUM             | Description                            |
|------------------|----------------------------------------|
| INITIAL_RELEASE  | Initial release of the collection      |
| VEX_UPDATED      | Updated the VEX artifact(s)            |
| ARTIFACT_UPDATED | Updated the artifact(s) other than VEX |
| ARTIFACT_REMOVED | Removal of artifact                    |
| ARTIFACT_ADDED   | Addition of an artifact                |

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
  "releaseDate": "2024-12-13T00:00:00Z",
  "updateReason": {
    "type": "ARTIFACT_UPDATED",
    "comment": "VDR file updated"
  },
  "artifacts": [
    {
      "uuid": "1cb47b95-8bf8-3bad-a5a4-0d54d86e10ce",
      "name": "Build SBOM",
      "type": "bom",
      "formats": [
        {
          "mime_type": "application/vnd.cyclonedx+xml",
          "description": "CycloneDX SBOM (XML)",
          "url": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3-cyclonedx.xml",
          "signature_url": "https://repo.maven.apache.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3-cyclonedx.xml.asc",
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
      "type": "vulnerability-assertion",
      "formats": [
        {
          "mime_type": "application/vnd.cyclonedx+xml",
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
