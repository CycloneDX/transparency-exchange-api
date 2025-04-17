# TEA Use Cases

An overview of use cases to consider for the Transparency Exchange API. The use
cases are marked with a short code for reference. All use cases needs a story
like "Alice has bought a..."

The use cases are divided in two categories:

- Use cases for **customers** (end-users, manufacturers) to find a repository
  with Transparency Artefacts for a single unit purchased
- Use cases where there are different **products**
  - This applies after discovery where we need to handle various things a
    customer may buy as a single unit

## Customer focused use cases

### C1: Consumer: Automated discovery based on SBOM identifier

As a consumer that has an SBOM for a product, I want to be able to retrieve VEX
and VDR files automatically both for current and old versions of the software.
In the SBOM the product is identified by a PURL or other means (CPE, …)

### C2: Consumer: Automation based on product name/identifier

As a consumer, I want to download artifacts for a product based on known data. A
combination of manufacturer, product name, vendor product ID, EAN bar code or
other unique identifier. After discovering the base repository URL I want to be
able to find a specific product variant and version.

If the consumer is a business, then the procurement process may include delivery
of an SBOM with proper identifiers and possibly URLs or identifiers in another
document, which may bootstrap the discovery process in a more exact way than in
the case of buying a product in a retail market. Alice bought a gadget at the
gadget store that contains a full Linux system. Where and how will she find the
SBOM and VEX for the gadget?

### C3: Consumer: Artifact retrieval

As a consumer, I want to retrieve one or more supply chain artifacts for the
products that I have access to, possibly through licensing or other means. As a
consumer, I should be able to retrieve all source artifacts such as xBOMs,
VDR/VEX, CDXA, and CLE.

### C4: Consumer: Summarized CLE

As a consumer, I want the ability to get the current lifecycle values for a
given product. A CLE captures all lifecycle events over time, however, there is
a need to retrieve only the current values for things like product name, vendor
name, and milestone events.

### C5: Consumer: Insights

As a consumer, I want the ability to simply ask the API questions rather than
having to download, process, and analyze raw supply chain artifacts on my own
systems. Common questions should be provided by the API by default along with
the ability to query for more complex answers using the Common Expression
Language (CEL).

_NOTE_: Project Hyades (Dependency-Track v5) already implements CEL with the
CycloneDX object model and has proven that this approach works for complex
queries.

### D1: Developer/PM - Components for inclusion in products by other projects/developers

A developer needs a component - software, library (open source and/or
proprietary) to include in a product - publicly available (may be commercial) or
in an internal system. Developer can be individual, company or public sector.
They need insight into the library or component and be able to automatically
keep upstream artefacts up to date.

### B1: Business consumer

Acme LLC buys 3 000 gadgetrons from Emca LTD to be distributed over a retail
chain. Acme runs an in-house vulnerability management system (Dependency Track)
to manage SBOMs and download VEX files to check for vulnerabilities. Acme has
products from exactly 14.385 vendors in the system. How will their systems get
continuous access to current and old documents - attestations, SBOM, VEX and
other files?

### E1: Third party: Regulators

Alice & Bob Enterprises AB has gotten a EUCC certification to get their Whola
Firewall certified for CRA-compatible CE labeling. In order to maintain the
certification the certifying body needs access to SBOM and VEX updates from A&BE
in an automated way.

### E2: External potential customer - insights before purchase

Palme Auditors INC wants to buy the ACME SWISH product from a vendor. They want
to examine vulnerability handling and get some insights into the products before
making a decision.

### E3: Hacker or competition

There are non-customers and not-to-be-customers that wants to get insight into
how a product is composed by getting the transparency docs.

### E4: Open Source user

Palme Inc considers using the asterisk.org open source telephony PBX. They need
to make an assessment before starting tests and possible production use. The can
either use the Debian package, the Alpine Linux Package or build a binary
themselves from source code. How can they find the transparency exchange data
sets?

### O1: Open Source project

The hatturl open source project publish a library and a server side software on
Github. This is later packaged as packages in many Linux distributions.

How does the project publish artefacts? What are the requirements?

## Product focused use cases

### P1: A standalone app

Customer buys a standalone application that is delivered as a binary

### P2: A OCI container

Customer gets a container with many third party components and a binary software
developed by the manufacturer

### P3: Embedded system

### P4: Package with many devices

### P5: An Open Source software library
