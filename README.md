[![License](https://img.shields.io/badge/license-Apache%202.0-brightgreen.svg)](LICENSE)
[![Website](https://img.shields.io/badge/https://-cyclonedx.org-blue.svg)](https://cyclonedx.org/)
[![Slack Invite](https://img.shields.io/badge/Slack-Join-blue?logo=slack&labelColor=393939)](https://cyclonedx.org/slack/invite)
[![Group Discussion](https://img.shields.io/badge/discussion-groups.io-blue.svg)](https://groups.io/g/CycloneDX)
[![Twitter](https://img.shields.io/twitter/url/http/shields.io.svg?style=social&label=Follow)](https://twitter.com/CycloneDX_Spec)
[![ECMA TC54](https://tc54.org)](https://tc54.org)

# CycloneDX Transparency Exchange API Standard

The Transparency API Exchange API is being worked on within the CycloneDX community
with the goal to standardise the API in ECMA. A working group within ECMA TC54 has been
formed - TC54 TG1. The working group has a slack channel in the CycloneDX slack space.

![](images/Project-Koala.svg)

## Introduction

This specification defines a standard, format agnostic, API for the exchange of
product related artefacts, like BOMs, between systems. The work includes:

- Discovery of servers
- Retrieval of artefacts
- Publication of artefacts
- Authentication and authorization
- Querying

System and tooling implementors are encouraged to adopt this API standard for
sending/receiving transparency artefacts between systems. 
This will enable more widespread
"out of the box" integration support in the BOM ecosystem.

## Previous work

- [The CycloneDX BOM Exchange API](/api/bomexchangeapi.md)
   Implemented in the [CycloneDX BOM Repo Server]
   (https://github.com/CycloneDX/cyclonedx-bom-repo-server)
