[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](LICENSE)
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)

# CycloneDX BOM Exchange API Standard

NOTICE: This standard is currently in draft pending feedback from the community.

## Introduction

This specification defines a standard, format agnostic, API for the exchange of
BOMs between systems.

System and tooling implementors are encouraged to adopt this API standard for
sending/receiving BOMs between systems. This will enable more widespread
"out of the box" integration support in the BOM ecosystem.

## Conventions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be
interpreted as described in [RFC2119](http://www.ietf.org/rfc/rfc2119.txt).

### ABNF Syntax

ABNF syntax used as per
[RFC5234: Augmented BNF for Syntax Specifications: ABNF](https://datatracker.ietf.org/doc/html/rfc5234).

ABNF rules are used from [RFC3986: Uniform Resource Identifier (URI): Generic Syntax - Appendix A. Collected ABNF for URI](https://datatracker.ietf.org/doc/html/rfc3986/#appendix-A).

These additional rules are defined:
```
system-url       = supported-scheme ":" hier-part
                       ; a system defined URL
                       ; hier-part as defined in RFC3986
supported-scheme = "http" / "https"
```

See also: [RFC7231: Hypertext Transfer Protocol (HTTP/1.1): Semantics and Content](https://datatracker.ietf.org/doc/html/rfc7231)

## Specification Compliance

An API server/client can be referred to as compliant if it correctly implements
any of the methods described within this specification. It is not a
requirement to implement all the methods described.

## BOM Retrieval

This method is for retrieving a BOM from a system.

The BOM retrieval URL MUST comply with this syntax:

```
bom-retrieval-url    = system-url "?" bom-identifier-query
bom-identifier-query = "bomIdentifier=" bom-identifier
bom-identifier       = *( pchar / "/" / "?" )
                        ; an identifier that uniquely identifies a BOM
                        ; pchar as defined in RFC3986
```

The HTTP request method MUST be `GET`.

For CycloneDX BOMs the `bom-identifier` MUST be either a CDX URN (https://www.iana.org/assignments/urn-formal/cdx)
or a BOM serial number UUID URN (https://cyclonedx.org/docs/1.4/json/#serialNumber).

For SPDX documents the `bom-identifier` MUST be the SPDX Document Namespace
(https://spdx.github.io/spdx-spec/document-creation-information/#65-spdx-document-namespace-field).

### Server Requirements

Servers MAY require authorization. If authorization is required it MUST
use the HTTP `Authorization` header. If a server requires authorization, and
no `Authorization` request header is supplied by the client, the server
MUST respond with a 401 Unauthorized response.

Servers MUST honour the requested content types in the `Accept` header. If
the server does not support any of the requested content types a HTTP 406 response
MUST be returned. The 406 response body MUST contain a list of server supported
content types in the below format with `text/plain` content type.

```
media-type *(", " media-type)
```

e.g. `application/vnd.cyclonedx+xml; version=1.4, application/vnd.cyclonedx+xml; version=1.3`

API servers MUST provide the correct `Content-Type` HTTP response header. For example:

```
Content-Type: application/vnd.cyclonedx+xml; version=1.4
```

If a BOM serial number UUID URN is used as the `bom-identifier`, the server
MUST respond with the latest available version of the BOM.

### Client Requirements

Clients MUST support an optional `Authorization` header being specified.

Clients MUST provide a `Accept` HTTP request header. For example:

```
Accept: application/vnd.cyclonedx+xml; version=1.4, application/vnd.cyclonedx+xml; version=1.3
```

## BOM Submission Endpoint

This method is for submitting a BOM to a system.

The BOM submission URL MUST comply with this syntax:

```
bom-submission-url = system-url
```

The HTTP request method MUST be `POST`.

### Server Requirements

Servers MAY require authorization. If authorization is required it MUST
use the HTTP `Authorization` header. If a server requires authorization, and
no `Authorization` request header is supplied by the client, the server
MUST respond with a 401 Unauthorized response.

Servers MUST honour the specified content type in the `Content-Type` header. If
the server does not support the supplied content type a HTTP 415 Unsupported
Media Type response MUST be returned. The 415 response body MUST contain a list
of server supported content types in the below format with `text/plain` content type.

```
media-type *(", " media-type)
```

e.g. `application/vnd.cyclonedx+xml; version=1.4, application/vnd.cyclonedx+xml; version=1.3`

If the submitted BOM has been successfully submitted the API server MUST
respond with an appropriate 2xx HTTP status code.

### Client Requirements

Clients MUST support an optional `Authorization` header being specified.

Clients MUST provide the correct `Content-Type` HTTP request header. For example:

```
Content-Type: application/vnd.cyclonedx+xml; version=1.4
```

## BOM Metadata Endpoint

This method is for retrieving metadata about a BOM from a system.

The BOM metadata URL MUST comply with this syntax:

```
bom-meta-url        = system-url "?" bom-identifier-query
bom-identifier-query = "bomIdentifier=" bom-identifier
bom-identifier       = *( pchar / "/" / "?" )
                        ; an identifier that uniquely identifies a BOM
                        ; pchar as defined in RFC3986
```

The HTTP request method MUST be `GET`.

For CycloneDX BOMs the `bom-identifier` MUST be either a CDX URN (https://www.iana.org/assignments/urn-formal/cdx)
or a BOM serial number UUID URN (https://cyclonedx.org/docs/1.4/json/#serialNumber).

For SPDX documents the `bom-identifier` MUST be the SPDX Document Namespace
(https://spdx.github.io/spdx-spec/document-creation-information/#65-spdx-document-namespace-field).

### Server Requirements

Servers MAY require authorization. If authorization is required it MUST
use the HTTP `Authorization` header. If a server requires authorization, and
no `Authorization` request header is supplied by the client, the server
MUST respond with a 401 Unauthorized response.

The server MUST respond with a JSON payload using the following syntax:

```json
{
  "identifier": "",
  "spec": {
    "format": "",
    "version": ""
  },
  "artifacts": [
    {
      "mime-type": "",
      "checksum": [
        { "alg":  "", "value":  ""}
      ]
    }
  ],
  "published": ""
}
```

#### The IDENTIFIER property
The "identifier" is a string value of the requested bom-identifier. This property is REQUIRED.

#### The SPEC property
The "spec" object is REQUIRED and MUST describe the "format" of a BOM. The "version" property is OPTIONAL and RECOMMENDED.

##### The FORMAT property
The "format" property is REQUIRED and MUST be a string. The value SHOULD be one of "CycloneDX" or "SPDX". 

##### The VERSION property
The "version" property is OPTIONAL. If specified, the property MUST be a string representation
of the BOM specification version.

#### The ARTIFACTS property
The "artifacts" property describes an array of BOMs for the different serialization formats available for a bom-identifier.
If no artifacts are available, the "artifacts" object MUST be an empty array.

##### The MIME-TYPES property
The "mime-type" property MUST be a string that conform to [RFC-2045](https://datatracker.ietf.org/doc/html/rfc2045).
This property specifies the media type the underlying server maintains which can be requested.

##### The CHECKSUM property
The "checksum" property describes an array of objects, each MUST contain "alg" and "value". The "alg" property
is a REQUIRED string and must be one of SHA-1, SHA-256, SHA-384, SHA-512, SHA3-256, SHA3-384, SHA3-512, BLAKE2b, or BLAKE3.
The "value" property is a hexadecimal string derived from the hashing function. All values MUST be uppercase.

In some cases, no checksums may be available. This may occur if a BOM is published in a single serialization format (e.g. JSON),
but the underlying server can transform the BOM into other serialization formats (e.g. XML), but has not proactively
done so. In this scenario, or whenever the checksum of a BOM artifact is not known, it MAY be omitted resulting in an 
artifact object containing only a mime-type.

##### The PUBLISHED property
The "published" property is OPTIONAL but RECOMMENDED. The value MUST conform to [RFC-3339 Section 5.6](https://datatracker.ietf.org/doc/html/rfc3339#section-5.6)

For example:
```json
{
  "identifier": "urn:cdx:f5375fa3-a234-4abc-b4d4-577583fc4d0f/1",
  "spec": {
    "format": "CycloneDX",
    "version": "1.4"
  },
  "artifacts": [
    {
      "mime-type": "application/vnd.cyclonedx+json",
      "checksum": [
        {
          "alg": "SHA-256",
          "value": "8d9754db9759ab1785644440dbf19f88ab45ae326e421da6c1cb6e45140d534f"
        }
      ]
    },
    {
      "mime-type": "application/vnd.cyclonedx+xml",
      "checksum": [
        {
          "alg":  "SHA-256",
          "value":  "cf80cd8aed482d5d1527d7dc72fceff84e6326592848447d2dc0b0e87dfc9a90"
        }
      ]
    }
  ],
  "published": "2022-06-07T15:50+00Z"
}
```

### Client Requirements

Clients MUST support an optional `Authorization` header being specified.
