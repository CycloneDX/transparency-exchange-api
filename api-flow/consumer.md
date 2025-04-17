# Transparency Exchange API: Consumer access

The consumer access starts with a TEI, A transparency Exchange Identifier. This
is used to find the API server as described in the
[discovery document](/discovery/readme.md).

## API usage

The standard TEI points to a product.

- **List of TEA leafs**: Leafs are components of something sold. Each leaf has
  it's own versioning and it's own set of artefacts. Note that a single artefact
  can belong to multiple versions of a leaf and multiple leafs.
- **List of TEA collections**: For each leaf, there is a list of TEA collections
  as indicated by release date and a version string. The TEA API has no
  requirements of type of version string (semantic or any other scheme) - it's
  just an identifier set by the manufacturer. It's sorted by release date as a
  default.
- **List of TEA artefacts**: The collection is unique for a version and contains
  a list of artefacts. This can be SBOM files, VEX, SCITT, IN-TOTO or other
  documents.
- **List of artefact formats**: An artefact can be published in multiple
  formats.

The user has to know product TEI and version of each component (TEA LEAF) to
find the list of artefacts for the used version.

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
    participant tea_leaf as TEA Leaf
    participant tea_collection as TEA Collection
    participant tea_artifact as TEA Artefact


    user ->> discovery: Discovery using DNS
    discovery ->> user: List of API servers

    user ->> tea_product: Finding all product parts
    tea_product ->> user: List of product parts

    user ->> tea_leaf: Finding all versions of a part
    tea_leaf ->> user: List of all available versions (paginated)

    user ->> tea_collection: Finding all artefacts for version in scope
    tea_collection ->> user: List of artefacts and formats available for each artefact

    user ->> tea_artifact: Download artefact
```
