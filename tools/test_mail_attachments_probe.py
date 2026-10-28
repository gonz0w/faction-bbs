"""Private fixture ownership and failure-path cleanup acceptance."""
import os
from pathlib import Path
import tempfile
import unittest
from probe_mail_attachments import owned, close_resources


class Controls(unittest.TestCase):
    def test_one_failed_cleanup_does_not_skip_other_children_or_handler(self):
        calls = []
        class Resource:
            def __init__(self, name, fail=False): self.name, self.fail = name, fail
            def close(self):
                calls.append(self.name)
                if self.fail: raise RuntimeError('synthetic failure')
        with self.assertRaisesRegex(RuntimeError, 'fixture cleanup failed'):
            close_resources([Resource('pty', True), Resource('second-pty')],
                            [Resource('home', True), Resource('second-home')],
                            Resource('handler'))
        self.assertEqual(calls, ['pty', 'second-pty', 'home', 'second-home', 'handler'])

    def test_private_owned_root_rejects_loose_directory_wrong_marker_and_symlink(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve(); root.chmod(0o700)
            marker = root/'OWNED'
            marker.write_bytes(b'Owned native attachment console proof\n')
            self.assertEqual(owned(root), root)
            root.chmod(0o755)
            with self.assertRaises(ValueError): owned(root)
            root.chmod(0o700); marker.write_bytes(b'other fixture')
            with self.assertRaises(ValueError): owned(root)
            marker.write_bytes(b'Owned native attachment console proof\n')
            link = root/'link'; link.symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError): owned(link)


if __name__ == '__main__': unittest.main()
