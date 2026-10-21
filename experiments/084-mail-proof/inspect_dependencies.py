#!/usr/bin/env python3
"""Inspect only the selected standalone reference runtime, never product deps.

Run inside its private venv after building Slixmpp's upstream Python JID fallback.
Wheel hashes are observations of tested platform artifacts, not a portable lock.
"""
import argparse
import hashlib
from importlib import metadata
import json
from pathlib import Path
import sys
import zipfile
from email.parser import Parser
from packaging.requirements import Requirement
from packaging.utils import canonicalize_name

FORBIDDEN = {'.js','.jsx','.ts','.tsx','.mjs','.cjs','.mts','.cts'}
MANIFESTS = {'package.json','package-lock.json','npm-shrinkwrap.json','yarn.lock','pnpm-lock.yaml','bun.lock','bun.lockb','deno.json','deno.jsonc'}
ROOTS = ['OMEMO==2.1.0', 'Twomemo[xml]==2.1.0', 'slixmpp==1.17.0']
BUILD = {'setuptools', 'setuptools-rust', 'setuptools-scm', 'packaging', 'semantic-version', 'vcs-versioning'}


def check(wheels):
    queue, selected = [Requirement(n) for n in ROOTS], {}
    while queue:
        req = queue.pop()
        name = canonicalize_name(req.name)
        previous = selected.get(name)
        if previous is not None and req.extras <= previous:
            continue
        extras = req.extras | (previous or set())
        selected[name] = extras
        dist = metadata.distribution(name)
        assert dist.version in req.specifier
        for raw in dist.requires or []:
            dep = Requirement(raw)
            if dep.marker is None or any(dep.marker.evaluate({'extra': x}) for x in ['']+sorted(extras)):
                queue.append(dep)
    artifacts = {}
    for path in wheels.glob('*.whl'):
        with zipfile.ZipFile(path) as z:
            msg = Parser().parsestr(z.read(next(n for n in z.namelist() if n.endswith('.dist-info/METADATA'))).decode())
            name = canonicalize_name(msg['Name'])
            if name not in selected and name not in BUILD: continue
            for n in z.namelist():
                p = Path(n)
                assert p.suffix.lower() not in FORBIDDEN and p.name not in MANIFESTS and 'node_modules' not in p.parts
            if name == 'slixmpp':
                assert not any(n.endswith(('.so','.dylib','.dll')) for n in z.namelist())
            artifacts[name] = {'wheel': path.name, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    packages = []
    for name in sorted(selected):
        dist = metadata.distribution(name)
        for n in dist.files or []:
            p = Path(n)
            assert p.suffix.lower() not in FORBIDDEN and p.name not in MANIFESTS and 'node_modules' not in p.parts
        assert name in artifacts
        license = dist.metadata.get('License-Expression') or dist.metadata.get('License') or '; '.join(x for x in dist.metadata.get_all('Classifier', []) if x.startswith('License ::'))
        assert license
        packages.append({'name': name, 'version': dist.version, 'license': license, **artifacts[name]})
    build = []
    for name in sorted(BUILD):
        dist = metadata.distribution(name)
        for n in dist.files or []:
            p = Path(n)
            assert p.suffix.lower() not in FORBIDDEN and p.name not in MANIFESTS
        assert name in artifacts
        license = dist.metadata.get('License-Expression') or dist.metadata.get('License') or '; '.join(x for x in dist.metadata.get_all('Classifier', []) if x.startswith('License ::'))
        assert license
        build.append({'name': name, 'version': dist.version, 'license': license, **artifacts[name]})
    import slixmpp.jid
    assert slixmpp.jid.__file__.endswith('.py')
    assert 'slixmpp-omemo' not in selected and 'pyo3' not in selected
    return {'roots': ROOTS, 'packages': packages, 'build_packages': build, 'build_count': len(build), 'count': len(packages), 'python': sys.version.split()[0], 'no_js_ts_selected_filenames': True, 'slixmpp_jid': 'upstream Python fallback', 'product_dependency': False, 'platform_artifact_lock': False, 'slixmpp_source': {'url': 'https://files.pythonhosted.org/packages/6c/ff/aac46de291c29162da5f1ef45d68a6f649a538942da28b514c866b909e35/slixmpp-1.17.0.tar.gz', 'sha256': '05257ee0f2aecb42d6479f6071c89ad3c734fda0ee753eb4625ecdc6114d90b1'}}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--wheels', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    result = check(a.wheels)
    a.output.write_text(json.dumps(result, indent=2, sort_keys=True)+'\n')
    print('Selected standalone reference packages inspected:', result['count'])


if __name__ == '__main__': main()
