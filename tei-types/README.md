# TEI type registry

This directory is the registry of Transparency Exchange Identifier (TEI) types.
The [discovery document](/discovery/readme.md) defines the TEI syntax
`tei://<domain-name>/<type>/<unique-identifier>`;
each file in this directory defines one value of `<type>`
and the form of the `<unique-identifier>` that goes with it.

The registry is maintained outside the TEA standard,
because types are added over time
and the standard should not need a new edition for each of them.
The standard includes examples of registered types for illustration only;
this registry is the authoritative list.

## Registered types

| Type | Unique identifier | Status | Definition |
|---|---|---|---|
| `purl` | Package URL, Base64URL-encoded without padding | provisional | [purl.md](purl.md) |
| `hash` | Hash of an object, `<hashtype>:<hex>` | provisional | [hash.md](hash.md) |
| `uuid` | UUID | provisional | [uuid.md](uuid.md) |
| `eanupc` | EAN or UPC number | provisional | [eanupc.md](eanupc.md) |
| `gtin` | GTIN | provisional | [gtin.md](gtin.md) |
| `asin` | Amazon Standard Identification Number | provisional | [asin.md](asin.md) |
| `udi` | Unique Device Identifier (device identifier part) | provisional | [udi.md](udi.md) |

## Registering a type

To register a new TEI type, fill in the [registration template](TEMPLATE.md)
and open a pull request adding it as `<type>.md` to this directory,
or open an issue in the TEA issue tracker at
https://github.com/CycloneDX/transparency-exchange-api/issues with the same content.
The TEA project reviews the registration for completeness of the template,
for a `<type>` value that is not already registered,
and for an identifier syntax that yields a single path segment.
A type is registered when its definition file is merged.

Each registration carries a status:
`provisional` (registered, may still change),
`permanent` (stable; changes require a new type)
or `deprecated` (not for new TEIs; the entry stays for existing ones).

A `<type>` value consists of lowercase ASCII letters and digits and starts with a letter.
A unique identifier is scoped to its type: the same value under two types gives two different TEIs.
