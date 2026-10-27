"""Historical research inventories stay fixed when the product graph advances."""
import importlib
import io
import json
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

import research_dependency_inventory as inventory
from evidence_sources import ROOT, network_dependency_baseline_manifest, source_bytes

GUARDS = ['check_ui_dependencies', 'check_shared_dependencies', 'check_headroom_dependencies', 'check_transport_dependencies']
TARGET = 'aarch64-unknown-netbsd'
NAME = 'dev-docs/dependencies/local-core.json'


def frozen_bytes():
    digest = network_dependency_baseline_manifest()['files'][NAME]
    return source_bytes(NAME, digest)


class ResearchInventory(unittest.TestCase):
    def test_exact_reviewed_pre209_smallvec_not_current_product_union(self):
        reviewed = inventory.reviewed_research_inventory()
        self.assertEqual(reviewed, json.loads(frozen_bytes()))
        for rows in reviewed.values():
            smallvec = [row for row in rows if row['name'] == 'smallvec']
            self.assertEqual(smallvec, [dict(name='smallvec', version='1.16.3', license='MIT OR Apache-2.0', features=[])])
            self.assertFalse(any(row['name'] == 'smallvec' and row['version'] == '1.16.4' for row in rows))

    def test_isolated_frozen_bytes_and_manifest_tampering_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest_path = root / 'dev-docs/evidence/native-network-dependency-baseline-manifest.json'
            manifest_path.parent.mkdir(parents=True)
            original = (ROOT / manifest_path.relative_to(root)).read_bytes()
            manifest_path.write_bytes(original)
            frozen = root / 'frozen.json'
            frozen.write_bytes(frozen_bytes())
            digest = json.loads(original)['files'][NAME]
            with patch.object(inventory, 'source_bytes', side_effect=lambda name, expected, root: frozen.read_bytes()) as resolve:
                self.assertEqual(inventory.reviewed_research_inventory(root), json.loads(frozen_bytes()))
                resolve.assert_called_once_with(NAME, digest, root=root)
                frozen.write_bytes(frozen.read_bytes() + b' ')
                with self.assertRaises(AssertionError):inventory.reviewed_research_inventory(root)
                frozen.write_bytes(frozen_bytes())
                manifest_path.write_bytes(original + b' ')
                with self.assertRaises(AssertionError):inventory.reviewed_research_inventory(root)

    def run_guard(self, guard_name, row, source, reviewed):
        guard = importlib.import_module(guard_name)
        package_id = 'registry+smallvec@' + row['version']
        metadata = dict(resolve=dict(nodes=[dict(id=package_id, features=row['features'])]),
                        packages=[dict(row, id=package_id, source='registry+https://github.com/rust-lang/crates.io-index', manifest_path=str(source / 'Cargo.toml'))])
        with patch.object(guard, 'reviewed_research_inventory', return_value=reviewed) as pinned, patch.object(guard, 'TARGETS', [TARGET]), patch.object(guard.subprocess, 'check_output', return_value=json.dumps(metadata)) as command, redirect_stdout(io.StringIO()):
            guard.check()
            pinned.assert_called_once_with()
            self.assertIn('--locked', command.call_args.args[0])
            self.assertIn('--filter-platform', command.call_args.args[0])

    def test_all_four_guards_accept_the_real_frozen_smallvec(self):
        reviewed = inventory.reviewed_research_inventory()
        row = next(row for row in reviewed[TARGET] if row['name'] == 'smallvec')
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            (source / 'Cargo.toml').write_text('[package]\nname="smallvec"\n')
            for guard in GUARDS:
                with self.subTest(guard=guard):self.run_guard(guard, row, source, reviewed)

    def test_all_four_guards_keep_version_license_and_feature_refusals(self):
        reviewed = inventory.reviewed_research_inventory()
        row = next(row for row in reviewed[TARGET] if row['name'] == 'smallvec')
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            for guard in GUARDS:
                for mutation in [dict(version='1.16.4'), dict(version='99.0.0'), dict(license='Unreviewed'), dict(features=['unreviewed-feature'])]:
                    with self.subTest(guard=guard,mutation=mutation), self.assertRaisesRegex(AssertionError, 'unreviewed experiment dependency/features'):
                        self.run_guard(guard, dict(row, **mutation), source, reviewed)

    def test_all_four_guards_keep_prohibited_source_scan(self):
        reviewed = inventory.reviewed_research_inventory()
        row = next(row for row in reviewed[TARGET] if row['name'] == 'smallvec')
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            # A filename-only synthetic policy violation; no executable JS source.
            (source / 'prohibited.js').touch()
            for guard in GUARDS:
                with self.subTest(guard=guard), self.assertRaisesRegex(AssertionError, 'prohibited dependency source'):
                    self.run_guard(guard, row, source, reviewed)


if __name__ == '__main__':unittest.main()
