# The TEA Collection object

For each product and version there is a Tea Collection object (TCO), which is a list
of available artifacts for this specific version.

The TEA collection is normally created by the TEA application server at
publication time of artifacts. The publisher may sign the collection
object as a JSON file at time of publication.

A release is never served without a collection. If no artifacts have been
published when the release first becomes retrievable, the server serves
collection version 1 with an empty `artifacts` list and
`updateReason.type: INITIAL_RELEASE`. That version is immutable like any
other; the first artifacts are published as version 2. Classification follows
the update-reason rules below. A publisher that
publishes the release and its artifacts together never has an empty version
and starts at version 1 with content.
In both cases version 1 is the first collection a client could have retrieved.
A server may also synthesize the collection dynamically (see below).

If there are any updates of artefacts within a collection for the same
version of a product, then a new TEA Collection object is created and optionally signed.
This update will have the same UUID, but a new version number. A reason
for the update will have to be provided. This shall be used to
correct mistakes, spelling errors as well as to provide new information
on dynamic artifact types such as CLE or VEX. If the product
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

### Structure

The TEA Collection object has the following parts:

- Preamble
- __uuid__: UUID of the TEA Collection object.
    A TEA Collection shall use the same UUID as its parent Product Release or Component Release.
    Within an API base, a Product Release and a Component Release shall not share a UUID.
    Consequently, Collections belonging to different parent releases have different UUIDs.
    Versions of the same Collection retain the same UUID.
    See [TEA UUID Scope and Stability](../doc/tea-uuid-scope.md).
    When updating a collection, only the `version` is changed.
- __version__: TEA Collection version, incremented by 1 each time its content changes.
    Versions start with 1. Content changes include replacing an embedded artifact with a
    newer revision (for example one published because an external `url` or `signatureUrl`
    changed). If a Collection adopts that new artifact revision, the server shall publish
    a new Collection version. Previously published Collection versions shall remain
    unchanged. Classification of that adoption follows the update-reason rules below.
- __createdDate__: Timestamp when the TEA Collection version was created.
- __belongsTo__: Indicates whether this collection belongs to a Component Release or a Product Release. Enum values `COMPONENT_RELEASE` or `PRODUCT_RELEASE`.
- __updateReason__: Reason for the update/release of the TEA Collection object.
  - __type__: Type of update reason.
      See [reasons for TEA Collection update](#the-reason-for-tco-update-enum) below.
  - __comment__: Free text description.
- __artifacts__: Array of TEA Artifact objects.
    See [below](#the-tea-artifact-object).

Required fields:

- `uuid`, `version`, `createdDate`, `belongsTo`, `updateReason`, `artifacts`, `updateReason.type`

## TEA Artifact object

A TEA Artifact object represents a security-related document or file linked to a component release,
such as an SBOM, VEX, attestation, or license.
TEA Artifacts are strictly **immutable** per revision: if the underlying document or
any published field, including external `url` / `signatureUrl` values, changes, a new
TEA Artifact revision shall be created. For a fixed artifact UUID, version, and
format, the server shall ensure that successful content retrieval through its download
endpoint, including retrieval after following redirects, yields unchanged artifact bytes
after HTTP transfer coding and content coding are removed. The server shall ensure that
successful signature retrieval through its signature download endpoint yields unchanged
signature bytes on the same terms. Changing only a download response's `Location`, or
only its HTTP content coding, while preserving those bytes, does not require a new
artifact or collection version.
For artifact content retrieved from an external `url`, the published checksums are the
integrity statement for the revision. These checksums do not cover detached signatures.

TEA Artifacts can be reused across multiple TEA Collections,
allowing the same document to be referenced by different component or product releases.
This promotes consistency and reduces duplication. Publishing a new artifact revision
does not by itself change Collections that continue to embed an older revision.

Optionally, each TEA Artifact can specify the `distributionIds` of the distributions it applies to.
If this field is absent, the TEA Artifact is considered applicable to all distributions of the release.

### Structure

A TEA Artifact object contains the following fields:

- __uuid__: The UUID of the TEA Artifact object. Together with *version* uniquely identifies the TEA Artifact.
- __version__:
  Revision number, starting at 1 and incremented by 1 for each new revision of the same artifact UUID.
  Together with *uuid* uniquely identifies the TEA Artifact.
  Successive revisions cover content changes and changes to any published field,
  including external `url` or `signatureUrl` values. Each published revision is immutable.
- __name__: A human-readable name for the artifact.
- __type__: The type of TEA artifact. See [TEA Artifact types](#tea-artifact-types) for allowed values (e.g., `BOM`, `VULNERABILITIES`, `LICENSE`).
- __createdDate__: The date and time the TEA Artifact revision was created.
- __distributionIds__: (optional): Array of TEA Component Release distributions that this TEA Artifact applies to. If absent or empty, the TEA Artifact applies to all distributions.
- __formats__:  
  A non-empty array of objects, each representing the same artifact content in a different format.
  The order of the list is not significant.
  Each format object includes:
  - __mediaType__: The media type of the document (e.g., `application/vnd.cyclonedx+xml`).
    Required. A media type appears at most once across the formats of a TEA Artifact revision,
    so that a format can be selected unambiguously by media type.
  - __description__: A free-text description of the artifact format.
  - __url__ (optional): An external download URL for the artifact, outside the TEA API.
    When present, clients shall retrieve the content from it.
    When absent, the TEA server hosts the content itself and clients shall retrieve it from the
    artifact download endpoint (`/artifact/{uuid}/{version}/download`), selecting the format by its media type.
    `url` and `signatureUrl` should be stable URLs. Servers should not publish pre-signed or
    otherwise expiring URLs in them. A server that serves content from expiring storage links
    shall omit them and answer from the download endpoints, with `302` where the bytes live elsewhere.
    Changing `url` shall create a new artifact revision, even when content and checksums are unchanged.
    The server shall preserve the URLs published on each historical artifact and collection version.
    Collection adoption follows the collection `version` rules above.
    Byte stability for content retrieved through the server's download endpoint follows
    the artifact revision rule above.
    Servers shall publish at least one checksum for every format that has a `url`; content that does
    not match them is not the content of that revision. The `latest` download endpoints are mutable
    by design and shall not be used as a format's `url` or `signatureUrl`.
  - __signatureUrl__ (optional): An external download URL for a detached digital signature of the artifact, outside the TEA API.
    If present, clients retrieve the signature from it.
    If absent, clients retrieve it from the artifact signature download endpoint
    (`/artifact/{uuid}/{version}/signature/download`), which answers `404` when no signature is published for the format.
    Changing `signatureUrl` follows the same revision and collection-adoption rules as `url`.
    Byte stability for signature retrieval through the server's signature download endpoint
    follows the artifact revision rule above.
  - __checksums__:
    When present, the array shall contain at least one entry. When `url` is present, `checksums`
    is required. An array of checksum objects for the artifact format's content bytes, each containing:
    - __algType__: The checksum algorithm used (e.g., `SHA-256`, `SHA3-512`).
    - __algValue__: The checksum value as a string.
    When `url` is present, these checksums are the integrity statement for the revision.
    Content that does not match them is not the content of that revision.

    Published checksums shall be calculated over the artifact format's bytes before HTTP
    content coding is applied. Clients verifying them shall first remove any HTTP transfer
    coding and content coding, and shall not otherwise transform or canonicalize the
    artifact bytes. An artifact that is itself compressed, such as a `.gz` file, stays in
    that form. The TEA servers shall not declare the artifact's own compression as HTTP
    content coding, because clients would then remove it and the checksum would not match.

Required fields:

- `uuid`, `type`, `formats`, `createdDate`, `version`

### Notes

- The `formats` array allows the same artifact to be provided in multiple encodings or serializations (e.g., JSON, XML).
- The `checksums` field provides integrity verification for each artifact format's
  bytes before HTTP content coding is applied. It does not cover detached signatures,
  and it is not a digest of an HTTP-encoded representation; `Repr-Digest` covers that.
- Detached signatures, whether at `signatureUrl` or served by the TEA server, enable consumers to verify the authenticity of the artifact.
- `url` and `signatureUrl` are always external locations; a TEA server that hosts content or signatures itself omits them and serves the bytes from its download endpoints.
  A TEA access token is sent only to the TEA server's own API, never to an external URL.

## The reason for TCO update enum

Table: TEA Collection update reasons

| ENUM             | Description                            |
|------------------|----------------------------------------|
| INITIAL_RELEASE  | Initial release of the collection      |
| VEX_UPDATED      | Adding or revising an artifact of type `VULNERABILITIES`, including a new revision whose external `url` or `signatureUrl` changed |
| ARTIFACT_UPDATED | A new revision of an artifact whose type is not `VULNERABILITIES`, including a revision whose external `url` or `signatureUrl` changed |
| ARTIFACT_REMOVED | Removal of artifact                    |
| ARTIFACT_ADDED   | Addition of an artifact whose type is not `VULNERABILITIES` |

Replacing an embedded artifact revision solely because its published `url` or `signatureUrl`
changed is `VEX_UPDATED` when the artifact type is `VULNERABILITIES`, and `ARTIFACT_UPDATED`
otherwise. `VEX_UPDATED` covers adding or revising an artifact of type `VULNERABILITIES`.
VEX and VDR share that type. Removing an artifact, including one of type `VULNERABILITIES`,
counts as `ARTIFACT_REMOVED`. The first Collection version shall use `INITIAL_RELEASE`.
In subsequent versions, adding or revising an artifact of type `VULNERABILITIES` counts as
`VEX_UPDATED`. When multiple change kinds occur, the server shall select `updateReason.type`
using the precedence rule below.

When a collection version other than the first contains changes of more than one kind,
the server shall set `updateReason.type` to the first applicable value in this order:
`VEX_UPDATED`, `ARTIFACT_REMOVED`, `ARTIFACT_ADDED`, `ARTIFACT_UPDATED`. `comment` can
describe the other changes. A client that needs the complete set of changes should compare
the version with the previous one.

Updates of VEX (CSAF) files may be handled in a different way by a TEA client,
producing different alerts than other changes of a collection.

## TEA Artifact types

Table: TEA Artifact types

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
  "uuid": "da89e38e-95e7-44ca-aa7d-f3b6b34c7fab",
  "version": 10,
  "createdDate": "2024-12-15T00:00:00Z",
  "belongsTo": "COMPONENT_RELEASE",
  "updateReason": {
    "type": "VEX_UPDATED",
    "comment": "VDR file updated"
  },
  "artifacts": [
    {
      "uuid": "1cb47b95-8bf8-3bad-a5a4-0d54d86e10ce",
      "version": 2,
      "createdDate": "2024-12-13T00:00:00Z",
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
              "algType": "SHA-256",
              "algValue": "e04c9d55986d7194822eaa4f8115a77f801844d807ad6e0d454ac31dd41861e5"
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
      "createdDate": "2024-12-15T00:00:00Z",
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
