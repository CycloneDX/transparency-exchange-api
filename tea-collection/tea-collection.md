# The TEA Collection object

For each product and version there is a Tea Collection object, which is a list
of available artifacts for this specific version. The TEA Index is a list of
TEA collections.

The TEA collection is normally created by the TEA application server at
publication time of artifacts.

If there are any updates of artifacts within a collection for the same
version of a product, then a new TEA Collection object is created and signed.
This update will have the same UUID, but a new version number. A reason
for the update will have to be provided. This shall only be used to
correct mistakes, spelling errors and similar things. If the product
is modified, that is a new product version and that should generate
a new collection object with a new UUID and updated metadata.

## Collection object

The TEA Collection object has the following parts

* Preamble
  * UUID of collection object
  * Product name
  * Product version
  * Product Release date (timestamp)
  * TEA Collection object release date (timestamp)
  * TEA Collection object version (integer starting with version 1)
  * Reason for update/release of TCO - clear text
    * "New product release"
    * "Corrected dependency in SBOM that was faulty"
    * "Added missing In-Toto build attestation"
* List of artifact objects (see below)
* Optional Signature

The artifact object has the following parts

* Artifact UUID
* Artifact name
* List of objects with the same content
  * UUID for subdoc
  * Optional BOM identifier
    * SPDX or CycloneDX reference to BOM
  * MIME media type
  * Artifact category (enum)
    * https://cyclonedx.org/docs/1.6/json/#externalReferences_items_type
  * Description in clear text
  * URL for downloads
  * Size in bytes
  * SHA384 checksum
