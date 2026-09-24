# TEI type `gtin`

## Type name

`gtin`

## Description

A GS1 Global Trade Item Number of the product as sold.

## Identifier syntax

The number as its digits.

```text
tei://<domain-name>/gtin/<gtin-number>
```

## Canonical form

The GTIN exactly as assigned, with its check digit, in its own length
(GTIN-8, GTIN-12, GTIN-13 or GTIN-14).
Not used: zero-padding a shorter GTIN to 14 digits, or an application identifier prefix such as `(01)`.

## Example

```text
tei://example.org/gtin/0234567890123
```

## Specification

GS1 GTIN, https://www.gs1.org/standards/id-keys/gtin.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`permanent`

## Interoperability considerations

As for `eanupc`: one GTIN may cover several product releases.
GTIN-8, GTIN-12, GTIN-13 and GTIN-14 spellings of one item give different TEIs; publishers should use one length consistently.

## Security and privacy considerations

None.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
