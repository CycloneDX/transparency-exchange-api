# TEI type `eanupc`

## Type name

`eanupc`

## Description

An EAN or UPC article number of the product as sold.

## Identifier syntax

The number as its digits.

```text
tei://<domain-name>/eanupc/<ean-or-upc-number>
```

## Canonical form

The number exactly as assigned, with its check digit:
12 digits for a UPC-A, 13 for an EAN-13, 8 for an EAN-8 or UPC-E.
Not used: a UPC-A written as a 13-digit EAN with a leading zero, or any zero-padding to 14 digits;
those spellings are different TEIs.

## Example

```text
tei://example.com/eanupc/1234567890123
```

## Specification

GS1 General Specifications, https://www.gs1.org/standards/barcodes-epcrfid-id-keys/gs1-general-specifications.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`provisional`

## Interoperability considerations

The same number may be used for several product releases (for example successive firmware versions of one boxed product),
so a discovery lookup may return several product releases.
Vendors that need one TEI per release should prefer `uuid` or `hash`.
EAN-13 and UPC-A numbers for the same article differ by a leading zero and give different TEIs.

## Security and privacy considerations

None.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
