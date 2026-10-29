#!/usr/bin/env python3
"""Qualify explicit provider embedding sources against existing locked archives.

Inspect complete selected Linux/NetBSD/macOS source trees, licenses and features,
including upstream build/test/CI files. Never execute upstream CI. This source
qualification does not establish native TLS behavior or a security audit.
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
from check_provider_auth_dependencies import existing_checks

ROOT = Path(__file__).resolve().parents[1]
REL = 'crates/faction-provider'
MANIFEST = ROOT / REL / 'Cargo.toml'
INVENTORY = ROOT / 'dev-docs/dependencies/provider.json'


def validate_profile(manifest, workspace):
    expected = {'package': {'name': 'faction-provider', 'version': '0.1.0-alpha.1',
                'edition': '2024', 'rust-version': '1.96', 'license': 'BSD-3-Clause', 'publish': False},
                'workspace': {}, 'features': {'default': [], 'blob-iq': ['dep:faction-xmpp']},
                'dependencies': {'faction-files': {'path': '../faction-files'}, 'openssl': '=0.10.81',
                                 'faction-xmpp': {'path': '../faction-xmpp', 'optional': True}}}
    require(manifest == expected, 'provider profile changed; selected source review required')
    require(not any(fnmatch.fnmatchcase(REL, member.rstrip('/'))
                    for member in workspace['workspace'].get('members', [])),
            'provider must remain outside the ordinary product workspace')
    require(not any(key in workspace for key in ['patch', 'replace']), 'unreviewed dependency replacement')


def validate_product_edges(manifests):
    for manifest in manifests.values():
        for section in ['dependencies', 'build-dependencies', 'dev-dependencies']:
            tables = [manifest.get(section, {})]
            tables += [target.get(section, {}) for target in manifest.get('target', {}).values()]
            for dependencies in tables:
                for alias, dependency in dependencies.items():
                    require(alias != 'faction-provider', 'provider cannot be an ordinary product dependency')
                    if isinstance(dependency, dict):
                        require(dependency.get('package') != 'faction-provider'
                                and Path(dependency.get('path', '')).name != 'faction-provider',
                                'provider cannot be an ordinary product dependency')


def check_inventory(path, current):
    require(path.is_file() and path.read_text() == current,
            'provider inventory changed; preserve evidence and review source/license/features before --write')


def _job(text, name):
    jobs = re.findall(r'(?ms)^  ' + re.escape(name) + r':\n(.*?)(?=^  [A-Za-z_][A-Za-z_0-9]*:\n|\Z)', text)
    require(len(jobs) == 1, name + ' CI job missing or duplicated')
    return jobs[0]


def _step(job, name):
    steps = re.findall(r'(?ms)^      - name: ' + re.escape(name) + r'\n(.*?)(?=^      - |\Z)', job)
    require(len(steps) == 1, name + ' CI step missing or duplicated')
    require(not re.search(r'^        (?:if|continue-on-error|shell):', steps[0], re.M), 'conditional or overridden CI step refused')
    return steps[0]


def _lines(step, allowed):
    blocks = re.findall(r'(?m)^        run: \|\n((?:          [^\n]*\n|\n)*)', step)
    require(len(blocks) == 1, 'literal straight-line CI run block missing')
    lines = [line.strip() for line in blocks[0].splitlines() if line.strip() and not line.strip().startswith('#')]
    for line in lines:
        require('`' not in line and '$(' not in line and not line.endswith('\\'), 'indirect shell control refused')
        lexer = shlex.shlex(line, posix=True, punctuation_chars=';&|<>'); lexer.whitespace_split = True
        words = list(lexer)
        require(words and words[0] in allowed, 'non-straight-line provider CI command refused')
        require(not any(word in {';', '&', '&&', '|', '||', '<', '<<', '<<<'} for word in words), 'shell control or inert heredoc refused')
    return lines


def validate_ci(text):
    # Recognize this reviewed workflow's literal jobs/steps only. This does not
    # interpret general YAML/shell semantics or establish global workflow trust.
    provider = _job(text, 'provider')
    require(not re.search(r'^    (?:if|continue-on-error|shell|strategy):', provider, re.M), 'conditional provider CI job refused')
    for setting in ['runs-on: ubuntu-24.04', 'timeout-minutes: 15']:
        require(provider.splitlines().count('    ' + setting) == 1, 'provider CI platform or timeout changed')
    step = _step(provider, 'Validate the opt-in attachment provider')
    for setting in ['CARGO_BUILD_JOBS: 2', 'RUSTUP_TOOLCHAIN: 1.96.0', 'CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider']:
        require(step.splitlines().count('          ' + setting) == 1, 'provider CI resource/toolchain environment changed')
    lines = _lines(step, {'rustup', 'cargo', 'python3'})
    commands = ['rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy']
    commands += ['cargo metadata --locked --manifest-path ' + REL + '/Cargo.toml --format-version 1 --filter-platform ' + target + ' > /dev/null' for target in TARGETS]
    commands += ['cargo metadata --locked --manifest-path ' + REL + '/Cargo.toml --format-version 1 --filter-platform ' + target + ' --features blob-iq > /dev/null' for target in TARGETS]
    commands += ['python3 tools/check_provider_dependencies.py',
                 'cargo fmt --manifest-path ' + REL + '/Cargo.toml -- --check',
                 'cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked -- -D warnings',
                 'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked -- --test-threads=1',
                 'cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --features blob-iq -- -D warnings',
                 'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --features blob-iq -- --test-threads=1']
    require(lines == commands, 'provider CI must qualify exact selected sources before locked validation')
    documentation = _job(text, 'documentation')
    require('    needs: [native_mail, provider]' in documentation.splitlines()
            and '    if: ${{ always() }}' in documentation.splitlines()
            and not re.search(r'^    (?:continue-on-error|shell):', documentation, re.M),
            'documentation must require native_mail and provider results')
    gate = _step(documentation, 'Require optional native tests')
    for name, job in [('NATIVE_MAIL_RESULT', 'native_mail'), ('PROVIDER_RESULT', 'provider')]:
        require(('          ' + name + ': ${{ needs.' + job + '.result }}') in gate.splitlines(), 'required CI result environment missing')
    require(_lines(gate, {'test'}) == ['test "$NATIVE_MAIL_RESULT" = success', 'test "$PROVIDER_RESULT" = success'],
            'documentation must fail closed on both optional test results')


def inventory():
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    validate_profile(tomllib.loads(MANIFEST.read_text()), workspace)
    validate_ci((ROOT / '.github/workflows/documentation.yml').read_text())
    product = {'workspace': workspace.get('workspace', {})}
    for member in workspace['workspace'].get('members', []):
        for path in ROOT.glob(member.rstrip('/') + '/Cargo.toml'):
            product[path.relative_to(ROOT).as_posix()] = tomllib.loads(path.read_text())
    validate_product_edges(product)
    lock = MANIFEST.parent / 'Cargo.lock'
    checks = existing_checks(tomllib.loads((ROOT / 'Cargo.lock').read_text()), tomllib.loads(lock.read_text()))
    profiles = {}; verified = {}
    for profile, features in [('default', []), ('blob-iq', ['blob-iq'])]:
        graphs = {}
        for target in TARGETS:
            command = ['cargo', 'metadata', '--locked', '--offline', '--manifest-path', str(MANIFEST),
                       '--format-version', '1', '--filter-platform', target]
            if features:
                command += ['--features', ','.join(features)]
            metadata = json.loads(subprocess.check_output(command, text=True))
            rows = []; local = []
            for package, selected_features in selected_packages(metadata):
                path = Path(package['manifest_path'])
                if package.get('source') is None:
                    local.append(scan_local(package, selected_features, ROOT))
                    continue
                key = (package['name'], package['version'], package['source'])
                require(key in checks, 'selected source missing from provider lock')
                require('vendored' not in selected_features, 'provider cannot silently vendor OpenSSL')
                row = scan_package(package, selected_features, checks[key])
                if key not in verified:
                    source = path.parent
                    archive = source.parents[2] / 'cache' / source.parent.name / (package['name'] + '-' + package['version'] + '.crate')
                    verified[key] = verify_archive_source(source, archive, checks[key])
                row.update(verified[key]); rows.append(row)
            graphs[target] = dict(registry=sorted(rows, key=lambda package: (package['name'], package['version'])),
                                  local=sorted(local, key=lambda package: package['name']))
        profiles[profile] = dict(selected_features=features, graphs=graphs)
    return json.dumps(dict(schema=2, scope='explicit-provider-embedding',
        root_lock_sha256=digest(ROOT / 'Cargo.lock'), provider_lock_sha256=digest(lock),
        manifest_sha256=digest(MANIFEST), profiles=profiles, upstream_ci_executed=False), indent=2, sort_keys=True) + '\n'



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='write only after deliberate selected source/license/features review')
    args = parser.parse_args(); current = inventory()
    if args.write:
        INVENTORY.write_text(current)
        print('Wrote explicit provider source inventory after reviewed preflight.')
    else:
        check_inventory(INVENTORY, current)
        print('Selected Linux/NetBSD/macOS provider sources match existing locked archives and reviewed profiles; no security audit claim.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, tarfile.TarError) as error:
        raise SystemExit(str(error)) from None
