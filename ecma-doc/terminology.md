
# Terminology (Glossary)

## Status

This document defines the terminology used in the Transparency Exchange API (TEA) specification.

It is **normative for terminology only**, and serves as a reference to ensure:

- consistent interpretation  
- aligned language across specifications  
- reduced ambiguity  


## Terms and Definitions

### Artifact

A binary or structured object distributed via TEA.

Examples include:

- SBOMs  
- firmware images  
- configuration files  

Artifacts are:

- immutable  
- identified by digest  
- validated using evidence bundles (in the Trust Architecture)

Artifacts in TEA can belong to multiple collections - for different products
or different releases of a component. Artifacts may also be published
in multiple formats.

### TEA Collection

A structured object that defines a component or product release by grouping artifacts.

A collection:

- references artifacts  
- includes metadata  
- does not prove artifact authenticity  

Collections can be updated without publishing a new release. For an update,
the version number is incremented and a reason for the update is added to the
collection. This can be fixing an error in a SBOM, updating a compliance
document or replacing an artifact.

### Compliance Document

A document describing compliance, certification, or regulatory status of a manufacturer or project.

Compliance documents:

- are not tied to a specific product or release  
- are associated with the domain owner  
- are treated as standalone artifacts in the TEA Architecture  

### CPE (Common Platform Enumeration)

A standardized identifier format for IT products, platforms, and versions.

In TEA, CPE may be used as an identifier type in API queries and object metadata.

### CSAF (Common Security Advisory Framework)

A standard for machine-readable security advisories.

CSAF documents provide structured information about:

- vulnerabilities  
- affected products  
- remediation guidance  

### TEA Discovery Document

A document retrieved via the TEA discovery process that defines:

- API endpoints  
- service capabilities
- failover policies

### SBOM (Software Bill of Materials)

A structured inventory of software components within a product.

SBOMs describe:

- components  
- dependencies  
- versions  
- identifiers

One format for SBOMs is OWASP CycloneDX, documented in ECMA-424.

### TEA (Transparency Exchange API)

A specification for exchanging software transparency information across the software supply chain.

TEA defines:

- APIs for publishing and retrieving artifacts  
- collections representing releases  
- discovery mechanisms using TEI  

### TEA Service

A service implementing TEA APIs for:

- publishing  
- discovery  
- retrieval  

### TEI (Transparency Exchange Identifier)

A URI-based identifier used to locate TEA services and resources.

A TEI:

- identifies a product  
- is scoped to a domain

### UUID (Universally Unique Identifier)

A standardized 128-bit identifier used to uniquely identify TEA objects such as:

- products  
- product releases  
- components  
- component releases  
- collections  
- artifacts  

### VEX (Vulnerability Exploitability eXchange)

A document format describing the exploitability status of vulnerabilities in software.

VEX documents indicate whether a vulnerability:

- is exploitable  
- is not exploitable  
- has mitigations  


## 3. References

- RFC 2119 — Key words for use in RFCs to Indicate Requirement Levels  
  https://www.rfc-editor.org/rfc/rfc2119  

- RFC 8174 — Ambiguity of Uppercase vs Lowercase in RFC 2119 Keywords  
  https://www.rfc-editor.org/rfc/rfc8174  

- RFC 3986 — Uniform Resource Identifier (URI): Generic Syntax  
  https://www.rfc-editor.org/rfc/rfc3986  

- RFC 4648 — Base64URL Encoding  
  https://www.rfc-editor.org/rfc/rfc4648  

- RFC 9110 — HTTP Semantics  
  https://www.rfc-editor.org/rfc/rfc9110  

- FIPS 180-4 — Secure Hash Standard (SHA-256)  
  https://csrc.nist.gov/publications/detail/fips/180/4/final  

- ECMA-428 — Common Lifecycle Enumeration (CLE)  
  https://ecma-international.org/publications-and-standards/standards/ecma-428/  

- ECMA-424 — CycloneDX (Software Bill of Materials)  
  https://ecma-international.org/publications-and-standards/standards/ecma-424/  

- ECMA-427 — Package URL (PURL)  
  https://ecma-international.org/publications-and-standards/standards/ecma-427/
