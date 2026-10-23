"""Filesystem-boundary tests for complete file-area bundles."""
import hashlib
import os
from pathlib import Path
import tempfile
import unittest
from file_area_backup import digest_copy, private


class FileBackupTests(unittest.TestCase):
    def test_copy_hashes_bounded_regular_bytes_and_refuses_aliases(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            source = p/'object'
            source.write_bytes(bytes(range(256))*300)
            source.chmod(0o600)
            private(p, os.geteuid())
            expected = {'bytes': source.stat().st_size, 'sha256': hashlib.sha256(source.read_bytes()).hexdigest()}
            self.assertEqual(digest_copy(source, p/'copy', os.geteuid(), 100000), expected)
            self.assertEqual((p/'copy').read_bytes(), source.read_bytes())
            with self.assertRaises(ValueError):
                digest_copy(source, None, os.geteuid(), 10)
            os.link(source, p/'hardlink')
            with self.assertRaises(ValueError):
                digest_copy(source, None, os.geteuid(), 100000)
            (p/'hardlink').unlink()
            (p/'symlink').symlink_to(source)
            with self.assertRaises(OSError):
                digest_copy(p/'symlink', None, os.geteuid(), 100000)
            os.mkfifo(p/'fifo', 0o600)
            with self.assertRaises(ValueError):
                digest_copy(p/'fifo', None, os.geteuid(), 100000)
            source.chmod(0o644)
            with self.assertRaises(ValueError):
                digest_copy(source, None, os.geteuid(), 100000)
