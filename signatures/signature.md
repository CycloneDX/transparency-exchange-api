# Trusting digital signatures in TEA

Software transparency is a lot about trust. Within the
API documents needs to be signed with an electronic
signature. CycloneDX boms supports signatures within
the JSON file, but other artefacts may need external
signature files.

- __Integrity__: Documents dowloaded needs to be the same
  as documents published
- __Identity__: Customers needs to be able to verify the 
  publisher of the documents and verify that it is
  the expected publisher

In order to sign, a pair of asymmetric keys will be needed.
The public key is used to create a certificate, signed
by a certificate authority (CA).

A software publisher may bye CA services from a commercial vendors
or set up an internal solution. The issue with that is that external
parties do not automatically trust that internal PKI.

This document outlines a proposal on how to build that trust and
make it possible for publishers to use an internal PKI. It is
of course important that this PKI is maintained according to 
best current practise.

## API trust

The TEA API is built on the HTTP protocol with TLS encryption
and authentication, using the https:// URL scheme.

The TLS server certificate is normally issued by a public Certificate
Authority that is part of the Web PKI. The client needs to validate
the TLS server certificate to make sure that the certificate name
(CN or Subject Alt Name) matches the host part of the URI.

If the certificate validates properly, the API can be trusted.
The server is the right server. This trust can be used to
implement trust in a private PKI used to sign documents.

## Getting trust anchors

Much like the EST protocol, the TEA protocol can be used
to download trust anchors for a private PKI. These are
PEM-encoded certificates that are in one text file.

The TEA API has a `/trust-anchors/` API that will download
the current trust anchor APIs. This file is not signed,
that would cause a chicken-and-egg problem.

## Validating the trust anchors using DNSsec (DANE)

## Digital signatures

### Digital signatures as specified for CycloneDX
"Digital signatures may be applied to a BOM or to an assembly within a BOM. CycloneDX supports XML Signature, JSON Web Signature (JWS), and JSON Signature Format (JSF). Signed BOMs benefit by providing advanced integrity and non-repudiation capabilities."
https://cyclonedx.org/use-cases/#authenticity


### External (deattached) digital signatures for other documents

- indication of hash algorithm
- indicator of cert used
- intermediate cert and sign cert

### Validating the digital signature

## Using Sigstore for signing

## Suggested PKI setup

### Root cert

#### Root cert validity and renewal

### Intermediate cert

#### Intermediate cert validity and renewal

### Signature

#### Time stamp services

### DNS entry



## References

- IETF RFC DANE
- IETF DANCE architecture (IETF draft)
- IETF Digital signature
- JSON web signatures (JWS) - https://datatracker.ietf.org/doc/html/rfc7515
- JSON signature format (JSF) - https://cyberphone.github.io/doc/security/jsf.html
- [IETF Enrollment over Secure Transport (EST) RFC 7030](https://www.rfc-editor.org/rfc/rfc7030)