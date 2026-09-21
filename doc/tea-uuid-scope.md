# TEA UUID Scope and Stability

This document defines the scope of uniqueness and the stability guarantees of UUIDs used by TEA objects (TEA Product, TEA Product Release, TEA Component, TEA Component Release, TEA Collection, TEA Artifact).

## Uniqueness scope

A TEA UUID alone is **not** unique across TEA servers. Within one authoritative domain, a UUID shall identify at most one TEA object, except as follows.

- The **authoritative domain** is supplied by the TEI (`tei://<domain-name>/<type>/<unique-identifier>`); see [discovery](../discovery/readme.md). TEA itself has no centralized authority that can police UUIDs across servers, so cross-server uniqueness is not enforceable and is not claimed.

**Exception: Collection and its parent release.** A TEA Collection is the versioned, mutable companion of a Product Release or Component Release and is addressable on its own. The Collection shall use the same UUID as that parent Product Release or Component Release. No other cross-type UUID reuse is permitted. In particular, a Product Release and a Component Release shall not share a UUID: that would yield two Collections with the same UUID under one authoritative domain.

Object types in scope are Product, Product Release, Component, Component Release, Collection, and Artifact.

Implementations may use any UUID generator; the choice neither violates nor strengthens this specification. The rules above are requirements on the server: it shall not assign a UUID that is already in use, except when creating the Collection that belongs to an existing Product Release or Component Release and reuses that release's UUID.

## Stability

All TEA UUIDs shall be stable for the lifetime of the resource they identify, within a given TEA server. Specifically:

- A UUID shall not be reassigned to a different resource.
- A UUID shall not change across backup/restore or re-hosting of the same TEA service instance.

Stability is required for **all** TEA object types, not only TEA Product Release. While TEI resolution targets only Product Release UUIDs, clients routinely bookmark and cache sub-resource URLs (e.g. `/component/{uuid}`, `/componentRelease/{uuid}`, `/productRelease/{uuid}`); changing those UUIDs would silently break those clients.
