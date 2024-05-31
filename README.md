[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](LICENSE)
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)
[![ECMA TC54](https://img.shields.io/badge/ECMA-TC54-404040?labelColor=FC7C00)](https://tc54.org)
[![ECMA TC54](https://img.shields.io/badge/ECMA-TC54--TG1-404040?labelColor=FC7C00)](https://ecma-international.org/task-groups/tc54-tg1/)

# CycloneDX Transparency Exchange API Standard

The Transparency API Exchange API is being worked on within the CycloneDX community
with the goal to standardise the API in ECMA. A working group within ECMA TC54 has been
formed - TC54 TG1. The working group has a slack channel in the CycloneDX slack space.

![](images/Project-Koala.svg)

## Introduction

This specification defines a standard, format agnostic, API for the exchange of
product related artefacts, like BOMs, between systems. The work includes:

- Discovery of servers
- Retrieval of artefacts
- Publication of artefacts
- Authentication and authorization
- Querying

System and tooling implementors are encouraged to adopt this API standard for
sending/receiving transparency artefacts between systems. 
This will enable more widespread
"out of the box" integration support in the BOM ecosystem.

## Use cases and requirements

The working group has produced a list of use cases and requirements for the protocol.

- [TEA requirements](doc/tea-requirements.md)
- [TEA use cases](doc/tea-usecases.md)

## Elements of the API

The Transparency Exchange API (TEA) supports publication and retrieval of a set of transparency exchange artefacts. A few examples:

### xBOM

Bill of materials for any type of component and service are supported. This includes, but is not limited to, SBOM, HBOM, AI/ML-BOM, SaaSBOM, and CBOM. The API provides a BOM format agnostic way of publishing, searching, and retrieval of xBOM artifacts. 

### CDXA

Standards and requirements along with attestations to those standards and requirements are captured and supported by CycloneDX Attestations (CDXA). Much like xBOM, these are supply chain artifacts that are captured allowing for consistent publishing, searching, and retrieval.

### VDR/VEX

Vulnerability Disclosure Reports (VDR) and Vulnerability Exploitability eXchange (VEX) are supported artifact types. Like the xBOM element, the VDR/VEX support is format agnostic. However, CSAF has its own distribution requirements that may not be compatible with APIs. Therefore, the initial focus will be on CycloneDX (VDR and VEX) and OpenVEX.

### CLE

Product lifecycle events that are captured and communicated through the Common Lifecycle Enumeration will be supported. This includes product rebranding, repackaging, mergers and acquisitions, and product milestone events such as end-of-life and end-of-support.

### Insights

Much of the focus on Software Transparency from the U.S. Government and others center around the concept of “full transparency”. Consumers often need to ingest, process, and analyze SBOMs or VEXs just to be able to answer simple questions such as:

- Do any of my licensed products from Vendor A use Apache Struts?
- Are any of my licensed products from Vendor A vulnerable to log4shell and is there any action I need to take?

Insights allows for “limited transparency” that can be asked and answered using an expression language that can be tightly scoped or outcome-driven. Insights also removes the complexities of BOM format conversion away from the consumers. An object model derived from CycloneDX will be an integral part of this API, since the objects within CycloneDX are self-contained (thus API friendly) and the specification supports all the necessary xBOM types along with CDXA.

### Transparency Exchange Identifier (TEI)
The TEI is a URN that is unique for a product. It can be resolved by DNS to locate a Collection

## Terminology

- API: Application programming interface
- Authorization (authz):
- Authentication (authn):
- Collection: A set of artifacts representing a version of a product
- Product: An item sold or delivered under one name (?)
- Product variant: A variant of a product
- Version:



## Previous work

- [The CycloneDX BOM Exchange API](/api/bomexchangeapi.md)
   Implemented in the [CycloneDX BOM Repo Server]
   (https://github.com/CycloneDX/cyclonedx-bom-repo-server)
