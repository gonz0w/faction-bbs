"""Exact default-off own-home dependency qualification contracts."""
import copy
import json
from unittest.mock import patch
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
import check_provider_dependencies as provider
import check_delegation_codec_dependencies as codec

class OwnHomeQualification(unittest.TestCase):
    def test_explicit_default_off_own_home_client_feature_is_allowed(self):
        manifest = tomllib.loads((ROOT / 'crates/faction-client/Cargo.toml').read_text())
        manifest['features']['own-home-provider'] = [
            'device-provider', 'faction-provider/delegation-verifier']
        provider.validate_product_edges({'crates/faction-client/Cargo.toml': manifest})

    def test_new_root_codec_checksums_cannot_approve_themselves(self):
        # This fixture exercises identity/checksum adoption, not Cargo resolution.
        # Keep each identity once even after the real root adopts these packages.
        actual = tomllib.loads((ROOT / 'Cargo.lock').read_text())
        root_lock = 'version = 4\n'
        for row in actual['package']:
            if (row['name'], row['version']) in provider.CODEC_ONLY:
                continue
            root_lock += '\n[[package]]\n'
            for key in ['name', 'version', 'source', 'checksum']:
                if key in row:
                    root_lock += key + ' = ' + json.dumps(row[key]) + '\n'
        for name, version in sorted(provider.CODEC_ONLY):
            approved = codec.APPROVED[(name, version)]
            checksum = ('0' * 64 if name == 'httparse' else approved['checksum'])
            root_lock += ('\n[[package]]\nname = ' + json.dumps(name) +
                          '\nversion = ' + json.dumps(version) +
                          '\nsource = ' + json.dumps(codec.REGISTRY) +
                          '\nchecksum = ' + json.dumps(checksum) + '\n')
        original_read = Path.read_text
        def read(path, *args, **kwargs):
            if path == ROOT / 'Cargo.lock':
                return root_lock
            return original_read(path, *args, **kwargs)
        metadata = json.dumps({'packages': [], 'resolve': {'nodes': []}})
        with patch.object(Path, 'read_text', read), \
             patch.object(provider.subprocess, 'check_output', return_value=metadata), \
             patch.object(provider, 'validate_graph'), \
             patch.object(provider, 'validate_ci'):
            with self.assertRaisesRegex(ValueError, 'unreviewed optional codec locked source'):
                provider.inventory()

    def test_ci_warms_own_home_graph_before_compile_and_keeps_legacy_pair(self):
        source = (ROOT / '.github/workflows/documentation.yml').read_text()
        for job in ['provider', 'provider_operator', 'provider_client']:
            section = provider._job(source, job)
            prefix = section[:section.index('python3 tools/check_provider_dependencies.py')]
            for target in provider.TARGETS:
                command = ('cargo metadata --locked --manifest-path Cargo.toml --format-version 1 --filter-platform ' + target +
                           ' --features faction-client/own-home-provider > /dev/null')
                self.assertEqual(prefix.splitlines().count('          ' + command), 1)
        native = provider._step(provider._job(source, 'provider_client'),
                                'Validate the native authenticated attachment client')
        for feature in ['device-provider', 'own-home-provider']:
            clippy = ('cargo clippy -p faction-client --features ' + feature +
                      ' --all-targets --release --locked -- -D warnings')
            self.assertIn('          ' + clippy + '\n', native)
        own_test = ('env FACTION_SETUP_CLIENT_EXE=\"$RUNNER_TEMP/faction322-setup-OWNED/faction-client\" '
                    'FACTION_SETUP_CAPTURE_DIR=\"$RUNNER_TEMP/faction322-setup-OWNED/captures\" '
                    'cargo test -p faction-client --features own-home-provider'
                    ' --all-targets --release --locked -- --test-threads=1')
        self.assertEqual(source.splitlines().count('          ' + own_test), 1)
        codec_step = provider._step(provider._job(source, 'documentation'),
                                    'Validate the isolated delegation wire codec')
        for target in provider.TARGETS:
            self.assertIn(' --filter-platform ' + target +
                          ' --features faction-client/own-home-provider > /dev/null', codec_step)

    def test_own_home_codec_graph_accepts_only_qualified_parent_union(self):
        packages = [
            {'id': 'client', 'name': 'faction-client', 'version': '0.1.0-alpha.1'},
            {'id': 'provider', 'name': 'faction-provider', 'version': '0.1.0-alpha.1'},
            {'id': 'codec', 'name': 'faction-delegation-codec', 'version': '0.1.0-alpha.1'},
        ]
        nodes = [
            {'id': 'client', 'features': ['default', 'device-provider', 'native-mail', 'network-mail', 'own-home-provider'], 'deps': [{'pkg': 'provider'}]},
            {'id': 'provider', 'features': ['delegation-issuer', 'delegation-records', 'delegation-services', 'delegation-verifier', 'delegation-wire', 'preparation'], 'deps': [{'pkg': 'codec'}]},
            {'id': 'codec', 'features': [], 'deps': []},
        ]
        for name, version, features in [
            ('base64', '0.22.1', ['alloc', 'default', 'std']),
            ('memchr', '2.8.3', ['alloc', 'default', 'std']),
            ('serde_json', '1.0.154', ['alloc', 'std']),
        ]:
            approved = codec.APPROVED[(name, version)]
            packages.append({'id': name, 'name': name, 'version': version,
                             'source': codec.REGISTRY, 'license': approved['license']})
            nodes.append({'id': name, 'features': features, 'deps': []})
            nodes[2]['deps'].append({'pkg': name})
        metadata = {'packages': packages, 'resolve': {'nodes': nodes}}
        codec.validate_own_home_graph(metadata)
        ordinary = copy.deepcopy(metadata)
        ordinary['resolve']['nodes'][0]['features'].remove('own-home-provider')
        with self.assertRaises(ValueError):
            codec.validate_own_home_graph(ordinary)
        extra = copy.deepcopy(metadata)
        extra['resolve']['nodes'][3]['features'].append('unreviewed')
        with self.assertRaises(ValueError):
            codec.validate_own_home_graph(extra)
        # The standalone wire composition must retain its narrower feature rules.
        with self.assertRaises(ValueError):
            codec.validate_wire_graph(metadata)

class OwnHomeConsoleCI(unittest.TestCase):
    def test_selected_console_proof_builds_both_real_paths_after_legacy(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text()
        job=provider._job(source,'provider_client')
        name='Validate the own-home encrypted attachment console'
        self.assertEqual(source.splitlines().count('      - name: '+name),1,
                         'explicit own-home console proof must run exactly once')
        proof=provider._step(job,name)
        self.assertLess(job.index('      - name: Validate the prepared provider operator console'),job.index('      - name: '+name))
        for setting in ['CARGO_BUILD_JOBS: 2','RUSTUP_TOOLCHAIN: 1.96.0','CARGO_TARGET_DIR: ${{ runner.temp }}/faction-provider']:
            self.assertEqual(proof.splitlines().count('          '+setting),1)
        expected = [
            'python3 tools/check_provider_dependencies.py',
            'cargo build --manifest-path crates/faction-provider/Cargo.toml --bin faction-provider --features operator,delegation-verifier --release --locked',
            'cargo build -p faction-client --features own-home-provider --bin faction-client --example device_provider_console_fixture --release --locked',
            'umask 077',
            'mkdir -m 700 "$RUNNER_TEMP/faction322-console-OWNED"',
            'printf \'%s\\n\' \'Owned two-home encrypted attachment console proof\' > "$RUNNER_TEMP/faction322-console-OWNED/OWNED"',
            'python3 tools/probe_device_provider_console.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/device_provider_console_fixture" --operator "$CARGO_TARGET_DIR/release/faction-provider" --route own-home --owned-root "$RUNNER_TEMP/faction322-console-OWNED" --openssl openssl --output "$RUNNER_TEMP/faction322-console-OWNED/result.json"',
        ]
        self.assertEqual(provider._lines(proof,{'python3','cargo','umask','mkdir','printf'}), expected)
        self.assertNotIn('continue-on-error',proof)
        self.assertNotIn('timeout-minutes:',proof)

class OwnHomeGuard(unittest.TestCase):
    def test_exact_added_step_and_its_negative_mutations(self):
        source=(ROOT/'.github/workflows/documentation.yml').read_text()
        provider.validate_ci(source)
        for old,new in [(' --route own-home',''),('--features own-home-provider --bin','--features device-provider --bin'),('--features operator,delegation-verifier --release','--features operator --release'),('Owned two-home encrypted attachment console proof','Wrong owner marker')]:
            with self.subTest(changed=old):
                self.assertIn(old,source)
                with self.assertRaises(ValueError):provider.validate_ci(source.replace(old,new,1))

if __name__ == '__main__':
    unittest.main()
