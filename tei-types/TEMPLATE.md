# TEI type registration template

Copy this file to `<type>.md`, fill in every field, and open a pull request.
Fields follow the shape of registration templates used by IETF registries.

## Type name

The `<type>` value: lowercase ASCII letters and digits, starting with a letter.

## Description

What identifier scheme the `<unique-identifier>` carries,
and what kind of object it identifies (a product release, a package, a device, ...).

## Identifier syntax

How the identifier is written inside a TEI:
allowed characters, encoding (for example Base64URL without padding),
case, separators, and anything a publisher must do to obtain the canonical spelling.
The result shall be a single path segment
and shall not contain `/`, neither literally nor percent-encoded as `%2F`.

## Canonical form

If the identifier scheme allows several equivalent spellings
(letter case, leading zeros or padding, prefixes, alternative encodings),
state the single spelling used inside a TEI.
TEIs are compared as character strings, so publishers shall use this spelling only;
list the equivalent forms that shall not be used.

## Example

At least one complete TEI using an example domain.

## Specification

The document that defines the identifier scheme
(a standard, an RFC, or the organisation that issues the identifiers), with a link.

## Change controller

Who may change this registration.
Use "TEA project (Ecma TC54 TG1)" unless the type is controlled by another organisation.

## Status

One of:

- `provisional`: registered, may still change;
- `permanent`: stable, changes require a new type;
- `deprecated`: should not be used for new TEIs; the entry stays for existing ones.

## Interoperability considerations

Anything a client or server must know to handle the type correctly,
for example whether the same identifier may map to several product releases,
or whether the scheme has variants that must not be mixed.

## Security and privacy considerations

Whether the identifier may reveal information about a person or an installation,
or can be guessed or enumerated.

## Contact

Name and email address, or a GitHub handle, of the person submitting the registration.
