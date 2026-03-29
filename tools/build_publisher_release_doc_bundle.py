#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tarfile
from datetime import datetime, timezone
from pathlib import Path

from render_publisher_conformance_report import (
    CHECKLIST,
    OPENAPI,
    build_stats,
    load_json,
    render_html_report,
    render_report,
    render_summary,
)

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT = ROOT / 'dist/publisher-release-doc-bundle'
SOURCE_DOCS = [
    (
        ROOT / 'docs/generated/sbom-tools-publisher-profile-examples.md',
        'sbom-tools-publisher-profile-examples.md',
        'Canonical publisher-profile JSON/curl examples for sbom-tools.',
    ),
    (
        ROOT / 'docs/generated/sbom-tools-publisher-reqwest-snippets.md',
        'sbom-tools-publisher-reqwest-snippets.md',
        'Canonical publisher-profile Rust/reqwest snippets for sbom-tools.',
    ),
    (
        ROOT / 'docs/sbom-tools-integration.md',
        'sbom-tools-integration.md',
        'Reference-server integration guide for sbom-tools.',
    ),
    (
        ROOT / 'spec/publisher/conformance-matrix.md',
        'publisher-conformance-matrix.md',
        'Canonical-vs-reference publisher capability matrix.',
    ),
]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for chunk in iter(lambda: handle.read(65536), b''):
            digest.update(chunk)
    return digest.hexdigest()



def git_revision() -> str:
    if os.environ.get('GITHUB_SHA'):
        return os.environ['GITHUB_SHA'][:12]
    try:
        result = subprocess.run(
            ['git', 'rev-parse', '--short=12', 'HEAD'],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except Exception:
        return 'unknown'



def generated_timestamp() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()



def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)



def write_text(path: Path, content: str) -> None:
    path.write_text(content)



def collect_file_entry(path: Path, description: str) -> dict:
    return {
        'path': path.name,
        'description': description,
        'sizeBytes': path.stat().st_size,
        'sha256': sha256_file(path),
    }



def render_index(files: list[dict]) -> str:
    items = '\n'.join(
        f'      <li><a href="{entry["path"]}"><code>{entry["path"]}</code></a> - {entry["description"]}</li>'
        for entry in files
    )
    return f'''<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Publisher Release Doc Bundle</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 2rem auto; max-width: 960px; line-height: 1.5; color: #1f2937; }}
    code {{ background: #f3f4f6; padding: 0.1rem 0.3rem; border-radius: 4px; }}
  </style>
</head>
<body>
  <h1>Publisher Release Doc Bundle</h1>
  <p>This lightweight bundle packages the generated publisher conformance report and sbom-tools integration docs for release review, handoff, static hosting, or archive upload.</p>
  <p>Local and CI builds also emit sibling <code>publisher-release-doc-bundle.tar.gz</code> and <code>publisher-release-doc-bundle.tar.gz.sha256</code> files for direct release publishing.</p>
  <ul>
{items}
  </ul>
</body>
</html>
'''



def build_archive(output_dir: Path) -> tuple[Path, Path, str]:
    archive_path = output_dir.parent / f'{output_dir.name}.tar.gz'
    checksum_path = output_dir.parent / f'{output_dir.name}.tar.gz.sha256'
    if archive_path.exists():
        archive_path.unlink()
    if checksum_path.exists():
        checksum_path.unlink()

    with tarfile.open(archive_path, 'w:gz') as archive:
        archive.add(output_dir, arcname=output_dir.name)

    archive_sha = sha256_file(archive_path)
    checksum_path.write_text(f'{archive_sha}  {archive_path.name}\n')
    return archive_path, checksum_path, archive_sha



def build_manifest(output_dir: Path, bundle_version: str) -> tuple[dict, list[dict]]:
    stats = build_stats(load_json(CHECKLIST), load_json(OPENAPI))
    file_entries: list[dict] = []

    report_md = output_dir / 'publisher-conformance-report.md'
    report_html = output_dir / 'publisher-conformance-report.html'
    report_summary = output_dir / 'publisher-conformance-summary.md'
    write_text(report_md, render_report(stats))
    write_text(report_html, render_html_report(stats))
    write_text(report_summary, render_summary(stats))
    file_entries.extend([
        collect_file_entry(report_md, 'Markdown publisher conformance/parity report.'),
        collect_file_entry(report_html, 'HTML publisher conformance/parity report.'),
        collect_file_entry(report_summary, 'Short markdown summary for CI surfaces.'),
    ])

    for source, target_name, description in SOURCE_DOCS:
        if not source.exists():
            raise SystemExit(f'missing required bundle source: {source}')
        target = output_dir / target_name
        shutil.copyfile(source, target)
        file_entries.append(collect_file_entry(target, description))

    index_path = output_dir / 'index.html'
    write_text(index_path, render_index(file_entries))
    file_entries.append(collect_file_entry(index_path, 'Bundle landing page with links to included docs.'))

    manifest = {
        'schemaVersion': 1,
        'bundleName': 'publisher-release-doc-bundle',
        'bundleVersion': bundle_version,
        'generatedAt': generated_timestamp(),
        'gitRevision': git_revision(),
        'canonicalContract': 'proto/tea/v1/publisher.proto',
        'publisherProfile': 'spec/publisher/openapi.json',
        'conformanceChecklist': 'spec/publisher/conformance-checklist.json',
        'distribution': {
            'directory': output_dir.name,
            'archive': f'{output_dir.name}.tar.gz',
            'archiveChecksumFile': f'{output_dir.name}.tar.gz.sha256',
        },
        'files': file_entries,
    }
    return manifest, file_entries



def write_manifest(output_dir: Path, manifest: dict) -> Path:
    manifest_path = output_dir / 'manifest.json'
    write_text(manifest_path, json.dumps(manifest, indent=2) + '\n')
    return manifest_path



def build_bundle(output_dir: Path, bundle_version: str) -> dict:
    ensure_parent(output_dir / 'index.html')
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    manifest, file_entries = build_manifest(output_dir, bundle_version)
    manifest_path = write_manifest(output_dir, manifest)
    archive_path, checksum_path, archive_sha = build_archive(output_dir)

    manifest['manifestPath'] = manifest_path.name
    manifest['distribution']['archiveSha256'] = archive_sha
    manifest['distribution']['archiveSizeBytes'] = archive_path.stat().st_size
    manifest['distribution']['checksumFileSha256'] = sha256_file(checksum_path)
    write_manifest(output_dir, manifest)
    return {
        'outputDir': output_dir,
        'archivePath': archive_path,
        'checksumPath': checksum_path,
        'manifestPath': manifest_path,
        'fileEntries': file_entries,
    }



def main() -> None:
    parser = argparse.ArgumentParser(description='Build a publish-friendly publisher release-doc bundle')
    parser.add_argument('--output-dir', default=str(DEFAULT_OUTPUT), help='directory to write the bundle into')
    parser.add_argument('--bundle-version', help='optional bundle version label; defaults to git revision when available')
    parser.add_argument('--check', action='store_true', help='fail if the bundle is missing required generated sources')
    args = parser.parse_args()

    if args.check:
        for source, _, _ in SOURCE_DOCS:
            if not source.exists():
                raise SystemExit(f'missing required generated source: {source}')
        print('publisher release-doc bundle inputs check ok')
        return

    bundle_version = args.bundle_version or git_revision()
    result = build_bundle(Path(args.output_dir), bundle_version)
    print(f'wrote {result["outputDir"]}')
    print(f'wrote {result["archivePath"]}')
    print(f'wrote {result["checksumPath"]}')


if __name__ == '__main__':
    main()
