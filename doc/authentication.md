# Authentication

TEA deployments need a way to distinguish public discovery from protected read
and write operations. This document describes common authentication patterns
without requiring one identity stack for every TEA deployment.

## Base expectations

The base TEA specification does not mandate a single identity provider, token
format, or client certificate profile. An implementation is compliant if it can:

- authenticate callers for protected operations
- bind authenticated identities to local authorization policy
- protect authenticated traffic with TLS

A deployment MAY expose some discovery or read-only endpoints publicly.
Publisher or other mutating operations SHOULD require an authenticated caller.

## Bearer tokens

Bearer tokens are a common choice for HTTP and gRPC deployments.

- Clients send bearer credentials in the `Authorization` header.
- Token format is deployment-defined. JWT is common, but opaque access tokens
    are also valid if the server can validate them.
- Servers SHOULD validate token lifetime, issuing authority, intended audience
    or resource, and any permissions or claims used for authorization.

Example:

```text
Authorization: Bearer <access-token>
```

## Mutual TLS

Mutual TLS is appropriate for closed ecosystems and higher-assurance
machine-to-machine integrations.

- The server validates the client certificate chain and certificate status
    according to local PKI policy.
- The server binds the certificate identity, such as subject, subject
    alternative name, or a mapped identifier, to local authorization policy.
- The base TEA guidance does not require one client certificate algorithm, one
    certificate subject model, or one CA topology for every deployment.

OWASP recommends considering mTLS for high-value applications and APIs.

## Transport security

Authenticated TEA endpoints, and any endpoint carrying non-public data, MUST
only be exposed over TLS.

This base guidance intentionally does not set one mandatory TLS version or one
universal certificate and key profile for every TEA deployment. Those choices
are environment-specific and should come from an applicable TEA profile or
deployment baseline.

Examples:

- OWASP recommends defaulting to TLS 1.3 and supporting TLS 1.2 if necessary
    for general-purpose web applications.
- NIST SP 800-52 Rev. 2 defines a stricter government TLS baseline for federal
    systems.
- Frameworks such as NIS2 or ITSG-33 can drive additional organizational
    requirements, but they do not by themselves create one universal TEA
    transport profile.

## Error responses

- `401 Unauthorized`: missing or invalid credentials
- `403 Forbidden`: authenticated, but not authorized for the requested
    operation

## Operational notes

- Log authentication and certificate validation failures for audit and incident
    response.
- Rotate bearer credentials and client certificates according to local policy.
- Prefer profile-level or deployment-level guidance for stricter transport
    baselines.

## Non-normative references

- RFC 6750: OAuth 2.0 Bearer Token Usage
- RFC 5280: Internet X.509 Public Key Infrastructure Certificate and CRL
    Profile
- OWASP Transport Layer Security Cheat Sheet
- NIST SP 800-52 Rev. 2: Guidelines for the Selection, Configuration, and Use
    of TLS Implementations
