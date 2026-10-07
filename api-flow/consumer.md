# Transparency Exchange API: Consumer access


The consumer access starts with a TEI, A Transparency Exchange Identifier. This is used to find the API server as
described in the [discovery document](/discovery/readme.md).

## API usage

The standard TEI points to a product release. A product release is something sold,
downloaded as an open source project or acquired by other means. A Product Release
contains component references. Each reference identifies a Component and can also
identify a particular Component Release.

- __List of component references__: each entry identifies a Component by UUID. Some
  entries also identify a particular Component Release by UUID; those without a
  release UUID do not select a release.
- Where a Component Release is retrieved, it has its own versioning and artefacts,
  a timestamp and a lifecycle enumeration. Component Releases are normally sorted
  by timestamps. The TEA API has no requirements of type of version string
  (semantic or any other scheme) - it's just an identifier set by the manufacturer.
- __List of TEA Collections__: For each release, there is a list of TEA collections as indicated
  by release date and a version integer starting with collection version 1.
- __List of TEA Artifacts__: The collection is unique for a version and contains a list of artefacts.
  This can be SBOM files, VEX, SCITT, IN-TOTO or other documents.  Note that a single artefact
  can belong to multiple Component or Product Releases.
- __List of artefact formats__: An artefact can be published in multiple formats.

Finding the list of artefacts for a Product Release starts from the product
release TEI. When a component reference includes a Component Release UUID, the
client can use it to retrieve that release directly.

## API flow based on TEI discovery

```mermaid

---
title: TEA consumer flow
---
sequenceDiagram
    autonumber
    actor manufacturer as Manufacturer
    actor user as TEA Client

    participant discovery as TEA Discovery / TEA Server
    participant tea_product_release as TEA Product Release
    participant tea_component_release as TEA Component Release

    manufacturer ->> user: Provides TEI (Transparency Exchange Identifier)

    user ->> discovery: GET https://<domain>/.well-known/tea
    discovery -->> user: TEA discovery document (API servers & endpoints)

    user ->> discovery: Call Discovery endpoint with TEI
    discovery -->> user: Product Release reference(s)

    alt Multiple Product Releases returned
        user ->> tea_product_release: Resolve Product Release(s)
        tea_product_release -->> user: Product Releases with component references
        opt Additional details needed for a referenced Component Release
            user ->> tea_component_release: Resolve the referenced Component Release
            tea_component_release -->> user: Component Release details
        end
        user ->> user: Identify the relevant Product Release
    end

    user ->> tea_product_release: Resolve Product Release Details
    tea_product_release -->> user: Product Release with its latest Collection and component references

    loop For each component reference that includes a Component Release UUID
        user ->> tea_component_release: Resolve that Component Release
        tea_component_release -->> user: Component Release with its latest Collection (list of TEA Artifacts)
    end

```

## API flow based on direct access to API

In this case, the client wants to search for a specific product release using the API

```mermaid

---
title: TEA client flow with search
---

sequenceDiagram
    autonumber
    actor user

    participant tea_product_release as TEA Product Release
    participant tea_component_release as TEA Component Release
    participant tea_collection as TEA Collection
    participant tea_artifact as TEA Artifact


    user ->> tea_product_release: Search for product releases based on identifier (CPE, PURL, TEI)
    tea_product_release ->> user: List of product releases

    user ->> tea_product_release: Finding component references and facts about chosen product
    tea_product_release ->> user: Product Release with component references

    opt Component reference includes a Component Release UUID
        user ->> tea_component_release: Retrieve the referenced Component Release
        tea_component_release ->> user: Component Release with its latest Collection
        user ->> tea_collection: Retrieve artefacts for that Component Release
        tea_collection ->> user: Artefacts and their available formats
        user ->> tea_artifact: Request to download TEA artifact
        tea_artifact ->> user: TEA Artifact
    end

```

## API flow based on cached data - checking for a new release

In this case a TEA client knows the component UUID and wants to check the status of the
used release and if there's a new release.

```mermaid

---
title: TEA client flow with direct query for release
---

sequenceDiagram
    autonumber
    actor user

    participant tea_component as TEA Component
    participant tea_component_release as TEA Component Release

    user ->> tea_component: Finding a specific version/release
    tea_component ->> user: List of releases and collection id for each release

    user ->> tea_component_release: Details for discovered release
    tea_component_release ->> user: Collection with artefact details for the release

```

## API flow based on cached data  - checking if a collection changed

In this case a TEA client knows the release UUID, the collection UUID, and the
collection version from previous queries. If the given version is not the same,
another query is done to get reason for update and new collection list of artefacts.


```mermaid

---
title: TEA client collection query
---

sequenceDiagram
    autonumber
    actor user

    participant tea_collection as TEA Collection


    user ->> tea_collection: Finding the current collection, including version
    tea_collection ->> user: List of artefacts and formats available for each artefact

    user ->> tea_collection: Request to access previous version of the collection to compare
    tea_collection ->> user: Previous version of collection

```