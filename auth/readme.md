# Transparency Exchange API - Authentication and authorization

This document covers authentication and authorization on the consumer side
of a TEA service - the discovery and download of software transparency artefacts.

## Requirements

__Authorization__: A user of a TEA service may get access to all objects (components, collections) and
artefacts or just a subset, depending on the publisher of the data. Authorization is connected
to __authentication__.

The level of authorization is up to the implementer of the TEA implementation and the publisher,
whether an identity gets access to all objects in a service or just a subset.

In order to get interoperability between clients and servers implementing the protocol, the
specification focuses on the authentication. After successful authentication, the authorization
may be implemented in multiple ways - on various levels of the API - depending on what information
the user can access.

As an example, one implementation may publish all information about existing artefacts and software
versions openly, but restrict access to artefacts to those that match the customers installation.
Another implementation can implement a filter that does not show products and versions ("components") that
the customer has not aquired.

For most Open Source projects, implementing authentication - setting up accounts and managing
authorization - does not make much sense, since the information is usually in the open any way.

## Scope of this specification

This specification does not require a TEA service to authenticate its users. A service that
publishes openly need not implement any of what follows.

Where a service does authenticate, interoperability requires that every TEA client can
authenticate against every TEA server without server-specific code. This specification therefore
defines a single mandatory baseline and leaves everything above it optional:

* A TEA server that requires authentication __shall__ implement the token endpoint (`POST /token`)
  described below, and __shall__ support the API key credential exchange on it.
* A TEA server __may__ support additional credential types on the same endpoint - federated
  identity from an external OpenID Connect or SAML provider, mutual TLS, or others.
* A client that holds an API key for a TEA service __shall__ be able to reach every endpoint it
  is authorized for using only the baseline exchange.

Two consequences are worth stating explicitly, because they are what make the baseline useful:

* All authenticated access goes through the token endpoint. A server __shall not__ accept an API
  key directly on the resource endpoints, and a client __shall not__ present one there. The API key
  is exchanged for an access token, and the access token is what the resource endpoints see.
* Whatever credential a client starts with, it ends up holding the same thing: a TEA access token
  presented as an HTTP bearer token. Clients therefore need one code path for the API itself,
  regardless of how the server manages identity.

## The token endpoint

The token endpoint is `POST /token`, relative to the TEA API base URL, and is defined in
[the OpenAPI specification](../spec/openapi.yaml). It is an OAuth 2.0 token endpoint as defined
in [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749) section 3.2; this specification constrains
which grant types a conforming server has to accept, and adds nothing to the wire format.

RFC 6749 leaves the location of the token endpoint outside its scope - `/token` appears only in its
examples - so TEA fixes the path here rather than requiring clients to discover it. TEA defines no
OAuth 2.0 authorization endpoint (RFC 6749 section 3.1): there is no interactive, browser-based
consent step in TEA, and every grant type described below is one a client can complete on its own.

### API key exchange (mandatory)

An API key consists of two parts issued together by the service: an __identifier__ and a
__secret__. The identifier need not be confidential; the secret is.

The client presents them using the HTTP Basic authentication scheme
([RFC 7617](https://www.rfc-editor.org/rfc/rfc7617)) - the identifier as the user-id and the secret
as the password - with the `client_credentials` grant type. This is OAuth 2.0 client
authentication as described in RFC 6749 section 2.3.1, which requires a token endpoint to support
Basic for clients that were issued a secret.

```http
POST /token HTTP/1.1
Host: tea.example.com
Authorization: Basic <base64 of "key-id:key-secret">
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials
```

Per RFC 6749 section 2.3.1 the identifier and secret are each `application/x-www-form-urlencoded`
encoded before being joined with a colon and Base64 encoded. This only matters when a credential
contains a colon or characters outside US-ASCII, but clients and servers __shall__ apply it so that
such credentials interoperate.

Servers __may__ additionally accept the credentials as `client_id` and `client_secret` form
parameters in the request body. RFC 6749 section 2.3.1 marks this as NOT RECOMMENDED and limits it
to clients that cannot use Basic, so clients __should__ use Basic where they can.

A successful response is the standard OAuth 2.0 token response (RFC 6749 section 5.1):

```http
HTTP/1.1 200 OK
Content-Type: application/json;charset=UTF-8
Cache-Control: no-store

{
  "access_token": "2YotnFZFEjr1zCsicMWpAA",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Errors are the standard OAuth 2.0 token error response (RFC 6749 section 5.2), for example
`invalid_client` for a bad API key or `unsupported_grant_type` for a grant the server does not
implement.

### Using the access token

The access token is presented on every other TEA endpoint as an HTTP bearer token
([RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)):

```http
GET /product/d4d9f54a-abcf-11ee-ac79-1a52914d44b HTTP/1.1
Host: tea.example.com
Authorization: Bearer 2YotnFZFEjr1zCsicMWpAA
```

__The access token is opaque to the client.__ Clients __shall not__ inspect, parse, or depend on the
contents of the token, and __shall not__ assume any particular format. Servers are free to issue a
random handle, a signed token such as a JWT access token
([RFC 9068](https://www.rfc-editor.org/rfc/rfc9068)), or anything else, and to change that choice
without notice; only the issuing server interprets it.

Token lifetime, storage, rotation, and revocation are implementation matters and are deliberately
not specified. Two rules keep clients simple in spite of that:

* Servers __should__ return `expires_in`, so that a client can obtain a new token before the current
  one expires rather than discovering expiry through a failed request.
* If a request to a resource endpoint fails with `401` and the `invalid_token` error code, the
  client __may__ obtain a new token from the token endpoint and retry the request once, as described
  in RFC 6750 section 3.1. A client __should not__ retry more than once for a single request.

Between them these cover server-side revocation without the client having to know it happened:
RFC 6750 defines `invalid_token` as covering tokens that are "expired, revoked, malformed, or invalid
for other reasons". This specification defines no revocation endpoint, and clients need not
implement one.

__No refresh tokens.__ Servers __should not__ issue refresh tokens with the `client_credentials`
grant, following RFC 6749 section 4.4.3. A refresh token exists so that a client can obtain a new
access token without re-presenting a credential, which matters when the credential belongs to a user
who is no longer present. A TEA client holds its own API key permanently and can simply call the
token endpoint again, so a refresh token would be a second long-lived secret to store and protect for
no benefit.

## Optional credential types

A server __may__ accept credentials other than an API key at the same token endpoint, selected by the
`grant_type` parameter. In every case the response is the same token response, and the resulting
access token is used the same way, so clients that already implement the baseline need only the
additional request.

The following are the cases we expect to be common. All are existing OAuth 2.0 profiles; TEA adds
nothing to them.

| Use case | Grant type | Specification |
|---|---|---|
| Enterprise SSO where the customer's identity provider issues SAML assertions | `urn:ietf:params:oauth:grant-type:saml2-bearer` | [RFC 7522](https://www.rfc-editor.org/rfc/rfc7522) |
| OpenID Connect, or any provider issuing signed JWTs, including workload identity in CI systems | `urn:ietf:params:oauth:grant-type:jwt-bearer` | [RFC 7523](https://www.rfc-editor.org/rfc/rfc7523) |
| A client already holding a token from another security domain, exchanged for a TEA token | `urn:ietf:params:oauth:grant-type:token-exchange` | [RFC 8693](https://www.rfc-editor.org/rfc/rfc8693) |
| A client authenticated by a TLS client certificate rather than a shared secret | `client_credentials` with mutual TLS client authentication | [RFC 8705](https://www.rfc-editor.org/rfc/rfc8705) |

[RFC 7521](https://www.rfc-editor.org/rfc/rfc7521) defines the common framework the two assertion
grants share.

Note that a server which delegates identity to an external provider still issues its own TEA access
token from its own token endpoint. The external provider authenticates the user; the TEA server
decides what that user may see. This keeps authorization with the party that owns the data, and
keeps the resource endpoints validating exactly one kind of token.

### Mutual TLS

Mutual TLS is a client authentication method at the token endpoint, not a separate way in. A client
authenticated by a certificate presents no `Authorization` header on its token request; the
certificate identifies it. RFC 8705 additionally allows the issued access token to be bound to the
certificate, so that a stolen token is useless without the corresponding private key.

Client certificates are managed by the service and provided in a service-specific way. Clients
__should__ be able to configure a separate client certificate and private key per TEA service, and
__should not__ assume that a client certificate for one service is trusted anywhere else.

## Transport security

All of the above assumes TLS. Credentials and bearer tokens are transmitted in the clear at the HTTP
layer, so a TEA server __shall__ be reachable only over TLS, and clients __shall__ verify the server
certificate. This restates RFC 6749 section 3.2 and RFC 6750 section 5.

## Discovery

A client that has an API key for a service knows to call the token endpoint. A client that does not
know whether a service requires authentication at all can simply issue the request: a server that
requires authentication answers `401` with a `WWW-Authenticate` header naming the `Bearer` scheme,
as described in RFC 6750 section 3.

Servers that delegate identity to an external provider __may__ publish OAuth 2.0 protected resource
metadata ([RFC 9728](https://www.rfc-editor.org/rfc/rfc9728)) and reference it from the
`WWW-Authenticate` challenge, allowing a client to locate the authorization server automatically.
This is optional; it does not replace the token endpoint, which remains the interoperable baseline.

## References

* RFC 6749: The OAuth 2.0 Authorization Framework (https://www.rfc-editor.org/rfc/rfc6749)
* RFC 6750: The OAuth 2.0 Authorization Framework: Bearer Token Usage (https://www.rfc-editor.org/rfc/rfc6750)
* RFC 7617: The 'Basic' HTTP Authentication Scheme (https://www.rfc-editor.org/rfc/rfc7617)
* RFC 7521: Assertion Framework for OAuth 2.0 Client Authentication and Authorization Grants (https://www.rfc-editor.org/rfc/rfc7521)
* RFC 7522: SAML 2.0 Profile for OAuth 2.0 Client Authentication and Authorization Grants (https://www.rfc-editor.org/rfc/rfc7522)
* RFC 7523: JWT Profile for OAuth 2.0 Client Authentication and Authorization Grants (https://www.rfc-editor.org/rfc/rfc7523)
* RFC 8693: OAuth 2.0 Token Exchange (https://www.rfc-editor.org/rfc/rfc8693)
* RFC 8705: OAuth 2.0 Mutual-TLS Client Authentication and Certificate-Bound Access Tokens (https://www.rfc-editor.org/rfc/rfc8705)
* RFC 9068: JSON Web Token (JWT) Profile for OAuth 2.0 Access Tokens (https://www.rfc-editor.org/rfc/rfc9068)
* RFC 9728: OAuth 2.0 Protected Resource Metadata (https://www.rfc-editor.org/rfc/rfc9728)

### A note on API keys and `X-API-Key`

There is no IETF specification for API keys, and no registered HTTP authentication scheme for them.
A survey of the IETF datatracker finds only working-group discussion material and an early
individual draft, nothing normative. The widespread `X-API-Key` header is additionally at odds with
[RFC 6648](https://www.rfc-editor.org/rfc/rfc6648), a Best Current Practice that deprecates the `X-`
prefix for new header fields.

This specification therefore carries the API key in the standard `Authorization` header using the
Basic scheme, which is registered, specified, supported directly by essentially every HTTP client
library, and already the mandatory-to-implement client authentication method for OAuth 2.0 token
endpoints. The result is no harder for a client than a custom header - it is one line in any HTTP
library - and it avoids inventing a TEA-specific credential format.
