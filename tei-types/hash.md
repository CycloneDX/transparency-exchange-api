# TEI type `hash`

## Type name

`hash`

## Description

A hash of an object chosen by the vendor, for example a distributed file.

## Identifier syntax

`<hashtype>:<hash>`, where `<hashtype>` is one of the values of `checksum-type` in the TEA OpenAPI specification
(for example `SHA-256`, `SHA-384` or `SHA-512`)
and `<hash>` is the digest in lowercase hexadecimal.
The colon keeps the identifier a single path segment.
What is hashed is up to the vendor to define.

```text
tei://<domain-name>/hash/<hashtype>:<hash>
```

## Canonical form

The hash type is spelled exactly as in `checksum-type` (for example `SHA-256`, not `sha256` or `SHA256`),
and the digest is lowercase hexadecimal.
Not used: uppercase hexadecimal, Base64 or other digest encodings, or a `<hashtype>/<hash>` form with a slash.

## Example

```text
tei://example.com/hash/SHA-256:fd44efd601f651c8865acf0dfeacb0df19a2b50ec69ead0262096fd2f67197b9
```

## Specification

The hash algorithms are those of `checksum-type` in the TEA OpenAPI specification.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`provisional`

## Interoperability considerations

The same bytes hashed with two algorithms give two different TEIs.
A client that holds only the bytes must know which algorithm the vendor used.

## Security and privacy considerations

A hash of a publicly distributed file can be computed by anyone; it identifies the file, not a customer.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
