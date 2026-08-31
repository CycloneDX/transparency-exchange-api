# Transparency Exchange API - Discovery

- [From product identifier to API endpoint](#from-product-identifier-to-api-endpoint)
- [Advertising the TEI](#advertising-the-tei)
- [TEA Discovery - defining an extensible identifier](#tea-discovery---defining-an-extensible-identifier)
- [The TEI URN: An extensible identifier](#the-tei-urn-an-extensible-identifier)
  - [TEI syntax](#tei-syntax)
  - [TEI types](#tei-types)
  - [TEI resolution using DNS](#tei-resolution-using-dns)
- [Connecting to the API](#connecting-to-the-api)
  - [Overview: Finding the Index using DNS result](#overview-finding-the-index-using-dns-result)
- [The TEA Version Index](#the-tea-version-index)
- [References](#references)

## From product identifier to API endpoint

Discovery is the **first step in all TEA interactions**, enabling a consumer to map an identifier to a service endpoint.

This specification defines:

- how discovery is initiated  
- how discovery documents are retrieved  
- how API endpoints are obtained

TEA separates:

- **identity** → TEI  
- **location** → discovery  
- **data retrieval** → API  

Discovery answers the question:

> “Where can I retrieve authoritative TEA data for this identifier?”

TEA Discovery is the connection between a product release identifier and the API endpoint.
A "product release" is something that the customer aquires or downloads - hardware and/or software.

It can be a bundle of many digital devices or software applications.
A "product release" normally also has an entry in a large corporation's asset inventory system.

A product release identifier is embedded in a URL where the identifier is one of many existing
identifiers or a random string - like an EAN or UPC bar code, UUID, product
number or PURL.

The goal is for a user to add this URL to the transparency platform (sometimes with an
associated authentication token) and have the platform access the required artefacts
in a highly automated fashion.

## Advertising the TEI

The TEI for a product release can be communicated to the user in many ways.

- A QR code on a box
- On the invoice or delivery note
- For software with a GUI, in an "about" box

The user needs to get the TEI from the manufacturer, through a reseller or directly. The TEI
is defined by the manufacturer and can normally not be derived from known information.

## TEA Discovery - defining an extensible identifier

TEA discovery is the process where a user with a product release identifier can discover and download
artefacts automatically, with or without authentication. A globally unique identifier is
required for a given product release. This identifier is called the Transparency Exchange Identifier (TEI).

The TEI identifier is based on DNS, which assures a uniqueness per vendor (or open source project)
and gives the vendor a namespace to define product release identifiers based on existing or new identifiers
like EAN/UPC bar code, PURLs or other existing schemes. A given product release may have multiple identifiers
as long as they all resolve into the same destination. In some cases, these identifiers has to be applied
for to the corresponding standards organisation.

The vendor needs to make sure that the TEI is unique within the vendor's namespace. There is no
intention to create any TEI registries.

## The TEI: URL - An extensible identifier

The TEI, __Transparency Exchange Identifier__, is a URL schema that is extensible based on existing
identifiers like EAN codes, PURL and other identifiers. It is based on a DNS name, which leads
to global uniqueness without new registries.

The TEI can be shown in the software itself, in shipping documentation, in web pages and app stores.

A TEI belongs to a single product release. A product release can have multiple TEIs - like one with a EAN/UPC
barcode and one with the vendor's product number.

### TEI syntax

The TEI consists of three core parts

```text
tei://<domain-name>/<type>/<unique-identifier>
````

- The **`domain-name`** part resolves into a web server, which may not be the API host.
  - The uniqueness of the name is the domain name part that has to be registred at creation of the TEI.
- The **`type`** which defines the syntax of the unique identifier part. Types are declared in the
  specification. If there is a need for new types, please inform ECMA TC54.
- The **`unique-identifier`** has to be unique within the `domain-name`.
  Recommendation is to use a UUID but it can be an existing article code too. The
  identifier is in some cases (depending on type) encoded using BASE64URL encoding (RFC 4648 section 5).


### TEI types

The below show examples of TEI where the types are specific known formats or types.

Reminder: the `unique-identifer` component of the TEI needs only be unique within the `domain-name`.

#### PURL - Package URL

Where the `unique-identifier` is a PURL in it's canonical string form.
A PURL identifier is encoded using BASE64URL.

Syntax:

```text
tei://<domain-name>/purl/<purl>
````

Example:

PURL: pkg:pypi/cyclonedx-python-lib@8.4.0?extension=whl&qualifier=py3-none-any

```text
tei://cyclonedx.org/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWJAOC40LjA_ZXh0ZW5zaW9uPXdobCZxdWFsaWZpZXI9cHkzLW5vbmUtYW55
```

#### SWID

Where the `unique-identifier` is a SWID.

Syntax:

```text
tei://<domain-name>/swid/<swid>
```

Note that there is a TEI SWID type as well as a PURL SWID type.

#### HASH

Where the `unique-identifier` is a Hash. Supports the following hash types:

- SHA256
- SHA384
- SHA512

```text
tei://<domain-name>/hash/<hashtype>/<hash>
````

Example:

```text
tei://cyclonedx.org/hash/SHA256/fd44efd601f651c8865acf0dfeacb0df19a2b50ec69ead0262096fd2f67197b9

```

The origin of the hash is up to the vendor to define.

#### UUID

Where the `unique-identifier` is a UUID.

Syntax:

```text
tei://<domain-name>/uuid/<uuid>
````

Example:

```text
tei://cyclonedx.org/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1
```

#### EAN/UPC

Where the `unique-identifier` is a EAN/UPC.

Syntax:

```text
tei://<domain-name>/eanupc/<ean/upc-number>
````

Example:

```text
tei://example.com/eanupc/1234567890123
```

#### GTIN

Where the `unique-identifier` is a [GTIN](https://www.gs1.org/standards/id-keys/gtin).

Syntax:

```text
tei://<domain-name>/gtin/<gtin-number>
````

Example:

```text
tei://example.org/gtin/0234567890123
```

#### ASIN

Where the `unique-identifier` is a [ASIN](https://sell.amazon.com/blog/what-is-an-asin).

Syntax:

```text
tei://<domain-name>/asin/<asin-identifier>
````

Example:

```text
tei://example.com/asin/B07FZ8S74R
```


#### UDI

Where the `unique-identifier` is a [UDI](https://www.gs1.org/industries/healthcare/udi).

Syntax:

```text
tei://<domain-name>/udi/<udi-identifier>
````

Example:

```text
tei://cyclonedx.org/udi/00123456789012
```

Note that if the same identifier, like EAN, is used for multiple different product releases
then this EAN code will not be unique for a given product. While this case is supported
by TEA, the vendor is recommended to create a separate TEI for each unique product sold,
like UUID or hash. In any case, the vendor SHOULD minimize the number of distinct product
releases returned per TEI. Preferable situation is to have a single product release
per TEI.

### TEI resolution using DNS

The `domain-name` part of the TEI is used in a DNS query to find one or multiple locations for
product transparency exchange information.

At the URL a well-known name space is used to find out where the API endpoint is hosted.
This is solved by using the ".well-known" name space as defined by the IETF.

- `tei://<domain-name>/uuid/ZDRkOWY1NGEtYWJjZi0xMWVlLWFjNzktMWE1MjkxNGQ0NGIx`
- Syntax: `tei://<domain-name>/uuid/<unique identifier>`

The name in the DNS name part points to a set of DNS records.

A TEI with `domain-name` `tea.example.com` queries DNS for `tea.example.com`, considering `A`, `AAAA` and `CNAME` records.
These point to the hosts available for the Transparency Exchange API.

The TEA client connects to the host using HTTPS and validates
the certificate. The URL is composed of the host name with the `/.well-known/tea` path added.

This results in the base URL such as
`https://products.example.com/.well-known/tea`

### TEA Discovery document

The response must contain a json object that lists the available TEA server endpoints and supported versions.
The json must conform to the [TEA Well-Known Schema](tea-well-known.schema.json).

Example:
```json
{
  "schemaVersion": 1,
  "endpoints": [
    {
      "url": "https://api.teaexample.com",
      "versions": 
        [
          "0.1.0-beta.1",
          "0.2.0-beta.2",
          "1.0.0"
        ],
      "priority": 1
    },
    {
      "url": "https://api2.teaexample.com/mytea",
      "versions": 
        [
          "1.0.0"
        ],
      "priority": 0.5
    }
  ]
}
```

### Discovery data caching and freshness

Discovery documents MAY be cached.

Implementations SHOULD:

- respect HTTP caching headers  
- periodically refresh discovery data  
- handle endpoint changes gracefully  

## Port resolution

Currently, the port number is not part of the TEI but it is needed to connect to the API.

The TEA API server may be hosted on any port, but the server that is part of the
first step of discovery will by default be running on the default HTTPS port 443.
In the JSON file found there, the server URLs may contain port numbers.

If the clients are known to support HTTPS/SVCB DNS records for failover and
load balancing, these may be used to redirect to other ports and servers.

Public servers are recommended to have the server hosting the DNS name used
in the TEI URI running on the default port to enable discovery of the API servers.

## Connecting to the API

Clients must pick any one of the endpoints listed in the `.well-known/tea` json
response. The client MUST pick an endpoint with the at least one version that is
supported by the client is using. The client MUST prioritize endpoints with the
highest matching version supported both by the client and the endpoint based on
SemVer 2.0.0 specification comparison [rules](https://semver.org/#spec-item-11).
If there are several endpoints like these and if the priority field is present,
the client SHOULD pick the endpoint with the highest priority value (a float
between 0 and 1).

The client must then construct the full URL to the API by appending the
"/v" plus one of the versions listed in the `versions` array of the selected endpoint,
plus "/discovery?tei=", plus the TEI that is url-encoded according to [RFC3986]
and [RFC3986]).

Examples:
1. For TEI `tei://<domain-name>/uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b`
`https://api.teaexample.com/v0.2.0-beta.2/discovery?tei=tei%3A//products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b`
2. For TEI `tei://<domain-name>/purl:products.example.com:pkg:deb/debian/curl@7.50.3-1?arch=i386&distro=jessie`
`https://api2.teaexample.com/mytea/v1.0.0/discovery?tei=tei%3A//products.example.com%3Apurl%3Apkg%3Adeb%2Fdebian%2Fcurl%407.50.3-1%3Farch%3Di386%26distro%3Djessie`

The discovery endpoint is a part of the TEA OpenAPI specification.

If the TEI is known to the TEA server, the discovery endpoint must return at least
the product release uuid, the root URL of the TEA server, the list of supported
versions, plus the response may have other fields based on the current version of
the TEA OpenAPI specification.

If the TEI is not known to the TEA server, the discovery endpoint must return a 404
status code with a response describing the error.

If the DNS record for the discovery endpoint cannot be resolved by the client, or
the discovery endpoint fails with 5xx error code, or the TLS certificate cannot be validated,
the client MUST retry the discovery endpoint with the next endpoint in the list, if another
endpoint is present. While doing so the client SHOULD preserve the priority order if provided
(from highest to lowest priority). If no other endpoint is available, the client MUST retry
the discovery endpoint with the first endpoint in the list. The client SHOULD implement an
exponential backoff strategy for retries.

Client implementations needs to indicate authentication errors clearly to the users,
to indicate that there are no updates. An expired token or TLS Client Cert will
mean that new versions of a product or updated artefacts will not be accessed.

### Error handling

Authentication error codes (401, 403) should not lead to failover to the next endpoint
in the list.

How this is communicated to the client users is implementation specific.

Common errors:

#### 404 Not Found

- discovery endpoint not present  

#### 503 Service Unavailable

- temporary failure  

#### TLS failure

- certificate validation error

### Client behavior

Clients SHOULD:

- retry with backoff  
- validate TLS certificates  
- fail closed if discovery cannot be validated

## Notes Regarding .well-known

Servers MUST NOT locate the actual TEA service endpoint at the
`.well-known` URI as per Section 1.1 of [RFC5785]. This endpoint is only for distribution
of the TEA discovery document.

### TLS Encryption

The .well-known endpoint must only be available via HTTPS. Using unencrypted HTTP is not valid.

- TEI: `tei://products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- URL: `https://products.example.com/.well-known/tea`

## References

- [IANA .well-known registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- [IANA URI registry](https://www.iana.org/assignments/urn-namespaces/urn-namespaces.xhtml#urn-namespaces-1)
