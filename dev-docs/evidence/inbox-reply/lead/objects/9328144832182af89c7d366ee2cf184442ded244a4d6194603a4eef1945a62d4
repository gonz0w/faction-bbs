"""Private native two-home fixture boundaries, without network or third parties."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import probe_home_mail as fixture


class PrivateFixtureBoundary(unittest.TestCase):
    def test_inspection_requires_authenticated_plaintext_and_ambiguous_outbox(self):
        history = [{'direction': 'received', 'subject': 'subject', 'body': 'body', 'sent_at': 42, 'address': 'alice@homea.test'}]
        data = dict(history=history, pending=[dict(operation='a' * 32, address='alice@homea.test')])
        self.assertEqual(fixture.inspection(json.dumps(data).encode()), data)
        for invalid in [dict(history='raw', pending=[]), dict(history=[dict(direction='received')], pending=[]), dict(history=[], pending=[dict(operation='x', address='peer')])]:
            with self.assertRaises(fixture.FixtureFailure):
                fixture.inspection(json.dumps(invalid).encode())
        fixture.require_history(data, [('received', 'subject', 'body')], 1)
        with self.assertRaises(fixture.FixtureFailure):
            fixture.require_history(data, [('received', 'changed', 'body')], 1)

    def test_passwords_are_stdin_only_and_diagnostics_stay_private(self):
        home = dict(store='/private/store', local_password='private-local', password='private-home')
        with patch.object(fixture.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, b'ok\n', b'')) as run:
            fixture.command(Path('/binary'), 'sync', home, '/private/config', home_password=True)
        args, kwargs = run.call_args
        self.assertNotIn('private-local', str(args[0]))
        self.assertNotIn('private-home', str(args[0]))
        self.assertEqual(kwargs['input'], b'private-local\nprivate-home\n')
        with patch.object(fixture.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1, b'private-remote-diagnostic', b'private-stderr')):
            with self.assertRaisesRegex(fixture.FixtureFailure, '^native action rejected$'):
                fixture.command(Path('/binary'), 'sync', home, '/private/config', home_password=True)

    def test_storage_scan_does_not_accept_plaintext_or_follow_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'archive').write_bytes(b'ciphertext bytes')
            fixture.assert_no_plaintext([root], [b'private-subject'])
            (root / 'archive').write_bytes(b'private-subject')
            with self.assertRaises(fixture.FixtureFailure):
                fixture.assert_no_plaintext([root], [b'private-subject'])
            (root / 'archive').unlink()
            (root / 'link').symlink_to('/tmp')
            with self.assertRaises(fixture.FixtureFailure):
                fixture.assert_no_plaintext([root], [b'private-subject'])

class SetupPolicy(unittest.TestCase):
    def test_two_home_configs_select_direct_tls_and_loopback_only(self):
        for side, port in [('a', 32222), ('b', 32223)]:
            config = fixture.server_configuration(side)
            self.assertIn('interfaces = { "127.0.0.1" }', config)
            self.assertIn('c2s_ports = { }', config)
            self.assertIn(f'c2s_direct_tls_ports = {{ {port} }}', config)
            self.assertIn('http_ports = { }', config)
            self.assertIn('s2s_secure_auth = true', config)
            self.assertIn('c2s_stanza_size_limit = 16384', config)
            self.assertNotIn('10.0.2.15', config)
        with self.assertRaises(fixture.FixtureFailure):
            fixture.server_configuration('public')

class AdversarialFixtureBoundary(unittest.TestCase):
    def test_attack_module_is_explicit_and_owned_fixture_only(self):
        directory = '/home/mailb/faction209-attack'
        config = fixture.server_configuration('b', attack_directory=directory)
        self.assertIn('"faction_probe_attack"', config)
        self.assertIn('faction_probe_attack_directory = "' + directory + '"', config)
        self.assertNotIn('"faction_probe_attack"', fixture.server_configuration('b'))
        with self.assertRaises(fixture.FixtureFailure):
            fixture.server_configuration('b', attack_directory='/tmp/unowned')

    def test_denial_requires_exact_native_status_and_unchanged_checkpoint(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'mail.fkv').write_bytes(b'canonical encrypted state')
            (root / 'mail.fkv').chmod(0o600)
            home = dict(store=str(root), local_password='local-private', password='home-private')
            denied = subprocess.CompletedProcess([], 1, b'', b'Transport(Pin)\n')
            with patch.object(fixture.subprocess, 'run', return_value=denied) as run:
                fixture.denied_cycle(Path('/binary'), home, '/config', 'Transport(Pin)')
            args, kwargs = run.call_args
            self.assertNotIn('home-private', str(args))
            self.assertEqual(kwargs['input'], b'local-private\nhome-private\n')
            for response in [subprocess.CompletedProcess([], 0, b'ok\n', b''), subprocess.CompletedProcess([], 1, b'', b'Transport(Auth)\n')]:
                with patch.object(fixture.subprocess, 'run', return_value=response), self.assertRaises(fixture.FixtureFailure):
                    fixture.denied_cycle(Path('/binary'), home, '/config', 'Transport(Pin)')
            def changed(*args, **kwargs):
                (root / 'mail.fkv').write_bytes(b'changed canonical state')
                return denied
            with patch.object(fixture.subprocess, 'run', side_effect=changed), self.assertRaises(fixture.FixtureFailure):
                fixture.denied_cycle(Path('/binary'), home, '/config', 'Transport(Pin)')

    def test_wrong_pin_configuration_is_private_changes_only_pin_and_is_removed(self):
        import os
        import tomllib
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / 'home.toml'
            good = dict(version=1, account='alice@homea.test', endpoint='127.0.0.1:32222', domain='homea.test', ca=str(root / 'ca.pem'), pin='12' * 32)
            path.write_text('\n'.join(k + '=' + (str(v) if isinstance(v, int) else json.dumps(v)) for k, v in good.items()) + '\n')
            path.chmod(0o600)
            with fixture.wrong_pin_configuration(dict(config=str(path))) as changed:
                actual = tomllib.loads(changed.read_text())
                self.assertEqual({k:v for k,v in actual.items() if k != 'pin'}, {k:v for k,v in good.items() if k != 'pin'})
                self.assertEqual(int(actual['pin'], 16) ^ int(good['pin'], 16), 1)
                self.assertEqual(changed.stat().st_mode & 0o777, 0o600)
                self.assertEqual(changed.stat().st_uid, os.geteuid())
                self.assertEqual(changed.stat().st_nlink, 1)
            self.assertFalse(changed.exists())
            self.assertEqual(tomllib.loads(path.read_text()), good)

    def test_receipt_requires_fresh_nonce_mode_and_bounded_counts(self):
        token = 'ab' * 16
        raw = ('faction209-v1\n' + token + '\nadded-device\n1\n1\n1\n0\n').encode()
        self.assertEqual(fixture.parse_attack_receipt(raw, token, 'added-device'), dict(devices=1, bundles=1, messages=1, unexpected=0))
        for bad in [raw.replace(token.encode(), b'00' * 16), raw.replace(b'added-device', b'replaced-device'), raw + b'extra\n', raw.replace(b'\n1\n', b'\n-1\n'), raw.replace(b'\n1\n', b'\n100000000\n')]:
            with self.assertRaises(fixture.FixtureFailure):
                fixture.parse_attack_receipt(bad, token, 'added-device')

    def test_public_results_have_all_named_controls_and_no_private_fields(self):
        negative = [
            'tls_wrong_leaf_pin_denied_without_checkpoint_mutation',
            'home_wrong_password_denied_without_checkpoint_mutation',
            'server_replaced_device_list_denied_without_checkpoint_mutation',
            'server_changed_identity_bundle_denied_without_checkpoint_mutation',
            'server_added_device_never_enrolled_or_addressed',
        ]
        result = fixture.public_result(fixture.POSITIVE_CHECKS + negative)
        self.assertEqual(result['checks'], fixture.POSITIVE_CHECKS + negative)
        self.assertEqual(result['scope'], 'complete')
        self.assertNotIn('pin', result)
        self.assertNotIn('password', result)
        self.assertEqual(fixture.public_result(negative, negative_only=True)['scope'], 'negative-controls-only')
        with self.assertRaises(fixture.FixtureFailure):
            fixture.public_result(negative)

    def test_server_attack_controls_are_private_scoped_and_removed(self):
        import os
        import pwd
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            profile = dict(alice=dict(jid='alice@homea.test'), bob=dict(jid='bob@homeb.test', device=2102))
            owner = pwd.getpwuid(os.geteuid())
            with patch.object(fixture, 'attack_identity', return_value=(root, owner)):
                with fixture.server_attack(profile, 'added-device') as receipt:
                    fields = (root / 'mode').read_text().splitlines()
                    self.assertEqual(fields[2:], ['added-device', 'bob@homeb.test', 'alice@homea.test', '2102', '999'])
                    self.assertEqual((root / 'mode').stat().st_mode & 0o777, 0o600)
                    self.assertEqual(receipt(), dict(devices=0, bundles=0, messages=0, unexpected=0))
                self.assertEqual(list(root.iterdir()), [])
                (root / 'receipt').write_text('preexisting control')
                with self.assertRaises(FileExistsError):
                    with fixture.server_attack(profile, 'added-device'):
                        self.fail('stale control accepted')
                self.assertEqual((root / 'receipt').read_text(), 'preexisting control')
                self.assertFalse((root / 'mode').exists())


if __name__ == '__main__':
    unittest.main()
