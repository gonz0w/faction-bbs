"""Signing succeeds only after a clean exit and real certificate verification."""
import json
from pathlib import Path
import tempfile
import subprocess
import unittest
from unittest.mock import patch
import probe_signing_exit as probe


class SigningExit(unittest.TestCase):
    def test_artifacts_never_override_exit_or_failed_verification(self):
        self.assertFalse(probe.accepted(-11, True, True))
        self.assertFalse(probe.accepted(1, True, True))
        self.assertFalse(probe.accepted(0, False, True))
        self.assertFalse(probe.accepted(0, True, False))
        self.assertTrue(probe.accepted(0, True, True))

    def test_error_cleanup_removes_private_fixture(self):
        with tempfile.TemporaryDirectory() as parent:
            def interrupted(argv, data=None):
                Path(argv[-1]).write_text('synthetic private fixture marker')
                raise RuntimeError('controlled failure')
            with patch.object(probe, '_run', side_effect=interrupted):
                with self.assertRaises(RuntimeError):
                    probe.run(Path('/usr/bin/ssh-keygen'), Path('/usr/bin/ssh-keygen'), Path(parent))
            self.assertEqual(list(Path(parent).iterdir()), [])

    def test_real_synthetic_certificates_and_negative_verifier_controls(self):
        with tempfile.TemporaryDirectory() as parent:
            result = probe.run(Path('/usr/bin/ssh-keygen'), Path('/usr/bin/ssh-keygen'), Path(parent))
            self.assertEqual(result['result'], 'pass')
            self.assertEqual(result['count'], 5)
            self.assertTrue(result['keys_removed'] and result['core_dumps_disabled'])
            self.assertTrue(all(c['accepted'] and c['signature_verified'] for c in result['controls']))
            self.assertTrue(all(c['tamper_denied'] and c['wrong_principal_denied'] and c['wrong_ca_denied'] for c in result['controls']))
            self.assertTrue(result['invalid_ca_denied'])
            self.assertEqual(list(Path(parent).iterdir()), [])
            raw = json.dumps(result)
            self.assertNotIn('BEGIN OPENSSH PRIVATE KEY', raw)
            self.assertNotIn('ssh-ed25519 ', raw)

    def test_crashed_negative_verifier_never_counts_as_a_valid_rejection(self):
        original = probe._run
        def crash(argv, data=None):
            if '-Y' in argv and 'verify' in argv and '-I' in argv and argv[argv.index('-I')+1] == 'other':
                return subprocess.CompletedProcess(argv, -11, b'', b'')
            return original(argv, data)
        with patch.object(probe, '_run', side_effect=crash):
            result = probe.run(Path('/usr/bin/ssh-keygen'), Path('/usr/bin/ssh-keygen'))
        self.assertEqual(result['result'], 'observed_failure')
        self.assertTrue(all(not c['wrong_principal_denied'] for c in result['controls']))

    def test_signing_timeout_is_recorded_failure_and_keys_are_removed(self):
        original = subprocess.run
        def timeout(argv, **kwargs):
            if '-s' in argv:
                raise subprocess.TimeoutExpired(argv, 10)
            return original(argv, **kwargs)
        with tempfile.TemporaryDirectory() as parent, patch.object(probe.subprocess, 'run', side_effect=timeout):
            result = probe.run(Path('/usr/bin/ssh-keygen'), Path('/usr/bin/ssh-keygen'), Path(parent))
            self.assertEqual(result['result'], 'observed_failure')
            self.assertTrue(all(c['timed_out'] and not c['accepted'] for c in result['controls']))
            self.assertFalse(result['invalid_ca_denied'])
            self.assertTrue(result['keys_removed'])
            self.assertEqual(list(Path(parent).iterdir()), [])

    def test_key_generation_timeout_is_an_explicit_fixture_error(self):
        with tempfile.TemporaryDirectory() as parent, patch.object(probe, '_run', return_value=subprocess.CompletedProcess([], None, b'', b'')):
            with self.assertRaisesRegex(RuntimeError, 'key generation failed'):
                probe.run(Path('/usr/bin/ssh-keygen'), Path('/usr/bin/ssh-keygen'), Path(parent))
            self.assertEqual(list(Path(parent).iterdir()), [])
