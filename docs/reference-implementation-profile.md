# Reference Implementation Profile

This repository is primarily two things:

1. the evolving TEA specification surface, centered on the protobuf contracts in
   [`proto/`](../proto)
2. a Rust reference server that demonstrates the normative flows safely and
   explicitly

It is not intended to claim that every optional TEA deployment capability is
finished, turnkey, or universally enabled.

## Contract

- The specification is the source of truth.
- The Rust server is the executable reference implementation.
- Unsupported optional capabilities must fail closed, not pretend to work.
- Production hardening is valuable, but it should not obscure where the spec
  ends and the reference profile begins.

## Current Reference Scope

### Canonical surfaces

- `proto/tea/v1/*.proto` defines the normative gRPC contract
- REST mappings in the protobuf annotations define the canonical HTTP shape
- `proto/README.md` explains the contract and current reference-server coverage

### Rust reference server

The current Rust server focuses on:

- discovery and consumer reads
- authenticated publisher metadata writes for products, product releases,
  components, and component releases
- artifact registration from a verified immutable URL
- collection creation and immutable versioned updates
- safe delete flows that preserve referential integrity

The current Rust server intentionally does not claim full support for:

- raw artifact upload streaming
- batch artifact upload
- collection signing
- bulk collection import

Those surfaces remain part of the canonical specification, but the reference
server returns explicit `UNIMPLEMENTED` errors until the backing storage,
signing, and lifecycle semantics are complete enough to demonstrate safely.

## Why this split matters

Keeping the spec and the reference profile distinct helps us:

- evolve the TEA contract without overstating implementation maturity
- give implementers a trustworthy baseline for the stable flows
- make unsupported areas obvious to integrators
- avoid shipping "demo success" behavior for operations that still need real
  storage, signing, or migration semantics

## Practical guidance

If you are extending TEA:

1. update the protobuf/OpenAPI contract first
2. document lifecycle and failure semantics
3. add or adjust reference-server behavior for the stable subset
4. leave incomplete optional features fail-closed until they are ready

That keeps the repository honest and makes the reference implementation more
useful to other teams.
