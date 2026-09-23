# TEA UUID Scope and Stability

This document defines the scope of uniqueness and the stability guarantees of UUIDs used by TEA objects (TEA Product, TEA Product Release, TEA Component, TEA Component Release, TEA Collection, TEA Artifact).

## Uniqueness scope

A TEA UUID alone is **not** globally unique. Global uniqueness in TEA is achieved by the tuple:

```text
authoritative domain  +  object type  +  UUID
```

- The **authoritative domain** is supplied by the TEI (`tei://<domain-name>/<type>/<unique-identifier>`); see [discovery](../discovery/readme.md). TEA itself has no centralised authority that can police UUIDs across servers, so cross-server uniqueness is not enforceable and is not claimed.
- The **object type** scopes uniqueness to a single object class (Product, Product Release, Component, Component Release, Collection, Artifact). A TEA server shall guarantee that UUIDs are unique within `(authoritative domain, object type)`. Except as noted below, UUIDs are not required to be unique across object types.

**Exception: Product Release and Component Release.** A TEA Collection shall use the same UUID as its parent Product Release or Component Release. Within an authoritative domain, a Product Release and a Component Release shall not share a UUID. Consequently, Collections belonging to different parent releases have different UUIDs. Versions of the same Collection retain the same UUID. Except for Product Release and Component Release, objects of different types may share a UUID.

The per-object-type scoping reflects how TEA servers are typically implemented: each object type lives in its own database table, where uniqueness is trivially enforced by a primary key. Cross-table uniqueness is not required in general (TEA has no operation that resolves a UUID without already knowing its object type), except that Product Release and Component Release UUIDs shall be disjoint as above, because those types share the Collection UUID namespace.

Implementations may use any UUID generator; the choice neither violates nor strengthens this specification. The exception above is a requirement on the server: it shall not assign a UUID that is already in use for the other release type, however UUIDs are produced.

## Stability

All TEA UUIDs shall be stable for the lifetime of the resource they identify, within a given TEA server. Specifically:

- A UUID shall not be reassigned to a different resource.
- A UUID shall not change across backup/restore or re-hosting of the same TEA service instance.

Stability is required for **all** TEA object types, not only TEA Product Release. While TEI resolution targets only Product Release UUIDs, clients routinely bookmark and cache sub-resource URLs (e.g. `/component/{uuid}`, `/componentRelease/{uuid}`, `/productRelease/{uuid}`); changing those UUIDs would silently break those clients.
