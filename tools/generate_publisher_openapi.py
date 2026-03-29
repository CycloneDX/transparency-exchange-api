#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'spec/publisher/openapi.json'


def ref(name):
    return {"$ref": f"#/components/schemas/{name}"}


def example_ref(name):
    return {"$ref": f"#/components/examples/{name}"}


def response_ref(name):
    return {"$ref": f"#/components/responses/{name}"}


def schema_array(items, min_items=None):
    schema = {"type": "array", "items": items}
    if min_items is not None:
        schema["minItems"] = min_items
    return schema


def obj(properties, required=None, **extra):
    schema = {"type": "object", "properties": properties}
    if required:
        schema["required"] = required
    schema.update(extra)
    return schema


def examples_map(*pairs):
    return {name: example_ref(example_name) for name, example_name in pairs}


def request_body(schema_name, description=None, content_type='application/json', examples=None):
    media_type = {"schema": ref(schema_name)}
    if examples:
        media_type["examples"] = examples
    body = {
        "required": True,
        "content": {content_type: media_type},
    }
    if description:
        body["description"] = description
    return body


def success_response(description, schema_name, examples=None):
    media_type = {"schema": ref(schema_name)}
    if examples:
        media_type["examples"] = examples
    return {
        "description": description,
        "content": {"application/json": media_type},
    }


def op(rpc, summary, description, tag, status, response, response_description,
       request=None, request_description=None, request_content_type='application/json',
       path_params=None, query_params=None, errors=None, streaming=False,
       request_examples=None, response_examples=None):
    operation = {
        "operationId": rpc,
        "summary": summary,
        "description": description,
        "tags": [tag],
        "security": [{"bearerAuth": []}],
        "x-tea-proto-rpc": rpc,
        "x-reference-status": status,
        "x-canonical-contract": "proto/tea/v1/publisher.proto",
        "responses": {
            "200": success_response(response_description, response, examples=response_examples),
        },
    }
    if streaming:
        operation["x-tea-streaming"] = "client"
    if path_params or query_params:
        operation["parameters"] = []
        for name in path_params or []:
            operation["parameters"].append({"$ref": f"#/components/parameters/{name}"})
        for name in query_params or []:
            operation["parameters"].append({"$ref": f"#/components/parameters/{name}"})
    if request:
        operation["requestBody"] = request_body(
            request,
            description=request_description,
            content_type=request_content_type,
            examples=request_examples,
        )

    code_by_error = {
        "InvalidRequest": "400",
        "Unauthenticated": "401",
        "NotFound": "404",
        "FailedPrecondition": "409",
        "InvalidContent": "422",
        "UpstreamFetchFailed": "502",
        "UnsupportedCapability": "501",
    }
    for error_name in errors or []:
        operation["responses"][code_by_error[error_name]] = response_ref(error_name)
    return operation


def enum_string(description, examples=None):
    schema = {"type": "string", "description": description}
    if examples:
        schema["examples"] = examples
    return schema


EXAMPLES = {
    "createProductRequest": {
        "summary": "Create a product for the Log4j product line",
        "value": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "name": "Apache Log4j 2",
            "description": "Java logging framework product line maintained by Apache.",
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core",
                }
            ],
            "vendor": {
                "name": "Apache Software Foundation",
                "url": "https://logging.apache.org/log4j/2.x/",
            },
            "homepageUrl": "https://logging.apache.org/log4j/2.x/",
            "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/",
            "vcsUrl": "https://github.com/apache/logging-log4j2",
        },
    },
    "product": {
        "summary": "Created product resource",
        "value": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "name": "Apache Log4j 2",
            "description": "Java logging framework product line maintained by Apache.",
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core",
                }
            ],
            "vendor": {
                "name": "Apache Software Foundation",
                "url": "https://logging.apache.org/log4j/2.x/",
            },
            "createdDate": "2026-03-28T15:00:00Z",
            "modifiedDate": "2026-03-28T15:00:00Z",
            "homepageUrl": "https://logging.apache.org/log4j/2.x/",
            "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/",
            "vcsUrl": "https://github.com/apache/logging-log4j2",
        },
    },
    "updateProductBody": {
        "summary": "Update selected product fields with an update mask",
        "value": {
            "updateMask": "description,documentationUrl",
            "description": "Java logging framework product line with maintained transparency metadata.",
            "documentationUrl": "https://logging.apache.org/log4j/2.x/manual/index.html",
        },
    },
    "deleteProductResponse": {
        "summary": "Deleted product with cascaded release cleanup",
        "value": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "releasesDeleted": 2,
        },
    },
    "createProductReleaseRequest": {
        "summary": "Create product release 2.24.3",
        "value": {
            "uuid": "22222222-2222-4222-8222-222222222222",
            "productUuid": "11111111-1111-4111-8111-111111111111",
            "version": "2.24.3",
            "releaseDate": "2026-03-01T00:00:00Z",
            "preRelease": False,
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_TEI",
                    "idValue": "urn:tei:uuid:cyclonedx.org:22222222-2222-4222-8222-222222222222",
                }
            ],
            "components": [
                {
                    "uuid": "33333333-3333-4333-8333-333333333333",
                    "release": "44444444-4444-4444-8444-444444444444",
                }
            ],
        },
    },
    "productRelease": {
        "summary": "Created product release resource",
        "value": {
            "uuid": "22222222-2222-4222-8222-222222222222",
            "product": "11111111-1111-4111-8111-111111111111",
            "version": "2.24.3",
            "createdDate": "2026-03-28T15:02:00Z",
            "releaseDate": "2026-03-01T00:00:00Z",
            "preRelease": False,
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_TEI",
                    "idValue": "urn:tei:uuid:cyclonedx.org:22222222-2222-4222-8222-222222222222",
                }
            ],
            "components": [
                {
                    "uuid": "33333333-3333-4333-8333-333333333333",
                    "release": "44444444-4444-4444-8444-444444444444",
                }
            ],
        },
    },
    "updateProductReleaseBody": {
        "summary": "Update selected product release fields",
        "value": {
            "updateMask": "releaseDate,preRelease",
            "releaseDate": "2026-03-02T00:00:00Z",
            "preRelease": False,
        },
    },
    "deleteProductReleaseResponse": {
        "summary": "Deleted product release",
        "value": {
            "uuid": "22222222-2222-4222-8222-222222222222",
        },
    },
    "createComponentRequest": {
        "summary": "Create a component lineage",
        "value": {
            "uuid": "33333333-3333-4333-8333-333333333333",
            "name": "Apache Log4j Core",
            "description": "Core logging library component.",
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core",
                }
            ],
            "componentType": "COMPONENT_TYPE_LIBRARY",
            "licenses": [{"spdxId": "Apache-2.0"}],
            "publisher": "Apache Software Foundation",
            "homepageUrl": "https://logging.apache.org/log4j/2.x/",
            "vcsUrl": "https://github.com/apache/logging-log4j2",
        },
    },
    "component": {
        "summary": "Created component resource",
        "value": {
            "uuid": "33333333-3333-4333-8333-333333333333",
            "name": "Apache Log4j Core",
            "description": "Core logging library component.",
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core",
                }
            ],
            "createdDate": "2026-03-28T15:03:00Z",
            "modifiedDate": "2026-03-28T15:03:00Z",
            "componentType": "COMPONENT_TYPE_LIBRARY",
            "licenses": [{"spdxId": "Apache-2.0"}],
            "publisher": "Apache Software Foundation",
            "homepageUrl": "https://logging.apache.org/log4j/2.x/",
            "vcsUrl": "https://github.com/apache/logging-log4j2",
        },
    },
    "updateComponentBody": {
        "summary": "Update a component description and homepage",
        "value": {
            "updateMask": "description,homepageUrl",
            "description": "Core logging library component with transparency metadata.",
            "homepageUrl": "https://logging.apache.org/log4j/2.x/components.html",
        },
    },
    "deleteComponentResponse": {
        "summary": "Deleted component with cascaded release cleanup",
        "value": {
            "uuid": "33333333-3333-4333-8333-333333333333",
            "releasesDeleted": 1,
        },
    },
    "createComponentReleaseRequest": {
        "summary": "Create component release 2.24.3",
        "value": {
            "uuid": "44444444-4444-4444-8444-444444444444",
            "componentUuid": "33333333-3333-4333-8333-333333333333",
            "version": "2.24.3",
            "releaseDate": "2026-03-01T00:00:00Z",
            "preRelease": False,
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                }
            ],
            "distributions": [
                {
                    "distributionType": "jar",
                    "description": "Maven Central JAR",
                    "url": "https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3.jar",
                    "checksums": [
                        {
                            "algType": "CHECKSUM_ALGORITHM_SHA256",
                            "algValue": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        }
                    ],
                    "sizeBytes": 2147483,
                    "mimeType": "application/java-archive",
                }
            ],
        },
    },
    "componentRelease": {
        "summary": "Created component release resource",
        "value": {
            "uuid": "44444444-4444-4444-8444-444444444444",
            "component": "33333333-3333-4333-8333-333333333333",
            "version": "2.24.3",
            "createdDate": "2026-03-28T15:04:00Z",
            "releaseDate": "2026-03-01T00:00:00Z",
            "preRelease": False,
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                }
            ],
            "distributions": [
                {
                    "distributionType": "jar",
                    "description": "Maven Central JAR",
                    "url": "https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.24.3/log4j-core-2.24.3.jar",
                    "checksums": [
                        {
                            "algType": "CHECKSUM_ALGORITHM_SHA256",
                            "algValue": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                        }
                    ],
                    "sizeBytes": 2147483,
                    "mimeType": "application/java-archive",
                }
            ],
        },
    },
    "updateComponentReleaseBody": {
        "summary": "Update selected component release fields",
        "value": {
            "updateMask": "releaseDate,identifiers",
            "releaseDate": "2026-03-02T00:00:00Z",
            "identifiers": [
                {
                    "idType": "IDENTIFIER_TYPE_PURL",
                    "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                }
            ],
        },
    },
    "deleteComponentReleaseResponse": {
        "summary": "Deleted component release",
        "value": {
            "uuid": "44444444-4444-4444-8444-444444444444",
        },
    },
    "uploadArtifactMetadataFrame": {
        "summary": "First upload frame containing artifact metadata",
        "value": {
            "metadata": {
                "uuid": "55555555-5555-4555-8555-555555555555",
                "name": "Log4j SBOM",
                "type": "ARTIFACT_TYPE_BOM",
                "mimeType": "application/vnd.cyclonedx+json",
                "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
                "subject": {
                    "type": "SUBJECT_TYPE_COMPONENT",
                    "name": "Apache Log4j Core",
                    "version": "2.24.3",
                    "identifiers": [
                        {
                            "idType": "IDENTIFIER_TYPE_PURL",
                            "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                        }
                    ],
                },
                "componentDistributions": ["jar"],
                "specVersion": "1.6",
                "expectedChecksums": [
                    {
                        "algType": "CHECKSUM_ALGORITHM_SHA256",
                        "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    }
                ],
            }
        },
    },
    "uploadArtifactContentFrame": {
        "summary": "Subsequent upload frame containing content bytes",
        "value": {
            "content": "ZXhhbXBsZS1hcnRpZmFjdC1ieXRlcy0x"
        },
    },
    "createArtifactFromUrlRequest": {
        "summary": "Register a CycloneDX SBOM from an immutable URL",
        "value": {
            "metadata": {
                "uuid": "55555555-5555-4555-8555-555555555555",
                "name": "Log4j SBOM",
                "type": "ARTIFACT_TYPE_BOM",
                "mimeType": "application/vnd.cyclonedx+json",
                "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
                "subject": {
                    "type": "SUBJECT_TYPE_COMPONENT",
                    "name": "Apache Log4j Core",
                    "version": "2.24.3",
                    "identifiers": [
                        {
                            "idType": "IDENTIFIER_TYPE_PURL",
                            "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                        }
                    ],
                },
                "componentDistributions": ["jar"],
                "specVersion": "1.6",
                "expectedChecksums": [
                    {
                        "algType": "CHECKSUM_ALGORITHM_SHA256",
                        "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                    }
                ],
            },
            "sourceUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
            "signatureUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json.sig",
            "expectedChecksums": [
                {
                    "algType": "CHECKSUM_ALGORITHM_SHA256",
                    "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                }
            ],
        },
    },
    "artifact": {
        "summary": "Registered immutable artifact resource",
        "value": {
            "uuid": "55555555-5555-4555-8555-555555555555",
            "name": "Log4j SBOM",
            "type": "ARTIFACT_TYPE_BOM",
            "componentDistributions": ["jar"],
            "formats": [
                {
                    "mimeType": "application/vnd.cyclonedx+json",
                    "description": "CycloneDX SBOM (JSON)",
                    "url": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
                    "signatureUrl": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json.sig",
                    "checksums": [
                        {
                            "algType": "CHECKSUM_ALGORITHM_SHA256",
                            "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                        }
                    ],
                    "sizeBytes": 20480,
                    "specVersion": "1.6",
                }
            ],
            "createdDate": "2026-03-28T15:05:00Z",
            "description": "CycloneDX SBOM for Log4j Core 2.24.3.",
            "subject": {
                "type": "SUBJECT_TYPE_COMPONENT",
                "name": "Apache Log4j Core",
                "version": "2.24.3",
                "identifiers": [
                    {
                        "idType": "IDENTIFIER_TYPE_PURL",
                        "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core@2.24.3",
                    }
                ],
            },
        },
    },
    "deleteArtifactResponse": {
        "summary": "Deleted artifact that was not referenced",
        "value": {
            "uuid": "55555555-5555-4555-8555-555555555555",
            "affectedCollectionUuids": [],
        },
    },
    "createCollectionRequest": {
        "summary": "Create version 1 of a component release collection",
        "value": {
            "uuid": "44444444-4444-4444-8444-444444444444",
            "belongsTo": "COLLECTION_SCOPE_RELEASE",
            "artifactUuids": ["55555555-5555-4555-8555-555555555555"],
            "updateReason": {
                "type": "UPDATE_REASON_TYPE_INITIAL_RELEASE",
                "comment": "Initial transparency artifact publication.",
                "affectedArtifactUuids": ["55555555-5555-4555-8555-555555555555"],
            },
        },
    },
    "updateCollectionBody": {
        "summary": "Create a new immutable collection version",
        "value": {
            "artifactUuids": [
                "55555555-5555-4555-8555-555555555555",
                "66666666-6666-4666-8666-666666666666"
            ],
            "updateReason": {
                "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
                "comment": "Add VEX alongside the original SBOM.",
                "affectedArtifactUuids": ["66666666-6666-4666-8666-666666666666"],
            },
        },
    },
    "collection": {
        "summary": "Versioned collection resource",
        "value": {
            "uuid": "44444444-4444-4444-8444-444444444444",
            "version": 2,
            "date": "2026-03-28T15:06:00Z",
            "belongsTo": "COLLECTION_SCOPE_RELEASE",
            "updateReason": {
                "type": "UPDATE_REASON_TYPE_ARTIFACT_ADDED",
                "comment": "Add VEX alongside the original SBOM.",
                "affectedArtifactUuids": ["66666666-6666-4666-8666-666666666666"],
            },
            "artifacts": [
                {
                    "uuid": "55555555-5555-4555-8555-555555555555",
                    "name": "Log4j SBOM",
                    "type": "ARTIFACT_TYPE_BOM",
                    "formats": [
                        {
                            "mimeType": "application/vnd.cyclonedx+json",
                            "url": "https://downloads.example.org/log4j/2.24.3/log4j-core-2.24.3.cdx.json",
                            "checksums": [
                                {
                                    "algType": "CHECKSUM_ALGORITHM_SHA256",
                                    "algValue": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                                }
                            ],
                        }
                    ],
                }
            ],
            "createdDate": "2026-03-28T15:06:00Z",
            "conformanceVectors": ["OWASP SCVS", "SLSA"],
        },
    },
    "signCollectionBody": {
        "summary": "Request signing of collection version 2",
        "value": {
            "version": 2,
            "keyId": "sigstore-prod-key",
            "useSigstore": True,
        },
    },
    "batchUploadArtifactsFrame": {
        "summary": "First batch upload frame with batch metadata",
        "value": {
            "batchMetadata": {
                "totalCount": 2,
                "collectionUuid": "44444444-4444-4444-8444-444444444444"
            }
        },
    },
    "importCollectionFrame": {
        "summary": "First import frame with session metadata",
        "value": {
            "importMetadata": {
                "sourceSystem": "legacy-tea",
                "totalCollections": 1,
                "totalArtifacts": 2,
                "overwrite": False
            }
        },
    },
    "errorResponse": {
        "summary": "Structured validation error",
        "value": {
            "code": "ERROR_CODE_INVALID_ARGUMENT",
            "message": "updateMask includes unsupported field 'fooBar'",
            "details": [
                {
                    "code": "ERROR_CODE_INVALID_ARGUMENT",
                    "message": "unsupported update mask path",
                    "field": "updateMask",
                    "metadata": {"path": "fooBar"}
                }
            ],
            "requestId": "req-1234abcd"
        },
    },
}

SCHEMA_EXAMPLE_MAP = {
    'CreateProductRequest': 'createProductRequest',
    'Product': 'product',
    'UpdateProductBody': 'updateProductBody',
    'DeleteProductResponse': 'deleteProductResponse',
    'CreateProductReleaseRequest': 'createProductReleaseRequest',
    'ProductRelease': 'productRelease',
    'UpdateProductReleaseBody': 'updateProductReleaseBody',
    'DeleteProductReleaseResponse': 'deleteProductReleaseResponse',
    'CreateComponentRequest': 'createComponentRequest',
    'Component': 'component',
    'UpdateComponentBody': 'updateComponentBody',
    'DeleteComponentResponse': 'deleteComponentResponse',
    'CreateComponentReleaseRequest': 'createComponentReleaseRequest',
    'ComponentRelease': 'componentRelease',
    'UpdateComponentReleaseBody': 'updateComponentReleaseBody',
    'DeleteComponentReleaseResponse': 'deleteComponentReleaseResponse',
    'UploadArtifactFrame': 'uploadArtifactMetadataFrame',
    'CreateArtifactFromUrlRequest': 'createArtifactFromUrlRequest',
    'Artifact': 'artifact',
    'DeleteArtifactResponse': 'deleteArtifactResponse',
    'CreateCollectionRequest': 'createCollectionRequest',
    'UpdateCollectionBody': 'updateCollectionBody',
    'Collection': 'collection',
    'SignCollectionBody': 'signCollectionBody',
    'BatchUploadArtifactsFrame': 'batchUploadArtifactsFrame',
    'ImportCollectionFrame': 'importCollectionFrame',
    'ErrorResponse': 'errorResponse',
}


schemas = {
    'Uuid': {'type': 'string', 'format': 'uuid'},
    'Timestamp': {'type': 'string', 'format': 'date-time'},
    'FieldMask': {
        'type': 'string',
        'description': 'Protobuf FieldMask JSON encoding: a comma-separated list of lowerCamel field paths.',
        'examples': ['name,description', 'releaseDate,preRelease'],
    },
    'Identifier': obj(
        {
            'idType': enum_string('Identifier enum name from tea.v1.IdentifierType.', ['IDENTIFIER_TYPE_PURL', 'IDENTIFIER_TYPE_TEI']),
            'idValue': {'type': 'string'},
        },
        ['idType', 'idValue'],
        description='Canonical identifier shape from proto/tea/v1/common.proto.',
    ),
    'Contact': obj(
        {
            'name': {'type': 'string'},
            'email': {'type': 'string', 'format': 'email'},
            'phone': {'type': 'string'},
        },
        description='Vendor contact details.',
    ),
    'Vendor': obj(
        {
            'name': {'type': 'string'},
            'uuid': ref('Uuid'),
            'url': {'type': 'string', 'format': 'uri'},
            'contacts': schema_array(ref('Contact')),
        },
        description='Product vendor or publisher metadata.',
    ),
    'LicenseInfo': obj(
        {
            'spdxId': {'type': 'string'},
            'name': {'type': 'string'},
            'url': {'type': 'string', 'format': 'uri'},
        },
        description='License metadata for a component.',
    ),
    'Checksum': obj(
        {
            'algType': enum_string('Checksum enum name from tea.v1.ChecksumAlgorithm.', ['CHECKSUM_ALGORITHM_SHA256']),
            'algValue': {'type': 'string', 'pattern': '^[a-f0-9]+$'},
        },
        ['algType', 'algValue'],
    ),
    'Distribution': obj(
        {
            'distributionType': {'type': 'string'},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'url': {'type': 'string', 'format': 'uri'},
            'signatureUrl': {'type': 'string', 'format': 'uri'},
            'checksums': schema_array(ref('Checksum')),
            'sizeBytes': {'type': 'integer'},
            'mimeType': {'type': 'string'},
        },
        description='A delivery-specific variant of a component release.',
    ),
    'Deprecation': {
        'type': 'object',
        'description': 'Canonical deprecation structure from proto/tea/v1/common.proto.',
        'additionalProperties': True,
    },
    'LifecycleStatus': {
        'type': 'object',
        'description': 'Lifecycle metadata from proto/tea/v1/product.proto.',
        'additionalProperties': True,
    },
    'ComponentRef': obj(
        {'uuid': ref('Uuid'), 'release': ref('Uuid')},
        ['uuid'],
        description='Reference to a component, optionally pinned to a specific component release.',
    ),
    'Product': obj(
        {
            'uuid': ref('Uuid'),
            'name': {'type': 'string'},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'vendor': ref('Vendor'),
            'createdDate': ref('Timestamp'),
            'modifiedDate': ref('Timestamp'),
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'documentationUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
            'deprecation': ref('Deprecation'),
        },
        ['uuid', 'name'],
        description='Canonical product resource defined in proto/tea/v1/product.proto.',
    ),
    'ProductRelease': obj(
        {
            'uuid': ref('Uuid'),
            'product': ref('Uuid'),
            'version': {'type': 'string'},
            'createdDate': ref('Timestamp'),
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'components': schema_array(ref('ComponentRef')),
            'lifecycleStatus': ref('LifecycleStatus'),
            'deprecation': ref('Deprecation'),
        },
        ['uuid', 'version'],
        description='Canonical product release resource.',
    ),
    'Component': obj(
        {
            'uuid': ref('Uuid'),
            'name': {'type': 'string'},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'createdDate': ref('Timestamp'),
            'modifiedDate': ref('Timestamp'),
            'componentType': enum_string('Enum name from tea.v1.ComponentType.', ['COMPONENT_TYPE_LIBRARY']),
            'licenses': schema_array(ref('LicenseInfo')),
            'publisher': {'type': 'string'},
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
            'deprecation': ref('Deprecation'),
        },
        ['uuid', 'name'],
        description='Canonical component resource defined in proto/tea/v1/component.proto.',
    ),
    'ComponentRelease': obj(
        {
            'uuid': ref('Uuid'),
            'component': ref('Uuid'),
            'version': {'type': 'string'},
            'createdDate': ref('Timestamp'),
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'distributions': schema_array(ref('Distribution')),
            'deprecation': ref('Deprecation'),
        },
        ['uuid', 'component', 'version'],
        description='Canonical component release resource.',
    ),
    'ArtifactFormat': obj(
        {
            'mimeType': {'type': 'string'},
            'description': {'type': 'string'},
            'url': {'type': 'string', 'format': 'uri'},
            'signatureUrl': {'type': 'string', 'format': 'uri'},
            'checksums': schema_array(ref('Checksum')),
            'sizeBytes': {'type': 'integer'},
            'encoding': {'type': 'string'},
            'specVersion': {'type': 'string'},
        },
        ['mimeType', 'url', 'checksums'],
    ),
    'ArtifactSubject': obj(
        {
            'type': enum_string('Enum name from tea.v1.SubjectType.', ['SUBJECT_TYPE_COMPONENT']),
            'identifiers': schema_array(ref('Identifier')),
            'name': {'type': 'string'},
            'version': {'type': 'string'},
        },
        description='What an artifact describes.',
    ),
    'ArtifactMetadata': obj(
        {
            'name': {'type': 'string'},
            'type': enum_string('Enum name from tea.v1.ArtifactType.', ['ARTIFACT_TYPE_BOM']),
            'mimeType': {'type': 'string'},
            'description': {'type': 'string'},
            'componentDistributions': schema_array({'type': 'string'}),
            'subject': ref('ArtifactSubject'),
            'specVersion': {'type': 'string'},
            'uuid': ref('Uuid'),
            'expectedChecksums': schema_array(ref('Checksum')),
        },
        ['name', 'type', 'mimeType'],
        description='Artifact metadata used for upload or URL-backed registration.',
    ),
    'Artifact': obj(
        {
            'uuid': ref('Uuid'),
            'name': {'type': 'string'},
            'type': enum_string('Enum name from tea.v1.ArtifactType.', ['ARTIFACT_TYPE_BOM']),
            'componentDistributions': schema_array({'type': 'string'}),
            'formats': schema_array(ref('ArtifactFormat'), min_items=1),
            'createdDate': ref('Timestamp'),
            'description': {'type': 'string'},
            'subject': ref('ArtifactSubject'),
            'deprecation': ref('Deprecation'),
        },
        ['uuid', 'name', 'type', 'formats'],
        description='Canonical artifact resource defined in proto/tea/v1/artifact.proto.',
    ),
    'UpdateReason': obj(
        {
            'type': enum_string('Enum name from tea.v1.UpdateReasonType.', ['UPDATE_REASON_TYPE_ARTIFACT_UPDATED']),
            'comment': {'type': 'string'},
            'affectedArtifactUuids': schema_array(ref('Uuid')),
        },
        description='Why a new collection version exists.',
    ),
    'SigstoreBundle': obj(
        {
            'rekorLogUrl': {'type': 'string', 'format': 'uri'},
            'rekorLogId': {'type': 'string'},
            'fulcioCertificate': {'type': 'string'},
            'timestampAuthorityResponse': {'type': 'string', 'contentEncoding': 'base64'},
        }
    ),
    'CollectionSignature': obj(
        {
            'algorithm': enum_string('Enum name from tea.v1.SignatureAlgorithm.', ['SIGNATURE_ALGORITHM_EDDSA']),
            'value': {'type': 'string'},
            'signedAt': ref('Timestamp'),
            'keyId': {'type': 'string'},
            'certificateChain': schema_array({'type': 'string'}),
            'sigstoreBundle': ref('SigstoreBundle'),
        },
        ['algorithm', 'value'],
    ),
    'Collection': obj(
        {
            'uuid': ref('Uuid'),
            'version': {'type': 'integer', 'minimum': 1},
            'date': ref('Timestamp'),
            'belongsTo': enum_string('Enum name from tea.v1.CollectionScope.', ['COLLECTION_SCOPE_RELEASE']),
            'updateReason': ref('UpdateReason'),
            'artifacts': schema_array(ref('Artifact')),
            'signature': ref('CollectionSignature'),
            'createdDate': ref('Timestamp'),
            'deprecation': ref('Deprecation'),
            'conformanceVectors': schema_array({'type': 'string'}),
        },
        ['uuid', 'version'],
        description='Canonical collection resource defined in proto/tea/v1/collection.proto.',
    ),
    'CreateProductRequest': obj(
        {
            'name': {'type': 'string', 'minLength': 1, 'maxLength': 512},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'vendor': ref('Vendor'),
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'documentationUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
            'uuid': ref('Uuid'),
        },
        ['name'],
    ),
    'UpdateProductBody': obj(
        {
            'updateMask': ref('FieldMask'),
            'name': {'type': 'string'},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'vendor': ref('Vendor'),
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'documentationUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
        },
        description='Body fields for UpdateProduct. The resource UUID is carried in the path and maps to the protobuf uuid field.',
    ),
    'DeleteProductResponse': obj({'uuid': ref('Uuid'), 'releasesDeleted': {'type': 'integer'}}, ['uuid']),
    'CreateProductReleaseRequest': obj(
        {
            'productUuid': ref('Uuid'),
            'version': {'type': 'string', 'minLength': 1},
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'components': schema_array(ref('ComponentRef')),
            'uuid': ref('Uuid'),
        },
        ['version'],
    ),
    'UpdateProductReleaseBody': obj(
        {
            'updateMask': ref('FieldMask'),
            'version': {'type': 'string'},
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'components': schema_array(ref('ComponentRef')),
        },
        description='Body fields for UpdateProductRelease. Unspecified fields must be preserved.',
    ),
    'DeleteProductReleaseResponse': obj({'uuid': ref('Uuid')}, ['uuid']),
    'CreateComponentRequest': obj(
        {
            'name': {'type': 'string', 'minLength': 1},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'componentType': enum_string('Enum name from tea.v1.ComponentType.'),
            'licenses': schema_array(ref('LicenseInfo')),
            'publisher': {'type': 'string'},
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
            'uuid': ref('Uuid'),
        },
        ['name'],
    ),
    'UpdateComponentBody': obj(
        {
            'updateMask': ref('FieldMask'),
            'name': {'type': 'string'},
            'description': {'type': 'string'},
            'identifiers': schema_array(ref('Identifier')),
            'componentType': enum_string('Enum name from tea.v1.ComponentType.'),
            'licenses': schema_array(ref('LicenseInfo')),
            'publisher': {'type': 'string'},
            'homepageUrl': {'type': 'string', 'format': 'uri'},
            'vcsUrl': {'type': 'string', 'format': 'uri'},
        },
        description='Body fields for UpdateComponent. The path UUID maps to the protobuf uuid field.',
    ),
    'DeleteComponentResponse': obj({'uuid': ref('Uuid'), 'releasesDeleted': {'type': 'integer'}}, ['uuid']),
    'CreateComponentReleaseRequest': obj(
        {
            'componentUuid': ref('Uuid'),
            'version': {'type': 'string', 'minLength': 1},
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'distributions': schema_array(ref('Distribution')),
            'uuid': ref('Uuid'),
        },
        ['componentUuid', 'version'],
    ),
    'UpdateComponentReleaseBody': obj(
        {
            'updateMask': ref('FieldMask'),
            'version': {'type': 'string'},
            'releaseDate': ref('Timestamp'),
            'preRelease': {'type': 'boolean'},
            'identifiers': schema_array(ref('Identifier')),
            'distributions': schema_array(ref('Distribution')),
        },
        description='Body fields for UpdateComponentRelease. Implementations must not allow preRelease to move from false back to true.',
    ),
    'DeleteComponentReleaseResponse': obj({'uuid': ref('Uuid')}, ['uuid']),
    'UploadArtifactFrame': {
        'oneOf': [
            obj({'metadata': ref('ArtifactMetadata')}, ['metadata'], description='First frame: artifact metadata.'),
            obj({'content': {'type': 'string', 'contentEncoding': 'base64'}}, ['content'], description='Subsequent frames: artifact content chunks encoded as base64.'),
        ],
        'description': 'Client-streaming upload frame. Metadata must be sent before any content chunks.',
    },
    'CreateArtifactFromUrlRequest': obj(
        {
            'metadata': ref('ArtifactMetadata'),
            'sourceUrl': {'type': 'string', 'format': 'uri'},
            'expectedChecksums': schema_array(ref('Checksum')),
            'signatureUrl': {'type': 'string', 'format': 'uri'},
        },
        ['metadata', 'sourceUrl'],
    ),
    'DeleteArtifactResponse': obj({'uuid': ref('Uuid'), 'affectedCollectionUuids': schema_array(ref('Uuid'))}, ['uuid']),
    'CreateCollectionRequest': obj(
        {
            'uuid': ref('Uuid'),
            'belongsTo': enum_string('Enum name from tea.v1.CollectionScope.', ['COLLECTION_SCOPE_RELEASE']),
            'artifactUuids': schema_array(ref('Uuid')),
            'updateReason': ref('UpdateReason'),
        },
        ['uuid', 'belongsTo'],
    ),
    'UpdateCollectionBody': obj(
        {'artifactUuids': schema_array(ref('Uuid')), 'updateReason': ref('UpdateReason')},
        ['updateReason'],
        description='Body fields for UpdateCollection. The path UUID maps to the protobuf uuid field and creates a new immutable collection version.',
    ),
    'SignCollectionBody': obj(
        {'version': {'type': 'integer', 'minimum': 1}, 'keyId': {'type': 'string'}, 'useSigstore': {'type': 'boolean'}},
        description='Signing parameters for one collection version.',
    ),
    'BatchArtifactMetadata': obj({'totalCount': {'type': 'integer', 'minimum': 1}, 'collectionUuid': ref('Uuid')}, ['totalCount']),
    'BatchUploadArtifactsFrame': {
        'oneOf': [
            obj({'batchMetadata': ref('BatchArtifactMetadata')}, ['batchMetadata'], description='First frame: batch metadata.'),
            obj({'artifact': ref('UploadArtifactFrame')}, ['artifact'], description='Subsequent frames: nested per-artifact frames.'),
        ],
        'description': 'Client-streaming batch upload frame.',
    },
    'BatchUploadError': obj({'index': {'type': 'integer'}, 'name': {'type': 'string'}, 'error': ref('ErrorDetail')}, ['index', 'error']),
    'BatchUploadArtifactsResponse': obj({'artifacts': schema_array(ref('Artifact')), 'errors': schema_array(ref('BatchUploadError'))}),
    'ImportMetadata': obj({'sourceSystem': {'type': 'string'}, 'totalCollections': {'type': 'integer'}, 'totalArtifacts': {'type': 'integer'}, 'overwrite': {'type': 'boolean'}}),
    'ArtifactFormatContent': obj({'mimeType': {'type': 'string'}, 'content': {'type': 'string', 'contentEncoding': 'base64'}}, ['mimeType', 'content']),
    'ArtifactWithContent': obj({'artifact': ref('Artifact'), 'formatContents': schema_array(ref('ArtifactFormatContent'))}, ['artifact', 'formatContents']),
    'ImportCollectionFrame': {
        'oneOf': [
            obj({'importMetadata': ref('ImportMetadata')}, ['importMetadata'], description='First frame: import session metadata.'),
            obj({'collection': ref('Collection')}, ['collection'], description='Collection payload frame.'),
            obj({'artifact': ref('ArtifactWithContent')}, ['artifact'], description='Artifact payload frame with embedded content.'),
        ],
        'description': 'Client-streaming import frame.',
    },
    'ImportError': obj({'entityType': {'type': 'string'}, 'uuid': ref('Uuid'), 'error': ref('ErrorDetail')}, ['entityType', 'error']),
    'ImportCollectionResponse': obj({'collectionsImported': {'type': 'integer'}, 'artifactsImported': {'type': 'integer'}, 'errors': schema_array(ref('ImportError'))}),
    'ErrorDetail': obj(
        {
            'code': enum_string('Enum name from tea.v1.ErrorCode.', ['ERROR_CODE_INVALID_ARGUMENT']),
            'message': {'type': 'string'},
            'field': {'type': 'string'},
            'metadata': {'type': 'object', 'additionalProperties': {'type': 'string'}},
        },
        ['code', 'message'],
    ),
    'ErrorResponse': obj(
        {
            'code': enum_string('Enum name from tea.v1.ErrorCode.', ['ERROR_CODE_INVALID_ARGUMENT']),
            'message': {'type': 'string'},
            'details': schema_array(ref('ErrorDetail')),
            'requestId': {'type': 'string'},
            'documentationUrl': {'type': 'string', 'format': 'uri'},
        },
        ['code', 'message'],
        description='Canonical error envelope from proto/tea/v1/common.proto.',
    ),
}

for schema_name, example_name in SCHEMA_EXAMPLE_MAP.items():
    schemas[schema_name]['example'] = EXAMPLES[example_name]['value']

responses = {
    'InvalidRequest': {
        'description': 'The request was malformed, failed validation, or violated a canonical publisher rule.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'invalidRequest': example_ref('errorResponse')},
            }
        },
    },
    'Unauthenticated': {
        'description': 'Authentication is required for every publisher operation.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'unauthenticated': example_ref('errorResponse')},
            }
        },
    },
    'NotFound': {
        'description': 'The addressed publisher resource does not exist.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'notFound': example_ref('errorResponse')},
            }
        },
    },
    'FailedPrecondition': {
        'description': 'The request was rejected because referential integrity or explicit cascade policy requirements were not met.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'failedPrecondition': example_ref('errorResponse')},
            }
        },
    },
    'InvalidContent': {
        'description': 'The request payload or fetched content failed checksum or semantic verification.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'invalidContent': example_ref('errorResponse')},
            }
        },
    },
    'UpstreamFetchFailed': {
        'description': 'The server could not safely fetch or verify the requested upstream artifact content.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'upstreamFetchFailed': example_ref('errorResponse')},
            }
        },
    },
    'UnsupportedCapability': {
        'description': 'This optional publisher capability remains intentionally unsupported in the current reference implementation and must fail explicitly.',
        'content': {
            'application/json': {
                'schema': ref('ErrorResponse'),
                'examples': {'unsupportedCapability': example_ref('errorResponse')},
            }
        },
    },
}

paths = {
    '/v1/publisher/products': {
        'post': op(
            'CreateProduct',
            'Create a product',
            'Create a product record. Client-supplied UUIDs may be preserved when provided.',
            'Products',
            'implemented',
            'Product',
            'The created product.',
            request='CreateProductRequest',
            errors=['InvalidRequest', 'Unauthenticated'],
            request_examples=examples_map(('log4jProduct', 'createProductRequest')),
            response_examples=examples_map(('createdProduct', 'product')),
        )
    },
    '/v1/publisher/products/{uuid}': {
        'put': op(
            'UpdateProduct',
            'Update a product',
            'Apply only the fields listed in updateMask and preserve all unspecified fields.',
            'Products',
            'implemented',
            'Product',
            'The updated product.',
            request='UpdateProductBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'NotFound'],
            request_examples=examples_map(('maskedUpdate', 'updateProductBody')),
            response_examples=examples_map(('updatedProduct', 'product')),
        ),
        'delete': op(
            'DeleteProduct',
            'Delete a product',
            'Delete a product. When cascade=false, implementations should reject deletion if dependent product releases still exist.',
            'Products',
            'implemented',
            'DeleteProductResponse',
            'Deletion result including cascade side effects.',
            path_params=['UuidPath'],
            query_params=['CascadeQuery'],
            errors=['Unauthenticated', 'NotFound', 'FailedPrecondition'],
            response_examples=examples_map(('deletedProduct', 'deleteProductResponse')),
        ),
    },
    '/v1/publisher/product-releases': {
        'post': op(
            'CreateProductRelease',
            'Create a product release',
            'Create a product release linked to an existing product when productUuid is supplied.',
            'Product Releases',
            'implemented',
            'ProductRelease',
            'The created product release.',
            request='CreateProductReleaseRequest',
            errors=['InvalidRequest', 'Unauthenticated', 'FailedPrecondition'],
            request_examples=examples_map(('productRelease', 'createProductReleaseRequest')),
            response_examples=examples_map(('createdProductRelease', 'productRelease')),
        )
    },
    '/v1/publisher/product-releases/{uuid}': {
        'put': op(
            'UpdateProductRelease',
            'Update a product release',
            'Apply only the fields listed in updateMask and preserve all unspecified fields.',
            'Product Releases',
            'implemented',
            'ProductRelease',
            'The updated product release.',
            request='UpdateProductReleaseBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'NotFound'],
            request_examples=examples_map(('maskedUpdate', 'updateProductReleaseBody')),
            response_examples=examples_map(('updatedProductRelease', 'productRelease')),
        ),
        'delete': op(
            'DeleteProductRelease',
            'Delete a product release',
            'Delete one product release.',
            'Product Releases',
            'implemented',
            'DeleteProductReleaseResponse',
            'Deletion result for the addressed product release.',
            path_params=['UuidPath'],
            errors=['Unauthenticated', 'NotFound'],
            response_examples=examples_map(('deletedProductRelease', 'deleteProductReleaseResponse')),
        ),
    },
    '/v1/publisher/components': {
        'post': op(
            'CreateComponent',
            'Create a component',
            'Create a component lineage record.',
            'Components',
            'implemented',
            'Component',
            'The created component.',
            request='CreateComponentRequest',
            errors=['InvalidRequest', 'Unauthenticated'],
            request_examples=examples_map(('log4jCore', 'createComponentRequest')),
            response_examples=examples_map(('createdComponent', 'component')),
        )
    },
    '/v1/publisher/components/{uuid}': {
        'put': op(
            'UpdateComponent',
            'Update a component',
            'Apply only the fields listed in updateMask and preserve all unspecified fields.',
            'Components',
            'implemented',
            'Component',
            'The updated component.',
            request='UpdateComponentBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'NotFound'],
            request_examples=examples_map(('maskedUpdate', 'updateComponentBody')),
            response_examples=examples_map(('updatedComponent', 'component')),
        ),
        'delete': op(
            'DeleteComponent',
            'Delete a component',
            'Delete a component. When cascade=false, implementations should reject deletion if dependent releases still exist.',
            'Components',
            'implemented',
            'DeleteComponentResponse',
            'Deletion result including any cascaded component release deletions.',
            path_params=['UuidPath'],
            query_params=['CascadeQuery'],
            errors=['Unauthenticated', 'NotFound', 'FailedPrecondition'],
            response_examples=examples_map(('deletedComponent', 'deleteComponentResponse')),
        ),
    },
    '/v1/publisher/component-releases': {
        'post': op(
            'CreateComponentRelease',
            'Create a component release',
            'Create a component release linked to an existing component.',
            'Component Releases',
            'implemented',
            'ComponentRelease',
            'The created component release.',
            request='CreateComponentReleaseRequest',
            errors=['InvalidRequest', 'Unauthenticated', 'FailedPrecondition'],
            request_examples=examples_map(('componentRelease', 'createComponentReleaseRequest')),
            response_examples=examples_map(('createdComponentRelease', 'componentRelease')),
        )
    },
    '/v1/publisher/component-releases/{uuid}': {
        'put': op(
            'UpdateComponentRelease',
            'Update a component release',
            'Apply only the fields listed in updateMask. Implementations must not allow preRelease to move from false back to true.',
            'Component Releases',
            'implemented',
            'ComponentRelease',
            'The updated component release.',
            request='UpdateComponentReleaseBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'NotFound'],
            request_examples=examples_map(('maskedUpdate', 'updateComponentReleaseBody')),
            response_examples=examples_map(('updatedComponentRelease', 'componentRelease')),
        ),
        'delete': op(
            'DeleteComponentRelease',
            'Delete a component release',
            'Delete one component release.',
            'Component Releases',
            'implemented',
            'DeleteComponentReleaseResponse',
            'Deletion result for the addressed component release.',
            path_params=['UuidPath'],
            errors=['Unauthenticated', 'NotFound'],
            response_examples=examples_map(('deletedComponentRelease', 'deleteComponentReleaseResponse')),
        ),
    },
    '/v1/publisher/artifacts': {
        'post': op(
            'UploadArtifact',
            'Upload an artifact',
            'Client-streaming artifact upload. The first frame carries metadata; subsequent frames carry content bytes. This remains intentionally unsupported in the current reference server.',
            'Artifacts',
            'intentionally_unimplemented',
            'Artifact',
            'The created immutable artifact.',
            request='UploadArtifactFrame',
            request_description='Documentation-only HTTP framing for the client-streaming UploadArtifact RPC.',
            request_content_type='application/x-ndjson',
            errors=['InvalidRequest', 'Unauthenticated', 'InvalidContent', 'UnsupportedCapability'],
            streaming=True,
            request_examples=examples_map(
                ('metadataFrame', 'uploadArtifactMetadataFrame'),
                ('contentFrame', 'uploadArtifactContentFrame'),
            ),
            response_examples=examples_map(('artifact', 'artifact')),
        )
    },
    '/v1/publisher/artifacts/from-url': {
        'post': op(
            'CreateArtifactFromUrl',
            'Register an artifact from a source URL',
            'Fetch immutable content from a source URL, verify declared checksums, and register the artifact metadata. Implementations may reject non-HTTPS or private-network URLs.',
            'Artifacts',
            'implemented',
            'Artifact',
            'The created immutable artifact.',
            request='CreateArtifactFromUrlRequest',
            errors=['InvalidRequest', 'Unauthenticated', 'InvalidContent', 'UpstreamFetchFailed'],
            request_examples=examples_map(('artifactFromUrl', 'createArtifactFromUrlRequest')),
            response_examples=examples_map(('createdArtifact', 'artifact')),
        )
    },
    '/v1/publisher/artifacts/{uuid}': {
        'delete': op(
            'DeleteArtifact',
            'Delete an artifact',
            'Delete an artifact only when it is not referenced, unless the implementation explicitly supports safe force-delete semantics.',
            'Artifacts',
            'implemented',
            'DeleteArtifactResponse',
            'Deletion result for the addressed artifact.',
            path_params=['UuidPath'],
            query_params=['ForceQuery'],
            errors=['Unauthenticated', 'NotFound', 'FailedPrecondition', 'UnsupportedCapability'],
            response_examples=examples_map(('deletedArtifact', 'deleteArtifactResponse')),
        )
    },
    '/v1/publisher/collections': {
        'post': op(
            'CreateCollection',
            'Create version 1 of a collection',
            'Create version 1 of a logical collection stream. All referenced artifacts must already exist.',
            'Collections',
            'implemented',
            'Collection',
            'The created collection version.',
            request='CreateCollectionRequest',
            errors=['InvalidRequest', 'Unauthenticated', 'FailedPrecondition'],
            request_examples=examples_map(('collectionV1', 'createCollectionRequest')),
            response_examples=examples_map(('collection', 'collection')),
        )
    },
    '/v1/publisher/collections/{uuid}/versions': {
        'post': op(
            'UpdateCollection',
            'Create a new immutable collection version',
            'Create the next immutable version of an existing collection. Prior versions remain addressable.',
            'Collections',
            'implemented',
            'Collection',
            'The new collection version.',
            request='UpdateCollectionBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'NotFound', 'FailedPrecondition'],
            request_examples=examples_map(('nextVersion', 'updateCollectionBody')),
            response_examples=examples_map(('collectionV2', 'collection')),
        )
    },
    '/v1/publisher/collections/{uuid}/sign': {
        'post': op(
            'SignCollection',
            'Sign a collection version',
            'Sign one collection version and return the updated metadata. This remains intentionally unsupported in the current reference server until a real signing and storage flow exists.',
            'Collections',
            'intentionally_unimplemented',
            'Collection',
            'The signed collection version.',
            request='SignCollectionBody',
            path_params=['UuidPath'],
            errors=['InvalidRequest', 'Unauthenticated', 'UnsupportedCapability'],
            request_examples=examples_map(('signVersion', 'signCollectionBody')),
            response_examples=examples_map(('collection', 'collection')),
        )
    },
    '/v1/publisher/artifacts/batch': {
        'post': op(
            'BatchUploadArtifacts',
            'Batch upload artifacts',
            'Advanced optional client-streaming batch upload capability. This remains intentionally unsupported in the current reference server.',
            'Artifacts',
            'intentionally_unimplemented',
            'BatchUploadArtifactsResponse',
            'Per-artifact batch upload results.',
            request='BatchUploadArtifactsFrame',
            request_description='Documentation-only HTTP framing for the client-streaming BatchUploadArtifacts RPC.',
            request_content_type='application/x-ndjson',
            errors=['InvalidRequest', 'Unauthenticated', 'UnsupportedCapability'],
            streaming=True,
            request_examples=examples_map(('batchMetadata', 'batchUploadArtifactsFrame')),
        )
    },
    '/v1/publisher/import': {
        'post': op(
            'ImportCollection',
            'Import collections and artifacts',
            'Advanced optional client-streaming migration capability for importing collections plus artifact data. This remains intentionally unsupported in the current reference server.',
            'Collections',
            'intentionally_unimplemented',
            'ImportCollectionResponse',
            'Import summary and per-entity failures.',
            request='ImportCollectionFrame',
            request_description='Documentation-only HTTP framing for the client-streaming ImportCollection RPC.',
            request_content_type='application/x-ndjson',
            errors=['InvalidRequest', 'Unauthenticated', 'UnsupportedCapability'],
            streaming=True,
            request_examples=examples_map(('importMetadata', 'importCollectionFrame')),
        )
    },
}

openapi = {
    '$schema': 'https://spec.openapis.org/oas/3.1/schema-base/2025-02-13',
    'openapi': '3.1.1',
    'jsonSchemaDialect': 'https://spec.openapis.org/oas/3.1/dialect/base',
    'info': {
        'title': 'TEA Publisher API Profile',
        'summary': 'Draft HTTP profile aligned with the canonical TEA publisher protobuf contract',
        'description': 'This OpenAPI profile mirrors the google.api.http annotations in proto/tea/v1/publisher.proto. The protobuf contract remains the canonical source of truth for publisher RPC semantics. Optional publisher capabilities that a given implementation does not support must fail explicitly rather than partially succeeding.',
        'version': '0.4.0-draft',
        'contact': {
            'name': 'TEA Working Group',
            'url': 'https://github.com/CycloneDX/transparency-exchange-api',
        },
        'license': {
            'name': 'Apache 2.0',
            'url': 'https://github.com/CycloneDX/transparency-exchange-api/blob/main/LICENSE',
        },
    },
    'servers': [
        {
            'url': 'http://localhost',
            'description': 'Local development base URL; publisher paths are rooted under /v1/publisher.',
        }
    ],
    'externalDocs': {
        'description': 'Canonical publisher protobuf contract',
        'url': '../proto/tea/v1/publisher.proto',
    },
    'security': [{'bearerAuth': []}],
    'tags': [
        {'name': 'Products', 'description': 'Publisher operations for product metadata and releases.'},
        {'name': 'Components', 'description': 'Publisher operations for component metadata.'},
        {'name': 'Product Releases', 'description': 'Publisher operations for product release lifecycle management.'},
        {'name': 'Component Releases', 'description': 'Publisher operations for component release lifecycle management.'},
        {'name': 'Artifacts', 'description': 'Publisher operations for immutable artifact registration and deletion.'},
        {'name': 'Collections', 'description': 'Publisher operations for versioned artifact collections.'},
    ],
    'x-tea-canonical-contract': 'proto/tea/v1/publisher.proto',
    'x-tea-reference-implementation': 'tea-server',
    'paths': paths,
    'components': {
        'securitySchemes': {
            'bearerAuth': {
                'type': 'http',
                'scheme': 'bearer',
                'bearerFormat': 'JWT',
                'description': 'All publisher operations require authentication; deployments define the exact JWT issuer, audience, and write scope vocabulary.',
            }
        },
        'parameters': {
            'UuidPath': {
                'name': 'uuid',
                'in': 'path',
                'required': True,
                'description': 'UUID of the addressed TEA resource.',
                'schema': ref('Uuid'),
            },
            'CascadeQuery': {
                'name': 'cascade',
                'in': 'query',
                'required': False,
                'description': 'Whether dependent releases should also be deleted.',
                'schema': {'type': 'boolean', 'default': False},
            },
            'ForceQuery': {
                'name': 'force',
                'in': 'query',
                'required': False,
                'description': 'Whether deletion should proceed even if the artifact is referenced. Implementations may reject this flag when safe cleanup is not supported.',
                'schema': {'type': 'boolean', 'default': False},
            },
        },
        'schemas': schemas,
        'responses': responses,
        'examples': EXAMPLES,
    },
}


def render_json():
    return json.dumps(openapi, indent=2) + '\n'


def main():
    parser = argparse.ArgumentParser(description='Generate the publisher OpenAPI profile from the checked-in reference model.')
    parser.add_argument('--check', action='store_true', help='fail if spec/publisher/openapi.json is not up to date')
    args = parser.parse_args()

    rendered = render_json()
    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else None
        if current != rendered:
            print('publisher OpenAPI profile is out of date; run tools/generate_publisher_openapi.py', file=sys.stderr)
            raise SystemExit(1)
        print('publisher OpenAPI generator check ok')
        return

    OUTPUT.write_text(rendered)
    print(f'wrote {OUTPUT.relative_to(ROOT)}')


if __name__ == '__main__':
    main()
