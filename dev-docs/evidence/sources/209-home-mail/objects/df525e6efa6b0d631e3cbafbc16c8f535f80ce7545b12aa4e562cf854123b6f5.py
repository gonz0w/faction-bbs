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


if __name__ == '__main__':
    unittest.main()

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
