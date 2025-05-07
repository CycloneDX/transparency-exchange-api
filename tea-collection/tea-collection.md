# TEA release and collections

## The TEA release object (TRO)

The TEA Component Release object corresponds to a specific variant
(version and packaging) of a component with a release identifier (string),
release timestamp and a lifecycle enumeration for the release.
The UUID of the TEA Component Release object matches the UUID of the associated TEA Collection objects (TCO).

A TEA Component Release object has the following parts:

- __uuid__: A unique identifier for the TEA Component Release
- __version__: Version number
- __release_date__: Timestamp of the release (for sorting purposes)
- __pre_release__: A flag indicating pre-release (or beta) status.
  May be disabled after the creation of the release object, but can't be enabled after creation of an object.
- __identifiers__: List of identifiers for the component
  - __idType__: Type of identifier, e.g. `tei`, `purl`, `cpe`
  - __idValue__: Identifier value

### Examples

A TEA Component Release object of the binary distribution of Apache Tomcat 11.0.6 will look like:

```json
{
  "uuid": "605d0ecb-1057-40e4-9abf-c400b10f0345",
  "version": "11.0.6",
  "release_date": "2025-04-01T15:43:00Z",
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.maven/maven@11.0.6?type=zip"
    }
  ]
}
```

Different versions of Apache Tomcat should have separate TEA Component Release objects:

```json
{
  "uuid": "da89e38e-95e7-44ca-aa7d-f3b6b34c7fab",
  "version": "10.1.4",
  "release_date": "2025-04-01T18:20:00Z",
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.maven/maven@10.1.4?type=zip"
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
  "release_date": "2024-09-13T17:49:00Z",
  "pre_release": true,
  "identifiers": [
    {
      "idType": "purl",
      "idValue": "pkg:maven/org.apache.maven/maven@11.0.0-M26?type=zip"
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

The TEA Collection object has the following parts

* Preamble
  * UUID of the TEA collection release object (TCO). Note that this
    is the same UUID as the release object for this version. When updating
    a collection, the version is changed.
  * Product Release date (timestamp)
  * Collection version release date (timestamp)
  * _Version_ of this collection object. Starting with 1.
  * Reason for update/release of TCO
    * ENUM reason
      See below
    * clear text description of change
      * "New product release"
      * "Corrected dependency in SBOM that was faulty"
      * "Added missing In-Toto build attestation"
* List of artifact objects (see below)
* Optional Signature of the collection object

The artifact object has the following parts

* Artifact UUID
* Artifact name
* Author of the artifact object
  * Name
  * Email
  * Organisation
* List of objects with the same content, but in different formats.
  The order of the list has no significance.
  * UUID for subdoc
  * Optional BOM identifier
    * SPDX or CycloneDX reference to BOM
  * MIME media type
  * Artifact category (enum)
    * <https://cyclonedx.org/docs/1.6/json/#externalReferences_items_type>
  * Description in clear text
  * Direct URL for downloads of artefact
  * Direct URL for download of external signature
  * Size in bytes
  * SHA384 checksum

### The reason for TCO update enum

| ENUM        | Explanation                    |
|-------------|--------------------------------|
| INITIAL_RELEASE | Initial release of the collection |
| VEX_UPDATED   | Updated the VEX artifact(s)    |
| ARTIFACT_UPDATED  | Updated the artifact(s) other than VEX |
| ARTIFACT_REMOVED | Removal of artifact       |
| ARTIFACT_ADDED | Addition of an artifact |

Updates of VEX (CSAF) files may be handled in a different way by a TEA client,
producing different alerts than other changes of a collection.
