# TEA Requirements

## Repository discovery

Based on an identifier a repository URL needs to be found. The identifier can
be:

- PURL
- Product name or Product SKU and vendor name
- EAN bar code
- Product SKU
- Vendor UUID
- Hash of object

At the base URL well known URLs (ref) needs to point to

- A lifecycle status document (using OWASP Common Lifecycle Enumeration, CLE)
- A version list. For each version, a URL will point to where a **collection**
  can be found
- Vendor Discovery, returns a list of Vendors represented in the repository
  - Vendor Name
  - Vendor ID

As an alternative, discovery using a company's ordinary web site should be
supported. This can be handled using the file security.txt (IETF RFC 9116)

## Artifact Discovery based on TEA collections

The API MUST provide a way to discover the artifacts that are available for
retrieval or further query. Discovery SHOULD group artifacts together that
represent a **collection** that are directly applicable to a given product with
a given version. Collections are OPTIONAL.

- SBOM - Software Bill of Material
- CBOM - Cryptography Bill of Material
- HBOM - Hardware Bill of Material
- VDR - Vulnerability Disclosure Report
- VEX - Vulnerability Exploitability eXchange
- CDXA - Attestation

Authn/Authz MUST be supported

## Collection Management

The API SHOULD provide a method to manage collections, such as adding new
collections, modifying collections, or deleting existing collections.

- Authn/Authz MUST be supported

## Artifact Retrieval

The API MUST provide a method in which to retrieve an artifact based on the
identity of the artifact. For example, using CycloneDX BOM-Link to retrieve
either the latest version or specific version of an artifact.

```text
urn:cdx:serialNumber
urn:cdx:serialNumber/version
```

The API needs to provide support for update checks, i.e. to check if a document
is updated without downloading. (possibly etag or HEAD method or similar)
Authn/Authz MUST be supported

## Artifact Publishing

The API MUST provide a way to publish an artifact, either standalone or to a
collection. The detection of duplicate artifacts with the same identity MUST be
handled and prevented. Authn/Authz MUST be supported

## Artifact Versioning

The system and API must support artifact versioning for formats that support
versioning such as CycloneDX. For example:

- The ability to retrieve the latest SBOM vs a previous (uncorrected) version of
  the same SBOM. Corrections to SBOMs is a supported use case in the NTIA
  framing document.
- The ability to retrieve the latest VEX along with previous VEX for the same
  product so that time-series decisions are transparently available.

Authn/Authz MUST be supported

## insights: Search Artifact Inventory

The API MUST provide a way to search the inventory of a specific BOM or all
available BOMs for a given component or service. The API SHOULD support multiple
identity formats including PURL, CPE, SWID, GAV, GTIN, and GMN.

For example:

- Return the identity of all BOMs that have a vulnerable version of Apache
  Log4J: `pkg:maven/org.apache.logging.log4j/log4j-core@2.10.0`

The API MUST provide a way to search for the metadata component across all
available BOMs. The API SHOULD support multiple identity formats including PURL,
CPE, SWID, GAV, GTIN, and GMN. For example:

- Return the identity of all artifacts that describe
  `cpe:/a:acme:commerce_suite:1.0`.

Authn/Authz MUST be supported
