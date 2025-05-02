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

This specification does not impose any requirement on authentication on a TEA service. But should
the provider implement authentication, two methods are supported in order to promote interoperability.

* HTTP Bearer Token Authentication
* Mutual TLS with verifiable client and server certificates

A client may use both HTTP bearer token auth and TLS client certificates
when accessing multiple TEA services. It is up to the service provider to select authentication.

## HTTP bearer token auth

The API will support HTTP bearer token in the __Authorization:__ http header.
How the token is aquired is out of scope for this
specification, as is the potential content of the token.

As an example the token can be downloaded from a customer support portal with a long-term
validity. This token is then installed into the software transparency platform (the TEA client)
and used to automatically retrieve software transparency artefacts.

For each TEA service, one bearer token is needed. The token itself (or the backend) should
specify authorization for the token.

## Mutual TLS

For Mutual TLS the client certificates will be managed by the service and provided
in a service-specific way. Clients should be able to configure a separate client certificate
(and private key) on a per-service level, not assuming that a client certificate
for one service is trusted anywhere else.

## References

* RFC 6750: The Oauth 2.0 Authorization Framework: Bearer Token 
  Usage (https://www.rfc-editor.org/rfc/rfc6750)
