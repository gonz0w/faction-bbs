"""Qualification contracts for the explicit opt-in provider embedding package."""
import copy
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import check_provider_dependencies as policy
import check_network_mail_dependencies as network
import check_provider_auth_dependencies as auth
import test_network_mail_policy as network_tests


def manifest():
    return {'package': {'name': 'faction-provider', 'version': '0.1.0-alpha.1',
            'edition': '2024', 'rust-version': '1.96', 'license': 'BSD-3-Clause', 'publish': False},
            'workspace': {}, 'features': {'default': [], 'blob-iq': ['dep:faction-xmpp']},
            'dependencies': {'faction-files': {'path': '../faction-files'}, 'openssl': '=0.10.81',
                             'faction-xmpp': {'path': '../faction-xmpp', 'optional': True}}}


class ProviderDependencies(unittest.TestCase):
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
        self.assertEqual(set(result['profiles']), {'default', 'blob-iq', 'device-provider'})
        self.assertEqual(execute.call_count, 18)
        expected = {(target, feature) for target in policy.TARGETS for feature in [None, 'blob-iq', 'faction-client/device-provider', 'faction-client/native-mail', 'faction-client/network-mail']}
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
        active = {'packages': [provider], 'resolve': {'nodes': [{'id': 'provider', 'features': []}]}}
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
        'device-provider': ['network-mail', 'dep:faction-provider', 'tokio/sync']},
        'dependencies': {'faction-provider': {'path': '../faction-provider', 'optional': True, 'default-features': False}}}


def native_ci_step():
    commands = ['cargo metadata --locked --format-version 1 --filter-platform ' + target + ' --features faction-client/device-provider > /dev/null' for target in policy.TARGETS]
    commands += ['python3 tools/check_provider_dependencies.py',
        'cargo clippy -p faction-client --features device-provider --all-targets --release --locked -- -D warnings',
        'cargo test -p faction-client --features device-provider --all-targets --release --locked -- --test-threads=1']
    commands += [
        'cargo build -p faction-client --features device-provider --bin faction-client --example device_provider_console_fixture --release --locked',
        'umask 077',
        'mkdir -m 700 "$RUNNER_TEMP/faction280-console-OWNED"',
        "printf '%s\\n' 'Owned authenticated-provider console proof' > \"$RUNNER_TEMP/faction280-console-OWNED/OWNED\"",
        'python3 tools/probe_device_provider_console.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --owned-root "$RUNNER_TEMP/faction280-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction280-console-OWNED/result.json"',
    ]
    return ('      - name: Validate the native authenticated attachment client\n        env:\n          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider\n          CARGO_BUILD_JOBS: 2\n          RUSTUP_TOOLCHAIN: 1.96.0\n        run: |\n'
            + ''.join('          ' + command + '\n' for command in commands))


def ci_fixture():
    commands = [
        'rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu > /dev/null',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd > /dev/null',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin > /dev/null',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features blob-iq > /dev/null',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-unknown-netbsd --features blob-iq > /dev/null',
        'cargo metadata --locked --manifest-path crates/faction-provider/Cargo.toml --format-version 1 --filter-platform aarch64-apple-darwin --features blob-iq > /dev/null',
        'python3 tools/check_provider_dependencies.py',
        'cargo fmt --manifest-path crates/faction-provider/Cargo.toml -- --check',
        'cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked -- -D warnings',
        'cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked -- --test-threads=1',
        'cargo clippy --manifest-path crates/faction-provider/Cargo.toml --all-targets --release --locked --features blob-iq -- -D warnings',
        'cargo test --manifest-path crates/faction-provider/Cargo.toml --release --locked --features blob-iq -- --test-threads=1',
    ]
    return ("jobs:\n  provider:\n    name: Optional attachment provider\n    runs-on: ubuntu-24.04\n    timeout-minutes: 15\n    steps:\n"
            "      - name: Validate the opt-in attachment provider\n        env:\n          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider\n          CARGO_BUILD_JOBS: 2\n          RUSTUP_TOOLCHAIN: 1.96.0\n        run: |\n"
            + ''.join('          ' + command + '\n' for command in commands)
            + native_ci_step() + '  documentation:\n    name: Documentation and policy\n    needs: [native_mail, provider]\n    if: ${{ always() }}\n    steps:\n      - name: Require optional native tests\n        env:\n          NATIVE_MAIL_RESULT: ${{ needs.native_mail.result }}\n          PROVIDER_RESULT: ${{ needs.provider.result }}\n        run: |\n          test "$NATIVE_MAIL_RESULT" = success\n          test "$PROVIDER_RESULT" = success\n')


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
        for old, new in [('needs: [native_mail, provider]', 'needs: native_mail'),
                         ('          test "$PROVIDER_RESULT" = success\n', ''),
                         ('          test "$NATIVE_MAIL_RESULT" = success\n', ''),
                         ('test "$PROVIDER_RESULT" = success', 'test "$PROVIDER_RESULT" = success || true'),
                         ('if: ${{ always() }}', 'if: false'),
                         ('        run: |\n          test', '        if: false\n        run: |\n          test')]:
            with self.subTest(change=new), self.assertRaises(ValueError):
                policy.validate_ci(good.replace(old, new))


if __name__ == '__main__':
    unittest.main()
