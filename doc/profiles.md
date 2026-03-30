# TEA Profiles

The base TEA specification defines the common contract. Some deployments need a
more specific subset or a stricter operating baseline. A TEA profile is a named
set of constraints and guidance applied on top of that base contract.

This is similar in spirit to a tailoring layer, as used by OSCAL profiles, but
applied to TEA APIs, messages, and deployment expectations.

## Why profiles exist

Profiles let TEA stay small and interoperable while still allowing communities
to publish sharper guidance for a given environment.

A profile can:

- require a subset of TEA capabilities
- tighten transport or authentication expectations
- add validation or lifecycle rules for a deployment community
- document interoperability expectations without redefining the base model

## Profile rules

A TEA profile:

- MUST identify the TEA version it applies to
- MAY tighten requirements from the base specification
- MUST NOT change the meaning of core fields or RPCs
- SHOULD describe any added transport, security, or operational rules as
  profile-level guidance rather than changing the base contract

## Examples

Possible TEA profiles include:

- a minimal discovery and consumer profile
- a publisher API profile
- a regulated deployment profile with stricter transport requirements

## Security and transport

Transport and authentication baselines vary by deployment. For example, a
higher-assurance profile may require mutual TLS, a specific TLS baseline, or a
particular identity-mapping process.

Those requirements should generally live in a TEA profile or deployment
baseline, not in the base TEA contract, when they are environment-specific.