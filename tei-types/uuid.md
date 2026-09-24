# TEI type `uuid`

## Type name

`uuid`

## Description

A UUID assigned by the vendor to a product release.

## Identifier syntax

The UUID in its standard textual form.

```text
tei://<domain-name>/uuid/<uuid>
```

## Canonical form

The 36-character hyphenated form in lowercase, as RFC 9562 specifies for output
(`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
Not used: uppercase hexadecimal, the form without hyphens, braces, or a `urn:uuid:` prefix.

## Example

```text
tei://cyclonedx.org/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1
```

## Specification

RFC 9562.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`permanent`

## Interoperability considerations

None.

## Security and privacy considerations

A UUID reveals nothing about the product; version 1 UUIDs may reveal the generating host's MAC address and the creation time.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
