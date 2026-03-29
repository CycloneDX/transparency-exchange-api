#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTO = ROOT / 'proto/tea/v1/publisher.proto'
CHECKLIST = ROOT / 'spec/publisher/conformance-checklist.json'
OPENAPI = ROOT / 'spec/publisher/openapi.json'
AGGREGATE_OPENAPI = ROOT / 'spec/openapi.yaml'
ALLOWED_REFERENCE_STATUS = {'implemented', 'intentionally_unimplemented'}
PUBLISHER_METHODS = {'post', 'put', 'delete'}
RPC_RE = re.compile(r'^\s*rpc\s+(\w+)\((stream\s+)?\w+\)\s+returns\s+\(\w+\)\s*\{$')
HTTP_RE = re.compile(r'\b(post|put|delete):\s+"([^"]+)"')


def fail(message: str) -> None:
    print(f'publisher OpenAPI validation failed: {message}', file=sys.stderr)
    raise SystemExit(1)


def load_proto_rpcs() -> dict[str, dict[str, object]]:
    if not PROTO.exists():
        fail(f'missing protobuf contract {PROTO}')

    rpcs: dict[str, dict[str, object]] = {}
    current_rpc = None
    current_streaming = False

    for line in PROTO.read_text().splitlines():
        rpc_match = RPC_RE.match(line)
        if rpc_match:
            current_rpc = rpc_match.group(1)
            current_streaming = bool(rpc_match.group(2))
            continue

        if current_rpc:
            http_match = HTTP_RE.search(line)
            if http_match:
                method = http_match.group(1)
                path = http_match.group(2)
                rpcs[current_rpc] = {
                    'method': method,
                    'path': path,
                    'streaming': current_streaming,
                }
                current_rpc = None
                current_streaming = False

    if not rpcs:
        fail('no publisher RPCs found in protobuf contract')
    return rpcs


def load_checklist_statuses() -> dict[str, str]:
    if not CHECKLIST.exists():
        fail(f'missing checklist {CHECKLIST}')
    data = json.loads(CHECKLIST.read_text())
    entries = data.get('entries')
    if not isinstance(entries, list) or not entries:
        fail('publisher conformance checklist must contain entries')
    statuses: dict[str, str] = {}
    for entry in entries:
        rpc = entry.get('rpc')
        status = entry.get('referenceStatus')
        if not rpc or not status:
            fail('every checklist entry must include rpc and referenceStatus')
        statuses[rpc] = status
    return statuses


def main() -> None:
    proto_rpcs = load_proto_rpcs()
    checklist_statuses = load_checklist_statuses()

    if not OPENAPI.exists():
        fail(f'missing OpenAPI profile {OPENAPI}')
    data = json.loads(OPENAPI.read_text())

    if data.get('openapi') != '3.1.1':
        fail('openapi version must be 3.1.1')
    if data.get('info', {}).get('title') != 'TEA Publisher API Profile':
        fail('info.title must be TEA Publisher API Profile')
    if data.get('x-tea-canonical-contract') != 'proto/tea/v1/publisher.proto':
        fail('x-tea-canonical-contract must point at proto/tea/v1/publisher.proto')
    if data.get('x-tea-reference-implementation') != 'tea-server':
        fail('x-tea-reference-implementation must be tea-server')

    if not AGGREGATE_OPENAPI.exists():
        fail(f'missing aggregate OpenAPI document {AGGREGATE_OPENAPI}')
    aggregate_text = AGGREGATE_OPENAPI.read_text()
    for marker in (
        'url: ./publisher/openapi.json',
        'x-tea-publisher-profile:',
        'fragment: spec/generated/publisher-profile-fragment.yaml',
        'generatedFrom: tools/generate_publisher_openapi.py',
        'fragmentGenerator: tools/render_aggregate_openapi_publisher_fragment.py',
        'validator: tools/validate_publisher_openapi.py',
        'syncTool: tools/sync_aggregate_openapi_publisher_block.py',
        'canonicalContract: proto/tea/v1/publisher.proto',
        'reqwestSnippetDoc: docs/generated/sbom-tools-publisher-reqwest-snippets.md',
        'x-tea-publisher-summary:',
        'x-tea-publisher-rpc-index:',
    ):
        if marker not in aggregate_text:
            fail(f'spec/openapi.yaml is missing publisher profile marker: {marker}')

    security_schemes = data.get('components', {}).get('securitySchemes', {})
    if 'bearerAuth' not in security_schemes:
        fail('components.securitySchemes.bearerAuth is required')

    root_security = data.get('security', [])
    has_root_bearer = any('bearerAuth' in item for item in root_security if isinstance(item, dict))

    seen_rpcs: set[str] = set()
    op_count = 0

    for path, methods in data.get('paths', {}).items():
        if not isinstance(methods, dict):
            fail(f'path item for {path} must be an object')
        for method, operation in methods.items():
            if method not in PUBLISHER_METHODS:
                continue
            op_count += 1
            if not isinstance(operation, dict):
                fail(f'operation {method.upper()} {path} must be an object')

            rpc = operation.get('x-tea-proto-rpc')
            if not rpc:
                fail(f'operation {method.upper()} {path} is missing x-tea-proto-rpc')
            if rpc in seen_rpcs:
                fail(f'RPC {rpc} is described more than once in the OpenAPI profile')
            seen_rpcs.add(rpc)

            if rpc not in proto_rpcs:
                fail(f'OpenAPI operation {method.upper()} {path} references unknown RPC {rpc}')
            proto_spec = proto_rpcs[rpc]
            if method != proto_spec['method']:
                fail(f'RPC {rpc} uses HTTP {method.upper()} in OpenAPI but {proto_spec["method"].upper()} in protobuf')
            if path != proto_spec['path']:
                fail(f'RPC {rpc} uses path {path!r} in OpenAPI but {proto_spec["path"]!r} in protobuf')
            if operation.get('operationId') != rpc:
                fail(f'RPC {rpc} must use operationId {rpc!r}')
            if operation.get('x-canonical-contract') != 'proto/tea/v1/publisher.proto':
                fail(f'RPC {rpc} must set x-canonical-contract to proto/tea/v1/publisher.proto')

            reference_status = operation.get('x-reference-status')
            if reference_status not in ALLOWED_REFERENCE_STATUS:
                fail(f'RPC {rpc} has invalid x-reference-status {reference_status!r}')
            checklist_status = checklist_statuses.get(rpc)
            if checklist_status is None:
                fail(f'RPC {rpc} is missing from spec/publisher/conformance-checklist.json')
            if reference_status != checklist_status:
                fail(
                    f'RPC {rpc} has x-reference-status {reference_status!r} but '
                    f'checklist says {checklist_status!r}'
                )

            op_security = operation.get('security', [])
            has_op_bearer = any('bearerAuth' in item for item in op_security if isinstance(item, dict))
            if not (has_root_bearer or has_op_bearer):
                fail(f'RPC {rpc} must require bearerAuth at the operation or root level')

            is_streaming = bool(proto_spec['streaming'])
            has_streaming_marker = operation.get('x-tea-streaming') == 'client'
            if is_streaming and not has_streaming_marker:
                fail(f'RPC {rpc} is client-streaming in protobuf and must set x-tea-streaming=client')
            if not is_streaming and 'x-tea-streaming' in operation:
                fail(f'RPC {rpc} is unary in protobuf and must not set x-tea-streaming')

    missing = set(proto_rpcs) - seen_rpcs
    if missing:
        fail(f'OpenAPI profile is missing RPCs: {sorted(missing)}')

    print(
        f'publisher OpenAPI profile ok: {len(proto_rpcs)} RPCs, '
        f'{op_count} documented HTTP operations, checklist parity preserved'
    )


if __name__ == '__main__':
    main()
