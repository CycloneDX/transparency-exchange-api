# Overview of the TEA API from a producer standpoint

## Bootstrapping

```mermaid
sequenceDiagram
    autonumber
    actor Vendor
    participant tea_product as TEA Product
    participant tea_leaf as TEA Leaf
    participant tea_collection as TEA Collection

    Vendor ->> tea_product: POST to /v1/product to create new product
    tea_product -->> Vendor: Product is created and TEA Product Identifier (PI) returned

    Vendor ->> tea_leaf: POST to /v1/leaf with the TEA PI and leaf version as the payload
    tea_leaf ->> Vendor: Leaf is created and a TEA Leaf ID is returned

    Vendor ->> tea_collection: POST to /v1/collection with the TEA Leaf ID as the and the artifact as payload
    tea_collection ->> Vendor: Collection is created with the collection ID returned
```

## Release life cycle

```mermaid
sequenceDiagram
    autonumber
    actor Vendor
    participant tea_product as TEA Product
    participant tea_leaf as TEA Leaf
    participant tea_collection as TEA Collection

    Note over Vendor,tea_leaf: Create new release

    Vendor ->> tea_leaf: POST to /v1/leaf with the TEA PI and leaf version as the payload
    tea_leaf ->> Vendor: Leaf is created and a TEA Leaf ID is returned

    Note over Vendor,TEA Leaf: Add an artifact (e.g. SBOM)
    Vendor ->> tea_collection: POST to /v1/collection with the TEA Leaf ID as the and the artifact as payload
    tea_collection ->> Vendor: Collection is created with the collection ID returned
```

## Adding a new artifact

```mermaid
sequenceDiagram
    autonumber
    actor Vendor
    participant tea_product as TEA Product
    participant tea_leaf as TEA Leaf
    participant tea_collection as TEA Collection

    Vendor ->> tea_leaf: GET to /v1/leaf with the TEA PI to get the latest version
    tea_leaf ->> Vendor: Leaf will be returned

    Vendor ->> tea_collection: POST to /v1/collection with the TEA Leaf ID as the and the artifact as payload
    tea_collection ->> Vendor: Collection is created with the collection ID returned
```
