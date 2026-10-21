#!/usr/bin/env python3
"""Scan selected Linux/NetBSD dependency sources and compare the reviewed inventory.

Python stdlib and Cargo required; Cargo may fetch registry sources. This checks
filenames/manifests, license declarations, package versions and selected features.
It is not a vulnerability, unsafe-code, generated-code or runtime security audit.
The lock may include unselected platform packages; this inventory covers the two
named graphs only. Review/update intentionally after any dependency/feature change.
"""
import argparse
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / 'dev-docs/dependencies/local-core.json'
TARGETS = ['aarch64-unknown-netbsd', 'x86_64-unknown-linux-gnu']
SUFFIXES = {'.js', '.jsx', '.mjs', '.cjs', '.ts', '.tsx', '.mts', '.cts'}
MANIFESTS = {'package.json', 'package-lock.json', 'npm-shrinkwrap.json', 'yarn.lock',
             'pnpm-lock.yaml', 'bun.lock', 'bun.lockb', 'deno.json', 'deno.jsonc'}


def reviewed_subset(actual, expected):
    """Standalone graphs may select fewer of the reviewed additive features."""
    return (expected is not None
            and {k: v for k, v in actual.items() if k != 'features'}
            == {k: v for k, v in expected.items() if k != 'features'}
            and set(actual['features']).issubset(expected['features']))


def inventory():
    graphs = {}
    for target in TARGETS:
        data = json.loads(subprocess.check_output([
            'cargo', 'metadata', '--locked', '--format-version', '1',
            '--filter-platform', target, '--manifest-path', str(ROOT / 'Cargo.toml')], text=True))
        selected = {n['id']: n['features'] for n in data['resolve']['nodes']}
        rows = []
        for package in data['packages']:
            if package['id'] not in selected or package['source'] is None:
                continue
            root = Path(package['manifest_path']).parent
            forbidden = sorted(str(f.relative_to(root)) for f in root.rglob('*') if
                (f.is_file() and (f.suffix.lower() in SUFFIXES or f.name in MANIFESTS)) or
                (f.is_dir() and f.name == 'node_modules'))
            if forbidden or not package['license']:
                raise SystemExit(f"Dependency policy failed: {package['name']} {forbidden}")
            rows.append({'name': package['name'], 'version': package['version'],
                         'license': package['license'], 'features': sorted(selected[package['id']])})
        graphs[target] = sorted(rows, key=lambda row: (row['name'], row['version']))
    return json.dumps(graphs, indent=2, sort_keys=True) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='Write a new inventory; review the diff before committing')
    args = parser.parse_args()
    current = inventory()
    if args.write:
        INVENTORY.parent.mkdir(parents=True, exist_ok=True)
        INVENTORY.write_text(current)
        print('Wrote selected dependency inventory; review versions, licenses and features.')
    elif not INVENTORY.is_file() or INVENTORY.read_text() != current:
        raise SystemExit('Dependency inventory changed; review sources/licenses/features and regenerate intentionally.')
    else:
        print('Selected Linux/NetBSD dependency sources passed no-JS filename policy; inventory unchanged.')
