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

TEA Discovery is the connection between a product identifier and the API endpoint.
A product identifier is embedded in a URN where the identifier is one of many existing
identifiers or a random string - like an EAN bar code, product
number or PURL.

The goal is for a user to add this URN to the transparency platform (sometimes with an
associated authentication token) and have the platform access the required artifacts
in a highly automated fashion.

## TEA Discovery - defining an extensible identifier

TEA discovery is the process where a user with a product identifier can discover and download
artifacts automatically, with or without authentication. A globally unique identifier is
required for a given product. This identifier is called the Transparency Exchange Identifier (TEI).

The TEI identifier is based on DNS, which assures a uniqueness per vendor (or open source project) 
and gives the vendor a name space to define product identifiers based on existing or new identifiers like EAN bar code, PURLs or other existing schemes. A given product may have multiple identifiers as long as they all resolve into the same destination.

## The TEI URN: An extensible identifier

The TEI, Transparency Exchange Identifier, is a URN schema that is extensible based on existing
identifiers like EAN codes, PURL and other identifiers. It is based on a DNS name, which leads
to global uniqueness without new registries.

The TEI can be shown in the software itself, in shipping documentation, in web pages and app stores.
TEI is unique for a product, not a version of a software. The TEI consist of three core parts

- The **`type`** which defines the syntax of the unique identifier part
- The **`domain-name`** part does not have to exist as a web server (HTTPS), but may do
  - The uniqueness of the name is the domain name part that has to be registred at creation of the TEI.
- The **`unique-identifier`** has to be unique within the `domain-name`. Recommendation is to use a UUID but it can be an existing article code too

A TEI belongs to a single product. A product can have multiple TEIs - like one with a EAN
barcode and one with the vendor's product number.

### TEI syntax

```text
urn:tei:<type>:<domain-name>:<unique-identifier>
````

**Note**: this requires a registration of the TEI URN schema with IANA - [see here](https://github.com/CycloneDX/transparency-exchange-api/issues/18)

### TEI types

The below show examples of TEI where the types are specific known formats or types.

Reminder: the `unique-identifer` component of the TEI needs only be unique within the `domain-name`.

#### PURL - Package URL

Where the `unique-identifier` is a PURL in it's canonical string form.

Syntax:

```text
urn:tei:purl:<domain-name>:<purl>
````

Example:
```text
urn:tei:org.cyclonedx:pkg:pypi/cyclonedx-python-lib@8.4.0?extension=whl&qualifier=py3-none-any
```

#### SWID

Where the `unique-identifier` is a SWID.

Syntax:

```text
urn:tei:swid:<domain-name>:<swid>
````

Note that there is a TEI SWID type as well as a PURL SWID type.

#### HASH

Where the `unique-identifier` is a Hash. Supports the following hash types:

* SHA256
* SHA384
* SHA512

```text
urn:tei:hash:<domain-name>:<hashtype>:<hash>
````

Example:
```text
urn:tei:cyclonedx.org:SHA256:fd44efd601f651c8865acf0dfeacb0df19a2b50ec69ead0262096fd2f67197b9
```

The origin of the hash is up to the vendor to define.

#### UUID

Where the `unique-identifier` is a UUID.

Syntax:

```text
urn:tei:uuid:<domain-name>:<uuid>
````

Has to be a valid UUID.

#### Other types to be defined

- EAN
- GS1
- STD


### TEI resolution using DNS

The name part of the TEI is used in a DNS query to find one or multiple locations for product transparency exchange information.
At the URL we need a well-known name space to find out more

- `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- Syntax: `urn:tei:uuid:<name based on domain>:<unique identifier>`

The name in the DNS name part points to a set of DNS records.
A TEI with name “tex.example.com" queries for `_tei._tcp.tex.example.com URI records`.
These point to URIs for the transparency exchange data.
If there are no records, try to resolve the name (using AAAA and A DNS records) and
append the /.well-known/tei prefix

```zone_file
_tei._tcp.tex.example.com.   3600 IN URI 10 1 “https://www.example.com/transparency“
```

Example response of DNS query including multiple URIs with a priority

```zone_file
_tei._tcp.tex.example.com.   3600 IN URI 10 1 “https://www.example.com/transparency“
_tei._tcp.tex.example.com.   3600 IN URI 20 1 “https://backup.example.com/transparency“
_tei._tcp.tex.example.com.   3600 IN URI 30 1 “https://thirdparty.example.org/example.com/transparency“
```

First try lowest priority then move up.

It is recommended to have a third party external repository as the last priority.
The URI in DNS does not have to belong to the same domain as the URI records. I.e.
it is valid for "tex.example.com" to resolve to "thirdparty.example.org".

### Finding the Index using DNS result

Append the product part of the TEI to the URI found

- TEI: `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- DNS record: `_tei._tcp.products.example.com`
- URI in DNS: `://www.example.com/transparency/`
- URL: `https://www.example.com/transparency/d4d9f54a-abcf-11ee-ac79-1a52914d44b1/`

If no DNS URI records are found the resolution defaults to A and AAAA records.
IP address is not valid in the domain name field.

Append the URL prefix `/.well-known/tei/` of the TEI to the DNS name found
Always prefix with the https:// scheme. http (unencrypted) is not valid.

- TEI: `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- URL: `https://products.example.com/.well-known/tei/d4d9f54a-abcf-11ee-ac79-1a52914d44b1/`

**NOTE:** The `/.well-known/tei`names space needs to be registred.

## The TEA Version Index

The resulting URL leads to the TEA version index, which is documented in another document.
One redirect (302) is allowed in order to provide for aliasing, where a single product
has many identifiers. The redirect should not lead to a separate web server.

## References

- [RFC 7553 - THE DNS URI record](https://datatracker.ietf.org/doc/html/rfc7553)
- [IANA .well-known registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- [IANA URI registry](https://www.iana.org/assignments/urn-namespaces/urn-namespaces.xhtml#urn-namespaces-1)
