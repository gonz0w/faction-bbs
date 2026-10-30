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
                'workspace': {}, 'features': {'default': [], 'blob-iq': ['dep:faction-xmpp'],
                    'durable-policy': ['dep:rustix'], 'delegation-records': [], 'delegation-services': [],
                    'preparation': ['dep:toml', 'toml/display'],
                    'operator': ['durable-policy', 'preparation', 'dep:faction-unix']},
                'dependencies': {'faction-files': {'path': '../faction-files'}, 'openssl': '=0.10.81',
                                 'faction-xmpp': {'path': '../faction-xmpp', 'optional': True},
                    'rustix': {'version': '=1.1.5', 'default-features': False,
                               'features': ['std', 'fs', 'process'], 'optional': True},
                    'toml': {'version': '=1.1.8', 'default-features': False,
                             'features': ['std', 'parse', 'serde'], 'optional': True},
                    'faction-unix': {'path': '../faction-unix', 'optional': True}},
                'bin': [{'name': 'faction-provider', 'path': 'src/main.rs', 'required-features': ['operator']}]}
    require(manifest == expected, 'provider profile changed; selected source review required')
    require(not any(fnmatch.fnmatchcase(REL, member.rstrip('/'))
                    for member in workspace['workspace'].get('members', [])),
            'provider must remain outside the ordinary product workspace')
    require(not any(key in workspace for key in ['patch', 'replace']), 'unreviewed dependency replacement')


def validate_product_edges(manifests):
    client = 'crates/faction-client/Cargo.toml'
    features = {'default': [],
                'native-mail': ['dep:faction-omemo', 'faction-omemo/native'],
                'network-mail': ['native-mail', 'dep:faction-xmpp', 'dep:tokio', 'dep:roxmltree'],
                'device-provider': ['network-mail', 'dep:faction-provider', 'faction-provider/preparation', 'tokio/sync']}
    edge = {'path': '../faction-provider', 'optional': True, 'default-features': False}
    for name, manifest in manifests.items():
        require(not any(key in manifest for key in ['patch', 'replace']), 'unreviewed dependency replacement')
        if name == client:
            require(manifest.get('features') == features, 'client provider feature profile changed')
            require(manifest.get('dependencies', {}).get('faction-provider') == edge,
                    'client provider must remain the exact default-off local dependency')
        for section in ['dependencies', 'build-dependencies', 'dev-dependencies']:
            tables = [(section, manifest.get(section, {}))]
            tables += [('target', target.get(section, {})) for target in manifest.get('target', {}).values()]
            tables += [('workspace', manifest.get('workspace', {}).get(section, {}))]
            for location, dependencies in tables:
                for alias, dependency in dependencies.items():
                    provider = alias == 'faction-provider' or (isinstance(dependency, dict) and
                        (dependency.get('package') == 'faction-provider'
                         or Path(dependency.get('path', '')).name == 'faction-provider'))
                    if provider:
                        require(name == client and location == 'dependencies'
                                and alias == 'faction-provider' and dependency == edge,
                                'provider cannot be an ordinary product dependency')


def validate_graph(metadata, enabled):
    selected = [(package, features) for package, features in selected_packages(metadata)
                if package['name'] == 'faction-provider']
    require(bool(selected) == enabled, 'provider leaked into an ordinary graph or is absent from the selected graph')
    if enabled:
        require(len(selected) == 1 and selected[0][1] == ['preparation'],
                'client provider must select only preparation; operator and durability stay separate')


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


TOOLCHAIN = 'rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy'


def root_warm_commands():
    return ['cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform ' + target
            + (' --features faction-client/' + feature if feature else '') + ' > /dev/null'
            for feature in ['', 'native-mail', 'network-mail', 'device-provider'] for target in TARGETS]


def provider_warm_commands():
    return ['cargo metadata --locked --manifest-path ' + REL + '/Cargo.toml --format-version 1 --filter-platform ' + target
            + (' --features ' + feature if feature else '') + ' > /dev/null'
            for feature in ['', 'blob-iq', 'durable-policy', 'preparation', 'operator', 'delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy'] for target in TARGETS]


def validate_warm_order(job):
    first = job.index('          python3 tools/check_provider_dependencies.py\n')
    prefix = job[:first]
    for command in root_warm_commands() + provider_warm_commands():
        require(prefix.splitlines().count('          ' + command) == 1,
                'every checked graph must warm online and locked before the first offline guard')
    require(not re.search(r'^          cargo (?:build|test|clippy) ', prefix, re.M),
            'source qualification must precede compilation')


def operator_commands():
    commands = [TOOLCHAIN] + root_warm_commands() + provider_warm_commands()
    commands += ['python3 tools/check_provider_dependencies.py',
                 'cargo fmt --manifest-path ' + REL + '/Cargo.toml -- --check']
    for feature in ['durable-policy', 'operator']:
        commands += ['cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --features ' + feature + ' -- -D warnings',
                     'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --features ' + feature + ' -- --test-threads=1']
    commands += ['cargo build --manifest-path ' + REL + '/Cargo.toml --bin faction-provider --features operator --release --locked',
                 'cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked',
                 'umask 077', 'mkdir -m 700 "$RUNNER_TEMP/faction284-console-OWNED"',
                 'printf \'%s\\n\' \'Owned maintained-provider operator proof\' > "$RUNNER_TEMP/faction284-console-OWNED/OWNED"',
                 'python3 tools/probe_provider_operator.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --operator "$CARGO_TARGET_DIR/release/faction-provider" --owned-root "$RUNNER_TEMP/faction284-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction284-console-OWNED/result.json" --prepared']
    return commands


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
    commands += ['cargo metadata --locked --manifest-path ' + REL + '/Cargo.toml --format-version 1 --filter-platform ' + target + ' --features ' + feature + ' > /dev/null' for feature in ['durable-policy', 'preparation', 'operator', 'delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy'] for target in TARGETS]
    commands += ['python3 tools/check_provider_dependencies.py',
                 'cargo fmt --manifest-path ' + REL + '/Cargo.toml -- --check',
                 'cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked -- -D warnings',
                 'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked -- --test-threads=1',
                 'cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --features blob-iq -- -D warnings',
                 'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --features blob-iq -- --test-threads=1']
    commands += ['cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --features preparation -- -D warnings',
                 'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --features preparation -- --test-threads=1']
    for feature in ['delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy']:
        commands += ['cargo clippy --manifest-path ' + REL + '/Cargo.toml --all-targets --release --locked --features ' + feature + ' -- -D warnings',
                     'cargo test --manifest-path ' + REL + '/Cargo.toml --release --locked --features ' + feature + ' -- --test-threads=1']
    require(lines == commands, 'provider CI must qualify exact selected sources before locked validation')
    native = _step(provider, 'Validate the native authenticated attachment client')
    for setting in ['CARGO_BUILD_JOBS: 2', 'RUSTUP_TOOLCHAIN: 1.96.0', 'CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider']:
        require(native.splitlines().count('          ' + setting) == 1, 'native provider CI resource/toolchain environment changed')
    commands = ['cargo metadata --locked --format-version 1 --filter-platform ' + target
                + ' --features faction-client/device-provider > /dev/null' for target in TARGETS]
    commands += ['python3 tools/check_provider_dependencies.py',
                 'cargo clippy -p faction-client --features device-provider --all-targets --release --locked -- -D warnings']
    commands += [
        'cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked',
        'umask 077',
        'mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED"',
        'cp "$CARGO_TARGET_DIR/release/faction-client" "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"',
        'sha256sum "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"',
        'mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED/captures"',
        'env FACTION_SETUP_CLIENT_EXE="$RUNNER_TEMP/faction286-setup-OWNED/faction-client" FACTION_SETUP_CAPTURE_DIR="$RUNNER_TEMP/faction286-setup-OWNED/captures" cargo test -p faction-client --features device-provider --all-targets --release --locked -- --test-threads=1',
        'mkdir -m 700 "$RUNNER_TEMP/faction280-console-OWNED"',
        "printf '%s\\n' 'Owned authenticated-provider console proof' > \"$RUNNER_TEMP/faction280-console-OWNED/OWNED\"",
        'python3 tools/probe_device_provider_console.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --owned-root "$RUNNER_TEMP/faction280-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction280-console-OWNED/result.json"',
    ]
    require(_lines(native, {'cargo', 'python3', 'umask', 'mkdir', 'printf', 'cp', 'sha256sum', 'env'}) == commands,
            'native provider CI must qualify selected sources before all-target locked validation')
    # Every fresh job must materialize every graph read by the offline guard.
    warm = _step(provider, 'Warm the selected native attachment source graphs')
    require(_lines(warm, {'rustup', 'cargo'}) == [TOOLCHAIN] + root_warm_commands(),
            'provider root warmup must cover every checked ordinary graph online and locked')
    validate_warm_order(provider)
    operator = _job(text, 'provider_operator')
    require(not re.search(r'^    (?:if|continue-on-error|shell|strategy):', operator, re.M), 'conditional operator CI job refused')
    for setting in ['runs-on: ubuntu-24.04', 'timeout-minutes: 15']:
        require(operator.splitlines().count('    ' + setting) == 1, 'operator CI platform or timeout changed')
    selected = _step(operator, 'Validate durable provider policy and operator')
    for setting in ['CARGO_BUILD_JOBS: 2', 'RUSTUP_TOOLCHAIN: 1.96.0', 'CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider-operator']:
        require(selected.splitlines().count('          ' + setting) == 1, 'operator CI resource/toolchain environment changed')
    require(_lines(selected, {'rustup', 'cargo', 'python3', 'umask', 'mkdir', 'printf'}) == operator_commands(),
            'operator CI must qualify all selected sources before locked API, CLI and native proof')
    validate_warm_order(operator)
    documentation = _job(text, 'documentation')
    require('    needs: [native_mail, provider, provider_operator]' in documentation.splitlines()
            and '    if: ${{ always() }}' in documentation.splitlines()
            and not re.search(r'^    (?:continue-on-error|shell):', documentation, re.M),
            'documentation must require all optional job results')
    gate = _step(documentation, 'Require optional native tests')
    for name, job in [('NATIVE_MAIL_RESULT', 'native_mail'), ('PROVIDER_RESULT', 'provider'), ('PROVIDER_OPERATOR_RESULT', 'provider_operator')]:
        require(('          ' + name + ': ${{ needs.' + job + '.result }}') in gate.splitlines(), 'required CI result environment missing')
    require(_lines(gate, {'test'}) == ['test "$NATIVE_MAIL_RESULT" = success', 'test "$PROVIDER_RESULT" = success', 'test "$PROVIDER_OPERATOR_RESULT" = success'],
            'documentation must fail closed on both optional test results')


def inventory():
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    validate_profile(tomllib.loads(MANIFEST.read_text()), workspace)
    validate_ci((ROOT / '.github/workflows/documentation.yml').read_text())
    product = {'Cargo.toml': workspace}
    for member in workspace['workspace'].get('members', []):
        for path in ROOT.glob(member.rstrip('/') + '/Cargo.toml'):
            product[path.relative_to(ROOT).as_posix()] = tomllib.loads(path.read_text())
    validate_product_edges(product)
    lock = MANIFEST.parent / 'Cargo.lock'
    checks = existing_checks(tomllib.loads((ROOT / 'Cargo.lock').read_text()), tomllib.loads(lock.read_text()))
    # An optional manifest edge is not a selected runtime edge. Check all three
    # ordinary product selections on every adopted platform before source review.
    for features in [[], ['faction-client/native-mail'], ['faction-client/network-mail']]:
        for target in TARGETS:
            command = ['cargo', 'metadata', '--locked', '--offline', '--manifest-path', str(ROOT / 'Cargo.toml'),
                       '--format-version', '1', '--filter-platform', target]
            if features:
                command += ['--features', ','.join(features)]
            validate_graph(json.loads(subprocess.check_output(command, text=True)), False)
    root_checks = existing_checks(tomllib.loads((ROOT / 'Cargo.lock').read_text()),
                                  tomllib.loads((ROOT / 'Cargo.lock').read_text()))
    profiles = {}; verified = {}
    for profile, features in [('default', []), ('blob-iq', ['blob-iq']),
                              ('durable-policy', ['durable-policy']), ('preparation', ['preparation']),
                              ('operator', ['operator']),
                              ('delegation-records', ['delegation-records']),
                              ('delegation-records,durable-policy', ['delegation-records', 'durable-policy']),
                              ('delegation-services', ['delegation-services']),
                              ('delegation-services,durable-policy', ['delegation-services', 'durable-policy']),
                              ('delegation-services,delegation-records,durable-policy', ['delegation-services', 'delegation-records', 'durable-policy']),
                              ('device-provider', ['faction-client/device-provider'])]:
        graphs = {}
        for target in TARGETS:
            selected_manifest = ROOT / 'Cargo.toml' if profile == 'device-provider' else MANIFEST
            command = ['cargo', 'metadata', '--locked', '--offline', '--manifest-path', str(selected_manifest),
                       '--format-version', '1', '--filter-platform', target]
            if features:
                command += ['--features', ','.join(features)]
            metadata = json.loads(subprocess.check_output(command, text=True))
            if profile == 'device-provider':
                validate_graph(metadata, True)
            selected_checks = root_checks if profile == 'device-provider' else checks
            rows = []; local = []
            for package, selected_features in selected_packages(metadata):
                path = Path(package['manifest_path'])
                if package.get('source') is None:
                    local.append(scan_local(package, selected_features, ROOT))
                    continue
                key = (package['name'], package['version'], package['source'])
                require(key in selected_checks, 'selected source missing from qualified lock')
                require('vendored' not in selected_features, 'provider cannot silently vendor OpenSSL')
                row = scan_package(package, selected_features, selected_checks[key])
                if key not in verified:
                    source = path.parent
                    archive = source.parents[2] / 'cache' / source.parent.name / (package['name'] + '-' + package['version'] + '.crate')
                    verified[key] = verify_archive_source(source, archive, selected_checks[key])
                row.update(verified[key]); rows.append(row)
            graphs[target] = dict(registry=sorted(rows, key=lambda package: (package['name'], package['version'])),
                                  local=sorted(local, key=lambda package: package['name']))
        profiles[profile] = dict(selected_features=features, graphs=graphs)
    return json.dumps(dict(schema=4, scope='explicit-provider-embedding',
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
