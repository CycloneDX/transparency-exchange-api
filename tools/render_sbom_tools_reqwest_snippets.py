#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT / 'spec/publisher/openapi.json'
OUTPUT = ROOT / 'docs/generated/sbom-tools-publisher-reqwest-snippets.md'

OPERATIONS = [
    {
        'title': 'CreateProduct',
        'function_name': 'create_product',
        'method': 'post',
        'path': '/v1/publisher/products',
        'response_label': 'Created product response',
    },
    {
        'title': 'CreateArtifactFromUrl',
        'function_name': 'create_artifact_from_url',
        'method': 'post',
        'path': '/v1/publisher/artifacts/from-url',
        'response_label': 'Registered artifact response',
    },
    {
        'title': 'CreateCollection',
        'function_name': 'create_collection',
        'method': 'post',
        'path': '/v1/publisher/collections',
        'response_label': 'Created collection response',
    },
    {
        'title': 'UpdateCollection',
        'function_name': 'create_next_collection_version',
        'method': 'post',
        'path': '/v1/publisher/collections/{uuid}/versions',
        'response_label': 'Next collection version response',
        'path_placeholder_note': 'Pass the logical collection UUID that should receive the next immutable version.',
    },
]

HEADER = '''# Generated sbom-tools Reqwest Publisher Snippets

This document is generated from `spec/publisher/openapi.json` by
`tools/render_sbom_tools_reqwest_snippets.py`.

These snippets target the draft canonical publisher HTTP profile under
`/v1/publisher/...`, which makes them a good fit when `sbom-tools` talks to a
transcoding gateway or another HTTP surface that follows the publisher protobuf
contract.

Important: these snippets are not the same thing as the current
`tea-server`-specific HTTP write surface described in
`docs/sbom-tools-integration.md`.

## Suggested Cargo dependencies

```toml
[dependencies]
anyhow = "1"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde_json = "1"
```

## Shared setup

```rust
use anyhow::Result;
use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};

fn publisher_client() -> Result<Client> {
    Ok(Client::builder().build()?)
}

fn parse_json(response: Response) -> Result<Value> {
    Ok(response.error_for_status()?.json()?)
}
```
'''


def fail(message: str) -> None:
    raise SystemExit(message)



def load_openapi() -> dict:
    if not OPENAPI.exists():
        fail(f'missing publisher OpenAPI profile {OPENAPI}')
    return json.loads(OPENAPI.read_text())



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



def render_json_block(value: object) -> str:
    return json.dumps(value, indent=4)



def render_url_line(path: str) -> str:
    if '{uuid}' in path:
        return '    let url = format!("{}/v1/publisher/collections/{}/versions", base_url.trim_end_matches(\'/\'), collection_uuid);'
    return f'    let url = format!("{{}}{path}", base_url.trim_end_matches(\'/\'));'



def render_signature(spec: dict) -> str:
    args = ['client: &Client', 'base_url: &str', 'token: &str']
    if '{uuid}' in spec['path']:
        args.append('collection_uuid: &str')
    return f"fn {spec['function_name']}({', '.join(args)}) -> Result<Value> {{"



def render_snippet(spec: dict, request_value: dict) -> str:
    lines = [
        '```rust',
        render_signature(spec),
        render_url_line(spec['path']),
        '    let response = client',
        f"        .{spec['method']}(url)",
        '        .bearer_auth(token)',
        '        .json(&json!(',
        render_json_block(request_value),
        '        ))',
        '        .send()?;',
        '',
        '    parse_json(response)',
        '}',
        '```',
    ]
    return '\n'.join(lines)



def render_section(spec: dict, components: dict) -> str:
    operation = spec['operation']
    request_name, request_value = request_example(operation, components)
    response_name, response_value = response_example(operation, components)

    lines = [
        f"## `{spec['title']}`",
        '',
        f"- HTTP: `{spec['method'].upper()} {spec['path']}`",
        f"- Example request: `{request_name}`",
        f"- Example response: `{response_name}`",
    ]
    if spec.get('path_placeholder_note'):
        lines.append(f"- Note: {spec['path_placeholder_note']}")
    lines.extend([
        '',
        'Generated Rust snippet:',
        '',
        render_snippet(spec, request_value),
        '',
        f"{spec['response_label']}:",
        '',
        '```json',
        json.dumps(response_value, indent=2),
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
    parser = argparse.ArgumentParser(description='Render generated sbom-tools reqwest publisher snippets from spec/publisher/openapi.json')
    parser.add_argument('--check', action='store_true', help='fail if the generated markdown is out of date')
    args = parser.parse_args()

    rendered = render_markdown(load_openapi())
    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else None
        if current != rendered:
            print('generated sbom-tools reqwest snippets are out of date; run tools/render_sbom_tools_reqwest_snippets.py', file=sys.stderr)
            raise SystemExit(1)
        print('sbom-tools reqwest snippets check ok')
        return

    OUTPUT.write_text(rendered)
    print(f'wrote {OUTPUT.relative_to(ROOT)}')


if __name__ == '__main__':
    import sys
    main()
