#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / 'spec/publisher/openapi.json'
OUTPUT = ROOT / 'docs/generated/sbom-tools-publisher-profile-examples.md'

OPERATIONS = [
    {
        'title': 'CreateProduct',
        'method': 'post',
        'path': '/v1/publisher/products',
        'base_url_var': '${TEA_PROFILE_BASE_URL}',
        'response_label': 'Created product response',
    },
    {
        'title': 'CreateArtifactFromUrl',
        'method': 'post',
        'path': '/v1/publisher/artifacts/from-url',
        'base_url_var': '${TEA_PROFILE_BASE_URL}',
        'response_label': 'Registered artifact response',
    },
    {
        'title': 'CreateCollection',
        'method': 'post',
        'path': '/v1/publisher/collections',
        'base_url_var': '${TEA_PROFILE_BASE_URL}',
        'response_label': 'Created collection response',
    },
    {
        'title': 'UpdateCollection',
        'method': 'post',
        'path': '/v1/publisher/collections/{uuid}/versions',
        'base_url_var': '${TEA_PROFILE_BASE_URL}',
        'path_placeholder_note': 'Replace `${COLLECTION_UUID}` with the logical collection UUID.',
        'response_label': 'Next collection version response',
    },
]

HEADER = '''# Generated sbom-tools Publisher Profile Examples

This document is generated from `spec/publisher/openapi.json` by
`tools/render_sbom_tools_publisher_examples.py`.

These examples target the draft canonical publisher HTTP profile under
`/v1/publisher/...`. They are useful when `sbom-tools` integrates through a
transcoding gateway or another HTTP layer that follows the publisher protobuf
contract.

Important: these examples are not the same thing as the current
`tea-server`-specific HTTP write surface described in `docs/sbom-tools-integration.md`.
Use this generated document when you want the canonical publisher payload
shapes, not the reference server's bespoke HTTP handlers.

## Environment

```bash
export TEA_PROFILE_BASE_URL=http://localhost:8080
export TEA_TOKEN=replace-with-a-real-writer-token
export COLLECTION_UUID=44444444-4444-4444-8444-444444444444
```
'''


def fail(message: str) -> None:
    raise SystemExit(message)


def load_openapi() -> dict:
    if not OPENAPI.exists():
        fail(f'missing publisher OpenAPI profile {OPENAPI}')
    return json.loads(OPENAPI.read_text())


def render_json_block(value: object) -> str:
    return json.dumps(value, indent=2)


def request_example(operation: dict, components: dict) -> tuple[str, dict]:
    content = operation['requestBody']['content']
    media_type = next(iter(content.values()))
    examples = media_type.get('examples', {})
    name, ref_obj = next(iter(examples.items()))
    example_name = ref_obj['$ref'].split('/')[-1]
    return name, components['examples'][example_name]['value']


def response_example(operation: dict, components: dict) -> tuple[str, dict]:
    content = operation['responses']['200']['content']
    media_type = next(iter(content.values()))
    examples = media_type.get('examples', {})
    name, ref_obj = next(iter(examples.items()))
    example_name = ref_obj['$ref'].split('/')[-1]
    return name, components['examples'][example_name]['value']


def curl_path(path: str) -> str:
    return path.replace('{uuid}', '${COLLECTION_UUID}')


def render_section(spec: dict, components: dict) -> str:
    operation = spec['operation']
    request_name, request_value = request_example(operation, components)
    response_name, response_value = response_example(operation, components)
    path = spec['path']
    if '{uuid}' in path:
        path = curl_path(path)

    lines = [
        f"## `{spec['title']}`",
        '',
        f"- HTTP: `{operation['method'].upper() if 'method' in operation else spec['method'].upper()} {spec['path']}`",
        f"- Example request: `{request_name}`",
        f"- Example response: `{response_name}`",
    ]
    if spec.get('path_placeholder_note'):
        lines.append(f"- Note: {spec['path_placeholder_note']}")
    lines.extend([
        '',
        'Request payload:',
        '',
        '```json',
        render_json_block(request_value),
        '```',
        '',
        'Example `curl` call:',
        '',
        '```bash',
        'curl -sS \\',
        '  -H "Authorization: Bearer ${TEA_TOKEN}" \\',
        '  -H "Content-Type: application/json" \\',
        f"  -X {spec['method'].upper()} {spec['base_url_var']}{path} \\",
        "  -d @<(cat <<'JSON'",
        render_json_block(request_value),
        'JSON',
        ')',
        '```',
        '',
        f"{spec['response_label']}:",
        '',
        '```json',
        render_json_block(response_value),
        '```',
        '',
    ])
    return '\n'.join(lines)


def render_markdown(openapi: dict) -> str:
    components = openapi['components']
    sections = [HEADER.strip(), '']
    for spec in OPERATIONS:
        operation = openapi['paths'][spec['path']][spec['method']]
        section_spec = dict(spec)
        section_spec['operation'] = operation
        sections.append(render_section(section_spec, components))
    return '\n'.join(sections).rstrip() + '\n'


def main() -> None:
    parser = argparse.ArgumentParser(description='Render generated sbom-tools publisher-profile examples from spec/publisher/openapi.json')
    parser.add_argument('--check', action='store_true', help='fail if the generated markdown is out of date')
    args = parser.parse_args()

    rendered = render_markdown(load_openapi())
    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else None
        if current != rendered:
            print('generated sbom-tools publisher examples are out of date; run tools/render_sbom_tools_publisher_examples.py', file=sys.stderr)
            raise SystemExit(1)
        print('sbom-tools publisher examples check ok')
        return

    OUTPUT.write_text(rendered)
    print(f'wrote {OUTPUT.relative_to(ROOT)}')


if __name__ == '__main__':
    import sys
    main()
