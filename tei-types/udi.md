# TEI type `udi`

## Type name

`udi`

## Description

A Unique Device Identifier of a medical device.

## Identifier syntax

The UDI device identifier (UDI-DI) part only, as its digits or characters,
without the production identifier (UDI-PI) part such as lot, serial number or expiry date.

```text
tei://<domain-name>/udi/<udi-di>
```

## Canonical form

The device identifier exactly as issued by the issuing agency, without application identifier or data identifier prefixes
(for a GS1 UDI-DI the 14-digit GTIN without the `(01)` prefix; for HIBCC, ICCBBA or IFA identifiers the string as printed in the UDI carrier).
Characters outside the unreserved set are percent-encoded when the TEI is written as a URL.
Not used: the full UDI including production identifiers, or a GTIN shortened by dropping leading zeros.

## Example

```text
tei://example.com/udi/00123456789012
```

## Specification

GS1 UDI, https://www.gs1.org/industries/healthcare/udi; the UDI rules of the relevant regulator (for example FDA, EU MDR).

## Change controller

TEA project (Ecma TC54 TG1)

## Status

`provisional`

## Interoperability considerations

UDI issuing agencies (GS1, HIBCC, ICCBBA, IFA) use different formats;
some formats contain characters that need percent-encoding in a URL path.
Only the device identifier is used, so one TEI covers all units of a model.

## Security and privacy considerations

A full UDI with production identifiers can identify an individual unit and, through it, a patient;
that is why only the device identifier part is used.

## Contact

TEA project, https://github.com/CycloneDX/transparency-exchange-api/issues
