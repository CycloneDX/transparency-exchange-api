# TEA UUID Scope and Stability

This document defines the scope of uniqueness and the stability guarantees of UUIDs used by TEA objects (TEA Product, TEA Product Release, TEA Component, TEA Release, TEA Collection, TEA Artifact).

## Uniqueness scope

A TEA UUID alone is **not** globally unique. Global uniqueness in TEA is achieved by the tuple:

```text
authoritative domain  +  object type  +  UUID
```

- The **authoritative domain** is supplied by the TEI (`urn:tei:<type>:<domain-name>:<unique-identifier>`); see [discovery](../discovery/readme.md). TEA itself has no centralized authority that can police UUIDs across servers, so cross-server uniqueness is not enforceable and is not claimed.
- The **object type** scopes uniqueness to a single object class (Product, Product Release, Component, Release, Collection, Artifact). A TEA server MUST guarantee that UUIDs are unique within `(authoritative domain, object type)`. UUIDs are not required to be unique across object types.

The per-object-type scoping reflects how TEA servers are typically implemented: each object type lives in its own database table, where uniqueness is trivially enforced by a primary key. Cross-table uniqueness offers no functional benefit - TEA has no operation that resolves a UUID without already knowing its object type.

Implementations MAY use generators that produce globally unique UUIDs (e.g. RFC 4122). Doing so neither violates nor strengthens this specification.

## Stability

All TEA UUIDs MUST be stable for the lifetime of the resource they identify, within a given TEA server. Specifically:

- A UUID MUST NOT be reassigned to a different resource.
- A UUID MUST NOT change across backup/restore or re-hosting of the same TEA service instance.

Stability is required for **all** TEA object types, not only TEA Product Release. While TEI resolution targets only Product Release UUIDs, clients routinely bookmark and cache sub-resource URLs (e.g. `/component/{uuid}`, `/release/{uuid}`, `/collection/{uuid}`); changing those UUIDs would silently break those clients.
