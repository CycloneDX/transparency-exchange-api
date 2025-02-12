# Transparency Exchange API - Authentication and authorization


**NOTE**: _This is a proposal for the WG_

This document covers authentication and authorization on the consumer side
of a TEA service - the discovery and download of software transparency artefacts.

## Requirements

__Authorization__: A user of a TEA service may get access to all objects (leaf, collections) and
artefacts or just a subset, depending on the publisher of the data. Authorization is connected
to __authentication__. 

The level of authorization is up to the implementer of the TEA implementation and the publisher.

In order to get interoperability between clients and servers implementing the protocol, the
specification focuses on the authentication. After successful authentication, the authorization
may be implemented in multiple ways - on various levels of the API - depending on what information
the user can access.

As an example, one implementation may publish all information about existing artefacts and software
versions openly, but restrict access to artefacts to those that match the customers installation.
Another implementation can implement a filter that does not show products and versions ("leafs") that
the customer has not aquired.

For most Open Source projects, implementing authentication - setting up accounts and managing
authorization - does not make much sense, since the information is usually in the open any way.

## HTTP bearer token auth

The API will support HTTP bearer token in the __Authorization:__ http header.
How the token is aquired is out of scope for this
specification, as is the potential content of the token.

As an example the token can be downloaded from a customer support portal with a long-term
validity. This token is then installed into the software transparency platform (the TEA client)
and used to automatically retrieve software transparency artefacts.

## References

- RFC 6750: The Oauth 2.0 Authorization Framework: Bearer Token Usage (https://www.rfc-editor.org/rfc/rfc6750)
