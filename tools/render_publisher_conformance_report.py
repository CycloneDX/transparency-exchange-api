#!/usr/bin/env python3
import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from html import escape
from pathlib import Path
from typing import Optional

ROOT = Path(__file__).resolve().parents[1]
CHECKLIST = ROOT / 'spec/publisher/conformance-checklist.json'
OPENAPI = ROOT / 'spec/publisher/openapi.json'


def load_json(path: Path) -> dict:
    if not path.exists():
        raise SystemExit(f'missing required file: {path}')
    return json.loads(path.read_text())


def load_openapi_ops(data: dict) -> dict[str, dict[str, str]]:
    ops: dict[str, dict[str, str]] = {}
    for path, methods in data.get('paths', {}).items():
        for method, operation in methods.items():
            if not isinstance(operation, dict):
                continue
            rpc = operation.get('x-tea-proto-rpc')
            if not rpc:
                continue
            ops[rpc] = {
                'method': method.upper(),
                'path': path,
                'status': operation.get('x-reference-status', 'unknown'),
            }
    return ops


def build_stats(checklist: dict, openapi: dict) -> dict:
    entries = checklist['entries']
    ops = load_openapi_ops(openapi)
    status_counts = Counter(entry['referenceStatus'] for entry in entries)
    coverage_counts = Counter(entry['coverage'] for entry in entries)
    linked_tests = sum(len(entry['tests']) for entry in entries)
    parity_ok = len(entries) == len(ops) and all(entry['rpc'] in ops for entry in entries)
    generated_at = datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    return {
        'entries': entries,
        'ops': ops,
        'status_counts': status_counts,
        'coverage_counts': coverage_counts,
        'linked_tests': linked_tests,
        'parity_ok': parity_ok,
        'generated_at': generated_at,
        'checklist': checklist,
    }


def render_report(stats: dict) -> str:
    entries = stats['entries']
    ops = stats['ops']
    status_counts = stats['status_counts']
    coverage_counts = stats['coverage_counts']
    linked_tests = stats['linked_tests']
    parity_ok = stats['parity_ok']
    generated_at = stats['generated_at']
    checklist = stats['checklist']

    lines = [
        '# Publisher Conformance Report',
        '',
        f'Generated: `{generated_at}`',
        '',
        '## Inputs',
        '',
        f'- Canonical contract: `{checklist["canonicalContract"]}`',
        f'- Reference implementation: `{checklist["referenceImplementation"]}`',
        f'- Checklist: `{CHECKLIST.relative_to(ROOT)}`',
        f'- OpenAPI profile: `{OPENAPI.relative_to(ROOT)}`',
        f'- Aggregate TEA HTTP profile: `spec/openapi.yaml`',
        f'- Generator: `tools/generate_publisher_openapi.py`',
        f'- Validators: `tools/validate_publisher_conformance.py`, `tools/validate_publisher_openapi.py`, `tools/sync_aggregate_openapi_publisher_block.py --check`',
        '',
        '## Summary',
        '',
        f'- RPCs tracked in checklist: `{len(entries)}`',
        f'- OpenAPI operations documented: `{len(ops)}`',
        f'- Implemented in reference server: `{status_counts.get("implemented", 0)}`',
        f'- Intentionally unimplemented: `{status_counts.get("intentionally_unimplemented", 0)}`',
        f'- Direct coverage entries: `{coverage_counts.get("direct", 0)}`',
        f'- Indirect coverage entries: `{coverage_counts.get("indirect", 0)}`',
        f'- Planned coverage entries: `{coverage_counts.get("planned", 0)}`',
        f'- Linked executable tests: `{linked_tests}`',
        f'- Checklist/OpenAPI parity: `{"ok" if parity_ok else "mismatch"}`',
        '',
        '## RPC Detail',
        '',
        '| RPC | Status | Coverage | HTTP | Linked tests |',
        '|-----|--------|----------|------|--------------|',
    ]

    for entry in entries:
        op = ops.get(entry['rpc'])
        http_cell = f"`{op['method']} {op['path']}`" if op else '`missing`'
        tests_cell = ', '.join(f'`{test}`' for test in entry['tests']) if entry['tests'] else 'none'
        lines.append(
            f"| `{entry['rpc']}` | `{entry['referenceStatus']}` | `{entry['coverage']}` | {http_cell} | {tests_cell} |"
        )

    lines.extend([
        '',
        '## Notes',
        '',
        '- This report summarizes contract metadata and parity state; it does not replace the executable publisher reference tests.',
        '- Optional publisher capabilities remain part of the canonical contract even when the current reference server intentionally returns `UNIMPLEMENTED`.',
    ])
    return '\n'.join(lines) + '\n'


def render_html_report(stats: dict) -> str:
    entries = stats['entries']
    ops = stats['ops']
    status_counts = stats['status_counts']
    coverage_counts = stats['coverage_counts']
    linked_tests = stats['linked_tests']
    parity_ok = stats['parity_ok']
    generated_at = stats['generated_at']
    checklist = stats['checklist']

    rows = []
    for entry in entries:
        op = ops.get(entry['rpc'])
        http_cell = f"{op['method']} {op['path']}" if op else 'missing'
        tests_cell = ', '.join(entry['tests']) if entry['tests'] else 'none'
        rows.append(
            '<tr>'
            f'<td><code>{escape(entry["rpc"])}</code></td>'
            f'<td><code>{escape(entry["referenceStatus"])}</code></td>'
            f'<td><code>{escape(entry["coverage"])}</code></td>'
            f'<td><code>{escape(http_cell)}</code></td>'
            f'<td>{escape(tests_cell)}</td>'
            '</tr>'
        )

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Publisher Conformance Report</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 2rem auto; max-width: 1100px; line-height: 1.5; color: #1f2937; }}
    code {{ background: #f3f4f6; padding: 0.1rem 0.3rem; border-radius: 4px; }}
    table {{ border-collapse: collapse; width: 100%; margin-top: 1rem; }}
    th, td {{ border: 1px solid #d1d5db; padding: 0.5rem; text-align: left; vertical-align: top; }}
    th {{ background: #f9fafb; }}
    .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 0.75rem; margin: 1rem 0 1.5rem; }}
    .card {{ border: 1px solid #d1d5db; border-radius: 8px; padding: 0.75rem 1rem; background: #ffffff; }}
  </style>
</head>
<body>
  <h1>Publisher Conformance Report</h1>
  <p>Generated: <code>{escape(generated_at)}</code></p>
  <div class="summary">
    <div class="card"><strong>Canonical contract</strong><br><code>{escape(checklist["canonicalContract"])}</code></div>
    <div class="card"><strong>Reference implementation</strong><br><code>{escape(checklist["referenceImplementation"])}</code></div>
    <div class="card"><strong>RPCs tracked</strong><br><code>{len(entries)}</code></div>
    <div class="card"><strong>OpenAPI operations</strong><br><code>{len(ops)}</code></div>
    <div class="card"><strong>Implemented</strong><br><code>{status_counts.get("implemented", 0)}</code></div>
    <div class="card"><strong>Intentionally unimplemented</strong><br><code>{status_counts.get("intentionally_unimplemented", 0)}</code></div>
    <div class="card"><strong>Direct coverage</strong><br><code>{coverage_counts.get("direct", 0)}</code></div>
    <div class="card"><strong>Indirect coverage</strong><br><code>{coverage_counts.get("indirect", 0)}</code></div>
    <div class="card"><strong>Planned coverage</strong><br><code>{coverage_counts.get("planned", 0)}</code></div>
    <div class="card"><strong>Linked tests</strong><br><code>{linked_tests}</code></div>
    <div class="card"><strong>Checklist/OpenAPI parity</strong><br><code>{"ok" if parity_ok else "mismatch"}</code></div>
  </div>
  <h2>RPC Detail</h2>
  <table>
    <thead>
      <tr>
        <th>RPC</th>
        <th>Status</th>
        <th>Coverage</th>
        <th>HTTP</th>
        <th>Linked tests</th>
      </tr>
    </thead>
    <tbody>
      {''.join(rows)}
    </tbody>
  </table>
  <h2>Notes</h2>
  <ul>
    <li>This report summarizes contract metadata and parity state; it does not replace the executable publisher reference tests.</li>
    <li>Optional publisher capabilities remain part of the canonical contract even when the current reference server intentionally returns <code>UNIMPLEMENTED</code>.</li>
  </ul>
</body>
</html>
"""


def render_summary(stats: dict) -> str:
    status_counts = stats['status_counts']
    coverage_counts = stats['coverage_counts']
    return '\n'.join([
        '## Publisher Conformance Summary',
        '',
        f"- Checklist/OpenAPI parity: `{'ok' if stats['parity_ok'] else 'mismatch'}`",
        f"- RPCs tracked: `{len(stats['entries'])}`",
        f"- Implemented: `{status_counts.get('implemented', 0)}`",
        f"- Intentionally unimplemented: `{status_counts.get('intentionally_unimplemented', 0)}`",
        f"- Direct coverage: `{coverage_counts.get('direct', 0)}`",
        f"- Indirect coverage: `{coverage_counts.get('indirect', 0)}`",
        f"- Planned coverage: `{coverage_counts.get('planned', 0)}`",
        f"- Linked executable tests: `{stats['linked_tests']}`",
        '',
    ]) + '\n'


def write_output(path_str: Optional[str], content: str) -> None:
    if not path_str:
        return
    output_path = Path(path_str)
    output_path.write_text(content)
    try:
        display_path = output_path.relative_to(ROOT)
    except ValueError:
        display_path = output_path
    print(f'wrote {display_path}')


def main() -> None:
    parser = argparse.ArgumentParser(description='Render a small publisher conformance/parity report')
    parser.add_argument('--output', required=True, help='path to the markdown file to write')
    parser.add_argument('--html-output', help='optional path to an HTML report')
    parser.add_argument('--summary-output', help='optional path to a short markdown summary')
    args = parser.parse_args()

    checklist = load_json(CHECKLIST)
    openapi = load_json(OPENAPI)
    stats = build_stats(checklist, openapi)
    write_output(args.output, render_report(stats))
    write_output(args.html_output, render_html_report(stats))
    write_output(args.summary_output, render_summary(stats))


if __name__ == '__main__':
    main()
