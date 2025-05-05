# TEA release and collections

## The TEA release object (TRO)

The TEA Component Release object is a list of releases (versions) of a component
with a release identifier (string), release timestamp and a lifecycle
enumeration for the release. Each release includes a UUID of a 
TEA Collection object (TCO).

* UUID: Release UUID
* Version: Version (string)
* Date: Timestamp of release (for sorting releases)
* Prerelease: A flag indicating pre-release (or beta) status. May be disabled
  after creation of release object, but can't be enabled after creation of
  an object.
* Lifecycle: An identifier indicating lifecycle status of a release

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
  * UUID of the TEA collection object (TCO)
  * Product Release date (timestamp)
  * Version of this collection object
  * Reason for update/release of TCO
    * ENUM reason
      See OpenAPI spec
    * clear text
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
| VEXUPDATE   | Updated the VEX artifact(s)    |
| UPDATE  | Updated the artifact(s) other than VEX |
| REMOVAL | Removal of artifact       |
| ADDITION | Addition of an artifact |

Updates of VEX (CSAF) files may be handled in a different way by a TEA client,
producing different alerts than other changes of a collection.
