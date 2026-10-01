# TEI type `purl`

## Type name

`purl`

## Description

A Package URL (PURL) identifying the package a product release is distributed as.

## Identifier syntax

The canonical PURL string, encoded using Base64URL (RFC 4648 section 5) without padding.
The encoded form consists only of the characters `A-Z`, `a-z`, `0-9`, `-` and `_`,
so a `purl` TEI never needs percent-encoding.
Decoders shall accept the unpadded form.

```text
tei://<domain-name>/purl/<base64url-of-purl>
```

## Canonical form

The PURL is canonicalised as defined by ECMA-427 before encoding
(lowercase scheme and type, percent-encoding as prescribed, qualifiers sorted by key, no empty qualifiers),
then Base64URL-encoded without padding.
Not used: a padded Base64URL string, standard Base64 (`+`, `/`), or a percent-encoded PURL in place of the encoding.

## Example

```text
tei://example.com/purl/cGtnOm1hdmVuL2NvbW1vbnMtaW8vY29tbW9ucy1pb0AyLjIyLjA

(the PURL `pkg:maven/commons-io/commons-io@2.22.0`)
```

## Specification

Package URL, ECMA-427.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`provisional`

## Interoperability considerations

Publishers shall canonicalize the PURL before Base64URL encoding. PURLs with the same
canonical form produce the same identifier segment. TEI comparison operates on the
encoded identifier and does not decode the PURL.

## Security and privacy considerations

None beyond those of the PURL itself.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
