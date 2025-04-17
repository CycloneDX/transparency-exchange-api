# Transparency Exchange API - Discovery

**NOTE**: _This is a proposal for the WG_

- [From product identifier to API endpoint](#from-product-identifier-to-api-endpoint)
- [TEA Discovery - defining an extensible identifier](#tea-discovery---defining-an-extensible-identifier)
- [The TEI URN: An extensible identifier](#the-tei-urn-an-extensible-identifier)
  - [TEI syntax](#tei-syntax)
  - [TEI types](#tei-types)
  - [TEI resolution using DNS](#tei-resolution-using-dns)
  - [Finding the Index using DNS result](#finding-the-index-using-dns-result)
- [The TEA Version Index](#the-tea-version-index)
- [References](#references)

## From product identifier to API endpoint

TEA Discovery is the connection between a product identifier and the API
endpoint. A "product" is something that the customer aquires or downloads. It
can be a bundle of many digital devices or software applications. A "product"
normally also has an entry in a large corporation's asset inventory system.

A product identifier is embedded in a URN where the identifier is one of many
existing identifiers or a random string - like an EAN or UPC bar code, UUID,
product number or PURL.

The goal is for a user to add this URN to the transparency platform (sometimes
with an associated authentication token) and have the platform access the
required artifacts in a highly automated fashion.

## Advertising the TEI

The TEI for a product can be communicated to the user in many ways.

- A QR code on a box
- On the invoice or delivery note
- For software with a GUI, in an "about" box

## TEA Discovery - defining an extensible identifier

TEA discovery is the process where a user with a product identifier can discover
and download artifacts automatically, with or without authentication. A globally
unique identifier is required for a given product. This identifier is called the
Transparency Exchange Identifier (TEI).

The TEI identifier is based on DNS, which assures a uniqueness per vendor (or
open source project) and gives the vendor a name space to define product
identifiers based on existing or new identifiers like EAN/UPC bar code, PURLs or
other existing schemes. A given product may have multiple identifiers as long as
they all resolve into the same destination.

## The TEI URN: An extensible identifier

The TEI, Transparency Exchange Identifier, is a URN schema that is extensible
based on existing identifiers like EAN codes, PURL and other identifiers. It is
based on a DNS name, which leads to global uniqueness without new registries.

The TEI can be shown in the software itself, in shipping documentation, in web
pages and app stores. TEI is unique for a product, not a version of a software.
The TEI consist of three core parts

A TEI belongs to a single product. A product can have multiple TEIs - like one
with a EAN/UPC barcode and one with the vendor's product number.

### TEI syntax

```text
urn:tei:<type>:<domain-name>:<unique-identifier>
```

- The **`type`** which defines the syntax of the unique identifier part
- The **`domain-name`** part resolves into a web server, which may not be the
  API host.
  - The uniqueness of the name is the domain name part that has to be registred
    at creation of the TEI.
- The **`unique-identifier`** has to be unique within the `domain-name`.
  Recommendation is to use a UUID but it can be an existing article code too

**Note**: this requires a registration of the TEI URN schema with IANA -
[see here](https://github.com/CycloneDX/transparency-exchange-api/issues/18)

### TEI types

The below show examples of TEI where the types are specific known formats or
types.

Reminder: the `unique-identifer` component of the TEI needs only be unique
within the `domain-name`.

#### PURL - Package URL

Where the `unique-identifier` is a PURL in it's canonical string form.

Syntax:

```text
urn:tei:purl:<domain-name>:<purl>
```

Example:

```text
urn:tei:purl:cyclonedx.org:pkg:pypi/cyclonedx-python-lib@8.4.0?extension=whl&qualifier=py3-none-any
```

#### SWID

Where the `unique-identifier` is a SWID.

Syntax:

```text
urn:tei:swid:<domain-name>:<swid>
```

Note that there is a TEI SWID type as well as a PURL SWID type.

#### HASH

Where the `unique-identifier` is a Hash. Supports the following hash types:

- SHA256
- SHA384
- SHA512

```text
urn:tei:hash:<domain-name>:<hashtype>:<hash>
```

Example:

```text
urn:tei:hash:cyclonedx.org:SHA256:fd44efd601f651c8865acf0dfeacb0df19a2b50ec69ead0262096fd2f67197b9
```

The origin of the hash is up to the vendor to define.

#### UUID

Where the `unique-identifier` is a UUID.

Syntax:

```text
urn:tei:uuid:<domain-name>:<uuid>
```

Example:

```text
urn:tei:uuid:cyclonedx.org:d4d9f54a-abcf-11ee-ac79-1a52914d44b1
```

#### Other types to be defined

- EAN
- GS1
- STD

### TEI resolution using DNS

The `domain-name` part of the TEI is used in a DNS query to find one or multiple
locations for product transparency exchange information.

At the URL a well-known name space is used to find out where the API endpoint is
hosted. This is solved by using the ".well-known" name space as defined by the
IETF.

- `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- Syntax: `urn:tei:uuid:<name based on domain>:<unique identifier>`

The name in the DNS name part points to a set of DNS records.

A TEI with name `tea.example.com` queries for `tea.example.com` `A` and `AAAA`
records. These point to the hosts available for the Transparency Exchange API.

The TEA client connects to the host using HTTPS and validates the certificate.
The URI is composed of the name with the `/.well-known/tea` prefix added.

This results in the base URI (without the product identifier)
`https://tea.example.com/.well-known/tea/`

## Connecting to the API

When connecting to the `.well-known/tea` URI with the unique identifier a HTTP
redirect is **required**.

The server MUST redirect HTTP requests for that resource to the actual "context
path" using one of the available mechanisms provided by HTTP (e.g., using a 301,
303, or 307 response). Clients MUST handle HTTP redirects on the `.well-known`
URI. Servers MUST NOT locate the actual TEA service endpoint at the
`.well-known` URI as per Section 1.1 of [RFC5785].

### Overview: Finding the Index using DNS result

Append the product part of the TEI to the URI found

- TEI: `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- DNS record: `products.example.com`
- URL:
  `https://products.example.com/.well-known/tea/d4d9f54a-abcf-11ee-ac79-1a52914d44b1/`
- HTTP 302 redirect to
  "https://teapot02.consumer.example.com/tea/v2/product-index/d4d9f54a-abcf-11ee-ac79-1a52914d44b1'

Always prefix with the https:// scheme. http (unencrypted) is not valid.

- TEI: `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- URL:
  `https://products.example.com/.well-known/tea/d4d9f54a-abcf-11ee-ac79-1a52914d44b1/`

**NOTE:** The `/.well-known/tea`names space needs to be registred.

## The TEA Version Index

The resulting URL leads to the TEA version index, which is documented in another
document. One redirect (302) is allowed in order to provide for aliasing, where
a single product has many identifiers. The redirect should not lead to a
separate web server.

## References

- [IANA .well-known registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- [IANA URI registry](https://www.iana.org/assignments/urn-namespaces/urn-namespaces.xhtml#urn-namespaces-1)
