#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path

REQUIRED_FILES = {
    'index.html',
    'publisher-conformance-report.md',
    'publisher-conformance-report.html',
    'publisher-conformance-summary.md',
    'publisher-conformance-matrix.md',
    'sbom-tools-integration.md',
    'sbom-tools-publisher-profile-examples.md',
    'sbom-tools-publisher-reqwest-snippets.md',
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for chunk in iter(lambda: handle.read(65536), b''):
            digest.update(chunk)
    return digest.hexdigest()



def fail(message: str) -> None:
    raise SystemExit(f'publisher release-doc bundle validation failed: {message}')



def load_manifest(bundle_dir: Path) -> dict:
    manifest_path = bundle_dir / 'manifest.json'
    if not manifest_path.exists():
        fail(f'missing manifest {manifest_path}')
    return json.loads(manifest_path.read_text())



def validate_checksum_file(path: Path, expected_sha: str, expected_name: str) -> None:
    if not path.exists():
        fail(f'missing checksum file {path}')
    line = path.read_text().strip()
    want = f'{expected_sha}  {expected_name}'
    if line != want:
        fail(f'checksum file {path} expected {want!r} but found {line!r}')



def validate_manifest(bundle_dir: Path, manifest: dict) -> None:
    for field in (
        'schemaVersion',
        'bundleName',
        'bundleVersion',
        'generatedAt',
        'gitRevision',
        'manifestPath',
        'canonicalContract',
        'publisherProfile',
        'conformanceChecklist',
        'distribution',
        'files',
    ):
        if field not in manifest:
            fail(f'manifest missing field {field}')

    if manifest['schemaVersion'] != 1:
        fail('schemaVersion must be 1')
    if manifest['bundleName'] != 'publisher-release-doc-bundle':
        fail('bundleName must be publisher-release-doc-bundle')
    if manifest['manifestPath'] != 'manifest.json':
        fail('manifestPath must be manifest.json')

    files = manifest['files']
    if not isinstance(files, list) or not files:
        fail('manifest files must be a non-empty list')

    seen = set()
    for entry in files:
        for field in ('path', 'description', 'sizeBytes', 'sha256'):
            if field not in entry:
                fail(f'file entry missing field {field}')
        rel = entry['path']
        seen.add(rel)
        path = bundle_dir / rel
        if not path.exists():
            fail(f'manifest entry points at missing file {path}')
        if path.stat().st_size != entry['sizeBytes']:
            fail(f'size mismatch for {rel}')
        if sha256_file(path) != entry['sha256']:
            fail(f'sha256 mismatch for {rel}')

    missing = REQUIRED_FILES - seen
    if missing:
        fail(f'manifest is missing required files: {sorted(missing)}')



def validate_distribution(bundle_root: Path, manifest: dict) -> None:
    distribution = manifest['distribution']
    for field in (
        'directory',
        'archive',
        'archiveChecksumFile',
        'archiveSha256',
        'archiveSizeBytes',
        'checksumFileSha256',
    ):
        if field not in distribution:
            fail(f'distribution missing field {field}')

    archive_path = bundle_root / distribution['archive']
    checksum_path = bundle_root / distribution['archiveChecksumFile']
    if not archive_path.exists():
        fail(f'missing archive {archive_path}')
    if archive_path.stat().st_size != distribution['archiveSizeBytes']:
        fail(f'archive size mismatch for {archive_path.name}')
    archive_sha = sha256_file(archive_path)
    if archive_sha != distribution['archiveSha256']:
        fail(f'archive sha256 mismatch for {archive_path.name}')
    checksum_sha = sha256_file(checksum_path)
    if checksum_sha != distribution['checksumFileSha256']:
        fail(f'checksum file sha256 mismatch for {checksum_path.name}')
    validate_checksum_file(checksum_path, archive_sha, archive_path.name)



def main() -> None:
    parser = argparse.ArgumentParser(description='Validate the publish-friendly publisher release-doc bundle')
    parser.add_argument('--bundle-dir', required=True, help='path to the unpacked bundle directory')
    args = parser.parse_args()

    bundle_dir = Path(args.bundle_dir)
    if not bundle_dir.exists():
        fail(f'bundle directory does not exist: {bundle_dir}')
    manifest = load_manifest(bundle_dir)
    validate_manifest(bundle_dir, manifest)
    validate_distribution(bundle_dir.parent, manifest)
    print(
        'publisher release-doc bundle ok: '
        f"{manifest['bundleName']} {manifest['bundleVersion']} with {len(manifest['files'])} tracked files"
    )


if __name__ == '__main__':
    main()
