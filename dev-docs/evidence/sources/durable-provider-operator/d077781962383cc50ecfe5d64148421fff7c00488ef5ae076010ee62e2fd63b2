#!/usr/bin/env python3
"""Qualify the isolated provider experiment against existing locked source only.

Reads complete selected Linux/NetBSD/macOS source trees and locked archives,
including upstream build/test/CI files. Never executes upstream CI or installs
packages. Source/license/feature integrity is not a security or native TLS audit.
"""
import argparse
import fnmatch
import json
import re
import shlex
import subprocess
import tarfile
import tomllib
from pathlib import Path
from check_network_mail_dependencies import (REGISTRY, TARGETS, digest, require,
    scan_tree, scan_package, scan_local, selected_packages, verify_archive_source)

ROOT=Path(__file__).resolve().parents[1]
REL='experiments/270-provider-auth'
MANIFEST=ROOT/REL/'Cargo.toml'
INVENTORY=ROOT/'dev-docs/dependencies/provider-auth.json'

def validate_profile(manifest,workspace):
    expected={'package':{'name':'faction-provider-auth-research','version':'0.0.0','edition':'2024','rust-version':'1.96','license':'BSD-3-Clause','publish':False},'workspace':{},'lib':{'path':'src/lib.rs'},'dependencies':{'faction-files':{'path':'../../crates/faction-files'},'openssl':'=0.10.81'}}
    require(manifest==expected,'provider experiment profile changed; source review required')
    require(not any(fnmatch.fnmatchcase(REL,m.rstrip('/')) for m in workspace['workspace'].get('members',[])), 'provider experiment must remain outside product workspace')

def existing_checks(root_lock,experiment_lock):
    roots={}
    for p in root_lock['package']:
        if p.get('source'):
            key=(p['name'],p['version'],p['source'])
            require(key not in roots,'duplicate root registry package')
            roots[key]=p.get('checksum')
    checks={}
    for p in experiment_lock['package']:
        if not p.get('source'):continue
        key=(p['name'],p['version'],p['source'])
        require(key not in checks,'duplicate experiment registry package')
        require(p['source']==REGISTRY and key in roots and p.get('checksum')==roots[key], 'experiment dependency is not an exact existing locked source')
        checks[key]=p['checksum']
    return checks

def validate_ci(text):
    commands=['python3 tools/check_provider_auth_dependencies.py',
        'cargo fmt --manifest-path '+REL+'/Cargo.toml -- --check',
        'cargo clippy --manifest-path '+REL+'/Cargo.toml --all-targets --release --locked -- -D warnings',
        'cargo test --manifest-path '+REL+'/Cargo.toml --release --locked -- --test-threads=1']
    # Recognize this repository's straight-line native run step, not arbitrary
    # YAML/shell semantics or a malicious workflow. Final source review remains required.
    job=re.search(r'(?ms)^  native_mail:\n(.*?)(?=^  [A-Za-z_][A-Za-z_0-9]*:\n|\Z)',text)
    require(job is not None,'native CI job missing')
    native=job.group(1)
    require(not re.search(r'^    (?:if|continue-on-error|shell):',native,re.M),'conditional native CI job refused')
    step=re.search(r'(?ms)^      - name: Validate the optional member-device checkpoint path\n(.*?)(?=^      - |\Z)',native)
    require(step is not None,'native checkpoint run step missing')
    body=step.group(1)
    require(not re.search(r'^        (?:if|continue-on-error|shell):',body,re.M),'conditional native CI step refused')
    run=re.search(r'(?m)^        run: \|\n((?:          [^\n]*\n|\n)*)',body)
    require(run is not None,'literal native run block missing')
    lines=[line.strip() for line in run.group(1).splitlines()]
    for line in lines:
        if not line or line.startswith('#'):continue
        require('`' not in line and '$(' not in line and not line.endswith('\\'),'indirect shell control refused')
        lexer=shlex.shlex(line,posix=True,punctuation_chars=';&|<>')
        lexer.whitespace_split=True
        words=list(lexer)
        require(words and words[0] in {'cargo','python3','rustup','mkdir','printf'},'non-straight-line native command refused')
        require(not any(w in {';','&','&&','|','||','<','<<','<<<'} for w in words),'shell control or inert heredoc refused')
    require(all(lines.count(c)==1 for c in commands),'CI must qualify and test isolated provider experiment with locked inputs')
    require([lines.index(c) for c in commands]==sorted(lines.index(c) for c in commands),'provider source qualification must precede tests')

def inventory():
    workspace=tomllib.loads((ROOT/'Cargo.toml').read_text())
    validate_profile(tomllib.loads(MANIFEST.read_text()),workspace)
    validate_ci((ROOT/'.github/workflows/documentation.yml').read_text())
    lock=MANIFEST.parent/'Cargo.lock'
    checks=existing_checks(tomllib.loads((ROOT/'Cargo.lock').read_text()),tomllib.loads(lock.read_text()))
    graphs={};verified={}
    for target in TARGETS:
        metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--offline','--manifest-path',str(MANIFEST),'--format-version','1','--filter-platform',target],text=True))
        rows=[];local=[]
        for package,features in selected_packages(metadata):
            path=Path(package['manifest_path'])
            if package.get('source') is None:
                if path==MANIFEST:
                    require(package.get('license')=='BSD-3-Clause','experiment license changed')
                    scan_tree(path.parent)
                    local.append(dict(name=package['name'],version=package['version'],license=package['license'],manifest=path.relative_to(ROOT).as_posix(),manifest_sha256=digest(path),features=sorted(features)))
                else:local.append(scan_local(package,features,ROOT))
                continue
            key=(package['name'],package['version'],package['source'])
            require(key in checks,'selected source missing from experiment lock')
            require('vendored' not in features,'provider experiment cannot silently vendor OpenSSL')
            row=scan_package(package,features,checks[key])
            if key not in verified:
                source=path.parent
                archive=source.parents[2]/'cache'/source.parent.name/(package['name']+'-'+package['version']+'.crate')
                verified[key]=verify_archive_source(source,archive,checks[key])
            row.update(verified[key]);rows.append(row)
        graphs[target]=dict(registry=sorted(rows,key=lambda p:(p['name'],p['version'])),local=sorted(local,key=lambda p:p['name']))
    return json.dumps(dict(schema=1,scope='isolated-provider-auth-experiment',root_lock_sha256=digest(ROOT/'Cargo.lock'),experiment_lock_sha256=digest(lock),manifest_sha256=digest(MANIFEST),graphs=graphs,upstream_ci_executed=False),indent=2,sort_keys=True)+'\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true',help='write only after deliberate selected source/license/features review')
    args=parser.parse_args();current=inventory()
    if args.write:
        INVENTORY.write_text(current)
        print('Wrote isolated provider source inventory after explicit preflight.')
    else:
        require(INVENTORY.is_file() and INVENTORY.read_text()==current,'provider inventory changed; preserve evidence and review source before --write')
        print('Isolated provider sources match existing locked archives and reviewed profiles for Linux/NetBSD/macOS.')

if __name__=='__main__':
    try:main()
    except (ValueError,OSError,tarfile.TarError) as e:raise SystemExit(str(e)) from None
