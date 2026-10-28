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
        for name in ('key_manifest', 'global_manifest', 'file_baseline_manifest', 'native_client_baseline_manifest', 'mail_client_baseline_manifest', 'room_reader_baseline_manifest', 'post_metadata_baseline_manifest', 'authored_articles_baseline_manifest', 'broader_articles_baseline_manifest', 'unicode_subjects_baseline_manifest', 'modern_terminals_baseline_manifest', 'kitty_graphics_baseline_manifest', 'terminal_compatibility_baseline_manifest', 'menu_themes_baseline_manifest', 'seasonal_themes_baseline_manifest', 'custom_menus_baseline_manifest', 'nightwire_baseline_manifest', 'operator_baseline_manifest', 'operator_host_baseline_manifest', 'operator_recovery_baseline_manifest', 'directory_baseline_manifest', 'board_launcher_baseline_manifest', 'room_browser_baseline_manifest', 'writing_desk_baseline_manifest', 'full_screen_baseline_manifest', 'community_wall_baseline_manifest', 'member_pages_baseline_manifest', 'local_mail_vault_baseline_manifest', 'client_vault_integration_baseline_manifest', 'gatherings_baseline_manifest', 'polls_baseline_manifest', 'catchup_baseline_manifest', 'discussion_baseline_manifest', 'gallery_baseline_manifest', 'travel_baseline_manifest', 'library_baseline_manifest', 'file_requests_baseline_manifest', 'magazine_baseline_manifest', 'editorial_baseline_manifest', 'social_baseline_manifest', 'boardwide_social_baseline_manifest', 'standalone_social_baseline_manifest', 'idle_notifications_baseline_manifest', 'who_baseline_manifest', 'connection_desk_baseline_manifest', 'passport_baseline_manifest', 'nightwire_refresh_baseline_manifest', 'checkpoint_ui_baseline_manifest', 'native_network_baseline_manifest', 'network_prekeys_baseline_manifest', 'network_dependency_baseline_manifest', 'home_mail_archive_baseline_manifest', 'archive_mail_picker_baseline_manifest', 'picker_mail_unread_baseline_manifest', 'attachment_mail_baseline_manifest', 'native_attachment_desk_baseline_manifest'):
            mocks.enter_context(patch.object(evidence,name,return_value={'files':{}}))
        yield

class ExecutedSources(unittest.TestCase):
    def test_attachment_desk_baseline_resolves_after_live_edits(self):
        baseline = evidence.native_attachment_desk_baseline_manifest()
        self.assertEqual(len(baseline['files']), 7)
        original_read = Path.read_bytes
        def changed_live(path):
            if path in {evidence.ROOT / name for name in baseline['files']}:
                return b'changed live attachment desk producer'
            return original_read(path)
        with patch.object(Path, 'read_bytes', changed_live):
            for name, digest in baseline['files'].items():
                self.assertEqual(hashlib.sha256(evidence.source_bytes(name, digest)).hexdigest(), digest)
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            path = root / 'dev-docs/evidence/native-attachment-desk-baseline-manifest.json'
            path.parent.mkdir(parents=True)
            path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError, 'frozen native attachment desk baseline changed'):
                evidence.native_attachment_desk_baseline_manifest(root)

    def test_archive_mail_picker_baseline_preserves_existing_objects_and_record(self):
        manifest=evidence.archive_mail_picker_baseline_manifest()
        self.assertEqual(len(manifest['files']),644)
        self.assertEqual(set(manifest['files']),set(manifest['source_paths']))
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        for name,digest in manifest['records'].items():
            self.assertEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            (base/'archive-mail-picker-baseline-manifest.json').write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen archive-mail picker baseline changed'):
                evidence.archive_mail_picker_baseline_manifest(root)

    def test_explicit_historical_object_binding_survives_edit_and_rejects_escape(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            old=b'exact executed object';digest=hashlib.sha256(old).hexdigest()
            (base/'object').write_bytes(old);(root/'same.rs').write_bytes(b'new current')
            data=dict(files={'same.rs':digest},source_root='unused',source_paths={'same.rs':'dev-docs/evidence/object'})
            raw=json.dumps(data).encode();(base/'manifest.json').write_bytes(raw)
            with isolated_history(hashlib.sha256(raw).hexdigest()):
                self.assertEqual(evidence.source_bytes('same.rs',digest,root),old)
                (base/'object').write_bytes(b'changed object')
                with self.assertRaisesRegex(AssertionError,'executed source bytes changed'):
                    evidence.source_bytes('same.rs',digest,root)
            data['source_paths']['same.rs']='../outside'
            raw=json.dumps(data).encode();(base/'manifest.json').write_bytes(raw)
            with isolated_history(hashlib.sha256(raw).hexdigest()):
                with self.assertRaisesRegex(AssertionError,'source path escaped repository'):
                    evidence.source_bytes('same.rs',digest,root)

    def test_home_mail_archive_baseline_preserves_prior_sources_and_records(self):
        manifest=evidence.home_mail_archive_baseline_manifest()
        self.assertEqual(len(manifest['files']),295)
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        for name,digest in manifest['records'].items():
            self.assertEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            (base/'home-mail-archive-baseline-manifest.json').write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen home-mail archive baseline changed'):
                evidence.home_mail_archive_baseline_manifest(root)

    def test_network_dependency_baseline_is_pinned(self):
        manifest=evidence.network_dependency_baseline_manifest()
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            (base/'native-network-dependency-baseline-manifest.json').write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen network dependency baseline changed'):
                evidence.network_dependency_baseline_manifest(root)

    def test_network_prekeys_baseline_is_pinned(self):
        manifest=evidence.network_prekeys_baseline_manifest()
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            (base/'native-network-prekeys-baseline-manifest.json').write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen network prekeys baseline changed'):
                evidence.network_prekeys_baseline_manifest(root)

    def test_native_network_baseline_preserves_executed_inputs(self):
        manifest=evidence.native_network_baseline_manifest()
        self.assertEqual(manifest['origin_commit'],'2224c41df1b6c633b27fd4243e60a99df2532dff')
        self.assertEqual(len(manifest['files']),9)
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        for name,digest in manifest['records'].items():
            self.assertEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            path=base/'native-network-baseline-manifest.json'
            path.write_bytes((evidence.ROOT/'dev-docs/evidence/native-network-baseline-manifest.json').read_bytes())
            self.assertEqual(evidence.native_network_baseline_manifest(root),manifest)
            path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen native network baseline changed'):
                evidence.native_network_baseline_manifest(root)

    def test_checkpoint_ui_preserves_measured_backend_and_pins_manifest(self):
        manifest=evidence.checkpoint_ui_baseline_manifest()
        self.assertEqual(manifest['origin_commit'],'af7d681dd1297c752286dda98cf1b25de09b1e0c')
        for name,digest in manifest['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        for name,digest in manifest['records'].items():
            self.assertEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            path=base/'checkpoint-ui-baseline-manifest.json'
            path.write_bytes((evidence.ROOT/'dev-docs/evidence/checkpoint-ui-baseline-manifest.json').read_bytes())
            self.assertEqual(evidence.checkpoint_ui_baseline_manifest(root),manifest)
            path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen checkpoint UI baseline changed'):
                evidence.checkpoint_ui_baseline_manifest(root)

    def test_nightwire_refresh_preserves_original_menu_and_pins_manifest(self):
        name='docs/images/nightwire-menu.svg'
        record=json.loads((evidence.ROOT/'dev-docs/evidence/board-guide-netbsd/results.json').read_text())
        digest=record['execution_sources'][name]
        self.assertEqual(hashlib.sha256(evidence.source_bytes(name,digest)).hexdigest(),digest)
        self.assertNotEqual(hashlib.sha256((evidence.ROOT/name).read_bytes()).hexdigest(),digest)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'dev-docs/evidence';base.mkdir(parents=True)
            path=base/'nightwire-refresh-baseline-manifest.json'
            path.write_bytes((evidence.ROOT/'dev-docs/evidence/nightwire-refresh-baseline-manifest.json').read_bytes())
            self.assertEqual(evidence.nightwire_refresh_baseline_manifest(root)['files'][name],digest)
            path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError,'frozen Nightwire refresh baseline changed'):
                evidence.nightwire_refresh_baseline_manifest(root)

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


    def test_attachment_mail_baseline_retains_checkpoint_after_live_edits(self):
        path = evidence.ROOT / 'dev-docs/evidence/attachment-mail-baseline-manifest.json'
        baseline = json.loads(path.read_bytes())
        name = 'crates/faction-client/src/mail_checkpoint.rs'
        expected = baseline['files'][name]
        saved = evidence.ROOT / 'dev-docs/evidence' / baseline['source_root'] / name
        original_read = Path.read_bytes
        def changed_live_source(path):
            if path == evidence.ROOT / name:
                return b'deliberately changed live checkpoint source for resolver regression'
            return original_read(path)
        with patch.object(Path, 'read_bytes', changed_live_source):
            self.assertEqual(evidence.source_bytes(name, expected), original_read(saved))
        self.assertEqual(evidence.attachment_mail_baseline_manifest(), baseline)
        for source, digest in baseline['files'].items():
            self.assertEqual(hashlib.sha256(evidence.source_bytes(source, digest)).hexdigest(), digest)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / 'dev-docs/evidence/attachment-mail-baseline-manifest.json'
            manifest.parent.mkdir(parents=True)
            manifest.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError, 'frozen attachment mail baseline changed'):
                evidence.attachment_mail_baseline_manifest(root)

if __name__=='__main__':unittest.main()
