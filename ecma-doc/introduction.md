# OWASP Transparency Exchange API Standard

## Introduction

The TEA API is created to support automation of the software supply chain. Upstream
vendors and open source projects can use this standard to keep downstream users
up to date with transparency artifacts such as, but not limited to, bill of materials,
VEX files, attestations and much more.

This specification defines a standard, format agnostic, API for the exchange of
product related artifacts, like BOMs, between systems. The work includes:

- __Discovery__ using the Transparency Exchange Identifier (TEI)
- __Retrieval__ of artifacts - from compliance documents to bill of materials and other artifacts
- __CLE__ - common lifecycle events delivering the status of a product or a release

System and tooling implementors are encouraged to adopt this API standard for
sending and receiving transparency artifacts between systems. 
This will enable more widespread "out of the box" integration support in the BOM ecosystem.
In addition, it will support automatic delivery of upstream vulnerability assessments,
such as VEX files.

## Data model

The data model is flexible to be able to handle many different use cases, from mobile applications and Open Source software to complex bundles of hardware sold in stores.

- __TEA Product Release__: The primary entry point. The __Transparency Exchange Identifier, TEI__ resolves to a specific Product Release. A Product Release belongs to a __TEA Product__.
- __TEA Product__: A higher-level object that groups a set of Product Releases for a product line or family. Products can be discovered and browsed.
- __TEA Component__: Represents a component lineage. A Component is a collection of Component Releases.
- __TEA Release__: A Component Release object. Each Component Release have its own TEA Collection.
- __TEA Collection__: A versioned list of artifacts for a specific Component Release or Product Release. Collections are versioned to indicate changes, e.g., an updated VEX or corrected SBOM.
- __TEA artifacts__: Files associated with a Collection. A single TEA artifact can appear in multiple Collections.

## Artifacts available using the API

The Transparency Exchange API (TEA) supports retrieval of a set of transparency exchange artifacts.
The API itself is not restricting the types of the artifacts published. A few examples:

### xBOM

Bill of materials for any type of component and service are supported. This includes, but is not limited to,
SBOM, HBOM, AI/ML-BOM, SaaSBOM, and CBOM. The API provides a BOM format agnostic way of publishing,
searching, and retrieval of xBOM artifacts.

### CDXA

Standards and requirements along with attestations to those standards and requirements are captured and supported
by CycloneDX Attestations (CDXA). Much like xBOM, these are supply chain artifacts that are captured allowing for
consistent publishing, searching, and retrieval.

### VDR/VEX

Vulnerability Disclosure Reports (VDR) and Vulnerability Exploitability eXchange (VEX) are supported artifact types.
Like the xBOM element, the VDR/VEX support is format agnostic. However, CSAF has its own distribution requirements
that may not be compatible with APIs. Therefore, the initial focus will be on CycloneDX (VDR and VEX) and OpenVEX.

### CLE

Product lifecycle events are communicated through the
ECMA-428 Common Lifecycle Enumeration standard.
This includes product rebranding, repackaging, mergers and acquisitions, and product milestone events such as end-of-life
and end-of-support.

Inclusion of CLE is optional and it may be introduced on the following levels:

- TEA Product
- TEA Component
- TEA Product Release
- TEA Component Release

If CLE is included, it is the responsibility of the TEA implementation to ensure consistency of
CLE events across the TEA Product and its releases and similarly across the TEA Component and its releases.

## Background

The Transparency Exchange API standard is created by OWASP CycloneDX in ECMA TC54 - Software and system transparency.
TEA is managed by ECMA TC54 TG1.
TEA depends on and is developed alongside with related ECMA standards, such as:

- PURL, Package URL ECMA-427
- CycloneDX ECMA-424
- Common Lifecycle Enumeration ECMA-428
- VERS, VErsion Range Specifier

For more information of these and releated standards, visit https://tc54.org
