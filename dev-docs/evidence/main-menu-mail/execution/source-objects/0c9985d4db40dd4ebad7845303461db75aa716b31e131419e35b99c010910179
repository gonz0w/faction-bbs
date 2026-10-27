"""The supported picker proof must never report skipped assertions as passes."""
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path

import probe_picker_home_mail as probe


class RequiredMailFixture(unittest.TestCase):
    def test_supported_api_requires_inspection_fixture_before_creating_private_state(self):
        with self.assertRaisesRegex(ValueError, 'mail_fixture is required'):
            probe.probe(Path('/missing/client'), Path('/missing/checkpoint-fixture'))

    def test_supported_cli_requires_inspection_fixture_before_opening_paths(self):
        with tempfile.TemporaryDirectory(prefix='faction214-picker-contract-OWNED-') as temp:
            output = Path(temp) / 'result.json'
            result = subprocess.run([sys.executable, str(Path(probe.__file__)),
                                     '--client', '/missing/client',
                                     '--checkpoint-fixture', '/missing/checkpoint-fixture',
                                     '--output', str(output)], capture_output=True, text=True)
            self.assertEqual(result.returncode, 2)
            self.assertIn('--mail-fixture is required', result.stderr)
            self.assertFalse(output.exists())


class ClosedEndpoint(unittest.TestCase):
    def test_only_actual_connection_refusal_establishes_unavailable_home(self):
        with mock.patch.object(probe.socket, 'create_connection', side_effect=ConnectionRefusedError):
            probe.require_closed_endpoint(('127.0.0.1', 9))

    def test_timeout_is_not_accepted_as_connection_refusal(self):
        with mock.patch.object(probe.socket, 'create_connection', side_effect=TimeoutError):
            with self.assertRaises(TimeoutError):
                probe.require_closed_endpoint(('127.0.0.1', 9))

    def test_successful_connection_is_rejected(self):
        with mock.patch.object(probe.socket, 'create_connection', return_value=mock.MagicMock()):
            with self.assertRaisesRegex(AssertionError, 'connection refusal'):
                probe.require_closed_endpoint(('127.0.0.1', 9))


if __name__ == '__main__':
    unittest.main()
