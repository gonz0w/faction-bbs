"""Git must retain protocol CRLF bytes in immutable #211 source objects."""
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ArchiveEvidenceBytes(unittest.TestCase):
    def test_staging_preserves_protocol_bytes_in_each_object_directory(self):
        raw = b'Subject: public protocol fixture\r\n\r\nbody\r\n'
        with tempfile.TemporaryDirectory(prefix='faction211-git-bytes-') as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(['git', '-c', 'core.autocrlf=false',
                    '-c', 'core.attributesfile=/dev/null', '-C', str(root), *args],
                    stderr=subprocess.PIPE)
            git('init', '-q')
            (root / '.gitattributes').write_bytes((ROOT / '.gitattributes').read_bytes())
            for folder in ['sources', 'native-source-objects', 'final-source-objects']:
                with self.subTest(folder=folder):
                    name = 'dev-docs/evidence/archive-mail/' + folder + '/' + 'a' * 64
                    path = root / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(raw)
                    git('add', '--', name)
                    self.assertEqual(git('show', ':' + name), raw,
                        'Git changed immutable protocol source bytes during staging')


if __name__ == '__main__':
    unittest.main()
