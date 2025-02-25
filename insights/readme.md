# Insights API

Much of the focus on Software Transparency from the U.S. Government (and others) centers around the concept of “full transparency.” In practice, this often means **consumers** need to ingest, process, and analyze SBOMs or VEX documents just to answer **simple questions**, such as:

- *Do any of my licensed products from Vendor A use Apache Log4J?*
- *Are any of my licensed products from Vendor A vulnerable to log4shell, and is there any action I need to take?*

However, “full transparency” can be cumbersome for consumers, requiring them to parse potentially large and complex SBOM or VEX data.

## Overview

**Insights** provides a more streamlined approach: **“limited transparency.”** It enables consumers to:

1. Ask specific, outcome-driven queries in an **expression language** (CEL) or via **natural-language prompts** (AI/LLM).
2. Receive just the **essential information** needed—without having to manage BOM conversions themselves.

Under the hood, Insights uses an **object model derived from CycloneDX** to generate responses. However, the **actual BOM consumption and processing** on the implementation side is **not** format-specific—implementers can support **CycloneDX, SPDX, Syft, or any other** current or future BOM format. Insights simply **normalizes** data into a CycloneDX-based response format for consistency and interoperability.

## Endpoints

Insights provides two primary endpoints to query the system:

### 1. **Static Insights** (`POST /insights/static`)

- **Method**: `POST`
- **Request Body**:
    - `expression` (string) – A [CEL](https://github.com/google/cel-spec) (Common Expression Language) expression describing the data you want to retrieve from the underlying model.
- **Response**: Returns a **CycloneDX** fragment (JSON) containing the filtered results.

Use this when you have a **precise, automatable condition** (e.g., `"component.name == 'Apache Struts'"`).

### 2. **Dynamic Insights** (`POST /insights/dynamic`)

- **Method**: `POST`
- **Request Body**:
    - `prompt` (string, **required**) – The main user-provided natural-language query.
    - `systemPrompt` (string, *optional*) – A higher-level or “system” instruction guiding the AI’s persona or context (e.g., “You are a security expert.”).
    - `modelSettings` (object, *optional*) – Additional parameters/flags for fine-tuning the AI model (e.g., `temperature`, `maxTokens`).
- **Response**: Returns a **CycloneDX** fragment (JSON) based on the AI’s interpretation of your prompt.

Use this when you want to **ask in natural language** without necessarily writing a formal query expression.

## Why “Limited Transparency”?

Instead of requiring **full ingestion and parsing** of entire SBOMs or VEX documents by the consumer, Insights:

- Lets you **ask specifically** for what you need.
- Returns **only** the relevant data.
- Offloads the complexity of BOM handling and format conversions to the **server** side, **regardless of the original BOM format**.

## Example Queries & Use Cases

Below are **typical questions** you might ask using either the static **(CEL)** or dynamic **(AI)** approach:

1. **List all third-party dependencies for a given product**
    - Static example (`expression`): `"productId == 'XYZ' && component.type == 'third-party'"`
    - Dynamic example (`prompt`): “Which third-party dependencies does product XYZ use?”

2. **Show only the open-source dependencies for a given product**
    - Static example: `"productId == 'XYZ' && component.licenseType == 'Open Source'"`
    - Dynamic example: “List all open-source dependencies in product XYZ.”

3. **List all vulnerabilities in a given product**
    - Static example: `"productId == 'XYZ' && component.vulnerabilities.size() > 0"`
    - Dynamic example: “What vulnerabilities exist in product XYZ?”

4. **Does the product use log4j and, if so, is it vulnerable to log4shell? If it is, what actions are needed to minimize risk?**
    - Static example: `"productId == 'XYZ' && component.name == 'log4j' && component.vulnerabilities.contains('log4shell')"`
    - Dynamic example: “Does product XYZ use log4j? If so, is it impacted by log4shell and how can I mitigate the risk?”

5. **Which cryptographic algorithms does a product use?**
    - Static example: `"productId == 'XYZ' && component.cryptography != null"`
    - Dynamic example: “Identify all cryptographic algorithms in product XYZ.”

6. **Provide an SSDF, BSIMM, or OWASP SAMM attestation on how a product was developed**
    - Static example: `"productId == 'XYZ' && product.processAttestation.type == 'OWASP SAMM'"`
    - Dynamic example: “Generate an attestation for product XYZ based on OWASP SAMM (or SSDF/BSIMM).”

Each of these examples demonstrates how you can **quickly retrieve insights** from product data that’s anchored in CycloneDX—either using structured CEL queries or direct natural-language questions.

## Architecture Highlights

- **CycloneDX-Derived Model**  
  While the **response** is always a **CycloneDX** structure (to maintain consistency), the underlying engine can **ingest data from any BOM format**—CycloneDX, SPDX, Syft, etc.—and normalize it for output.

- **Expression-Language Queries (CEL)**  
  A concise, powerful way to specify filtering and logic—ideal for CI pipelines or automated workflows.

- **AI/Natural Language Queries**  
  Enables a more conversational style, letting you “speak” to the system in everyday language while still retrieving detailed SBOM or VEX insights.

- **Implementation Flexibility**
    - `systemPrompt`: Optional top-level context (e.g., “You are a security guru”).
    - `modelSettings`: An open-ended object for advanced model parameters (e.g., `temperature`, `maxTokens`).

## Error Handling

- **400 Bad Request**: Returned if the CEL expression is invalid or if the request body is malformed.
- **Other 4xx/5xx**: Could be used for authentication issues, server errors, etc., depending on your deployment environment.

## Future Enhancements

- **Session/Conversation Support**: To enable multi-turn conversations with context retained across requests.
- **Streaming Responses**: For long or incremental AI responses, possibly via WebSockets or Server-Sent Events.
- **Extended Model Parameters**: For more sophisticated control over AI behavior.

## Conclusion

**Insights** streamlines software transparency by letting consumers query or ask questions **on-demand**—without the burden of parsing raw SBOM or VEX formats. **Regardless of which BOM standard** you use—CycloneDX, SPDX, Syft, or another—Insights can unify the data and produce a consistent, CycloneDX-based response. By leveraging **CEL** and **AI** queries, it accommodates both automated and human-friendly workflows alike.

For more details on the endpoints and schema definitions, refer to the [OpenAPI specification](./openapi.json) in this repository. If you have further questions, please open an issue or reach out to the maintainers.

