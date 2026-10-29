"""Contracts for the owned authenticated-provider console proof."""
import importlib
import os
from pathlib import Path
import stat
import subprocess
from unittest.mock import patch
import tempfile
import tomllib
import unittest


class ProviderConsoleProbeTests(unittest.TestCase):
    def test_fixture_target_requires_explicit_provider_feature(self):
        manifest = tomllib.loads((Path(__file__).resolve().parents[1] /
            'crates/faction-client/Cargo.toml').read_text())
        examples = {entry['name']: entry for entry in manifest['example']}
        self.assertIn('device_provider_console_fixture', examples)
        self.assertEqual(examples['device_provider_console_fixture']['required-features'],
                         ['device-provider'])

    def test_provider_readiness_failure_reaps_its_child(self):
        probe = importlib.import_module('probe_device_provider_console')
        created = []
        original = subprocess.Popen
        def record(*args, **kwargs):
            child = original(*args, **kwargs)
            created.append(child)
            return child
        with tempfile.TemporaryDirectory(prefix='faction280-unready-OWNED-') as name:
            root = Path(name).resolve();root.chmod(0o700)
            script = root/'unready'
            script.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n')
            script.chmod(0o700)
            try:
                with patch.object(probe.subprocess, 'Popen', side_effect=record):
                    with self.assertRaises(ValueError):
                        probe.Provider(script, root)
                self.assertEqual(len(created), 1)
                self.assertIsNotNone(created[0].poll(), 'readiness failure left a running child')
                self.assertTrue(created[0].stdout.closed)
                self.assertTrue(created[0].stderr.closed)
            finally:
                for child in created:
                    if child.poll() is None:child.kill()
                    child.wait(timeout=5)
                    child.stdout.close();child.stderr.close()

    def test_probe_cli_requires_explicit_local_inputs(self):
        probe = importlib.import_module('probe_device_provider_console')
        parsed = probe.make_parser().parse_args(['--client', '/client', '--fixture', '/fixture', '--owned-root', '/owned', '--openssl', 'openssl', '--output', '/owned/result.json'])
        self.assertEqual(parsed.client, Path('/client'))
        self.assertEqual(parsed.fixture, Path('/fixture'))
        self.assertEqual(parsed.openssl, 'openssl')
        self.assertEqual(parsed.owned_root, Path('/owned'))
        self.assertEqual(parsed.output, Path('/owned/result.json'))
        self.assertTrue(callable(probe.probe))

    def test_ready_is_one_bounded_public_port(self):
        probe = importlib.import_module('probe_device_provider_console')
        self.assertEqual(probe.parse_ready(b'ready 12345\n'), 12345)
        for raw in [b'ready 0\n', b'ready 65536\n', b'ready 001\n',
                    b'ready 12\nsecret\n', b'private endpoint:123\n', b'x'*8192]:
            with self.subTest(raw=raw[:30]), self.assertRaises(ValueError):
                probe.parse_ready(raw)

    def test_capture_is_private_bounded_and_cannot_escape(self):
        probe = importlib.import_module('probe_device_provider_console')
        with tempfile.TemporaryDirectory(prefix='faction280-probe-test-OWNED-') as name:
            root = Path(name).resolve()
            root.chmod(0o700)
            destination = probe.retain_capture(root, 'failed-console', b'failed private run')
            self.assertEqual(destination.parent, root)
            self.assertEqual(destination.read_bytes(), b'failed private run')
            self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o600)
            for label, data in [('../escape', b'x'), ('secret/path', b'x'),
                                ('oversize', b'x'*(4*1024*1024+1))]:
                with self.subTest(label=label), self.assertRaises(ValueError):
                    probe.retain_capture(root, label, data)
            self.assertEqual(len(list(root.iterdir())), 1)


if __name__ == '__main__':
    unittest.main()
