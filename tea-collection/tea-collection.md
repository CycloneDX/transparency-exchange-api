# The TEA Collection object

For each product and version there is a Tea Collection object, which is a list
of available artifacts for this specific version. The TEA Index is a list of
TEA collections.

The TEA collection is normally created by the TEA application server at
publication time of artifacts.

## Collection object

The TEA Collection object has the following parts

* Preamble
  * UUID of collection object
  * Product name
  * Product version
  * Release date
  * Last update
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
  * Description in clear text
  * URL for downloads
  * Size in bytes
  * SHA384 checksum
