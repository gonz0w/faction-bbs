#!/usr/bin/env python3
"""Inventory/scan the pinned NetBSD crate sources (Cargo and fetched crates required).

Prints credential-free JSON; no binaries are built. Cargo may retrieve missing
registry metadata/sources. This is a filename/manifest policy screen, not an audit
of unsafe code, vulnerability advisories, generated code or every Cargo target.
"""
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
SUFFIXES = {'.js','.jsx','.mjs','.cjs','.ts','.tsx','.mts','.cts'}
MANIFESTS = {'package.json','package-lock.json','npm-shrinkwrap.json','yarn.lock',
             'pnpm-lock.yaml','bun.lock','bun.lockb','deno.json','deno.jsonc'}

def inventory():
    raw = subprocess.check_output(['cargo','metadata','--locked','--format-version','1',
                                  '--filter-platform','aarch64-unknown-netbsd',
                                  '--manifest-path',str(HERE/'Cargo.toml')],text=True)
    metadata = json.loads(raw)
    selected = {node['id'] for node in metadata['resolve']['nodes']}
    rows = []
    for p in metadata['packages']:
        if p['id'] not in selected or p['source'] is None: continue
        root = Path(p['manifest_path']).parent
        forbidden = sorted(str(f.relative_to(root)) for f in root.rglob('*') if
                           (f.is_file() and (f.suffix.lower() in SUFFIXES or f.name in MANIFESTS)) or
                           (f.is_dir() and f.name == 'node_modules'))
        rows.append({'name':p['name'],'version':p['version'],'license':p['license'],
                     'forbidden_files':forbidden})
    return sorted(rows,key=lambda p:p['name'])

if __name__ == '__main__':
    rows = inventory()
    print(json.dumps(rows,indent=2))
    if any(p['forbidden_files'] or not p['license'] for p in rows):
        raise SystemExit(1)
