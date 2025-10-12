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

TEA Discovery is the connection between a product identifier and the API endpoint.
A "product" is something that the customer aquires or downloads - hardware and/or software.

It can be a bundle of many digital devices or software applications.
A "product" normally also has an entry in a large corporation's asset inventory system.

A product identifier is embedded in a URN where the identifier is one of many existing
identifiers or a random string - like an EAN or UPC bar code, UUID, product
number or PURL.

The goal is for a user to add this URN to the transparency platform (sometimes with an
associated authentication token) and have the platform access the required artifacts
in a highly automated fashion.

## Advertising the TEI

The TEI for a product can be communicated to the user in many ways.

- A QR code on a box
- On the invoice or delivery note
- For software with a GUI, in an "about" box

The user needs to get the TEI from the manufacturer, through a reseller or directly. The TEI
is defined by the manufacturer and can normally not be derived from known information.

## TEA Discovery - defining an extensible identifier

TEA discovery is the process where a user with a product identifier can discover and download
artifacts automatically, with or without authentication. A globally unique identifier is
required for a given product. This identifier is called the Transparency Exchange Identifier (TEI).

The TEI identifier is based on DNS, which assures a uniqueness per vendor (or open source project)
and gives the vendor a name space to define product identifiers based on existing or new identifiers
like EAN/UPC bar code, PURLs or other existing schemes. A given product may have multiple identifiers
as long as they all resolve into the same destination.

The vendor needs to make sure that the TEI is unique within the vendor's name space. There is no
intention to create any TEI registries.

## The TEI URN: An extensible identifier

The TEI, Transparency Exchange Identifier, is a URN schema that is extensible based on existing
identifiers like EAN codes, PURL and other identifiers. It is based on a DNS name, which leads
to global uniqueness without new registries.

The TEI can be shown in the software itself, in shipping documentation, in web pages and app stores.
TEI is unique for a product, not a version of a product.

A TEI belongs to a single product. A product can have multiple TEIs - like one with a EAN/UPC
barcode and one with the vendor's product number.

### TEI syntax

The TEI consists of three core parts

```text
urn:tei:<type>:<domain-name>:<unique-identifier>
````

- The **`type`** which defines the syntax of the unique identifier part
- The **`domain-name`** part resolves into a web server, which may not be the API host.
  - The uniqueness of the name is the domain name part that has to be registred at creation of the TEI.
- The **`unique-identifier`** has to be unique within the `domain-name`.
  Recommendation is to use a UUID but it can be an existing article code too

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
urn:tei:purl:cyclonedx.org:pkg:pypi/cyclonedx-python-lib@8.4.0?extension=whl&qualifier=py3-none-any
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

- SHA256
- SHA384
- SHA512

```text
urn:tei:hash:<domain-name>:<hashtype>:<hash>
````

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
````

Example:
```text
urn:tei:uuid:cyclonedx.org:d4d9f54a-abcf-11ee-ac79-1a52914d44b1
```


#### Other types to be defined

- EAN
- GS1
- STD

Note that if an identifier, like EAN, is used for multiple different products then this
EAN code will not be unique for a given product and should not be used as an identifier.
In this case, the vendor is recommended to create a separate identifier for each unique
product sold by other means, like UUID or hash.

### TEI resolution using DNS

The `domain-name` part of the TEI is used in a DNS query to find one or multiple locations for
product transparency exchange information.

At the URL a well-known name space is used to find out where the API endpoint is hosted.
This is solved by using the ".well-known" name space as defined by the IETF.

- `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- Syntax: `urn:tei:uuid:<name based on domain>:<unique identifier>`

The name in the DNS name part points to a set of DNS records.

A TEI with `domain-name` `tea.example.com` queries DNS for `tea.example.com`, considering `A`, `AAAA` and `CNAME` records.
These point to the hosts available for the Transparency Exchange API.

The TEA client connects to the host using HTTPS and validates the certificate.
The URI is composed of the name with the `/.well-known/tea` prefix added.

This results in the base URI 
`https://tea.example.com/.well-known/tea`

This URI must contain static json that lists the available TEA server endpoints and supported versions.
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
1. For TEI `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b`
`https://api.teaexample.com/v0.2.0-beta.2/discovery?tei=urn%3Atei%3Auuid%3Aproducts.example.com%3Ad4d9f54a-abcf-11ee-ac79-1a52914d44b`
2. For TEI `urn:tei:purl:products.example.com:pkg:deb/debian/curl@7.50.3-1?arch=i386&distro=jessie`
`https://api2.teaexample.com/mytea/v1.0.0/discovery?tei=urn%3Atei%3Apurl%3Aproducts.example.com%3Apkg%3Adeb%2Fdebian%2Fcurl%407.50.3-1%3Farch%3Di386%26distro%3Djessie`

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

TODO: Handle Auth errors (401, 403) and corresponding messages.

## Notes Regarding .well-known
Servers MUST NOT locate the actual TEA service endpoint at the
`.well-known` URI as per Section 1.1 of [RFC5785].

### TLS Encryption

The .well-known endpoint must only be available via HTTPS. Using unencrypted HTTP is not valid.

- TEI: `urn:tei:uuid:products.example.com:d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- URL: `https://products.example.com/.well-known/tea`

**NOTE:** The `/.well-known/tea` names space needs to be registred.


## References

- [IANA .well-known registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
- [IANA URI registry](https://www.iana.org/assignments/urn-namespaces/urn-namespaces.xhtml#urn-namespaces-1)
