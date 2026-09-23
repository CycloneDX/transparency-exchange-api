# Transparency Exchange API - Discovery

- [From product identifier to API endpoint](#from-product-identifier-to-api-endpoint)
- [Advertising the TEI](#advertising-the-tei)
- [TEA Discovery - defining an extensible identifier](#tea-discovery---defining-an-extensible-identifier)
- [The TEI URL: An extensible identifier](#the-tei-url---an-extensible-identifier)
  - [TEI syntax](#tei-syntax)
  - [TEI types](#tei-types)
  - [Comparing TEIs](#comparing-teis)
  - [TEI resolution using DNS](#tei-resolution-using-dns)
- [Connecting to the API](#connecting-to-the-api)
  - [Selecting an API base from `.well-known/tea`](#selecting-an-api-base-from-well-knowntea)
  - [Discovery response](#discovery-response)
  - [Discovery by PURL](#discovery-by-purl)
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

The goal is for a user to add this URL to the transparency platform (sometimes with
credentials for the selected TEA service, such as an API key) and have the platform
access the required artefacts in a highly automated fashion. Those credentials are
provisioned separately and are not embedded in the TEI.

## Advertising the TEI

The TEI for a product release can be communicated to the user in many ways.

- A QR code on a box
- On the invoice or delivery note
- For software with a GUI, in an "about" box

The user obtains the TEI from the manufacturer, through a reseller, or directly. The TEI
is defined by the manufacturer and can normally not be derived from known information.

## TEA Discovery - defining an extensible identifier

TEA discovery is the process where a user with a product release identifier can discover and download
artefacts automatically, with or without authentication. A globally unique identifier is
required for a given product release. This identifier is called the Transparency Exchange Identifier (TEI).

The TEI identifier is based on DNS, which assures a uniqueness per vendor (or open source project)
and gives the vendor a namespace to define product release identifiers based on existing or new identifiers
like EAN/UPC bar code, PURLs or other existing schemes. A given product release may have multiple identifiers
as long as they all resolve into the same destination. Some identifier schemes require registration
with the corresponding standards organisation.

The vendor should ensure that the TEI is unique within the vendor's namespace. There is no
intention to create any TEI registries.

## The TEI: URL - An extensible identifier

The TEI, __Transparency Exchange Identifier__, is a URL schema that is extensible based on existing
identifiers like EAN codes, PURL and other identifiers. It is based on a DNS name, which leads
to global uniqueness without new registries.

The TEI can be shown in the software itself, in shipping documentation, in web pages and app stores.

A TEI identifies product release(s) under a vendor domain. Prefer one product release
per TEI. A TEI may resolve to multiple product releases when the same identifier is
shared (for example a non-unique EAN/UPC); vendors should minimize that case. A product
release can have multiple TEIs — for example one with an EAN/UPC barcode and one with
the vendor's product number.

### TEI syntax

The TEI consists of three core parts

```text
tei://<domain-name>/<type>/<unique-identifier>
````

- The **`domain-name`** part resolves into a web server, which may not be the API host.
  - The domain-name part is a DNS name under the vendor's control. It identifies the DNS
    namespace used by the TEI.
- The **`type`** which defines the syntax of the unique identifier part. Types are declared in the
  specification. If there is a need for new types, please inform ECMA TC54.
- The **`unique-identifier`** shall be unique within the `domain-name`.
  Recommendation is to use a UUID but it can be an existing article code too. The
  identifier is in some cases (depending on type) encoded using BASE64URL encoding (RFC 4648 section 5).
- Port number is not allowed in the `domain-name` part of a TEI URL.


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
by TEA — a successful `/discovery` lookup may return multiple `discovery-info` entries —
the vendor should create a separate TEI for each unique product sold,
like UUID or hash. In any case, the vendor should minimize the number of distinct product
releases returned per TEI. Preferable situation is to have a single product release
per TEI. When multiple releases are returned, clients shall treat array order as
priority (first entry highest).

### Comparing TEIs

A TEI is an identifier, not a locator.
Two TEIs are equal if and only if they are the same sequence of characters.
The comparison is case-sensitive and applies to the TEI as written:
percent-escapes are neither added nor removed,
Base64URL-encoded identifiers are not decoded,
and the domain name is neither resolved nor normalized.

The following TEIs are therefore all distinct:

```text
tei://example.com/uuid/62f2cf92-ae88-11f1-a698-1a52914d44b2
tei://Example.com/uuid/62f2cf92-ae88-11f1-a698-1a52914d44b2
tei://example.com/uuid/62F2CF92-AE88-11F1-A698-1A52914D44B2
tei://example.com/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWI
tei://example.com/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWI=
```

So that one identifier has one spelling,
a vendor shall publish a TEI in its canonical form:
the domain name in lowercase,
the unique identifier exactly as its TEI type defines it
(for example lowercase hexadecimal for a hash, unpadded Base64URL for a PURL),
and no percent-escaping of characters that do not require it.
A client shall use a TEI exactly as received and shall not rewrite it.

This rule defines identity only.
Resolving a TEI to an API endpoint follows the rules below,
where the domain name is used as a DNS name and is therefore case-insensitive.

### TEI resolution using DNS

The `domain-name` part of the TEI is used in a DNS query to find one or multiple locations for
product transparency exchange information.

At the URL a well-known name space is used to find out where the API endpoint is hosted.
This is solved by using the ".well-known" name space as defined by the IETF.

- `tei://<domain-name>/uuid/62f2cf92-ae88-11f1-a698-1a52914d44b2`
- Syntax: `tei://<domain-name>/uuid/<unique identifier>`

The name in the DNS name part points to a set of DNS records.

A TEI with `domain-name` `tea.example.com` queries DNS for `tea.example.com`, considering `A`, `AAAA` and `CNAME` records.
These point to the hosts available for the Transparency Exchange API.

The TEA client connects to the host using HTTPS and shall verify the server
certificate, including the server identity check of [RFC 9525](https://www.rfc-editor.org/rfc/rfc9525).
The URL is composed of the host name with the `/.well-known/tea` path added.

This results in the base URL such as
`https://products.example.com/.well-known/tea`

### TEA Discovery document

This response shall contain a JSON object that lists the available TEA server endpoints and supported versions.
The JSON shall conform to the [TEA Well-Known Schema](tea-well-known.schema.json).

Example:
```json
{
  "schemaVersion": 1,
  "endpoints": [
    {
      "url": "https://api.teaexample.com",
      "versions": 
        [
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

Discovery documents may be cached.

Implementations should:

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

Discovery proceeds in two stages that share the same retry, backoff, and attempt-bound
rules below:

1. **Well-known stage** — retrieve and use `/.well-known/tea` for the TEI’s domain host
2. **API discovery stage** — call `/discovery` on a selected API base URL constructed from
   a well-known endpoint entry

### Selecting an API base from `.well-known/tea`

Clients shall pick an endpoint from the `.well-known/tea` JSON response that lists
at least one API version supported by the client. The client shall prefer endpoints
whose highest mutually supported version is greatest, based on SemVer 2.0.0
specification comparison [rules](https://semver.org/#spec-item-11).
Advertised API versions are full SemVer 2.0.0 (`MAJOR.MINOR.PATCH` with optional
prerelease and build metadata). Build metadata is ignored when comparing versions
([SemVer 2.0.0 §10](https://semver.org/#spec-item-10)), both for precedence and for
determining whether the client supports a version. If the highest mutually supported
version is advertised with more than one build-metadata spelling, the client may select
any of them, and shall use the selected string exactly as advertised when constructing
the path (for example `/v1.0.0+build.5`; `+` is a valid path character per [RFC3986]
and shall not be percent-encoded). If several endpoints remain after that preference,
the client should pick the endpoint with the highest `priority` value (a float between
0 and 1). If `priority` is absent on a well-known endpoint object, the client shall
treat it as `1` for ordering (JSON Schema `default` does not populate omitted fields in
the JSON response).

The client shall then construct the full URL to the API by selecting that highest
mutually supported version and appending `/v` followed by that exact advertised
version string (for example `/v1.0.0`), then `/discovery?tei=` plus the TEI,
url-encoded according to [RFC3986].

Examples:
1. For TEI `tei://products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
`https://api.teaexample.com/v1.0.0/discovery?tei=tei%3A//products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
2. For TEI `tei://products.example.com/purl/cGtnOmRlYi9kZWJpYW4vY3VybEA3LjUwLjMtMT9hcmNoPWkzODYmZGlzdHJvPWplc3NpZQ`
`https://api2.example.com/mytea/v1.0.0/discovery?tei=tei%3A//products.example.com/purl/cGtnOmRlYi9kZWJpYW4vY3VybEA3LjUwLjMtMT9hcmNoPWkzODYmZGlzdHJvPWplc3NpZQ`

The discovery endpoint is a part of the TEA OpenAPI specification. Unlike `/token`,
`/discovery` is required on a conforming TEA API base: a conforming server that exposes
a given API version shall implement it.

### Discovery response

A successful `/discovery` response is a JSON array of `discovery-info` objects. Each
element identifies one resolved product release and the TEA servers that serve it:

- `productReleaseUuid` — UUID of the TEA Product Release
- `servers` — non-empty array of `server-info` objects (`rootUrl`, `versions`, and
  optional `priority`)

`.well-known/tea` and `servers[]` are related but distinct:

- **Well-known** lists candidate API bases for a domain. That is where the client calls
  `/discovery` (after appending `/v{version}`).
- **`servers[]`** in the discovery response lists API bases that serve the resolved
  product release. The client selects among them with the same version-preference and
  `priority` rules as for well-known endpoints. A `rootUrl` need not appear in the
  well-known endpoint list. All further requests for that product release are built as
  `rootUrl` + `/v` + the selected version + path. The API base used for `/discovery` is
  not used again for that release unless it is also listed in `servers[]`.

When multiple product releases match, the array is ordered by priority (first entry
highest). Vendors should prefer returning a single release when possible.

A successful lookup shall return a non-empty array. If the server does not resolve the
identifier, the server shall respond with `404` and a TEA error body with
`error: OBJECT_UNKNOWN` — not `200` with an empty array. If `priority` is absent on a
discovery `servers[]` entry, the client shall treat it as `1` for ordering, the same as
for well-known endpoints.

Example (one match):

```json
[
  {
    "productReleaseUuid": "d4d9f54a-abcf-11ee-ac79-1a52914d44b1",
    "servers": [
      {
        "rootUrl": "https://api.teaexample.com",
        "versions": ["1.0.0"]
      }
    ]
  }
]
```

### Discovery by PURL

Clients that already know a TEA API base URL (for example from a prior TEI discovery,
configuration, or cache) may call `/discovery` with the `purl` query parameter instead of
`tei`. Exactly one of `tei` or `purl` shall be provided; a request with neither or both is
rejected with `400`.

Discovery by PURL resolves within the inventory of the TEA server that receives the
request. It does not replace TEI-based DNS / `.well-known` discovery for finding an API
host from an identifier alone.

Example:

`https://api.teaexample.com/v1.0.0/discovery?purl=pkg%3Amaven%2Forg.apache.logging.log4j%2Flog4j-core%402.24.3`

The response shape, non-empty success array, and `404` with `OBJECT_UNKNOWN` when the
server does not resolve the identifier are the same as for TEI lookup.

If this server does not resolve the TEI (or PURL, when discovering by PURL), whether
because it is unknown or because the server withholds it, the discovery endpoint
shall return `404` with a TEA error response body. A response is a
TEA error response only when its `Content-Type` is `application/json` (optionally with
parameters such as `charset`) and the body is a JSON object with a string `error`
property (typically `OBJECT_UNKNOWN`). Clients shall ignore properties they do not
recognize and shall not reject the response for an `error` value they do not know, so a
later TEA version can extend `error-response` without turning its `404`s into failover
triggers. Other JSON 404 bodies (for example `{"message":"Not Found"}`) are not TEA
error responses.

That conforming TEA `404` means this server does not resolve the identifier, whether
because it is unknown or because the server withholds it (see
`404-object-by-id-not-found`); the client shall not infer which from the status alone.
The client shall not treat it as “`/discovery` is missing” and shall not fail over to
another endpoint solely because of it. The client shall stop discovery for that
identifier at this authority and report that the identifier could not be resolved
there, including the `error` value received. That outcome shall not be reported as
evidence that no updates are available.

### Failover, invalid documents, and attempt bounds

The following failure classes count as a failed attempt for the current candidate and
are subject to the same total attempt bound, backoff, and TLS verification rules,
whether they occur in the well-known stage or the API discovery stage:

- DNS resolution failure for the host being contacted
- TLS certificate validation failure
- HTTP `5xx` from the host being contacted
- A `404` that is not a TEA error response as defined above (for example a web server
  answering for an unmounted path, or a JSON body without an `error` property)
- A response body that is not usable JSON, or that does not conform to the expected
  schema for that stage (malformed or non-conforming `.well-known/tea`, or an invalid
  `/discovery` success body)

A conforming TEA `/discovery` `404` (`error-response`, typically `OBJECT_UNKNOWN`) is
not a failover trigger; see above.

On such a failure, the client shall select the next untried well-known endpoint that
supports a compatible API version, if one is available. While doing so the client
should preserve priority order from highest to lowest (applying the absent-`priority`
equals `1` rule). Each failover connection is subject to the same TLS verification
requirement. Clients should limit the total number of attempts across both stages for
a discovery operation. Additional attempts should use exponential backoff. When a retry
limit is reached, the client shall report that discovery could not be completed,
indicating which stage failed when that is known.

### Authentication and authorization

Where authentication is required, clients use credentials configured for the selected
TEA service, such as an API key, to obtain a TEA access token from that service’s
`/token` endpoint. Credentials shall not be embedded in a TEI. API keys are exchanged
only at the selected API’s `/token` endpoint; clients shall not probe `/token` to discover
whether authentication is required.

A protected TEA resource endpoint (excluding `/token`) shall respond to a request without
valid authentication with `401 Unauthorized` and a `WWW-Authenticate: Bearer` challenge.
When that challenge contains `error="invalid_token"` (RFC 6750 section 3.1), the client
may obtain a replacement access token from the same service and retry the original
request once. Clients should not repeat this recovery attempt for the same request. This
is not OAuth refresh-token use.

If authentication cannot be completed or recovery fails, the client shall indicate that
update status could not be determined. Failures that may require user or administrator
intervention include rejected or revoked credentials, expired client certificates,
persistent rejection of a replacement token, and insufficient permissions. A
`403 Forbidden` response indicates denied authorization and shall not trigger
token-replacement attempts solely because of that status. Clients shall not fail over to
another endpoint solely in response to `401` or `403`.

Clients shall verify server certificates for every HTTPS connection used in discovery and
subsequent API access, including the server identity check of
[RFC 9525](https://www.rfc-editor.org/rfc/rfc9525), and shall not use connections that
fail validation.

Clients shall not automatically forward a TEA access token to a different origin, or
outside the authorized API base URL of the service that issued it. API-key Basic
credentials shall not be forwarded based merely on a discovery redirect; a different
service requires independently configured credentials. Redirect targets used during
discovery or API access shall use HTTPS and are subject to the same certificate
verification requirement.

The full client authentication flow is described in [Authentication](../auth/readme.md).
The rules above align discovery with that model and do not replace it.

How authentication or authorization failures are presented to end users is implementation
specific, but they shall not be reported as evidence that no updates are available.

### Common authentication-related responses

#### 401 Unauthorized

- For an initially unauthenticated resource request, a `WWW-Authenticate: Bearer`
  challenge without an `error` parameter indicates that authentication is required.
  A client with configured credentials may obtain an access token from the selected
  API’s `/token` endpoint and retry the resource request.
- For a resource request rejected with `error="invalid_token"`, the client may obtain
  a replacement access token from the same service and retry the original request once.
  Clients should not repeat this recovery attempt for the same request.

Other challenges shall not be interpreted as instructions to repeatedly obtain
replacement tokens.

#### 403 Forbidden

- Authenticated, but not authorized for this resource. Do not treat as token expiry and do
  not fail over solely because of this status.

Common errors:

#### 404 Not Found

- On `/discovery`, a TEA error response (`application/json` body with a string `error`
  property, typically `OBJECT_UNKNOWN`): this server does not resolve the TEI or PURL,
  whether unknown or withheld. Do not fail over. Stop and report that the identifier
  could not be resolved at this authority, with the `error` value; do not report that
  as evidence that no updates are available.
- A `404` that is not a TEA error response may mean the path is not mounted or the host
  is not a TEA API base; treat that as a failed discovery attempt and failover if another
  compatible endpoint remains. This is distinct from `/token`, where `404` means only
  that the token endpoint is not implemented.

#### 503 Service Unavailable

- temporary failure  

#### TLS failure

- certificate validation error

### Client behavior

Clients should:

- retry with backoff  
- validate TLS certificates  
- fail closed if discovery cannot be validated

## Notes Regarding .well-known

Servers shall not locate the actual TEA service endpoint at the
`.well-known` URI as per Section 1.1 of [RFC5785]. This endpoint is only for distribution
of the TEA discovery document.

### TLS Encryption

The `.well-known` endpoint shall only be available via HTTPS. Using unencrypted HTTP is not
valid. Clients shall verify the server certificate for this connection as for any other
TEA HTTPS request, including the server identity check of [RFC 9525](https://www.rfc-editor.org/rfc/rfc9525).

Conforming deployments shall advertise only lowercase `https` base URLs, in
`.well-known/tea` `endpoints[].url` and in `/discovery` `servers[].rootUrl` alike. A
client shall reject an `http` base URL unless it has been explicitly configured to
allow that specific base for local testing. The allowance is per configured base URL,
never a global setting, so it cannot apply to a base the client learned from discovery.
Credentials may be sent to a base allowed this way; a deployment that relies on it is
not conforming.

- TEI: `tei://products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1`
- URL: `https://products.example.com/.well-known/tea`

## References

- [IANA .well-known registry](https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml)
