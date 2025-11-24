# Transparency Exchange API: Consumer access


The consumer access starts with a TEI, A transparency Exchange Identifier. This is used to find the API server as
described in the [discovery document](/discovery/readme.md).

## API usage

The standard TEI points to a product release. A product release is something sold, downloaded as an opensource project or aquired by other means. It contains one or multiple component releases.

- __List of TEA Component Rleases__: Component releases are components of a product release.
  Each Component release has its own versioning and its own set of artefacts, they have a timestamp and   a lifecycle enumeration. They are normally sorted by timestamps. The TEA API has no requirements of type of version string (semantic or any other scheme) - it's just an identifier set by the manufacturer.
- __List of TEA Collections__: For each release, there is a list of TEA collections as indicated
  by release date and a version integer starting with collection version 1. 
- __List of TEA Artefacts__: The collection is unique for a version and contains a list of artefacts.
  This can be SBOM files, VEX, SCITT, IN-TOTO or other documents.  Note that a single artefact
  can belong to multiple Component or Product Releases.
- __List of artefact formats__: An artefact can be published in multiple formats.

The user has to know product release TEI and in some cases version of each component (TEA 
Component Release) to find the list of artefacts for the particular Product Release.

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
        tea_product_release -->> user: List of Component Releases
        user ->> user: Identify Discriminating Component Releases
        user ->> tea_component_release: Resolve Discriminating Component Releases
        tea_component_release -->> user: Discriminating Component Release Details
        user ->> user: Converge on a Single Product Release
    end

    user ->> tea_product_release: Resolve Product Release Details
    tea_product_release -->> user: List of Component Releases

    loop For each tea_component_release
        user ->> tea_component_release: Obtain latest collections
        tea_component_release -->> user: List of Artefacts
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
    participant tea_artefact as TEA Artefact


    user ->> tea_product_release: Search for product releases based on identifier (CPE, PURL, name)
    tea_product_release ->> user: List of product releases

    user ->> tea_product_release: Finding all product parts (TEA Component Releases) and facts     about choosen product
    tea_product_release ->> user: List of TEA Component Releases

    user ->> tea_component_release: Finding information of a component release
    tea_component_release ->> user: List of releases and collection id for each release

    user ->> tea_collection: Finding all artefacts for TEA Component Release
    tea_collection ->> user: List of artefacts and formats available for each artefact

    user ->> tea_artefact: Request to download artefact
    tea_artefact ->> user: Artefact

```

## API flow based on cached data - checking for a new release

In this case a TEA client knows the component UUID and wants to check the status of the
used release and if there's a new release. The client may limit the query with a given date
for a release.

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