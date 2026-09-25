# Discovery

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

## Overview

Discovery answers the question:

> “Where can I retrieve authoritative TEA data for this identifier?”

TEA discovery is the process by which a user utilizes a product release identifier in
order to obtain information about the TEA server, then download artefacts relating to a
specific product release or releases automatically with or without authentication.

A "product release" is something the user acquires or downloads in the form of a digital
device or software application. It can be a single item or a bundle of many items.
A product release normally also has an entry in a large corporation's asset inventory system.

A "product release identifier" is a string embedded in a URL which represents the product
release, such as an EAN or UPC bar code, UUID, product
number or PURL.

## TEI extensible URL schema

TEI, <dfn>Transparency Exchange Identifier</dfn>, is a URL schema that is extensible
by taking advantage of existing identifiers like EAN codes, PURLs, UUIDs, and others.
It is based on a DNS name, which enables global uniqueness across vendors and projects
without the need for new registries.

This URL is intended to then be added to a transparency platform for the purpose of
accessing the relevant artefacts in an automated fashion. Any necessary credentials
are to be provisioned separately and are not embedded in a TEI (Transparency Exchange Identifier).

A product release can have multiple TEIs—for example one with an EAN/UPC barcode and one with
the vendor's product number. TEIs can be communicated to the user in many ways, such as:

- a QR code on a box;
- on the invoice or delivery note;
- for software with a GUI, in an "about" box.

A user might obtain a TEI from the manufacturer, through a reseller, or directly. The TEI
is defined by the manufacturer and would not be expected to be derived from known information.

Since TEIs are based on DNS, manufacturers are provided a namespace to define product release
identifiers based on existing or new identifiers (see
<emu-xref href="#sec-discovery-5-tei-types"></emu-xref>). A given product release may
have multiple identifiers
as long as they all resolve into the same destination. Some identifier schemes require registration
with corresponding standards organizations.

There is no registry of TEIs; only TEI _types_ are registered.

### TEI syntax

A TEI is a URI as defined in RFC 3986, of the form:

```text
tei://<domain-name>/<type>/<unique-identifier>
```

- The scheme shall be `tei`.
- The `domain-name` shall be a DNS name under the vendor's control and resolve to a web
  server which need not be the API host. The domain name shall not include a port number.
- The `type` shall consist of lowercase ASCII letters and digits, and start with a letter.
  The type names the identifier scheme of the `unique-identifier`
  and shall be registered as described in
  <emu-xref href="#sec-discovery-5-tei-types"></emu-xref>.
- The `unique-identifier` shall be a single path segment in the syntax its type defines.
  The `unique-identifier` shall identify one or more product releases within the `domain-name`.
  It shall not contain `/`, either literally or percent-encoded as `%2F`.
  Any other characters that are not permitted in a path segment by RFC 3986 shall be percent-encoded with uppercase hexadecimal digits.

A TEI may identify several product releases,
for example when a boxed product keeps its EAN across firmware versions.
A discovery lookup therefore returns a list,
and clients shall treat its order as priority (first entry highest).
Manufacturers should strive for each TEI to identify a single product release;
types such as `uuid` or `hash` allows for this level of specificity.

### TEI types

TEI types are defined in the TEI type registry at
https://github.com/CycloneDX/transparency-exchange-api/tree/main/tei-types.
Registration refers to the TEA community process for adding a type, described in the registry.

The following table is **informative**.
It illustrates the types registered at the time of publication of this edition of the Standard;
the registry is the authoritative list.

Table: Informative: registered TEI types

| Type     | Unique identifier                              | Example                                                                                             |
|----------|------------------------------------------------|-----------------------------------------------------------------------------------------------------|
| `purl`   | Package URL, Base64URL-encoded without padding | `tei://cyclonedx.org/purl/cGtnOm1hdmVuL2NvbW1vbnMtaW8vY29tbW9ucy1pb0AyLjIyLjA`                      |
| `hash`   | Hash of an object, `<hashtype>:<hex>`          | `tei://cyclonedx.org/hash/SHA-256:fd44efd601f651c8865acf0dfeacb0df19a2b50ec69ead0262096fd2f67197b9` |
| `uuid`   | UUID                                           | `tei://cyclonedx.org/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1`                                     |
| `eanupc` | EAN or UPC number                              | `tei://example.com/eanupc/1234567890123`                                                            |
| `gtin`   | GTIN                                           | `tei://example.org/gtin/0234567890123`                                                              |
| `asin`   | Amazon Standard Identification Number          | `tei://example.com/asin/B07FZ8S74R`                                                                 |
| `udi`    | Unique Device Identifier                       | `tei://cyclonedx.org/udi/00123456789012`                                                            |

### Comparing TEIs

For the purpose of comparison, a TEI is an opaque identifier, not a locator.
Two TEIs are equal if and only if they are the same sequence of characters.
The comparison is case-sensitive and applies to the TEI as written:

- percent-escapes are neither added nor removed,
- Base64URL-encoded identifiers are not decoded, and
- the domain name is neither resolved nor normalized.

The following TEIs are therefore all distinct:

```text
tei://example.com/uuid/62f2cf92-ae88-11f1-a698-1a52914d44b2
tei://Example.com/uuid/62f2cf92-ae88-11f1-a698-1a52914d44b2
tei://example.com/uuid/62F2CF92-AE88-11F1-A698-1A52914D44B2
tei://example.com/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWI
tei://example.com/purl/cGtnOnB5cGkvY3ljbG9uZWR4LXB5dGhvbi1saWI=
```

A TEI shall be published in its "canonical" form, constrained by the following requirements:

- the scheme `tei` shall be lowercase,
- the domain name shall be lowercase ASCII, using A-labels for internationalized names,
- the type shall be lowercase,
- the unique identifier shall be formatted exactly as its registry entry defines it
  (for example lowercase for a UUID, lowercase hexadecimal for a hash, unpadded Base64URL for a PURL), and
- characters that do not require percent-escaping shall not be escaped.

A client shall use a TEI exactly as received and shall not rewrite it.

When a TEI is carried inside another URL,
for example as the `tei` query parameter of the discovery endpoint,
it shall be percent-encoded for transport.
Servers comparing received TEIs against their own data shall only do so after decoding an
encoded TEI.

Strict lowercase interpretation of the domain name is limited to comparison for identity.

### TEI resolution using DNS

The `domain-name` part of the TEI is used in a DNS query to find one or multiple locations for
product transparency exchange information.
The name in the DNS name part points to a set of DNS records.

For a TEI with `domain-name` `tea.example.com`, a client shall query DNS for `tea.example.com`, considering `A`, `AAAA` and `CNAME` records.

The `.well-known` endpoint shall only be available via HTTPS.
Upon connecting to the host using HTTPS, the TEA client shall verify the server
certificate, including the server identity check as described in RFC 9525.
The TEA client shall request the URL composed of the host name with the `/.well-known/tea` path added.

This results in the base URL such as
`https://products.example.com/.well-known/tea`

### TEA Discovery document

The TEA server shall respond to this request to the well-known tea path with a JSON
object that lists the available TEA server endpoints and supported versions.
The JSON shall conform to the [TEA Well-Known Schema](tea-well-known.schema.json).

Example:
```json
{
  "schemaVersion": 1,
  "endpoints": [
    {
      "url": "https://api.example.com",
      "versions":
        [
          "1.0.0"
        ],
      "priority": 1
    },
    {
      "url": "https://api2.example.com/mytea",
      "versions":
        [
          "1.0.0"
        ],
      "priority": 0.5
    }
  ]
}
```

Servers shall not locate the actual TEA service endpoint at the
`/.well-known/tea` URI. This URI is reserved for the TEA discovery
document and uses the well-known URI mechanism defined in RFC 8615.

### Discovery data caching and freshness

Discovery documents may be cached.

Implementations should:

- respect HTTP caching headers;
- periodically refresh discovery data;
- handle endpoint changes gracefully.

## Port resolution

Currently, the port number is not part of the TEI, but it is needed to connect to the API.

The TEA API server may be hosted on any port, but the server hosting the well-known
discovery document shall by default be running on the default HTTPS port 443.
Server URLs described in the discovery document may contain port numbers.

HTTPS/SVCB DNS records for failover and load balancing may be used to redirect to
other ports and servers for supported clients.

## Connecting to the API

Discovery proceeds in two stages that share the same retry, backoff, and attempt-bound
rules below:

1. **Well-known stage** — retrieve and use `/.well-known/tea` for the TEI’s domain host
2. **API discovery stage** — call `/discovery` on a selected API base URL constructed from
   a well-known endpoint entry

### Selecting an API base from `.well-known/tea`

Clients shall choose an endpoint from the `.well-known/tea` JSON response which lists
at least one API version supported by the client. The client shall prefer endpoints
with the highest mutually supported version, based on the SemVer [comparison clause](https://semver.org/#spec-item-11).
Advertised API versions shall fully conform to SemVer 2.0.0 (`MAJOR.MINOR.PATCH` with optional
prerelease and build metadata). Build metadata shall be ignored when comparing versions.

If the highest mutually supported
version is advertised with more than one build-metadata spelling, the client may select
any of them, and shall use the selected string exactly as advertised when constructing
the path (for example `/v1.0.0+build.5`). `+` is a valid path character per RFC 3986
and shall not be percent-encoded.

If several endpoints remain after that preference,
the client should pick the endpoint with the highest `priority` value (a float between
0 and 1). If `priority` is absent on a well-known endpoint object, the client shall
treat it as `1` for ordering (JSON Schema `default` does not populate omitted fields in
the JSON response).

The client shall then construct the full URL to the API by selecting that highest
mutually supported version and appending `/v` followed by that exact advertised
version string (for example `/v1.0.0`), then `/discovery?tei=` plus the TEI,
url-encoded according to RFC 3986.

<emu-example>

tei://products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1  
https://api.example.com/v1.0.0/discovery?tei=tei%3A//products.example.com/uuid/d4d9f54a-abcf-11ee-ac79-1a52914d44b1

</emu-example>

<emu-example>

tei://products.example.com/purl/cGtnOmRlYi9kZWJpYW4vY3VybEA3LjUwLjMtMT9hcmNoPWkzODYmZGlzdHJvPWplc3NpZQ  
https://api2.example.com/mytea/v1.0.0/discovery?tei=tei%3A//products.example.com/purl/cGtnOmRlYi9kZWJpYW4vY3VybEA3LjUwLjMtMT9hcmNoPWkzODYmZGlzdHJvPWplc3NpZQ

</emu-example>

The discovery endpoint is a part of the TEA OpenAPI specification.

A conforming TEA API base shall implement `/discovery` for any exposed API version.

### Discovery response

A successful `/discovery` response shall consist of a JSON array of `discovery-info` objects. Each
element in the array shall identify one resolved product release and the TEA servers that serve it:

- `productReleaseUuid` — UUID of the TEA Product Release
- `servers` — non-empty array of `server-info` objects (`rootUrl`, `versions`, and
  optional `priority`)

`.well-known/tea` and `servers[]` are related but distinct:

- **Well-known** lists candidate API bases for a domain. That is where the client calls
  `/v{version}/discovery`.
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
        "rootUrl": "https://api.example.com",
        "versions": ["1.0.0"]
      }
    ]
  }
]
```

Conforming deployments shall advertise only lowercase `https` base URLs, in
`.well-known/tea` `endpoints[].url` and in `/discovery` `servers[].rootUrl` alike. A
client shall reject an `http` base URL unless it has been explicitly configured to
allow that specific base URL for local testing. The allowance is per configured base URL,
never a global setting, so it cannot apply to a base URL the client learned from discovery.

### Discovery by PURL

Clients that already know a TEA API base URL (for example from a prior TEI discovery,
configuration, or cache) may call `/discovery` with the `purl` query parameter instead of
`tei`. Exactly one of either `tei` or `purl` shall be provided; a request with neither `tei` nor
`purl`, or both `tei` and `purl`, is rejected with `400`.

Discovery by PURL resolves within the inventory of the TEA server that receives the
request. It does not replace TEI-based DNS / `.well-known` discovery for finding an API
host from an identifier alone.

<emu-example>

https://api.example.com/v1.0.0/discovery?purl=pkg%3Amaven%2Forg.apache.logging.log4j%2Flog4j-core%402.24.3

</emu-example>

The response shape, non-empty success array, and `404` with `OBJECT_UNKNOWN` when the
server does not resolve the identifier are the same as for TEI lookup.

If this server does not resolve the TEI (or PURL, when discovering by PURL)—whether
because it is unknown or because the server withholds it—the discovery endpoint
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

A TEA server that requires no authentication on any endpoint need not implement the
`/token` endpoint, shall not answer any resource request with `401`, and shall ignore,
rather than reject, an `Authorization: Bearer` header a client presents anyway.

If authentication cannot be completed or recovery fails, the client shall indicate that
update status could not be determined. Failures that may require user or administrator
intervention include rejected or revoked credentials, expired client certificates,
persistent rejection of a replacement token, and insufficient permissions. A
`403 Forbidden` response indicates denied authorization and shall not trigger
token-replacement attempts solely because of that status. Clients shall not fail over to
another endpoint solely in response to `401` or `403`.

Clients shall verify server certificates for every HTTPS connection used in discovery and
subsequent API access, including the server identity check of
RFC 9525, and shall not use connections that
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

### Client behaviour

Clients should:

- retry with backoff
- validate TLS certificates
- fail closed if discovery cannot be validated
