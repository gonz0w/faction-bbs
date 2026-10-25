"""Behavioral contract for the optional product dependency adoption gate."""
import copy
import hashlib
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path
import check_network_mail_dependencies as policy


class NetworkMailPolicy(unittest.TestCase):
    def fixture(self, directory):
        root = directory / 'synthetic-1'
        root.mkdir()
        files = {'Cargo.toml': b'[package]\nname="synthetic"\n', 'src/lib.rs': b'pub fn reviewed() {}\n', 'LICENSE': b'Synthetic MIT terms\n'}
        archive = directory / 'synthetic-1.crate'
        with tarfile.open(archive, 'w:gz') as tar:
            for name, content in files.items():
                p = root / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(content)
                member = tarfile.TarInfo('synthetic-1/' + name)
                member.size = len(content)
                tar.addfile(member, io.BytesIO(content))
        return root, archive, hashlib.sha256(archive.read_bytes()).hexdigest()

    def package(self, root):
        return dict(id='synthetic-id', name='synthetic', version='1', license='MIT', source=policy.REGISTRY, manifest_path=str(root / 'Cargo.toml'))

    def profiles(self):
        return (
            {'workspace': {'exclude': ['crates/faction-xmpp']}},
            {'features': {'default': [], 'native-mail': ['dep:faction-omemo', 'faction-omemo/native'], 'network-mail': ['native-mail', 'dep:faction-xmpp', 'dep:tokio', 'dep:roxmltree']},
             'dependencies': {'faction-xmpp': {'path': '../faction-xmpp', 'optional': True}, 'tokio': {'version': '=1.54.1', 'default-features': False, 'features': ['rt', 'time', 'net', 'io-util', 'macros'], 'optional': True}, 'roxmltree': {'workspace': True, 'optional': True}}},
            {'package': {'publish': False}, 'workspace': {}, 'dependencies': {
                'tokio-xmpp': {'version': '=6.0.0', 'default-features': False, 'features': ['native-tls']},
                'native-tls': '=0.2.18', 'tokio-native-tls': '=0.3.1',
                'tokio': {'version': '=1.54.1', 'default-features': False, 'features': ['rt', 'time', 'net', 'io-util', 'macros']},
                'rxml': {'version': '=0.14.0', 'default-features': False},
                'sasl': '=0.5.2', 'futures-util': {'version': '=0.3.34', 'default-features': False, 'features': ['sink']},
                'sha2': '=0.10.9', 'log': {'version': '=0.4.34', 'features': ['max_level_off', 'release_max_level_off']},
                'xso': {'version': '=0.4.0', 'default-features': False}},
             'dev-dependencies': {'base64': '=0.22.1'}})

    def test_selected_source_license_and_feature_inventory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, _, checksum = self.fixture(Path(tmp))
            row = policy.scan_package(self.package(root), ['z', 'a'], checksum)
            self.assertEqual(row, dict(name='synthetic', version='1', license='MIT', features=['a', 'z'], checksum=checksum))

    def test_every_selected_source_subdirectory_obeys_no_js(self):
        for path in ['src/hidden.js', 'build/helper.TS', '.github/workflows/helper.mjs', 'fixtures/package.json', 'doc/node_modules/file.txt']:
            with self.subTest(path=path), tempfile.TemporaryDirectory() as tmp:
                root, _, checksum = self.fixture(Path(tmp))
                file = root / path
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text('unreviewed')
                with self.assertRaises(ValueError):
                    policy.scan_package(self.package(root), [], checksum)

    def test_license_source_and_symlink_refusal(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, _, checksum = self.fixture(Path(tmp))
            for field, value in [('license', None), ('license', 'Proprietary'), ('source', 'git+https://example.invalid/repo')]:
                p = self.package(root)
                p[field] = value
                with self.assertRaises(ValueError):
                    policy.scan_package(p, [], checksum)
            (root / 'alias.rs').symlink_to(root / 'src/lib.rs')
            with self.assertRaises(ValueError):
                policy.scan_package(self.package(root), [], checksum)

    def test_locked_archive_and_every_extracted_byte_match(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, archive, checksum = self.fixture(Path(tmp))
            (root / '.cargo-ok').write_text('local marker')
            evidence = policy.verify_archive_source(root, archive, checksum)
            self.assertEqual(evidence['source_files'], 3)
            self.assertEqual(evidence['source_bytes'], sum(p.stat().st_size for p in root.rglob('*') if p.is_file() and p.name != '.cargo-ok'))
            self.assertEqual(len(evidence['source_sha256']), 64)
            (root / 'src/lib.rs').write_text('tampered')
            with self.assertRaises(ValueError):
                policy.verify_archive_source(root, archive, checksum)

    def test_archive_tampering_extra_files_and_nested_markers_are_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            root, archive, checksum = self.fixture(Path(tmp))
            with self.assertRaises(ValueError):
                policy.verify_archive_source(root, archive, '0' * 64)
            (root / 'src/.cargo-ok').write_text('hidden source')
            with self.assertRaises(ValueError):
                policy.verify_archive_source(root, archive, checksum)
            (root / 'src/.cargo-ok').unlink()
            archive.write_bytes(archive.read_bytes() + b'changed archive')
            with self.assertRaises(ValueError):
                policy.verify_archive_source(root, archive, checksum)

    def test_archive_links_traversal_and_duplicate_members_are_refused(self):
        for kind in ['link', 'traversal', 'duplicate', 'wrong-prefix']:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as tmp:
                directory = Path(tmp)
                root = directory / 'synthetic-1'
                root.mkdir()
                (root / 'lib.rs').write_bytes(b'x')
                archive = directory / 'bad.crate'
                with tarfile.open(archive, 'w:gz') as tar:
                    member = tarfile.TarInfo('synthetic-1/lib.rs')
                    member.size = 1
                    if kind == 'link':
                        member.type = tarfile.SYMTYPE
                        member.linkname = '/outside'
                    if kind == 'traversal':
                        member.name = 'synthetic-1/../lib.rs'
                    if kind == 'wrong-prefix':
                        member.name = 'other-1/lib.rs'
                    tar.addfile(member, io.BytesIO(b'x'))
                    if kind == 'duplicate':
                        tar.addfile(member, io.BytesIO(b'x'))
                checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
                with self.assertRaises(ValueError):
                    policy.verify_archive_source(root, archive, checksum)

    def test_only_selected_graph_is_scanned_but_no_selected_node_is_lost(self):
        data = {'packages': [{'id': 'yes'}, {'id': 'unselected-js-package'}], 'resolve': {'nodes': [{'id': 'yes', 'features': ['selected']}]}}
        self.assertEqual(policy.selected_packages(data), [({'id': 'yes'}, ['selected'])])
        bad = copy.deepcopy(data)
        bad['resolve']['nodes'].append({'id': 'missing', 'features': []})
        with self.assertRaises(ValueError):
            policy.selected_packages(bad)
        bad = copy.deepcopy(data)
        bad['resolve']['nodes'].append({'id': 'yes', 'features': []})
        with self.assertRaises(ValueError):
            policy.selected_packages(bad)

    def test_default_off_and_exact_transport_profile_are_mandatory(self):
        policy.validate_profile(*self.profiles())
        for mutation in ['enabled', 'tls', 'logging', 'required', 'patch', 'unexpected']:
            workspace, client, transport = self.profiles()
            if mutation == 'enabled': client['features']['default'] = ['network-mail']
            if mutation == 'tls': transport['dependencies']['tokio-xmpp']['features'] = ['rustls-tls']
            if mutation == 'logging': transport['dependencies']['log']['features'] = []
            if mutation == 'required': client['dependencies']['faction-xmpp']['optional'] = False
            if mutation == 'patch': workspace['patch'] = {'crates-io': {}}
            if mutation == 'unexpected': transport['dependencies']['new-library'] = '1'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                policy.validate_profile(workspace, client, transport)

    def test_inventory_change_requires_deliberate_write(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'inventory.json'
            expected = json.dumps({'schema': 1, 'features': ['native-tls']}) + '\n'
            path.write_text(expected)
            policy.check_inventory(path, expected)
            with self.assertRaises(ValueError):
                policy.check_inventory(path, expected.replace('native-tls', 'other-tls'))
            self.assertEqual(path.read_text(), expected)
            with self.assertRaises(ValueError):
                policy.check_inventory(path.with_name('absent'), expected)

if __name__ == '__main__':
    unittest.main()
