"""Prior picker runs must remain bound to their actual source/result bytes."""
import hashlib,tempfile,unittest
from pathlib import Path
import evidence_sources as evidence

class UnreadBaseline(unittest.TestCase):
    def test_immutable_picker_sources_and_record(self):
        m=evidence.picker_mail_unread_baseline_manifest()
        self.assertEqual(m['base_commit'],'8768bb41a49ab1c15c834fbad2102d7522c48937')
        self.assertEqual(len(m['files']),684)
        self.assertEqual(set(m['files']),set(m['source_paths']))
        for name,digest in m['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        for name,digest in m['records'].items():
            self.assertEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
    def test_baseline_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            (base/'picker-mail-unread-baseline-manifest.json').write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen picker-mail unread baseline changed'):
                evidence.picker_mail_unread_baseline_manifest(root)
if __name__=='__main__':unittest.main()
