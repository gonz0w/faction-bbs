"""Qualification contracts for the explicit opt-in provider embedding package."""
import copy
import json
import re
import tempfile
import unittest
import tomllib
from pathlib import Path
from unittest.mock import patch

import check_provider_dependencies as policy
import check_network_mail_dependencies as network
import check_provider_auth_dependencies as auth
import test_network_mail_policy as network_tests


def manifest():
    return {'package': {'name': 'faction-provider', 'version': '0.1.0-alpha.1',
            'edition': '2024', 'rust-version': '1.96', 'license': 'BSD-3-Clause', 'publish': False},
            'workspace': {}, 'features': {'default': [], 'blob-iq': ['dep:faction-xmpp'],
                'durable-policy': ['dep:rustix'], 'delegation-records': [], 'delegation-services': [], 'delegation-issuer': ['delegation-records', 'delegation-services'], 'delegation-wire': ['delegation-issuer', 'dep:faction-delegation-codec'], 'delegation-verifier': ['delegation-wire'],
                'preparation': ['dep:toml', 'toml/display'],
                'operator': ['durable-policy', 'preparation', 'dep:faction-unix']},
            'dependencies': {'faction-delegation-codec': {'path': '../faction-delegation-codec', 'optional': True, 'default-features': False}, 'faction-files': {'path': '../faction-files'}, 'openssl': '=0.10.81',
                             'faction-xmpp': {'path': '../faction-xmpp', 'optional': True},
                'rustix': {'version': '=1.1.5', 'default-features': False, 'features': ['std', 'fs', 'process'], 'optional': True},
                'toml': {'version': '=1.1.8', 'default-features': False, 'features': ['std', 'parse', 'serde'], 'optional': True},
                'faction-unix': {'path': '../faction-unix', 'optional': True}},
            'bin': [{'name': 'faction-provider', 'path': 'src/main.rs', 'required-features': ['operator']}]}


class ProviderDependencies(unittest.TestCase):
    def test_verifier_exact_feature_and_separate_bounded_ci_consumers(self):
        actual=tomllib.loads(policy.MANIFEST.read_text())
        self.assertEqual(actual['features'].get('delegation-verifier'),['delegation-wire'])
        for feature in ['delegation-verifier','delegation-verifier,durable-policy']:
            for target in policy.TARGETS:
                self.assertIn('cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform '+target+' --features '+feature+' > /dev/null',policy.provider_warm_commands())
        workflow=(policy.ROOT/'.github/workflows/documentation.yml').read_text()
        for feature,job in [('delegation-verifier','provider'),('delegation-verifier,durable-policy','provider_operator')]:
            section=policy._job(workflow,job)
            for command in ['cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features '+feature+' -- -D warnings','cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features '+feature+' -- --test-threads=1']:
                self.assertEqual(sum(line.strip()==command for line in workflow.splitlines()),1)
                self.assertIn(command,section)
        policy.validate_ci(workflow)

    def test_issuer_is_exact_default_off_core_composition_and_qualified_before_build(self):
        actual = tomllib.loads(policy.MANIFEST.read_text())
        self.assertEqual(actual['features'].get('delegation-issuer'), ['delegation-records', 'delegation-services'])
        self.assertNotIn('dep:faction-delegation-codec', actual['features']['delegation-issuer'])
        self.assertNotIn('delegation-wire', actual['features']['delegation-issuer'])
        for feature in ['delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier,durable-policy']:
            for target in policy.TARGETS:
                command = ('cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml '
                           '--format-version 1 --filter-platform ' + target + ' --features ' + feature + ' > /dev/null')
                self.assertIn(command, policy.provider_warm_commands())
            for command in ['cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features ' + feature + ' -- -D warnings',
                            'cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features ' + feature + ' -- --test-threads=1']:
                self.assertIn(command, policy.operator_commands())
        policy.validate_ci((policy.ROOT / '.github/workflows/documentation.yml').read_text())

    def test_actual_services_selection_is_empty_default_off_and_qualified(self):
        actual = tomllib.loads(policy.MANIFEST.read_text())
        self.assertEqual(actual['features'].get('delegation-services'), [])
        policy.validate_profile(actual, {'workspace': {'members': []}})
        for feature in ['delegation-services', 'delegation-services,durable-policy',
                        'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy']:
            for target in policy.TARGETS:
                command = ('cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml '
                           '--format-version 1 --filter-platform ' + target + ' --features ' + feature + ' > /dev/null')
                self.assertIn(command, policy.provider_warm_commands())
        policy.validate_ci((policy.ROOT / '.github/workflows/documentation.yml').read_text())

    def test_exact_standalone_profile(self):
        policy.validate_profile(manifest(), {'workspace': {'members': ['crates/faction-files']}})

    def test_automatic_product_membership_and_edges_refused(self):
        for member in ['crates/faction-provider', 'crates/*', 'crates/faction-*']:
            with self.subTest(member=member), self.assertRaises(ValueError):
                policy.validate_profile(manifest(), {'workspace': {'members': [member]}})
        workspace = {'workspace': {'members': ['crates/faction-client']}}
        active = {'dependencies': {'faction-provider': {'path': '../faction-provider', 'optional': True}}}
        with self.assertRaises(ValueError):
            policy.validate_product_edges({'client': active})
        with self.assertRaises(ValueError):
            policy.validate_product_edges({'client': {'dependencies': {'renamed': {'path': '../faction-provider'}}}})
        policy.validate_product_edges({'client': {'dependencies': {'existing': '1'}}})

    def test_profile_drift_features_vendoring_and_replacements_refused(self):
        cases = [('patch', {}), ('replace', {}), ('features', {'default': ['online']}),
                 ('dev-dependencies', {'extra': '*'}), ('build-dependencies', {'extra': '*'}),
                 ('dependencies', {'openssl': {'version': '=0.10.81', 'features': ['vendored']}})]
        for key, value in cases:
            changed = manifest(); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})
        for key, value in [('publish', True), ('license', 'MIT'), ('version', '0.0.0'), ('edition', '2021')]:
            changed = manifest(); changed['package'][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})
        changed = manifest(); changed['dependencies']['faction-files']['path'] = '../../elsewhere'
        with self.assertRaises(ValueError):
            policy.validate_profile(changed, {'workspace': {'members': []}})
        with self.assertRaises(ValueError):
            policy.validate_profile(manifest(), {'workspace': {'members': []}, 'patch': {}})

    def test_iq_feature_is_exact_optional_and_default_off(self):
        changes = [({'default': ['blob-iq'], 'blob-iq': ['dep:faction-xmpp']}, None),
                   ({'default': [], 'blob-iq': ['dep:faction-files']}, None),
                   ({'default': [], 'blob-iq': ['dep:faction-xmpp'], 'extra': []}, None),
                   (None, {'path': '../faction-xmpp'}),
                   (None, {'path': '../other', 'optional': True}),
                   (None, {'path': '../faction-xmpp', 'optional': True, 'features': ['extra']})]
        for features, dependency in changes:
            changed = manifest()
            if features is not None: changed['features'] = features
            if dependency is not None: changed['dependencies']['faction-xmpp'] = dependency
            with self.subTest(profile=changed), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})

    def test_inventory_qualifies_default_and_iq_graphs_on_every_target(self):
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(policy.subprocess, 'check_output', return_value=metadata) as execute, patch.object(policy, 'validate_ci'), patch.object(policy, 'validate_product_edges'), patch.object(policy, 'validate_graph'):
            result = json.loads(policy.inventory())
        self.assertEqual(set(result['profiles']), {'default', 'blob-iq', 'device-provider', 'durable-policy', 'preparation', 'operator', 'delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy'})
        self.assertEqual(execute.call_count, 60)
        expected = {(target, feature) for target in policy.TARGETS for feature in [None, 'blob-iq', 'durable-policy', 'preparation', 'operator', 'delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy', 'faction-client/device-provider', 'faction-client/native-mail', 'faction-client/network-mail']}
        actual = set()
        for call in execute.call_args_list:
            command = call.args[0]
            self.assertIn('--locked', command); self.assertIn('--offline', command)
            feature = command[command.index('--features') + 1] if '--features' in command else None
            actual.add((command[command.index('--filter-platform') + 1], feature))
        self.assertEqual(actual, expected)
        for profile in result['profiles'].values():
            self.assertEqual(set(profile['graphs']), set(policy.TARGETS))


    def test_exact_client_device_provider_edge_is_accepted(self):
        policy.validate_product_edges({'crates/faction-client/Cargo.toml': client_manifest()})

    def test_device_provider_edge_cannot_leak_or_change_other_profiles(self):
        original = client_manifest()
        for section in ['dependencies', 'build-dependencies', 'dev-dependencies']:
            for target in [None, 'cfg(unix)']:
                changed = copy.deepcopy(original)
                deps = changed if target is None else changed.setdefault('target', {}).setdefault(target, {})
                deps.setdefault(section, {})['renamed'] = {'package': 'faction-provider', 'path': '../faction-provider'}
                with self.subTest(section=section, target=target), self.assertRaises(ValueError):
                    policy.validate_product_edges({'crates/faction-client/Cargo.toml': changed})
        changes = [(['dependencies', 'faction-provider', 'optional'], False),
                   (['dependencies', 'faction-provider', 'default-features'], True),
                   (['dependencies', 'faction-provider', 'path'], '../other'),
                   (['features', 'default'], ['device-provider']),
                   (['features', 'native-mail'], ['device-provider']),
                   (['features', 'network-mail'], ['device-provider']),
                   (['features', 'device-provider'], ['network-mail', 'dep:faction-provider']),
                   (['features', 'alias'], ['device-provider']), (['patch'], {}), (['replace'], {})]
        for keys, value in changes:
            changed = copy.deepcopy(original); destination = changed
            for key in keys[:-1]: destination = destination[key]
            destination[keys[-1]] = value
            with self.subTest(keys=keys), self.assertRaises(ValueError):
                policy.validate_product_edges({'crates/faction-client/Cargo.toml': changed})
        for dependency in ['*', {'workspace': True}, {'path': '../faction-provider', 'optional': True}]:
            changed = copy.deepcopy(original); changed['dependencies']['faction-provider'] = dependency
            with self.subTest(dependency=dependency), self.assertRaises(ValueError):
                policy.validate_product_edges({'crates/faction-client/Cargo.toml': changed})
        with self.assertRaises(ValueError):
            policy.validate_product_edges({'crates/faction-other/Cargo.toml': original})

    def test_optional_manifest_edges_do_not_activate_ordinary_graphs(self):
        provider = {'id': 'provider', 'name': 'faction-provider'}
        empty = {'packages': [], 'resolve': {'nodes': []}}
        active = {'packages': [provider], 'resolve': {'nodes': [{'id': 'provider', 'features': ['preparation']}]}}
        policy.validate_graph(empty, False)
        policy.validate_graph(active, True)
        with self.assertRaises(ValueError): policy.validate_graph(active, False)
        with self.assertRaises(ValueError): policy.validate_graph(empty, True)
        active['resolve']['nodes'][0]['features'] = ['blob-iq']
        with self.assertRaises(ValueError): policy.validate_graph(active, True)

    def test_reuses_existing_verified_source_and_lock_functions(self):
        for name in ['selected_packages', 'scan_package', 'scan_local', 'verify_archive_source', 'scan_tree']:
            self.assertIs(getattr(policy, name), getattr(network, name))
        self.assertIs(policy.existing_checks, auth.existing_checks)
        package = {'name': 'openssl', 'version': '0.10.81', 'source': policy.REGISTRY, 'checksum': 'a' * 64}
        root = {'package': [package]}
        self.assertEqual(len(policy.existing_checks(root, root)), 1)
        for key, value in [('version', '0.10.82'), ('checksum', 'b' * 64), ('source', 'git+https://example.invalid')]:
            changed = copy.deepcopy(root); changed['package'][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.existing_checks(root, changed)

    def test_source_archive_checksum_license_and_no_js_refusals(self):
        fixtures = network_tests.NetworkMailPolicy()
        with tempfile.TemporaryDirectory() as temporary:
            root, archive, checksum = fixtures.fixture(Path(temporary))
            package = fixtures.package(root)
            self.assertEqual(policy.verify_archive_source(root, archive, checksum)['source_files'], 3)
            for field, value in [('license', 'unknown'), ('source', 'git+https://example.invalid')]:
                changed = dict(package, **{field: value})
                with self.assertRaises(ValueError): policy.scan_package(changed, [], checksum)
            for name in ['build/tool.ts', '.github/helper.js', 'fixtures/package.json']:
                path = root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text('unreviewed')
                with self.assertRaises(ValueError): policy.scan_package(package, [], checksum)
                path.unlink()
            with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, '0' * 64)
            (root / 'src/lib.rs').write_text('tampered')
            with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, checksum)

    def test_inventory_drift_is_read_only_until_deliberate_write(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'provider.json'; path.write_text('reviewed')
            policy.check_inventory(path, 'reviewed')
            with self.assertRaises(ValueError): policy.check_inventory(path, 'unreviewed')
            self.assertEqual(path.read_text(), 'reviewed')


def client_manifest():
    return {'features': {'default': [],
        'native-mail': ['dep:faction-omemo', 'faction-omemo/native'],
        'network-mail': ['native-mail', 'dep:faction-xmpp', 'dep:tokio', 'dep:roxmltree'],
        'device-provider': ['network-mail', 'dep:faction-provider', 'faction-provider/preparation', 'tokio/sync']},
        'dependencies': {'faction-provider': {'path': '../faction-provider', 'optional': True, 'default-features': False}}}


def native_ci_step():
    commands = ['cargo metadata --locked --format-version 1 --filter-platform ' + target + ' --features faction-client/device-provider > /dev/null' for target in policy.TARGETS]
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
    return ('      - name: Validate the native authenticated attachment client\n        env:\n          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider\n          CARGO_BUILD_JOBS: 2\n          RUSTUP_TOOLCHAIN: 1.96.0\n        run: |\n'
            + ''.join('          ' + command + '\n' for command in commands))


def joined_ci_fixture():
    return """jobs:
  provider:
    name: Optional attachment provider
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - name: Fetch the public repository revision
        env:
          REPOSITORY_URL: ${{ github.server_url }}/${{ github.repository }}.git
          CHECKOUT_SHA: ${{ github.sha }}
        run: |
          git init .
          git remote add origin "$REPOSITORY_URL"
          git fetch --no-tags --depth=1 origin "$CHECKOUT_SHA"
          git checkout --detach FETCH_HEAD
      - name: Warm the selected native attachment source graphs
        env:
          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider
          CARGO_BUILD_JOBS: 2
          RUSTUP_TOOLCHAIN: 1.96.0
        run: |
          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/device-provider > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/device-provider > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/device-provider > /dev/null
      - name: Validate the opt-in attachment provider
        env:
          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider
          CARGO_BUILD_JOBS: 2
          RUSTUP_TOOLCHAIN: 1.96.0
        run: |
          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-verifier,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-verifier,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-verifier,durable-policy > /dev/null
          python3 tools/check_provider_dependencies.py
          cargo fmt --manifest-path crates/faction-provider/Cargo.toml -- --check
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features blob-iq -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features blob-iq -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features preparation -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features preparation -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-records -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-records -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-records,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-records,durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-verifier -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-verifier -- --test-threads=1
      - name: Validate the native authenticated attachment client
        env:
          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider
          CARGO_BUILD_JOBS: 2
          RUSTUP_TOOLCHAIN: 1.96.0
        run: |
          cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/device-provider > /dev/null
          cargo metadata --locked --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/device-provider > /dev/null
          cargo metadata --locked --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/device-provider > /dev/null
          python3 tools/check_provider_dependencies.py
          cargo clippy -p faction-client --features device-provider --all-targets --release --locked -- -D warnings
          cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked
          umask 077
          mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED"
          cp "$CARGO_TARGET_DIR/release/faction-client" "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"
          sha256sum "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"
          mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED/captures"
          env FACTION_SETUP_CLIENT_EXE="$RUNNER_TEMP/faction286-setup-OWNED/faction-client" FACTION_SETUP_CAPTURE_DIR="$RUNNER_TEMP/faction286-setup-OWNED/captures" cargo test -p faction-client --features device-provider --all-targets --release --locked -- --test-threads=1
          mkdir -m 700 "$RUNNER_TEMP/faction280-console-OWNED"
          printf '%s\\n' 'Owned authenticated-provider console proof' > "$RUNNER_TEMP/faction280-console-OWNED/OWNED"
          python3 tools/probe_device_provider_console.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --owned-root "$RUNNER_TEMP/faction280-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction280-console-OWNED/result.json"
  provider_operator:
    name: Optional durable provider operator
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - name: Fetch the public repository revision
        env:
          REPOSITORY_URL: ${{ github.server_url }}/${{ github.repository }}.git
          CHECKOUT_SHA: ${{ github.sha }}
        run: |
          git init .
          git remote add origin "$REPOSITORY_URL"
          git fetch --no-tags --depth=1 origin "$CHECKOUT_SHA"
          git checkout --detach FETCH_HEAD
      - name: Validate durable provider policy and operator
        env:
          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider-operator
          CARGO_BUILD_JOBS: 2
          RUSTUP_TOOLCHAIN: 1.96.0
        run: |
          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/native-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/network-mail > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/device-provider > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/device-provider > /dev/null
          cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/device-provider > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features blob-iq > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features preparation > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features operator > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-records > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-services,delegation-records,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-issuer > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-issuer,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-wire > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-wire,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-verifier > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features delegation-verifier,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features delegation-verifier,durable-policy > /dev/null
          cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features delegation-verifier,durable-policy > /dev/null
          python3 tools/check_provider_dependencies.py
          cargo fmt --manifest-path crates/faction-provider/Cargo.toml -- --check
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features operator -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features operator -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-services -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-services -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-services,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-services,durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-services,delegation-records,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-services,delegation-records,durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-issuer -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-issuer -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-issuer,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-issuer,durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-wire -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-wire -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-wire,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-wire,durable-policy -- --test-threads=1
          cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features delegation-verifier,durable-policy -- -D warnings
          cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features delegation-verifier,durable-policy -- --test-threads=1
          cargo build --manifest-path crates/faction-provider/Cargo.toml --bin faction-provider --features operator --release --locked
          cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked
          umask 077
          mkdir -m 700 "$RUNNER_TEMP/faction284-console-OWNED"
          printf '%s\\n' 'Owned maintained-provider operator proof' > "$RUNNER_TEMP/faction284-console-OWNED/OWNED"
          python3 tools/probe_provider_operator.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --operator "$CARGO_TARGET_DIR/release/faction-provider" --owned-root "$RUNNER_TEMP/faction284-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction284-console-OWNED/result.json" --prepared
  documentation:
    name: Documentation and policy
    needs: [native_mail, provider, provider_operator]
    if: ${{ always() }}
    runs-on: ubuntu-24.04
    timeout-minutes: 15
    steps:
      - name: Require optional native tests
        env:
          NATIVE_MAIL_RESULT: ${{ needs.native_mail.result }}
          PROVIDER_RESULT: ${{ needs.provider.result }}
          PROVIDER_OPERATOR_RESULT: ${{ needs.provider_operator.result }}
        run: |
          test "$NATIVE_MAIL_RESULT" = success
          test "$PROVIDER_RESULT" = success
          test "$PROVIDER_OPERATOR_RESULT" = success
"""


def ci_fixture():
    source = joined_ci_fixture()
    native = native_ci_step()
    assert source.count(native) == 1
    provider = policy._job(source, 'provider')
    prefix = provider[:provider.index('          python3 tools/check_provider_dependencies.py\n')]
    metadata = [line for line in prefix.splitlines() if line.strip().startswith('cargo metadata ')]
    assert len(metadata) == 60
    fetch = '      - name: Fetch the public repository revision\n' + policy._step(provider, 'Fetch the public repository revision')
    client = ('  provider_client:\n'
              '    name: Optional authenticated provider client\n'
              '    runs-on: ubuntu-24.04\n'
              '    timeout-minutes: 15\n'
              '    steps:\n' + fetch +
              '      - name: Warm the selected provider client source graphs\n'
              '        env:\n'
              '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider\n'
              '          CARGO_BUILD_JOBS: 2\n'
              '          RUSTUP_TOOLCHAIN: 1.96.0\n'
              '        run: |\n'
              '          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy\n'
              + '\n'.join(metadata) + '\n' + native)
    source = source.replace(native, '').replace('  provider_operator:\n', client + '  provider_operator:\n', 1)
    source = source.replace('needs: [native_mail, provider, provider_operator]',
                            'needs: [native_mail, provider, provider_operator, provider_client]')
    source = source.replace('          PROVIDER_OPERATOR_RESULT: ${{ needs.provider_operator.result }}\n',
                            '          PROVIDER_OPERATOR_RESULT: ${{ needs.provider_operator.result }}\n'
                            '          PROVIDER_CLIENT_RESULT: ${{ needs.provider_client.result }}\n')
    return source.replace('          test "$PROVIDER_OPERATOR_RESULT" = success\n',
                           '          test "$PROVIDER_OPERATOR_RESULT" = success\n'
                           '          test "$PROVIDER_CLIENT_RESULT" = success\n')


class ProviderClientCI(unittest.TestCase):
    def test_complete_split_client_layout_is_accepted(self):
        policy.validate_ci(ci_fixture())

    def test_joined_client_and_provider_layout_is_refused(self):
        with self.assertRaises(ValueError):
            policy.validate_ci(joined_ci_fixture())

    def test_repository_runs_client_in_its_own_required_job(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        client = policy._job(source, 'provider_client')
        self.assertNotIn(native_ci_step(), policy._job(source, 'provider'))
        self.assertIn(native_ci_step(), client)
        self.assertEqual(source.count(native_ci_step()), 1)
        policy.validate_warm_order(client)
        documentation = policy._job(source, 'documentation')
        self.assertIn('    needs: [native_mail, provider, provider_operator, provider_client]', documentation.splitlines())
        self.assertIn('          PROVIDER_CLIENT_RESULT: ${{ needs.provider_client.result }}', documentation.splitlines())
        self.assertIn('          test "$PROVIDER_CLIENT_RESULT" = success', documentation.splitlines())
        policy.validate_ci(source)

    def test_client_warmup_source_guard_and_each_validation_command_are_required(self):
        source = ci_fixture()
        client = policy._job(source, 'provider_client')
        for line in client.splitlines():
            if line.strip().startswith(('rustup ', 'cargo ', 'python3 ', 'umask ', 'mkdir ', 'printf ', 'cp ', 'sha256sum ', 'env ', 'git ')):
                with self.subTest(missing=line), self.assertRaises(ValueError):
                    policy.validate_ci(source.replace(client, client.replace(line + '\n', '', 1)))
        first = client.index('          python3 tools/check_provider_dependencies.py\n')
        warm = policy._step(client, 'Warm the selected provider client source graphs')
        metadata = [line for line in warm.splitlines() if line.strip().startswith('cargo metadata ')]
        self.assertEqual(len(metadata), 60)
        self.assertEqual(sum(line.strip().startswith('cargo metadata ') for line in client[:first].splitlines()), 63)
        for line in metadata:
            for altered in [line.replace(' --locked', ''), line.replace(' --locked', ' --locked --offline')]:
                with self.subTest(altered=altered), self.assertRaises(ValueError):
                    policy.validate_ci(source.replace(client, client.replace(line, altered, 1)))
        premature = client.replace('          python3 tools/check_provider_dependencies.py\n',
                                   '          cargo build --release --locked\n          python3 tools/check_provider_dependencies.py\n', 1)
        with self.assertRaises(ValueError):
            policy.validate_ci(source.replace(client, premature))

    def test_client_job_cannot_skip_change_limits_duplicate_or_escape_required_results(self):
        source = ci_fixture()
        client = policy._job(source, 'provider_client')
        for old, new in [('timeout-minutes: 15', 'timeout-minutes: 30'),
                         ('runs-on: ubuntu-24.04', 'runs-on: ubuntu-latest'),
                         ('CARGO_BUILD_JOBS: 2', 'CARGO_BUILD_JOBS: 4'),
                         ('RUSTUP_TOOLCHAIN: 1.96.0', 'RUSTUP_TOOLCHAIN: stable'),
                         ('    steps:', '    if: false\n    steps:'),
                         ('    steps:', '    continue-on-error: true\n    steps:'),
                         ('        run: |', '        if: false\n        run: |'),
                         ('        run: |', '        continue-on-error: true\n        run: |')]:
            with self.subTest(altered=new), self.assertRaises(ValueError):
                policy.validate_ci(source.replace(client, client.replace(old, new)))
        with self.assertRaises(ValueError):
            policy.validate_ci(source + '  provider_client:\n' + client)
        with self.assertRaises(ValueError):
            policy.validate_ci(source.replace('  provider_operator:\n', native_ci_step() + '  provider_operator:\n', 1))
        for old, new in [('needs: [native_mail, provider, provider_operator, provider_client]', 'needs: [native_mail, provider, provider_operator]'),
                         ('          PROVIDER_CLIENT_RESULT: ${{ needs.provider_client.result }}\n', ''),
                         ('PROVIDER_CLIENT_RESULT: ${{ needs.provider_client.result }}', 'PROVIDER_CLIENT_RESULT: success'),
                         ('          test "$PROVIDER_CLIENT_RESULT" = success\n', ''),
                         ('test "$PROVIDER_CLIENT_RESULT" = success', 'test "$PROVIDER_CLIENT_RESULT" = success || true')]:
            with self.subTest(altered=new), self.assertRaises(ValueError):
                policy.validate_ci(source.replace(old, new))


class ProviderCI(unittest.TestCase):
    def assert_root_graphs_warmed_before_source_guard(self, source):
        provider = policy._job(source, 'provider')
        name = 'Warm the selected native attachment source graphs'
        step = policy._step(provider, name)
        for setting in ['CARGO_BUILD_JOBS: 2', 'RUSTUP_TOOLCHAIN: 1.96.0',
                        'CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider']:
            self.assertEqual(step.splitlines().count('          ' + setting), 1)
        metadata = []
        for feature in [None, 'native-mail', 'network-mail', 'device-provider']:
            for target in policy.TARGETS:
                command = 'cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform ' + target
                if feature:
                    command += ' --features faction-client/' + feature
                metadata.append(command + ' > /dev/null')
        self.assertEqual(policy._lines(step, {'rustup', 'cargo'}),
                         ['rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy'] + metadata)
        first_guard = provider.index('          python3 tools/check_provider_dependencies.py\n')
        prefix = provider[:first_guard]
        for command in metadata:
            self.assertEqual(prefix.splitlines().count('          ' + command), 1,
                             'every root graph must warm online before the first offline source guard')
        self.assertFalse(re.search(r'^          cargo (?:build|test|clippy) ', prefix, re.M),
                         'source qualification must precede compilation')

    def test_repository_warms_all_root_graphs_before_first_offline_provider_guard(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        self.assert_root_graphs_warmed_before_source_guard(source)

    def test_root_warmup_cannot_be_missing_offline_unlocked_or_after_guard(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        provider = policy._job(source, 'provider')
        step = policy._step(provider, 'Warm the selected native attachment source graphs')
        block = '      - name: Warm the selected native attachment source graphs\n' + step
        for line in step.splitlines():
            if line.strip().startswith('cargo metadata '):
                with self.subTest(missing=line), self.assertRaises((AssertionError, ValueError)):
                    self.assert_root_graphs_warmed_before_source_guard(source.replace(line + '\n', '', 1))
        for old, new in [('cargo metadata --locked --manifest-path Cargo.toml', 'cargo metadata --locked --offline --manifest-path Cargo.toml'),
                         ('cargo metadata --locked --manifest-path Cargo.toml', 'cargo metadata --manifest-path Cargo.toml')]:
            with self.subTest(change=new), self.assertRaises((AssertionError, ValueError)):
                self.assert_root_graphs_warmed_before_source_guard(source.replace(old, new))
        later = source.replace(block, '')
        later = later.replace('      - name: Validate the native authenticated attachment client\n',
                              block + '      - name: Validate the native authenticated attachment client\n')
        with self.assertRaises((AssertionError, ValueError)):
            self.assert_root_graphs_warmed_before_source_guard(later)

    def test_repository_provider_job_exists_and_is_required(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        self.assertIn('  provider:\n', source)
        policy.validate_ci(source)

    def test_repository_ci_exercises_explicit_blob_iq(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        job = source.split('  provider:\n', 1)[1].split('  documentation:', 1)[0]
        for target in policy.TARGETS:
            self.assertIn('--filter-platform ' + target + ' --features blob-iq > /dev/null', job)
        self.assertIn('--all-targets --release --locked --features blob-iq -- -D warnings', job)
        self.assertIn('--release --locked --features blob-iq -- --test-threads=1', job)

    def test_repository_ci_exercises_native_device_provider(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        self.assertIn(native_ci_step(), source)

    def test_native_selected_sources_and_all_targets_cannot_be_removed(self):
        good = ci_fixture()
        with self.assertRaises(ValueError): policy.validate_ci(good.replace(native_ci_step(), ''))
        for command in native_ci_step().splitlines():
            if command.strip().startswith(('cargo ', 'python3 ')):
                with self.subTest(command=command), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace(command + '\n', '', 1 if 'device-provider' in command else 2))

    def test_inventory_calls_ci_guard_before_metadata(self):
        with patch.object(policy, 'validate_ci', side_effect=ValueError('refused CI contract')) as validate:
            with self.assertRaisesRegex(ValueError, 'refused CI contract'):
                policy.inventory()
            validate.assert_called_once()

    def test_positive_straight_line_provider_job(self):
        policy.validate_ci(ci_fixture())

    def test_missing_conditional_inert_or_unlocked_commands_refused(self):
        good = ci_fixture()
        command = 'python3 tools/check_provider_dependencies.py'
        for old, new in [(command, '# ' + command), (command, 'echo ' + command),
                         (command, command + ' || true'), (' --locked', ''),
                         (' --test-threads=1', ''), ('    steps:', '    if: false\n    steps:'),
                         ('        run: |', '        if: false\n        run: |'),
                         ('CARGO_BUILD_JOBS: 2', 'CARGO_BUILD_JOBS: 8'),
                         ('timeout-minutes: 15', 'timeout-minutes: 60'),
                         ('    runs-on: ubuntu-24.04', '    runs-on: windows-latest')]:
            with self.subTest(change=new), self.assertRaises(ValueError):
                policy.validate_ci(good.replace(old, new))
        for wrapper in ["cat <<'INERT'\n", 'if false; then\n']:
            with self.assertRaises(ValueError):
                policy.validate_ci(good.replace('          ' + command, '          ' + wrapper + '          ' + command))
        test = 'cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked -- --test-threads=1'
        with self.assertRaises(ValueError):
            policy.validate_ci(good.replace('          ' + test + '\n', '').replace('          ' + command, '          ' + test + '\n          ' + command))

    def test_iq_metadata_and_runtime_coverage_cannot_be_removed(self):
        good = ci_fixture()
        for command in good.splitlines():
            if '--features blob-iq' in command:
                with self.subTest(command=command), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace(command + '\n', ''))

    def test_documentation_fails_closed_on_both_native_and_provider(self):
        good = ci_fixture()
        for old, new in [('needs: [native_mail, provider, provider_operator, provider_client]', 'needs: native_mail'),
                         ('          test "$PROVIDER_RESULT" = success\n', ''),
                         ('          test "$NATIVE_MAIL_RESULT" = success\n', ''),
                         ('test "$PROVIDER_RESULT" = success', 'test "$PROVIDER_RESULT" = success || true'),
                         ('if: ${{ always() }}', 'if: false'),
                         ('        run: |\n          test', '        if: false\n        run: |\n          test')]:
            with self.subTest(change=new), self.assertRaises(ValueError):
                policy.validate_ci(good.replace(old, new))


def durable_manifest():
    result = manifest()
    result['features'].update({'durable-policy': ['dep:rustix'],
        'preparation': ['dep:toml', 'toml/display'],
        'operator': ['durable-policy', 'preparation', 'dep:faction-unix']})
    result['dependencies'].update({
        'rustix': {'version': '=1.1.5', 'default-features': False,
                   'features': ['std', 'fs', 'process'], 'optional': True},
        'toml': {'version': '=1.1.8', 'default-features': False,
                 'features': ['std', 'parse', 'serde'], 'optional': True},
        'faction-unix': {'path': '../faction-unix', 'optional': True}})
    result['bin'] = [{'name': 'faction-provider', 'path': 'src/main.rs',
                      'required-features': ['operator']}]
    return result


class DurableProviderProfiles(unittest.TestCase):
    def test_exact_optional_durable_operator_profile_accepted(self):
        policy.validate_profile(durable_manifest(), {'workspace': {'members': []}})

    def test_actual_manifest_has_deliberate_operator_binary(self):
        import tomllib
        actual = tomllib.loads(policy.MANIFEST.read_text())
        self.assertEqual(actual, durable_manifest())

    def test_inventory_qualifies_durable_and_operator_on_every_target(self):
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(policy.subprocess, 'check_output', return_value=metadata), patch.object(policy, 'validate_ci'), patch.object(policy, 'validate_product_edges'), patch.object(policy, 'validate_graph'):
            result = json.loads(policy.inventory())
        self.assertEqual(set(result['profiles']), {'default', 'blob-iq', 'device-provider', 'durable-policy', 'preparation', 'operator', 'delegation-records', 'delegation-records,durable-policy', 'delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy'})
        for profile in ['durable-policy', 'operator']:
            self.assertEqual(set(result['profiles'][profile]['graphs']), set(policy.TARGETS))

    def test_ci_without_durable_operator_validation_is_refused(self):
        old = ci_fixture()
        job = '  provider_operator:\n' + policy._job(old, 'provider_operator')
        with self.assertRaises(ValueError):
            policy.validate_ci(old.replace(job, ''))

    def test_actual_workflow_selects_both_profiles_before_compile(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        job = policy._job(source, 'provider_operator')
        for feature in ['durable-policy', 'operator']:
            self.assertIn('--features ' + feature + ' -- -D warnings', job)
            self.assertIn('--features ' + feature + ' -- --test-threads=1', job)
        self.assertIn('needs: [native_mail, provider, provider_operator, provider_client]', source)


class DurableProviderRefusals(unittest.TestCase):
    def test_manifest_optional_edges_and_binary_selection_cannot_drift(self):
        changes = [(['features', 'default'], ['operator']),
                   (['features', 'durable-policy'], ['dep:rustix', 'dep:toml']),
                   (['features', 'operator'], ['durable-policy']),
                   (['dependencies', 'rustix', 'version'], '=1.1.6'),
                   (['dependencies', 'rustix', 'optional'], False),
                   (['dependencies', 'rustix', 'features'], ['std', 'fs']),
                   (['dependencies', 'toml', 'optional'], False),
                   (['dependencies', 'toml', 'features'], ['std', 'parse', 'serde', 'display']),
                   (['dependencies', 'faction-unix', 'path'], '../other')]
        for keys, value in changes:
            changed = durable_manifest(); target = changed
            for key in keys[:-1]: target = target[key]
            target[keys[-1]] = value
            with self.subTest(keys=keys), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})
        for key, value in [('required-features', []), ('path', 'src/other.rs'), ('name', 'other')]:
            changed = durable_manifest(); changed['bin'][0][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})

    def test_all_fresh_jobs_warm_each_graph_online_before_first_source_guard(self):
        good = ci_fixture()
        for jobname in ['provider', 'provider_operator', 'provider_client']:
            job = policy._job(good, jobname)
            first = job.index('          python3 tools/check_provider_dependencies.py\n')
            prefix = job[:first]
            metadata = [line for line in prefix.splitlines() if line.strip().startswith('cargo metadata ')]
            self.assertEqual(len(metadata), 63 if jobname == 'provider_client' else 60)
            for line in metadata:
                for replacement in ['', line.replace(' --locked', ''), line.replace(' --locked', ' --locked --offline')]:
                    changed = job.replace(line + '\n', replacement + '\n', 1)
                    with self.subTest(job=jobname, line=line, replacement=replacement), self.assertRaises(ValueError):
                        policy.validate_ci(good.replace(job, changed))
            # A full correct warmup after compilation is still too late.
            changed = job.replace('          python3 tools/check_provider_dependencies.py\n',
                '          cargo build --release --locked\n          python3 tools/check_provider_dependencies.py\n', 1)
            with self.assertRaises(ValueError): policy.validate_ci(good.replace(job, changed))

    def test_operator_validation_probe_and_required_result_cannot_be_bypassed(self):
        good = ci_fixture(); job = policy._job(good, 'provider_operator')
        for line in job.splitlines():
            if line.strip().startswith(('cargo ', 'python3 ')):
                with self.subTest(line=line), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace(job, job.replace(line + '\n', '', 1)))
        for old, new in [('needs: [native_mail, provider, provider_operator, provider_client]', 'needs: [native_mail, provider]'),
                         ('test "$PROVIDER_OPERATOR_RESULT" = success', 'test "$PROVIDER_OPERATOR_RESULT" = success || true'),
                         ('PROVIDER_OPERATOR_RESULT: ${{ needs.provider_operator.result }}', 'PROVIDER_OPERATOR_RESULT: success'),
                         ('  provider_operator:\n', '  provider_operator:\n    if: false\n')]:
            with self.subTest(new=new), self.assertRaises(ValueError): policy.validate_ci(good.replace(old, new))


class OperatorCIMarkerIntegration(unittest.TestCase):
    def test_actual_ci_marker_is_accepted_by_maintained_probe(self):
        import shlex
        import subprocess
        import probe_provider_operator as probe
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        job = policy._job(source, 'provider_operator')
        step = policy._step(job, 'Validate durable provider policy and operator')
        commands = policy._lines(step, {'rustup', 'cargo', 'python3', 'umask', 'mkdir', 'printf'})
        producers = [shlex.split(command) for command in commands if command.startswith('printf ')]
        self.assertEqual(len(producers), 1)
        producer = producers[0]
        self.assertEqual(len(producer), 5)
        self.assertEqual(producer[:2], ['printf', '%s\\n'])
        self.assertEqual(producer[3:], ['>', '$RUNNER_TEMP/faction284-console-OWNED/OWNED'])
        # Obtain the real printf bytes from the workflow, without copying a marker oracle
        # or executing the workflow as arbitrary shell code.
        result = subprocess.run(['/usr/bin/printf', *producer[1:3]], check=True, capture_output=True)
        with tempfile.TemporaryDirectory(prefix='provider-ci-marker-OWNED-') as temporary:
            root = Path(temporary) / 'faction284-console-OWNED'
            root.mkdir(mode=0o700)
            (root / 'OWNED').write_bytes(result.stdout)
            self.assertEqual(probe.require_root(root), root)


class PreparationProfiles(unittest.TestCase):
    def test_preparation_profile_is_accepted_without_operator_or_durability(self):
        expected = manifest()
        expected['features']['preparation'] = ['dep:toml', 'toml/display']
        expected['features']['operator'] = ['durable-policy', 'preparation', 'dep:faction-unix']
        policy.validate_profile(expected, {'workspace': {'members': []}})

    def test_native_selection_allows_only_preparation_and_toml(self):
        provider = {'id': 'provider', 'name': 'faction-provider'}
        metadata = {'packages': [provider], 'resolve': {'nodes': [
            {'id': 'provider', 'features': ['preparation']}]}}
        policy.validate_graph(metadata, True)
        for leaked in ['operator', 'durable-policy', 'blob-iq']:
            changed = copy.deepcopy(metadata)
            changed['resolve']['nodes'][0]['features'].append(leaked)
            with self.subTest(leaked=leaked), self.assertRaises(ValueError):
                policy.validate_graph(changed, True)

    def test_inventory_selects_preparation_on_all_three_platforms(self):
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(policy.subprocess, 'check_output', return_value=metadata) as execute, patch.object(policy, 'validate_ci'), patch.object(policy, 'validate_product_edges'), patch.object(policy, 'validate_graph'):
            result = json.loads(policy.inventory())
        self.assertIn('preparation', result['profiles'])
        self.assertEqual(set(result['profiles']['preparation']['graphs']), set(policy.TARGETS))
        self.assertEqual(execute.call_count, 60)

    def test_current_ci_warms_preparation_before_each_first_guard_and_tests_it(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        for jobname in ['provider', 'provider_operator', 'provider_client']:
            job = policy._job(source, jobname)
            prefix = job[:job.index('          python3 tools/check_provider_dependencies.py\n')]
            for target in policy.TARGETS:
                self.assertIn('--filter-platform ' + target + ' --features preparation > /dev/null', prefix)
        provider = policy._job(source, 'provider')
        self.assertIn('--features preparation -- -D warnings', provider)
        self.assertIn('--features preparation -- --test-threads=1', provider)


class PreparationConsoleCI(unittest.TestCase):
    def test_current_client_tests_use_staged_built_binary_and_private_capture_directory(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        step = policy._step(policy._job(source, 'provider_client'), 'Validate the native authenticated attachment client')
        build = 'cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked'
        stage = 'cp "$CARGO_TARGET_DIR/release/faction-client" "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"'
        digest = 'sha256sum "$RUNNER_TEMP/faction286-setup-OWNED/faction-client"'
        directory = 'mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED/captures"'
        consumer = 'env FACTION_SETUP_CLIENT_EXE="$RUNNER_TEMP/faction286-setup-OWNED/faction-client" FACTION_SETUP_CAPTURE_DIR="$RUNNER_TEMP/faction286-setup-OWNED/captures" cargo test -p faction-client --features device-provider --all-targets --release --locked -- --test-threads=1'
        for command in [build, stage, digest, directory, consumer]:
            self.assertEqual(step.splitlines().count('          ' + command), 1)
        self.assertLess(step.index(build), step.index(stage))
        self.assertLess(step.index(stage), step.index(digest))
        self.assertLess(step.index(digest), step.index(consumer))
        self.assertLess(step.index(directory), step.index(consumer))


    def test_staged_client_identity_environment_and_private_capture_cannot_be_removed(self):
        good = ci_fixture()
        step = policy._step(policy._job(good, 'provider_client'), 'Validate the native authenticated attachment client')
        for command in step.splitlines():
            if 'faction286-setup-OWNED' in command:
                with self.subTest(missing=command), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace(command + '\n', '', 1))
        for old, new in [('env FACTION_SETUP_CLIENT_EXE=', 'env OMITTED_CLIENT='),
                         ('FACTION_SETUP_CAPTURE_DIR=', 'OMITTED_CAPTURE='),
                         ('mkdir -m 700 "$RUNNER_TEMP/faction286-setup-OWNED/captures"',
                          'mkdir -m 755 "$RUNNER_TEMP/faction286-setup-OWNED/captures"')]:
            with self.subTest(change=new), self.assertRaises(ValueError):
                policy.validate_ci(good.replace(old, new))



class PreparedOperatorCI(unittest.TestCase):
    def test_actual_operator_probe_selects_prepared_mode_only_at_the_operator_consumer(self):
        import shlex
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        job = policy._job(source, 'provider_operator')
        lines = [line.strip() for line in job.splitlines()
                 if line.strip().startswith('python3 tools/probe_provider_operator.py ')]
        self.assertEqual(len(lines), 1)
        words = shlex.split(lines[0])
        self.assertEqual(words.count('--prepared'), 1)
        self.assertEqual(words[-1], '--prepared')
        native = policy._job(source, 'provider_client')
        for line in native.splitlines():
            if line.strip().startswith('python3 tools/probe_device_provider_console.py '):
                self.assertNotIn('--prepared', shlex.split(line))

    def test_prepared_mode_cannot_be_omitted_or_moved_to_the_native_probe(self):
        source = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        with self.assertRaises(ValueError):
            policy.validate_ci(source.replace(' --prepared', ''))
        moved = source.replace(' --prepared', '')
        moved = moved.replace('tools/probe_device_provider_console.py --client',
                              'tools/probe_device_provider_console.py --prepared --client')
        with self.assertRaises(ValueError):
            policy.validate_ci(moved)


if __name__ == '__main__':
    unittest.main()

class RetrievalRecordsProfiles(unittest.TestCase):
    def test_empty_default_off_feature_and_exact_selected_graphs(self):
        policy.validate_profile(manifest(), {'workspace': {'members': []}})
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(policy.subprocess, 'check_output', return_value=metadata), patch.object(policy, 'validate_ci'), patch.object(policy, 'validate_product_edges'), patch.object(policy, 'validate_graph'):
            result = json.loads(policy.inventory())
        for profile in ['delegation-records', 'delegation-records,durable-policy']:
            self.assertEqual(result['profiles'][profile]['selected_features'], profile.split(','))
            self.assertEqual(set(result['profiles'][profile]['graphs']), set(policy.TARGETS))
        changed = manifest(); changed['features']['delegation-records'] = ['blob-iq']
        with self.assertRaises(ValueError): policy.validate_profile(changed, {'workspace': {'members': []}})
        changed = manifest(); changed['features']['default'] = ['delegation-records']
        with self.assertRaises(ValueError): policy.validate_profile(changed, {'workspace': {'members': []}})

    def test_current_ci_qualifies_both_record_profiles_before_compilation(self):
        text = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        policy.validate_ci(text)
        for job_name in ['provider', 'provider_operator', 'provider_client']:
            job = policy._job(text, job_name)
            first_guard = job.index('          python3 tools/check_provider_dependencies.py\n')
            for profile in ['delegation-records', 'delegation-records,durable-policy']:
                for target in policy.TARGETS:
                    command = 'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform ' + target + ' --features ' + profile + ' > /dev/null'
                    self.assertIn('          ' + command + '\n', job[:first_guard])
                    with self.assertRaises(ValueError): policy.validate_ci(text.replace('          ' + command + '\n', '', 1))
        provider = policy._job(text, 'provider')
        for profile in ['delegation-records', 'delegation-records,durable-policy']:
            for operation in ['clippy', 'test']:
                command = 'cargo ' + operation + ' --manifest-path crates/faction-provider/Cargo.toml' + (' --all-targets' if operation == 'clippy' else '') + ' --release --locked --features ' + profile + (' -- -D warnings' if operation == 'clippy' else ' -- --test-threads=1')
                self.assertIn(command, provider)
                with self.assertRaises(ValueError): policy.validate_ci(text.replace('          ' + command + '\n', '', 1))


class ServicesProfiles(unittest.TestCase):
    def test_services_profiles_are_qualified_on_all_three_targets(self):
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(policy.subprocess, 'check_output', return_value=metadata), patch.object(policy, 'validate_ci'), patch.object(policy, 'validate_product_edges'), patch.object(policy, 'validate_graph'):
            result = json.loads(policy.inventory())
        for profile in ['delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy']:
            self.assertEqual(result['profiles'][profile]['selected_features'], profile.split(','))
            self.assertEqual(set(result['profiles'][profile]['graphs']),set(policy.TARGETS))
        for key,value in [('default',['delegation-services']),('delegation-services',['blob-iq'])]:
            changed=manifest();changed['features'][key]=value
            with self.assertRaises(ValueError):policy.validate_profile(changed, {'workspace':{'members':[]}})

    def test_omitted_service_graph_or_validation_fails_closed_in_actual_ci(self):
        text=(policy.ROOT/'.github/workflows/documentation.yml').read_text()
        for profile in ['delegation-services', 'delegation-services,durable-policy', 'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier', 'delegation-verifier,durable-policy']:
            for job_name in ['provider','provider_operator']:
                job=policy._job(text,job_name);first=job.index('          python3 tools/check_provider_dependencies.py\n')
                for target in policy.TARGETS:
                    command='cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform '+target+' --features '+profile+' > /dev/null'
                    self.assertIn('          '+command+'\n',job[:first])
                    with self.assertRaises(ValueError):policy.validate_ci(text.replace('          '+command+'\n','',1))
            for operation in ['clippy','test']:
                command='cargo '+operation+' --manifest-path crates/faction-provider/Cargo.toml'+(' --all-targets' if operation=='clippy' else '')+' --release --locked --features '+profile+(' -- -D warnings' if operation=='clippy' else ' -- --test-threads=1')
                self.assertIn(command,policy._job(text,'provider' if profile=='delegation-verifier' else 'provider_operator'))
                with self.assertRaises(ValueError):policy.validate_ci(text.replace('          '+command+'\n','',1))


class ServiceProfileSchedulingCI(unittest.TestCase):
    @staticmethod
    def service_commands():
        commands = []
        for profile in ['delegation-services', 'delegation-services,durable-policy',
                        'delegation-services,delegation-records,durable-policy', 'delegation-issuer', 'delegation-issuer,durable-policy', 'delegation-wire', 'delegation-wire,durable-policy', 'delegation-verifier,durable-policy']:
            commands += [
                'cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features ' + profile + ' -- -D warnings',
                'cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features ' + profile + ' -- --test-threads=1',
            ]
        return commands

    @classmethod
    def scheduled_fixture(cls):
        text = ci_fixture()
        for command in cls.service_commands():
            text = text.replace('          ' + command + '\n', '')
        build = '          cargo build --manifest-path crates/faction-provider/Cargo.toml --bin faction-provider --features operator --release --locked\n'
        return text.replace(build, ''.join('          ' + command + '\n'
                                          for command in cls.service_commands()) + build)

    def test_actual_service_pairs_run_once_in_operator_after_qualification_before_build(self):
        text = (policy.ROOT / '.github/workflows/documentation.yml').read_text()
        provider = policy._job(text, 'provider')
        operator = policy._job(text, 'provider_operator')
        previous = operator.index('          python3 tools/check_provider_dependencies.py\n')
        build = operator.index('          cargo build --manifest-path crates/faction-provider/Cargo.toml --bin faction-provider ')
        for command in self.service_commands():
            with self.subTest(command=command):
                self.assertEqual(text.splitlines().count('          ' + command), 1)
                self.assertNotIn('          ' + command + '\n', provider)
                position = operator.index('          ' + command + '\n')
                self.assertLess(previous, position)
                self.assertLess(position, build)
                previous = position

    def test_guard_accepts_complete_scheduled_fixture(self):
        policy.validate_ci(self.scheduled_fixture())

    def test_service_pairs_cannot_be_omitted_duplicated_or_moved_to_provider(self):
        good = self.scheduled_fixture()
        provider_anchor = '      - name: Validate the native authenticated attachment client\n'
        for command in self.service_commands():
            line = '          ' + command + '\n'
            for candidate in [good.replace(line, ''), good.replace(line, line + line),
                              good.replace(line, '').replace(provider_anchor, line + provider_anchor)]:
                with self.subTest(command=command), self.assertRaises(ValueError):
                    policy.validate_ci(candidate)

    def test_service_pairs_cannot_be_changed_unlocked_parallelized_or_skipped(self):
        good = self.scheduled_fixture()
        for command in self.service_commands():
            variants = ['# ' + command, 'echo ' + command, command + ' || true',
                        command + ' &', command.replace(' --locked', ''),
                        command.replace(' --release', ''), 'if false; then ' + command + '; fi']
            if command.startswith('cargo test '):
                variants.append(command.replace(' --test-threads=1', ' --test-threads=2'))
            else:
                variants.append(command.replace(' -- -D warnings', ''))
            for changed in variants:
                with self.subTest(command=command, changed=changed), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace('          ' + command + '\n',
                                                    '          ' + changed + '\n'))

    def test_service_consumer_cannot_be_conditional_or_continue_on_error(self):
        good = self.scheduled_fixture()
        for prefix in ['    ', '        ']:
            anchor = ('    name: Optional durable provider operator\n' if prefix == '    '
                      else '      - name: Validate durable provider policy and operator\n')
            for setting in ['if: false', 'continue-on-error: true']:
                with self.subTest(prefix=prefix, setting=setting), self.assertRaises(ValueError):
                    policy.validate_ci(good.replace(anchor, anchor + prefix + setting + '\n'))

    def test_both_jobs_require_every_online_locked_warmup_before_compilation(self):
        good = self.scheduled_fixture()
        for name in ['provider', 'provider_operator', 'provider_client']:
            job = policy._job(good, name)
            for command in policy.root_warm_commands() + policy.provider_warm_commands():
                line = '          ' + command + '\n'
                for replacement in ['', line.replace(' --locked', ''),
                                    line.replace(' --locked', ' --locked --offline')]:
                    with self.subTest(job=name, command=command, replacement=replacement), self.assertRaises(ValueError):
                        policy.validate_ci(good.replace(job, job.replace(line, replacement, 1)))
            command = '          ' + self.service_commands()[0] + '\n'
            early = job.replace(command, '').replace('          python3 tools/check_provider_dependencies.py\n',
                                                    command + '          python3 tools/check_provider_dependencies.py\n', 1)
            with self.subTest(job=name), self.assertRaises(ValueError):
                policy.validate_ci(good.replace(job, early))


    def test_service_pairs_cannot_be_repeated_in_an_extra_step_or_job(self):
        good = self.scheduled_fixture()
        for command in self.service_commands():
            step = '      - name: Duplicate service check\n        run: |\n          ' + command + '\n'
            extra_job = '  duplicate_service:\n    runs-on: ubuntu-24.04\n    steps:\n' + step
            for extra in [step, extra_job]:
                candidate = good.replace('  documentation:\n', extra + '  documentation:\n')
                with self.subTest(command=command, extra=extra), self.assertRaises(ValueError):
                    policy.validate_ci(candidate)

class QualifiedIssuerWireFeature(unittest.TestCase):
    def test_actual_wire_feature_is_exact_default_off_and_qualified(self):
        actual=tomllib.loads(policy.MANIFEST.read_text())
        self.assertEqual(actual['features'].get('delegation-wire'),['delegation-issuer','dep:faction-delegation-codec'])
        self.assertEqual(actual['dependencies'].get('faction-delegation-codec'),{'path':'../faction-delegation-codec','optional':True,'default-features':False})
        for feature in ['delegation-wire','delegation-wire,durable-policy','delegation-verifier','delegation-verifier,durable-policy']:
            for target in policy.TARGETS:
                self.assertIn('cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform '+target+' --features '+feature+' > /dev/null',policy.provider_warm_commands())
        policy.validate_ci((policy.ROOT/'.github/workflows/documentation.yml').read_text())
    def test_codec_only_registry_sources_are_refused_outside_exact_wire_composition(self):
        import check_delegation_codec_dependencies as codec
        for key in [('httparse','1.10.1'),('form_urlencoded','1.2.2'),('percent-encoding','2.3.2')]:
            approved=codec.APPROVED[key]
            package={'name':key[0],'version':key[1],'source':policy.REGISTRY,'license':approved['license']}
            policy.validate_codec_composition('delegation-wire',package,approved['features'])
            for profile in ['default','operator','device-provider','delegation-issuer','delegation-issuer,durable-policy','delegation-services','delegation-records']:
                with self.subTest(key=key,profile=profile),self.assertRaises(ValueError):policy.validate_codec_composition(profile,package,approved['features'])
            with self.assertRaises(ValueError):policy.validate_codec_composition('delegation-wire',package,approved['features']+['unreviewed'])
    def test_root_graph_refuses_codec_only_registry_even_without_local_codec_node(self):
        for name in ['httparse','form_urlencoded','percent-encoding']:
            metadata={'packages':[{'id':'injected','name':name,'version':'999'}], 'resolve':{'nodes':[{'id':'injected','features':[]}]}}
            for enabled in [False,True]:
                with self.subTest(name=name,enabled=enabled),self.assertRaises(ValueError):policy.validate_graph(metadata,enabled)
