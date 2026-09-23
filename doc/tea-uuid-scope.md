# TEA UUID Scope and Stability

This document defines the scope of uniqueness and the stability guarantees of UUIDs used by TEA objects (TEA Product, TEA Product Release, TEA Component, TEA Component Release, TEA Collection, TEA Artifact).

## Uniqueness scope

Paths such as `/productRelease/{uuid}` name an object by the API base and the UUID.
The request does not carry the TEI authoritative domain, so two different objects
that share a UUID at the same API base cannot be distinguished.

A TEA UUID alone is **not** globally unique. Uniqueness for retrieval is the tuple:

```text
API base + object type + UUID
```

- The **API base** is the base URL of the TEA API the client calls. For each object
  type, a TEA server shall ensure that a UUID refers to at most one object served at
  that API base.
- The **authoritative domain** is supplied by the TEI
  (`tei://<domain-name>/<type>/<unique-identifier>`); see [discovery](../discovery/readme.md).
  It tells a client which API to call. It is not part of a UUID path. TEA has no
  centralized authority that assigns UUIDs, so uniqueness across different API bases
  is not enforced by the protocol.
- The **object type** scopes uniqueness to a single object class (Product, Product
  Release, Component, Component Release, Collection, Artifact). Except as noted below,
  UUIDs are not required to be unique across object types.

If two authoritative domains would assign the same UUID to different objects of the
same type, those domains shall be served from different API bases. One API base shall
not host both.

**Exception: Product Release and Component Release.** A TEA Collection shall use the same UUID as its parent
Product Release or Component Release. Within an API base, a Product Release and a Component Release
shall not share a UUID. Consequently, Collections belonging to different parent releases have different UUIDs.
Versions of the same Collection retain the same UUID. Except for Product Release and Component Release, objects
of different types may share a UUID.

The per-object-type scoping reflects how TEA servers are typically implemented: each object type lives in its
own database table, where uniqueness is trivially enforced by a primary key. Cross-table uniqueness is not
required in general (TEA has no operation that resolves a UUID without already knowing its object type),
except that Product Release and Component Release UUIDs shall be disjoint as above, because those types
share the Collection UUID namespace.

Implementations may use any UUID version defined in RFC 9562. Implementations should
create UUIDs with a cryptographically secure random number generator: UUID version 4,
or version 7 whose random bits are drawn that way. Independently created UUIDs are
then unique across TEA servers for practical purposes. This does not replace the rule
above. A server shall not assign a UUID that is already in use at its API base for
that object type, including a UUID already used for the other release type.

## Stability

All TEA UUIDs shall be stable for the lifetime of the resource they identify, within a given TEA server. Specifically:

- A UUID shall not be reassigned to a different resource.
- A UUID shall not change across backup/restore or re-hosting of the same TEA service instance.

Stability is required for **all** TEA object types, not only TEA Product Release. While TEI resolution targets only Product Release UUIDs, clients routinely bookmark and cache sub-resource URLs (e.g. `/component/{uuid}`, `/componentRelease/{uuid}`, `/productRelease/{uuid}`); changing those UUIDs would silently break those clients.
