[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](LICENSE)
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)
[![ECMA TC54](https://img.shields.io/badge/ECMA-TC54-FC7C00?labelColor=404040)](https://tc54.org)
[![ECMA TC54](https://img.shields.io/badge/ECMA-TC54--TG1-FC7C00?labelColor=404040)](https://ecma-international.org/task-groups/tc54-tg1/)

# CycloneDX Transparency Exchange API Standard

The Transparency Exchange API (TEA) is being worked on within the CycloneDX community
with the goal to standardise the API in ECMA. A working group within ECMA TC54 has been
formed - TC54 TG1. The working group has a slack channel in the CycloneDX slack space.

![](images/tealogo.png)

## Status of the standard: Beta 1

TEA is now in beta 1. This beta focuses on the consumer side of the API. Work on the
publisher API will start after the beta. The idea is to get implementation feedback
early on the current specification in order to move forward towards a first official
version of the standard. Feedback will be gathered in the Hackathon at OWASP AppSec
Global in Barcelona May 28 as well as in the meetings and slack channel.

We encourage developers to start with both client and server implementations of TEA and
participate in interoperability tests. These will be organised both as hackathons and
informally using the Slack channel.

There will likely be multiple beta releases. We will announce these by adding new
tags in the repository as well as in the slack channel.

## Introduction

This specification defines a standard, format agnostic, API for the exchange of
product related artefacts, like BOMs, between systems. The work includes:

- [Discovery of servers](/discovery/readme.md): Describes discovery using the Transparency Exchange Identifier (TEI)
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

## Data model

- [TEA Product](tea-product/tea-product): This is the starting point. A "product" is something for sale or distributed as an Open Source project. The [Transparency Exchange Identifier, TEI](/discovery/readme.md) points to a single product.
- [TEA Component index](tea-component/tea-component.md): A Component index is a version entry. The Component version index has one entry per version of the product.
- [TEA Collection](tea-collection/tea-collection.md): The collection is a list of artefacts for a specific version. The collection can be dynamic or static, depending on the implemenation.

## Artefacts available of the API

The Transparency Exchange API (TEA) supports publication and retrieval of a set of transparency exchange artefacts. The API itself should not be restricting the types of the artefacts. A few examples:

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

## Presentations and videos

- You can find presentations in the repository in the [Presentations](/presentations) directory
- Our biweekly meetings are available on [YouTube playlist: Project Koala](https://www.youtube.com/playlist?list=PLqjEqUxHjy1XtSzGYL7Dj_WJbiLu_ty58)
- KoalaCon 2024 - an introduction to the project - can be [viewed on YouTube](https://youtu.be/NStzYW4WnEE?si=ihLirpGVjHc7K4bL)

## Contributors

Contributors are listed in the [Contributors](contributors.md) file.

## Terminology

- API: Application programming interface
- Authorization (authz):
- Authentication (authn):
- Collection: A set of artifacts representing a version of a product
- Product: An item sold or delivered under one name
- Product variant: A variant of a product
- Version:

![](images/Project-Koala.svg)

## Contributing

### Markdown Formatting

This repository uses a Rust-based Markdown formatter (dprint) to ensure
consistent documentation formatting. When submitting pull requests that include
Markdown files, the formatter will automatically check for formatting issues.

To run the formatter locally:

1. Install dprint:
   ```bash
   cargo install dprint
   ```

2. Check for formatting issues:
   ```bash
   dprint check "**/*.md"
   ```

3. Automatically format all Markdown files:
   ```bash
   dprint fmt "**/*.md"
   ```

The formatter enforces a maximum line length of 80 characters and consistent
formatting across all Markdown files.

## Previous work

- [The CycloneDX BOM Exchange API](/api/bomexchangeapi.md)
   Implemented in the [CycloneDX BOM Repo Server](https://github.com/CycloneDX/cyclonedx-bom-repo-server)
