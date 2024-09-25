# Transparency Exchange API: Consumer access


The consumer access starts with a TEI, A transparency Exchange Identifier. This is used to find the API server as 
described in the [discovery document](/discovery/readme.md).

## API usage

The standard TEI points to a product.

- __List of TEA leafs__: Leafs are components of something sold. Each leaf has it's own versioning and it's own set of artefacts. Note that a single artefact can belong to multiple versions of a leaf and multiple leafs.
- __List of TEA collections__: For each leaf, there is a list of TEA collections as indicated by release date and a version string. The TEA API has no requirements of type of version string (semantic or any other scheme) - it's just an identifier set by the manufacturer. It's sorted by release date as a default.
- __List of TEA artefacts__: The collection is unique for a version and contains a list of artefacts. This can be SBOM files, VEX, SCITT, IN-TOTO or other documents.
- __List of artefact formats__: An artefact can be published in multiple formats.

The user has to know product TEI and version of each component (TEA LEAF) to find the list of artefacts for the used version.

## API flow

```mermaid

---
title: TEA consumer
---

sequenceDiagram
    autonumber
    actor user
    participant discovery as TEA Discovery with TEI
    box LightGrey TEA API service
    participant teaindex as TEA Index
    end
   
  

    user ->> discovery: Discovery using DNS
    discovery ->> user: List of API servers

    user ->> teaindex: Finding all product parts
    teaindex ->> user: List of product parts
    create participant tealeaf as TEA Leaf Index
    user ->> tealeaf: Finding all versions of a part
    tealeaf ->> user: List of all available versions (paginated)
    create  participant teacoll as TEA Collection
    user ->> teacoll: Finding all artefacts for version in scope
    teacoll ->> user: List of artefacts and formats available for each artefact
    create participant artefact as Artefact
    user ->> artefact: Download artefact



```
