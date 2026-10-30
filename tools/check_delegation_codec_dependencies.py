#!/usr/bin/env python3
"""Offline exact-source gate for the standalone unverified wire codec.

Upstream tests and CI are inspected inert source, never selected or executed.
This is source/license/feature integrity, not a security or native runtime audit.
"""
import argparse
import fnmatch
import json
import re
import subprocess
import tarfile
import tomllib
from pathlib import Path
from check_network_mail_dependencies import (REGISTRY, TARGETS, digest, require,
    scan_package, scan_local, scan_tree, selected_packages, verify_archive_source)

ROOT = Path(__file__).resolve().parents[1]
REL = 'crates/faction-delegation-codec'
MANIFEST = ROOT / REL / 'Cargo.toml'
INVENTORY = ROOT / 'dev-docs/dependencies/delegation-codec.json'
# Independently adopted archive identities, licenses and selected features; changing
# the candidate lock/inventory alone cannot introduce another input.
APPROVED = {('base64', '0.22.1'): {'checksum': '72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6',
                        'features': ['alloc', 'std'],
                        'license': 'MIT OR Apache-2.0'},
 ('form_urlencoded', '1.2.2'): {'checksum': 'cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf',
                                'features': ['alloc', 'std'],
                                'license': 'MIT OR Apache-2.0'},
 ('httparse', '1.10.1'): {'checksum': '6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87',
                          'features': ['std'],
                          'license': 'MIT OR Apache-2.0'},
 ('itoa', '1.0.18'): {'checksum': '8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682',
                      'features': [],
                      'license': 'MIT OR Apache-2.0'},
 ('memchr', '2.8.3'): {'checksum': 'cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98',
                       'features': ['alloc', 'std'],
                       'license': 'Unlicense OR MIT'},
 ('percent-encoding', '2.3.2'): {'checksum': '9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220',
                                 'features': ['alloc', 'std'],
                                 'license': 'MIT OR Apache-2.0'},
 ('serde', '1.0.229'): {'checksum': '4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba',
                        'features': ['std'],
                        'license': 'MIT OR Apache-2.0'},
 ('serde_core', '1.0.229'): {'checksum': '67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48',
                             'features': ['result', 'std'],
                             'license': 'MIT OR Apache-2.0'},
 ('serde_json', '1.0.154'): {'checksum': 'e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6',
                             'features': ['std'],
                             'license': 'MIT OR Apache-2.0'},
 ('zmij', '1.0.23'): {'checksum': '29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b',
                      'features': [],
                      'license': 'MIT'}}

LOCK_SHA256 = '5c0ae3b676e1e6aba6fa64f4a560de32a88d3bb1b9c0767f77a616ae94aa4a39'

def expected_manifest():
    return {'package': {'name': 'faction-delegation-codec', 'version': '0.1.0-alpha.1',
        'edition': '2024', 'rust-version': '1.96', 'license': 'BSD-3-Clause', 'publish': False},
        'workspace': {}, 'dependencies': {
            name: {'version': '=' + version, 'default-features': False, 'features': ['std']}
            for name, version in [('httparse', '1.10.1'), ('form_urlencoded', '1.2.2'),
                                  ('serde_json', '1.0.154'), ('serde', '1.0.229'), ('base64', '0.22.1')]}}


def validate_profile(manifest, workspace):
    require(manifest == expected_manifest(), 'standalone codec profile changed')
    require(not any('patch' in p or 'replace' in p for p in [manifest, workspace]), 'dependency replacement refused')
    require(not any(fnmatch.fnmatchcase(REL, member.rstrip('/'))
                    for member in workspace['workspace'].get('members', [])), 'codec exposed to product workspace')


def validate_product_edges(manifests):
    edge={'path':'../faction-delegation-codec','optional':True,'default-features':False}
    def visit(value):
        if isinstance(value, dict):
            for key, child in value.items():
                require(key != 'faction-delegation-codec', 'product codec dependency edge')
                if key in {'path', 'package'} and isinstance(child, str):
                    require('faction-delegation-codec' not in child, 'renamed product codec dependency edge')
                require(key not in {'patch', 'replace'}, 'product dependency replacement refused')
                visit(child)
        elif isinstance(value, list):
            for child in value: visit(child)
    import copy
    for path, original in manifests.items():
        manifest=copy.deepcopy(original)
        selected=manifest.get('dependencies',{}).get('faction-delegation-codec')
        if selected is not None:
            require(path=='crates/faction-provider/Cargo.toml' and selected==edge, 'only exact optional provider codec edge allowed')
            features=manifest.get('features',{})
            require(features.get('default')==[] and features.get('delegation-wire')==['delegation-issuer','dep:faction-delegation-codec'], 'codec must be selected only by explicit wire composition')
            for name, values in features.items():
                if name!='delegation-wire':
                    require(all(v not in {'delegation-wire','dep:faction-delegation-codec','faction-delegation-codec'} and not v.startswith('faction-delegation-codec/') for v in values), 'ordinary/core feature activates codec')
            del manifest['dependencies']['faction-delegation-codec']
        visit(manifest)


def validate_lock(lock):
    selected = {}
    for p in lock['package']:
        if not p.get('source'): continue
        key = (p['name'], p['version'])
        # Inactive cfg(any()) serde_derive closure is fixed by LOCK_SHA256 at the
        # file boundary, and is not approved as a selected source below.
        if key not in APPROVED:
            require(key in {('proc-macro2', '1.0.107'), ('quote', '1.0.47'),
                            ('serde_derive', '1.0.229'), ('syn', '3.0.9'), ('unicode-ident', '1.0.27')},
                    'unreviewed lock identity')
            continue
        require(key not in selected and p['source'] == REGISTRY
                and p.get('checksum') == APPROVED[key]['checksum'], 'unreviewed locked archive identity')
        selected[key] = p['checksum']
    require(set(selected) == set(APPROVED), 'qualified registry package missing')
    return selected


def validate_selected(package, features):
    key = (package['name'], package['version'])
    require(key in APPROVED, 'unreviewed selected package')
    expected = APPROVED[key]
    require(package.get('source') == REGISTRY and package.get('license') == expected['license']
            and sorted(features) == expected['features'], 'selected source/license/features changed')


def check_inventory(path, current):
    require(path.is_file() and path.read_text() == current, 'codec inventory drift; review before --write')


def validate_isolation_graph(metadata):
    packages = {p['id']: p for p in metadata['packages']}
    require(all(packages[n['id']]['name'] not in {'faction-delegation-codec','httparse','form_urlencoded','percent-encoding'}
                for n in metadata['resolve']['nodes']), 'codec leaked into a product graph')



def validate_wire_graph(metadata):
    packages={p['id']:p for p in metadata['packages']}
    nodes={n['id']:n for n in metadata['resolve']['nodes']}
    codecs=[n for n in nodes.values() if packages[n['id']]['name']=='faction-delegation-codec']
    require(len(codecs)==1 and codecs[0]['features']==[], 'wire graph must select exactly one inert codec')
    providers=[n for n in nodes.values() if packages[n['id']]['name']=='faction-provider']
    require(len(providers)==1 and 'delegation-wire' in providers[0]['features'], 'codec requires explicit provider wire feature')
    pending=[codecs[0]['id']];seen=set()
    while pending:
        identity=pending.pop()
        if identity in seen:continue
        seen.add(identity);node=nodes[identity];package=packages[identity]
        if identity!=codecs[0]['id']:
            key=(package['name'],package['version']);require(key in APPROVED,'unreviewed codec composition dependency')
            approved=APPROVED[key];expected=approved['features']
            if key==('serde_core','1.0.229'):expected=['alloc','result','std']
            require(package.get('source')==REGISTRY and package.get('license')==approved['license'] and sorted(node['features'])==expected, 'wire codec identity/license/feature union changed')
        pending.extend(d['pkg'] for d in node.get('deps',[]))
    return seen


def isolation_graphs():
    profiles = [('default', ROOT / 'Cargo.toml', None),
                ('native-mail', ROOT / 'Cargo.toml', 'faction-client/native-mail'),
                ('network-mail', ROOT / 'Cargo.toml', 'faction-client/network-mail'),
                ('device-provider', ROOT / 'Cargo.toml', 'faction-client/device-provider'),
                ('provider', ROOT / 'crates/faction-provider/Cargo.toml', None),
                ('provider-operator', ROOT / 'crates/faction-provider/Cargo.toml', 'operator'),
                ('provider-issuer',ROOT/'crates/faction-provider/Cargo.toml','delegation-issuer'),
                ('provider-issuer-durable',ROOT/'crates/faction-provider/Cargo.toml','delegation-issuer,durable-policy'),
                ('provider-wire',ROOT/'crates/faction-provider/Cargo.toml','delegation-wire'),
                ('provider-wire-durable',ROOT/'crates/faction-provider/Cargo.toml','delegation-wire,durable-policy')]
    graphs = {}
    for name, manifest, feature in profiles:
        graphs[name] = {}
        for target in TARGETS:
            command = ['cargo', 'metadata', '--locked', '--offline', '--manifest-path', str(manifest),
                       '--format-version', '1', '--filter-platform', target]
            if feature: command += ['--features', feature]
            metadata = json.loads(subprocess.check_output(command, text=True))
            if name.startswith('provider-wire'): validate_wire_graph(metadata)
            else: validate_isolation_graph(metadata)
            packages = {p['id']: p for p in metadata['packages']}
            rows = [dict(name=packages[n['id']]['name'], version=packages[n['id']]['version'],
                         features=sorted(n['features'])) for n in metadata['resolve']['nodes']]
            graphs[name][target] = sorted(rows, key=lambda row: (row['name'], row['version']))
    return graphs


def ci_commands():
    commands = ['rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy']
    for manifest, feature in [(REL + '/Cargo.toml', ''),
                              ('Cargo.toml', ' --features faction-client/device-provider'),
                              ('crates/faction-provider/Cargo.toml', ' --features operator'),
                              ('crates/faction-provider/Cargo.toml', ' --features delegation-wire'),
                              ('crates/faction-provider/Cargo.toml', ' --features delegation-wire,durable-policy')]:
        for target in TARGETS:
            commands.append('cargo metadata --locked --manifest-path ' + manifest
                + ' --format-version 1 --filter-platform ' + target + feature + ' > /dev/null')
    return commands + ['python3 tools/check_delegation_codec_dependencies.py',
        'cargo fmt --manifest-path ' + REL + '/Cargo.toml -- --check',
        'cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --offline -- -D warnings',
        'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --offline']


def ci_step():
    return ('      - name: Validate the isolated delegation wire codec\n'
        '        env:\n'
        '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-delegation-codec\n'
        '          CARGO_BUILD_JOBS: 2\n'
        '          RUSTUP_TOOLCHAIN: 1.96.0\n'
        '        run: |\n' + ''.join('          ' + c + '\n' for c in ci_commands()))


def validate_ci(source):
    job = re.search(r'(?ms)^  documentation:\n(.*?)(?=^  [a-zA-Z_][a-zA-Z_0-9]*:\n|\Z)', source)
    require(job is not None, 'required documentation job missing')
    body = job.group(1)
    for line in ['    name: Documentation and policy',
                 '    needs: [native_mail, provider, provider_operator]',
                 '    if: ${{ always() }}', '    timeout-minutes: 15']:
        require(body.splitlines().count(line) == 1, 'required CI semantics changed')
    step = ci_step()
    require(not re.search(r'^    (?:continue-on-error|shell):', body, re.M),
            'required CI job cannot tolerate or override codec execution')
    actual_step = re.search(r'(?ms)^      - name: Validate the isolated delegation wire codec\n.*?(?=^      - |\Z)', body)
    require(actual_step is not None and actual_step.group(0) == step
            and source.count(step) == 1, 'codec CI source gate/warm/build step changed or missing')
    # Exact literal step excludes conditional skips, shell substitutions, reordered
    # gates and loss of offline/locked flags. Extra selected builds elsewhere fail.
    remaining = source.replace(step, '')
    require(REL + '/Cargo.toml' not in remaining, 'codec build outside guarded CI step')


def inventory():
    validate_ci((ROOT / '.github/workflows/documentation.yml').read_text())
    manifests = {p.relative_to(ROOT).as_posix(): tomllib.loads(p.read_text())
                 for p in [ROOT / 'Cargo.toml', *sorted((ROOT / 'crates').glob('*/Cargo.toml'))] if p != MANIFEST}
    validate_profile(tomllib.loads(MANIFEST.read_text()), manifests['Cargo.toml'])
    validate_product_edges(manifests)
    lock = MANIFEST.parent / 'Cargo.lock'
    require(digest(lock) == LOCK_SHA256, 'standalone lock bytes changed')
    checks = validate_lock(tomllib.loads(lock.read_text()))
    graphs, verified = {}, {}
    for target in TARGETS:
        data = json.loads(subprocess.check_output(['cargo', 'metadata', '--manifest-path', str(MANIFEST),
            '--locked', '--offline', '--format-version', '1', '--filter-platform', target], text=True))
        require(data['workspace_members'] == [data['resolve']['root']], 'codec workspace gained members')
        rows, local = [], []
        for package, features in selected_packages(data):
            source = Path(package['manifest_path']).parent
            if package.get('source') is None:
                require(source == MANIFEST.parent and features == [], 'unexpected local codec graph edge/features')
                local.append(scan_local(package, features, ROOT))
                continue
            validate_selected(package, features)
            key = (package['name'], package['version'])
            row = scan_package(package, features, checks[key])
            if key not in verified:
                archive = source.parents[2] / 'cache' / source.parent.name / (source.name + '.crate')
                verified[key] = verify_archive_source(source, archive, checks[key])
            row.update(verified[key]); rows.append(row)
        require({(r['name'], r['version']) for r in rows} == set(APPROVED), 'selected target graph incomplete')
        graphs[target] = dict(registry=sorted(rows, key=lambda r: r['name']), local=local)
    return json.dumps(dict(schema=1, scope='standalone-unverified-delegation-codec',
        upstream_ci_selected=False, upstream_dependency_tests_selected=False,
        manifest_sha256=digest(MANIFEST), lock_sha256=digest(lock), graphs=graphs, isolation=isolation_graphs()), indent=2, sort_keys=True) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='write only after explicit source review')
    args = parser.parse_args(); current = inventory()
    if args.write:
        INVENTORY.write_text(current)
        print('Wrote independently pinned codec source inventory.')
    else:
        check_inventory(INVENTORY, current)
        print('Standalone codec Linux/NetBSD/macOS source/license/features match approved archives; no runtime claim.')

if __name__ == '__main__':
    try: main()
    except (ValueError, OSError, tarfile.TarError) as error: raise SystemExit(str(error)) from None
