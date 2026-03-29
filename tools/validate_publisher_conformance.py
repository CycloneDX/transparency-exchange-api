#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKLIST = ROOT / 'spec/publisher/conformance-checklist.json'
TEST_FILES = {
    'grpc_smoke': ROOT / 'tea-server/tests/grpc_smoke.rs',
    'publisher_conformance': ROOT / 'tea-server/tests/publisher_conformance.rs',
    'publisher_capability_coverage': ROOT / 'tea-server/tests/publisher_capability_coverage.rs',
}
ALLOWED_REFERENCE_STATUS = {'implemented', 'intentionally_unimplemented'}
ALLOWED_COVERAGE = {'direct', 'indirect', 'planned'}
TEST_RE = re.compile(r'#\[(?:tokio::)?test\]\s*(?:async\s+)?fn\s+([a-zA-Z0-9_]+)', re.MULTILINE)


def fail(message: str) -> None:
    print(f'publisher conformance validation failed: {message}', file=sys.stderr)
    raise SystemExit(1)


def load_tests() -> dict[str, set[str]]:
    discovered: dict[str, set[str]] = {}
    for module, path in TEST_FILES.items():
        if not path.exists():
            fail(f'missing expected test file {path}')
        discovered[module] = set(TEST_RE.findall(path.read_text()))
    return discovered


def main() -> None:
    if not CHECKLIST.exists():
        fail(f'missing checklist file {CHECKLIST}')

    data = json.loads(CHECKLIST.read_text())
    if data.get('version') != 1:
        fail('checklist version must be 1')
    if data.get('canonicalContract') != 'proto/tea/v1/publisher.proto':
        fail('canonicalContract must point at proto/tea/v1/publisher.proto')
    if data.get('referenceImplementation') != 'tea-server':
        fail('referenceImplementation must be tea-server')

    entries = data.get('entries')
    if not isinstance(entries, list) or not entries:
        fail('entries must be a non-empty list')

    seen_ids = set()
    tests_by_module = load_tests()

    for entry in entries:
        entry_id = entry.get('id')
        rpc = entry.get('rpc')
        reference_status = entry.get('referenceStatus')
        coverage = entry.get('coverage')
        tests = entry.get('tests')

        if not entry_id or entry_id in seen_ids:
            fail(f'entry id missing or duplicated: {entry_id!r}')
        seen_ids.add(entry_id)

        if not rpc:
            fail(f'entry {entry_id} is missing rpc')
        if reference_status not in ALLOWED_REFERENCE_STATUS:
            fail(f'entry {entry_id} has invalid referenceStatus {reference_status!r}')
        if coverage not in ALLOWED_COVERAGE:
            fail(f'entry {entry_id} has invalid coverage {coverage!r}')
        if not isinstance(tests, list):
            fail(f'entry {entry_id} tests must be a list')
        if coverage in {'direct', 'indirect'} and not tests:
            fail(f'entry {entry_id} has {coverage} coverage but no tests listed')

        for test_ref in tests:
            if '::' not in test_ref:
                fail(f'entry {entry_id} has malformed test reference {test_ref!r}')
            module, test_name = test_ref.split('::', 1)
            if module not in tests_by_module:
                fail(f'entry {entry_id} references unknown test module {module!r}')
            if test_name not in tests_by_module[module]:
                fail(f'entry {entry_id} references missing test {test_ref!r}')

    print(
        f'publisher conformance checklist ok: {len(entries)} entries, '
        f"{sum(len(e['tests']) for e in entries)} linked test references"
    )


if __name__ == '__main__':
    main()
