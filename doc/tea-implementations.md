![](/images/tealogo.png)

# TEA Implementations


## Open Source clients

### py-libtea

A Python client library and CLI client for TEA.

- [GitHub](https://github.com/sbomify/py-libtea)

### PyPi TEA

A TEA bridge for accessing Python SBOMs from PyPi (via [PEP 770](https://peps.python.org/pep-0770/)).

- [GitHub](https://github.com/sbomify/pypi-tea)

### ReARM CLI

The ReARM CLI supports TEA.

- [Documentation](https://github.com/relizaio/rearm-cli/blob/main/docs/tea.md)
- [GitHub](https://github.com/relizaio/rearm-cli)

## Open Source Servers

### Oolong

This project is a lightweight implementation of Transparency Exchange API which uses NestJS framework.

- [GitHub](https://github.com/relizaio/oolong)

### ReARM

ReARM is a Release-Level Supply Chain Evidence Platform. It supports TEA for standardized discovery and retrieval of SBOMs and other security artefacts.

- [Documentation](https://docs.rearmhq.com/tea/)
- [GitHub](https://github.com/relizaio/rearm)

### sbomify

sbomify is a Software Bill of Materials (SBOM) and document management platform that can be self-hosted or accessed through [app.sbomify.com](https://app.sbomify.com). The platform provides a centralized location to upload and manage your SBOMs and related documentation, allowing you to share them with stakeholders or make them publicly accessible.

- Implements the Transparency Exchange API
- Standardized SBOM discovery via .well-known/tea endpoints
- Enables automated discovery and retrieval of SBOMs across the supply chain

- [Documentation](https://sbomify.com/faq/how-do-i-enable-tea-in-sbomify/)
- [GitHub](https://github.com/sbomify/sbomify)

## Other implementations

### CyBeats SBOM Studio (commercial)

Cybeats SBOM Studio centralizes the SBOM lifecycle and product vulnerability monitoring and exposes a CycloneDX Transparency Exchange API endpoint through standardized .well-known/tea discovery, enabling automated distribution of SBOMs and related security artifacts across the supply chain. *Curently for demonstration purposes only.

- [Product Details](https://www.cybeats.com/product/sbom-studio)
- [TEA Endpoint](https://us.sbom.cybeats.com/.well-known/tea)

### CyBeats SBOM Consumer (commercial)

Cybeats SBOM Consumer enables IT teams to configure a vendor TEA domain and automatically discover, retrieve, and import supplier SBOMs into the Consumer instance for validation and continuous risk monitoring.

- [Product Details](https://www.cybeats.com/product/sbom-consumer)

If you want to have your implementation listed here, please provide a pull request.

