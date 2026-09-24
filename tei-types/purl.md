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
tei://cyclonedx.org/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWJAOC40LjA_ZXh0ZW5zaW9uPXdobCZxdWFsaWZpZXI9cHkzLW5vbmUtYW55

(the PURL `pkg:pypi/cyclonedx-python-lib@8.4.0?extension=whl&qualifier=py3-none-any`)
```

## Specification

Package URL, ECMA-427.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`permanent`

## Interoperability considerations

The PURL is encoded, not percent-escaped, so a TEI comparison never decodes it;
two PURLs that differ only in canonicalisation produce different TEIs.
Publishers shall canonicalise the PURL before encoding.

## Security and privacy considerations

None beyond those of the PURL itself.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
