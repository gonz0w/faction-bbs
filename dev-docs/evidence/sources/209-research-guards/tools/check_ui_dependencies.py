#!/usr/bin/env python3
"""Check standalone research dependencies against the reviewed native inventory."""
import json
import subprocess
from pathlib import Path
from check_rust_dependencies import ROOT,INVENTORY,TARGETS,SUFFIXES,MANIFESTS,reviewed_subset


def check():
    reviewed=json.loads(INVENTORY.read_text())
    for target in TARGETS:
        data=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',target,
            '--manifest-path',str(ROOT/'experiments/055-shared-ui/Cargo.toml')],text=True))
        selected={n['id']:n['features'] for n in data['resolve']['nodes']}
        expected={(p['name'],p['version']):p for p in reviewed[target]}
        rows=[]
        for p in data['packages']:
            if p['id'] not in selected or p['source'] is None:continue
            row={'name':p['name'],'version':p['version'],'license':p['license'],'features':sorted(selected[p['id']])}
            assert reviewed_subset(row,expected.get((p['name'],p['version']))),'unreviewed experiment dependency/features'
            for f in Path(p['manifest_path']).parent.rglob('*'):
                assert not (f.is_file() and (f.suffix.lower() in SUFFIXES or f.name in MANIFESTS)),'prohibited dependency source'
                assert not (f.is_dir() and f.name=='node_modules'),'prohibited dependency directory'
            rows.append(row)
        print(target+': standalone experiment uses '+str(len(rows))+' already reviewed registry packages/features; no JS/TS filenames.')


if __name__=='__main__':check()
