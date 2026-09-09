# TEA Collection object

For each product and version there is a Tea Collection object (TCO), which is a list
of available artefacts for this specific version.

The TEA collection is normally created by the TEA application server at
publication time of artefacts. The publisher may sign the collection
object as a JSON file at time of publication.

If there are any updates of artefacts within a collection for the same
version of a product, then a new TEA Collection object is created and optionally signed.
This update will have the same UUID, but a new version number. A reason
for the update will have to be provided. This shall be used to
correct mistakes, spelling errors as well as to provide new information
on dynamic artefact types such as LCE or VEX. If the product
is modified, that is a new product version and that should generate
a new collection object with a new UUID and updated metadata.

The API allows for retrieving the latest version of the collection,
or a specific version.

### Dynamic or static Collection objects

The TCO is managed by the TEA software platform. There are two ways
to implement this:

- __Dynamic__: The TCO is built for each API request and created
  dynamically.
- __Static__: The TCO is built at publication time as a static
  object by the publisher. This object can be digitally signed at
  publication time and version controlled.

### Collection object

The TEA Collection object has the following parts:

- Preamble
- __uuid__: UUID of the TEA Collection object.
    This matches the UUID of the associated TEA Component Release object.
    When updating a collection, only the `version` is changed.
- __version__: TEA Collection version, incremented each time its content changes.
    Versions start with 1.
- __createdDate__: Timestamp when the TEA Collection version was created.
- __belongsTo__: Indicates whether this collection belongs to a Component Release or a Product Release. Enum values `COMPONENT_RELEASE` or `PRODUCT_RELEASE`.
- __updateReason__: Reason for the update/release of the TEA Collection object.
  - __type__: Type of update reason.
      See [reasons for TEA Collection update](#the-reason-for-tco-update-enum) below.
  - __comment__: Free text description.
- __artifacts__: Array of TEA Artifact objects.
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

- __uuid__: The UUID of the TEA Artifact object. Together with *version* uniquely identifies the TEA Artifact.
- __version__:
  An integer with default value 1.
  Together with *uuid* uniquely identifies the TEA Artifact.
  This field can be used to designate successive, immutable revisions of an artifact content (e.g. an updated VEX file).
- __name__: A human-readable name for the artifact.
- __type__: The type of TEA artifact. See [TEA Artifact types](#tea-artifact-types) for allowed values (e.g., `BOM`, `VULNERABILITIES`, `LICENSE`).
- __createdDate__: The date and time the TEA Artifact revision was created.
- __distributionIds__: (optional): Array of TEA Component Release distributions that this TEA Artifact applies to. If absent or empty, the TEA Artifact applies to all distributions.
- __formats__:  
  An array of objects, each representing the same artifact content in a different format.
  The order of the list is not significant.
  Each format object includes:
  - __mediaType__: The MIME type of the document (e.g., `application/vnd.cyclonedx+xml`).
  - __description__: A free-text description of the artifact format.
  - __url__: A direct download URL for the artefact. This must point to an immutable resource.
  - __signatureUrl__ (optional): A direct download URL for a detached digital signature of the artefact, if available.
  - __checksums__:  
    An array of checksum objects for the artefact, each containing:
    - __algType__: The checksum algorithm used (e.g., `SHA_256`, `SHA3_512`).
    - __algValue__: The checksum value as a string.

Required fields:

- uuid, type, formats

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
  "version": 10,
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
          "mediaType": "application/vnd.cyclonedx+xml",
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
      "version": 7,
      "name": "Vulnerability Disclosure Report",
      "type": "VULNERABILITIES",
      "formats": [
        {
          "mediaType": "application/vnd.cyclonedx+xml",
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
