# TEA design requirements

This document records design goals considered during the development of TEA.
It is informative and does not define conformance requirements for TEA 1.0.
Some goals concern capabilities outside the scope of that release.

For current API behaviour, see the [OpenAPI specification](../spec/openapi.yaml),
[discovery specification](../discovery/readme.md), and
[authentication specification](../auth/readme.md).

## Repository discovery

Based on an identifier a repository URL needs to be found. The identifier can be:

- PURL
- Product name or Product SKU and vendor name
- EAN bar code
- Product SKU
- Vendor UUID
- Hash of object

At the base URL well known URLs (ref) needs to point to:

- A lifecycle status document (using OWASP Common Lifecycle Enumeration, CLE)
- A version list. For each version, a URL will point to where a **collection** can be found
- Vendor Discovery, returns a list of Vendors represented in the repository
  - Vendor Name
  - Vendor ID

As an alternative, discovery using a company's ordinary website should be supported.
This can be handled using the file security.txt (IETF RFC 9116).

## Artefact discovery based on collections

The API MUST provide a way to discover the artefacts that are available for retrieval or further query.
Discovery SHOULD group artefacts together that represent a **collection**
that are directly applicable to a given product with a given version.
Every release has a collection, which may be empty.

- SBOM - Software Bill of Material
- CBOM - Cryptography Bill of Material
- HBOM - Hardware Bill of Material
- VDR - Vulnerability Disclosure Report
- VEX - Vulnerability Exploitability eXchange
- CDXA - Attestation

Authn/Authz is optional, but supported by the API.

## Collection management

The API SHOULD provide a method to manage collections, such as adding new collections,
modifying collections, or deleting existing collections.

Authn/Authz is optional, but supported by the API.

## Artefact retrieval

The API MUST provide a method in which to retrieve an artefact based on the identity of the artefact.
For example, using CycloneDX BOM-Link to retrieve either the
latest version or specific version of an artefact.

```text
urn:cdx:serialNumber
urn:cdx:serialNumber/version
```

The API needs to provide support for update checks, i.e. to check if a document is
updated without downloading.

> Note: Possible approaches include ETag or HEAD method.

Authn/Authz is optional, but supported by the API.

## Artefact versioning

The system and API must support artefact versioning for formats that support
versioning such as CycloneDX. For example:

- The ability to retrieve the latest SBOM vs a previous (uncorrected) version of the same SBOM.
  Corrections to SBOMs is a supported use case in the NTIA framing document.
- The ability to retrieve the latest VEX along with previous VEX for the same product so
  that time-series decisions are transparently available.

Authn/Authz is optional, but supported by the API.

## Artefact inventory search

The API MUST provide a way to search the inventory of a specific BOM or all available BOMs
for a given component or service. The API SHOULD support multiple identity formats including
PURL, CPE, GAV, GTIN, and GMN.

For example:

- Return the identity of all BOMs that have a vulnerable version of Apache Log4j:
  `pkg:maven/org.apache.logging.log4j/log4j-core@2.10.0`

The API should support multiple identity formats including PURL, CPE, SWID, GAV, GTIN, and GMN.

For example:

- Return the identity of all artefacts that describe `cpe:/a:acme:commerce_suite:1.0`.

Authn/Authz is optional, but supported by the API.
