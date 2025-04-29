# Transparency Exchange API: Consumer access


The consumer access starts with a TEI, A transparency Exchange Identifier. This is used to find the API server as
described in the [discovery document](/discovery/readme.md).

## API usage

The standard TEI points to a product.

- __TEA Product__: This is the delivered goods, an open source software, library or a product sold. It consists of one or multiple TEA Components.
- __TEA Components__: Components are components of something sold. Each Component has it's own versioning and it's own set of artefacts. Note that a single artefact can belong to multiple versions of a Component and multiple Components.
- __TEA Component index__: A list of all the versions available for a TEA Component
- __TEA Collection__: For each Component version, there is TEA collection as indicated by release date and a version string. The TEA API has no requirements of type of version string (semantic or any other scheme) - it's just an identifier set by the manufacturer. It's sorted by release date as a default.
- __List of TEA artefacts__: The TEA Collection is unique for a version and contains a list of artefacts. This can be SBOM files, VEX, SCITT, IN-TOTO or other documents.
- __List of artefact formats__: An artefact can be published in multiple formats.

The user has to know product TEI and version of each component (TEA Component) to find the list of artefacts for the used version.

## API flow

```mermaid

---
title: TEA consumer
---

sequenceDiagram
    autonumber
    actor user
    participant discovery as TEA Discovery with TEI

    participant tea_product as TEA Product
    participant tea_component as TEA Component
    participant tea_collection as TEA Collection
    participant tea_artifact as TEA Artefact


    user ->> discovery: Discovery using DNS
    discovery ->> user: List of API servers

    user ->> tea_product: Finding all product parts (TEA Components) and facts about the product
    tea_product ->> user: List of product parts

    user ->> tea_component: Finding all versions of a TEA Component
    tea_component ->> user: List of all available versions (paginated)

    user ->> tea_collection: Finding all artefacts for version in scope
    tea_collection ->> user: List of artefacts and formats available for each artefact

    user ->> tea_artifact: Download artefact



```
