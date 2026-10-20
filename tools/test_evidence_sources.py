"""Historical checks require executed bytes; new source pins never silently fall back."""
import hashlib,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import evidence_sources as evidence

class ExecutedSources(unittest.TestCase):
    def test_historical_source_survives_current_edit_but_not_snapshot_tampering(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence/sources/revision';base.mkdir(parents=True)
            old=b'executed bytes';new=b'current bytes'
            (root/'Cargo.toml').write_bytes(new);(base/'Cargo.toml').write_bytes(old)
            oldhash=hashlib.sha256(old).hexdigest();newhash=hashlib.sha256(new).hexdigest()
            raw=json.dumps({'files':{'Cargo.toml':oldhash},'source_root':'sources/revision'}).encode()
            (root/'dev-docs/evidence/manifest.json').write_bytes(raw)
            with patch.object(evidence,'MANIFEST_SHA256',hashlib.sha256(raw).hexdigest()):
                self.assertEqual(evidence.source_bytes('Cargo.toml',oldhash,root),old)
                self.assertEqual(evidence.source_bytes('Cargo.toml',newhash,root),new)
                with self.assertRaises(AssertionError):evidence.source_bytes('../outside',newhash,root)
                (base/'Cargo.toml').write_bytes(b'tampered')
                with self.assertRaises(AssertionError):evidence.source_bytes('Cargo.toml',oldhash,root)
                (root/'Cargo.toml').write_bytes(b'different current')
                with self.assertRaises(AssertionError):evidence.source_bytes('Cargo.toml',newhash,root)
                (root/'dev-docs/evidence/manifest.json').write_bytes(b'{}')
                with self.assertRaises(AssertionError):evidence.source_bytes('Cargo.toml',newhash,root)

    def test_two_revisions_of_same_path_resolve_by_recorded_digest(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            versions=[b'first executed',b'second executed']
            snapshots=[]
            for i,raw in enumerate(versions):
                folder=base/str(i);folder.mkdir();(folder/'same.rs').write_bytes(raw)
                snapshots.append({'source_root':str(i),'files':{'same.rs':hashlib.sha256(raw).hexdigest()}})
            (root/'same.rs').write_bytes(b'current is different')
            raw=json.dumps(dict(snapshots[0],snapshots=[snapshots[1]])).encode()
            (base/'manifest.json').write_bytes(raw)
            with patch.object(evidence,'MANIFEST_SHA256',hashlib.sha256(raw).hexdigest()):
                for version,snapshot in zip(versions,snapshots):
                    self.assertEqual(evidence.source_bytes('same.rs',snapshot['files']['same.rs'],root),version)
                (base/'1/same.rs').write_bytes(versions[0])
                with self.assertRaisesRegex(AssertionError,'executed source bytes changed'):
                    evidence.source_bytes('same.rs',snapshots[1]['files']['same.rs'],root)

if __name__=='__main__':unittest.main()
