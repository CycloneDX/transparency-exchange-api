# TEI type `asin`

## Type name

`asin`

## Description

An Amazon Standard Identification Number of the product as sold on Amazon.

## Identifier syntax

The ten-character ASIN, uppercase letters and digits.

```text
tei://<domain-name>/asin/<asin-identifier>
```

## Canonical form

The ten-character ASIN in uppercase, as displayed by Amazon.
Not used: lowercase letters, or an ISBN-10 in place of the ASIN of a book even though Amazon treats them as equal.

## Example

```text
tei://example.com/asin/B07FZ8S74R
```

## Specification

Amazon, https://sell.amazon.com/blog/what-is-an-asin.

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`provisional`

## Interoperability considerations

ASINs are assigned by a single commercial party and may be reassigned or retired.
One ASIN may cover several product releases.

## Security and privacy considerations

None.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
