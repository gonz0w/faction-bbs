"""Historical checks require executed bytes; new source pins never silently fall back."""
import hashlib,json,tempfile,unittest
from contextlib import ExitStack,contextmanager
from pathlib import Path
from unittest.mock import patch
import evidence_sources as evidence


@contextmanager
def isolated_history(digest):
    # ExitStack avoids Python 3.12's static block limit as snapshots accumulate.
    with ExitStack() as mocks:
        mocks.enter_context(patch.object(evidence,'MANIFEST_SHA256',digest))
        for name in ('key_manifest', 'global_manifest', 'file_baseline_manifest', 'native_client_baseline_manifest', 'mail_client_baseline_manifest', 'room_reader_baseline_manifest', 'post_metadata_baseline_manifest', 'authored_articles_baseline_manifest', 'broader_articles_baseline_manifest', 'unicode_subjects_baseline_manifest', 'modern_terminals_baseline_manifest', 'kitty_graphics_baseline_manifest', 'terminal_compatibility_baseline_manifest', 'menu_themes_baseline_manifest', 'seasonal_themes_baseline_manifest', 'custom_menus_baseline_manifest', 'nightwire_baseline_manifest', 'operator_baseline_manifest', 'operator_host_baseline_manifest', 'operator_recovery_baseline_manifest', 'directory_baseline_manifest', 'board_launcher_baseline_manifest', 'room_browser_baseline_manifest', 'writing_desk_baseline_manifest', 'full_screen_baseline_manifest', 'community_wall_baseline_manifest', 'member_pages_baseline_manifest', 'local_mail_vault_baseline_manifest', 'client_vault_integration_baseline_manifest', 'gatherings_baseline_manifest', 'polls_baseline_manifest', 'catchup_baseline_manifest', 'discussion_baseline_manifest', 'gallery_baseline_manifest', 'travel_baseline_manifest', 'library_baseline_manifest', 'file_requests_baseline_manifest', 'magazine_baseline_manifest', 'editorial_baseline_manifest'):
            mocks.enter_context(patch.object(evidence,name,return_value={'files':{}}))
        yield

class ExecutedSources(unittest.TestCase):
    def test_historical_source_survives_current_edit_but_not_snapshot_tampering(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence/sources/revision';base.mkdir(parents=True)
            old=b'executed bytes';new=b'current bytes'
            (root/'Cargo.toml').write_bytes(new);(base/'Cargo.toml').write_bytes(old)
            oldhash=hashlib.sha256(old).hexdigest();newhash=hashlib.sha256(new).hexdigest()
            raw=json.dumps({'files':{'Cargo.toml':oldhash},'source_root':'sources/revision'}).encode()
            (root/'dev-docs/evidence/manifest.json').write_bytes(raw)
            with isolated_history(hashlib.sha256(raw).hexdigest()):
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
            with isolated_history(hashlib.sha256(raw).hexdigest()):
                for version,snapshot in zip(versions,snapshots):
                    self.assertEqual(evidence.source_bytes('same.rs',snapshot['files']['same.rs'],root),version)
                (base/'1/same.rs').write_bytes(versions[0])
                with self.assertRaisesRegex(AssertionError,'executed source bytes changed'):
                    evidence.source_bytes('same.rs',snapshots[1]['files']['same.rs'],root)

    def test_key_snapshot_and_manifest_cannot_be_substituted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            frozen=evidence.ROOT/'dev-docs/evidence/key-lookup-manifest.json'
            (base/frozen.name).write_bytes(frozen.read_bytes())
            data=evidence.key_manifest(root)
            self.assertEqual(data['origin_commit'],'71491cc61bf4a6606ec65bb73ad72ed585f18927')
            (base/frozen.name).write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen key manifest changed'):
                evidence.key_manifest(root)

    def test_global_snapshot_manifest_is_immutable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            frozen=evidence.ROOT/'dev-docs/evidence/global-admission-manifest.json'
            (base/frozen.name).write_bytes(frozen.read_bytes())
            data=evidence.global_manifest(root)
            self.assertEqual(data['origin_commit'],'5b62d0a705c988e47ec2cc2ce4b74f87de600386')
            self.assertEqual(len(data['files']),145)
            (base/frozen.name).write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen global manifest changed'):
                evidence.global_manifest(root)

if __name__=='__main__':unittest.main()
