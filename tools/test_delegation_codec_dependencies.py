"""Independent fail-closed contracts for the isolated codec source gate."""
import copy
import unittest
from pathlib import Path
from unittest.mock import patch
import check_delegation_codec_dependencies as policy
import check_network_mail_dependencies as network
import test_network_mail_policy as fixtures
import tempfile
import io
import tarfile
import hashlib


def manifest():
    return {'package': {'name': 'faction-delegation-codec', 'version': '0.1.0-alpha.1',
        'edition': '2024', 'rust-version': '1.96', 'license': 'BSD-3-Clause', 'publish': False},
        'workspace': {}, 'dependencies': {
            'httparse': {'version': '=1.10.1', 'default-features': False, 'features': ['std']},
            'form_urlencoded': {'version': '=1.2.2', 'default-features': False, 'features': ['std']},
            'serde_json': {'version': '=1.0.154', 'default-features': False, 'features': ['std']},
            'serde': {'version': '=1.0.229', 'default-features': False, 'features': ['std']},
            'base64': {'version': '=0.22.1', 'default-features': False, 'features': ['std']}}}


class CodecDependencies(unittest.TestCase):
    def test_exact_profile_and_no_product_membership(self):
        policy.validate_profile(manifest(), {'workspace': {'members': ['crates/faction-client']}})
        for member in ['crates/faction-delegation-codec', 'crates/*', 'crates/faction-*']:
            with self.subTest(member=member), self.assertRaises(ValueError):
                policy.validate_profile(manifest(), {'workspace': {'members': [member]}})
        for key, value in [('features', {'default': ['bad']}), ('patch', {}), ('replace', {}),
                           ('build-dependencies', {}), ('dev-dependencies', {})]:
            changed = manifest(); changed[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_profile(changed, {'workspace': {'members': []}})
        changed = manifest(); changed['dependencies']['serde']['features'] = ['derive']
        with self.assertRaises(ValueError):
            policy.validate_profile(changed, {'workspace': {'members': []}})

    def test_independent_registry_allowlist_not_replacement_lock(self):
        rows = [dict(name=name, version=version, source=policy.REGISTRY, checksum=row['checksum'])
                for (name, version), row in policy.APPROVED.items()]
        self.assertEqual(len(policy.validate_lock({'package': rows})), len(rows))
        for field, value in [('checksum', '0' * 64), ('version', '999.0.0'), ('source', 'git+https://example.invalid')]:
            changed = copy.deepcopy(rows); changed[0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                policy.validate_lock({'package': changed})
        with self.assertRaises(ValueError): policy.validate_lock({'package': rows[:-1]})
        with self.assertRaises(ValueError): policy.validate_lock({'package': rows + rows[:1]})

    def test_edges_refused_in_any_dependency_table_even_renamed(self):
        for table in ['dependencies', 'dev-dependencies', 'build-dependencies']:
            for dependency in [{'path': '../faction-delegation-codec'}, {'package': 'faction-delegation-codec', 'version': '*'}]:
                for value in [{table: {'renamed': dependency}}, {'target': {'cfg(unix)': {table: {'renamed': dependency}}}}]:
                    with self.subTest(value=value), self.assertRaises(ValueError):
                        policy.validate_product_edges({'other/Cargo.toml': value})
        policy.validate_product_edges({'other/Cargo.toml': {'dependencies': {'safe': '1'}}})

    def test_reuses_unchanged_archive_and_source_policy(self):
        for name in ['scan_package', 'scan_local', 'verify_archive_source', 'scan_tree', 'selected_packages']:
            self.assertIs(getattr(policy, name), getattr(network, name))
        f = fixtures.NetworkMailPolicy()
        with tempfile.TemporaryDirectory() as temporary:
            root, archive, checksum = f.fixture(Path(temporary))
            package = f.package(root)
            self.assertEqual(policy.verify_archive_source(root, archive, checksum)['source_files'], 3)
            for name in ['build/tool.ts', '.github/helper.js', 'fixtures/package.json']:
                path = root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text('bad')
                with self.assertRaises(ValueError): policy.scan_package(package, [], checksum)
                path.unlink()
            with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, '0' * 64)
            (root / 'src/lib.rs').write_text('tampered')
            with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, checksum)

    def test_selected_identity_license_and_features_are_exact(self):
        for key, approved in policy.APPROVED.items():
            package = dict(name=key[0], version=key[1], license=approved['license'], source=policy.REGISTRY)
            policy.validate_selected(package, approved['features'])
            for field, value in [('name', 'extra'), ('license', 'unknown')]:
                with self.assertRaises(ValueError): policy.validate_selected(dict(package, **{field: value}), approved['features'])
            with self.assertRaises(ValueError): policy.validate_selected(package, approved['features'] + ['unreviewed'])

    def test_selected_isolation_graph_refuses_even_renamed_codec(self):
        policy.validate_isolation_graph({'packages': [], 'resolve': {'nodes': []}})
        package = {'id': 'codec-id', 'name': 'faction-delegation-codec'}
        with self.assertRaises(ValueError):
            policy.validate_isolation_graph({'packages': [package], 'resolve': {'nodes': [{'id': 'codec-id'}]}})

    def test_unsafe_archive_entries_and_extra_source_refused(self):
        for name, kind in [('synthetic-1/../escape', tarfile.REGTYPE),
                           ('/synthetic-1/absolute', tarfile.REGTYPE),
                           ('synthetic-1/link', tarfile.SYMTYPE),
                           ('synthetic-1/hardlink', tarfile.LNKTYPE)]:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root, archive, _ = fixtures.NetworkMailPolicy().fixture(Path(temporary))
                with tarfile.open(archive, 'w:gz') as tar:
                    member = tarfile.TarInfo(name); member.type = kind
                    member.size = 1 if kind == tarfile.REGTYPE else 0
                    tar.addfile(member, io.BytesIO(b'x') if member.size else None)
                checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
                with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, checksum)
        with tempfile.TemporaryDirectory() as temporary:
            root, archive, checksum = fixtures.NetworkMailPolicy().fixture(Path(temporary))
            (root / 'unreviewed.rs').write_text('extra')
            with self.assertRaises(ValueError): policy.verify_archive_source(root, archive, checksum)

    def test_required_ci_warms_cold_sources_before_gate_and_builds(self):
        source = ci_fixture()
        policy.validate_ci(source)
        for old, new in [
            ('        run: |', '        if: false\n        run: |'),
            ('CARGO_BUILD_JOBS: 2', 'CARGO_BUILD_JOBS: 8'),
            ('timeout-minutes: 15', 'timeout-minutes: 30'),
            ('--locked --offline -- -D warnings', '--locked -- -D warnings'),
            ('          python3 tools/check_delegation_codec_dependencies.py\n', ''),
            ('          python3 tools/check_delegation_codec_dependencies.py', '          # python3 tools/check_delegation_codec_dependencies.py')]:
            with self.subTest(new=new), self.assertRaises(ValueError): policy.validate_ci(source.replace(old,new))
        for line in source.splitlines():
            if 'cargo metadata' in line:
                with self.subTest(missing=line), self.assertRaises(ValueError):
                    policy.validate_ci(source.replace(line+'\n',''))
        gate = '          python3 tools/check_delegation_codec_dependencies.py\n'
        moved = source.replace(gate,'') + gate
        with self.assertRaises(ValueError): policy.validate_ci(moved)
        with self.assertRaises(ValueError): policy.validate_ci(source.replace('needs: [native_mail, provider, provider_operator]', 'needs: [native_mail]'))

    def test_ci_post_run_conditionals_and_job_tolerance_refused(self):
        source = ci_fixture()
        for key in ['if: false', 'continue-on-error: true']:
            with self.subTest(key=key), self.assertRaises(ValueError):
                policy.validate_ci(source + '        ' + key + '\n')
        with self.assertRaises(ValueError):
            policy.validate_ci(source.replace('    steps:', '    continue-on-error: true\n    steps:'))

    def test_actual_repository_workflow_codec_gate_is_mandatory(self):
        # Independently invoked by the normal Documentation unittest discovery.
        # A skipped codec step cannot skip this check of the actual YAML source.
        policy.validate_ci((policy.ROOT / '.github/workflows/documentation.yml').read_text())

    def test_inventory_is_read_only(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'profile.json'; path.write_text('expected')
            policy.check_inventory(path, 'expected')
            with self.assertRaises(ValueError): policy.check_inventory(path, 'drift')
            self.assertEqual(path.read_text(), 'expected')



def ci_fixture():
    lines = ['jobs:', '  documentation:', '    name: Documentation and policy',
             '    needs: [native_mail, provider, provider_operator]', '    if: ${{ always() }}',
             '    runs-on: ubuntu-24.04', '    timeout-minutes: 15', '    steps:',
             '      - name: Validate the isolated delegation wire codec', '        env:',
             '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-delegation-codec',
             '          CARGO_BUILD_JOBS: 2', '          RUSTUP_TOOLCHAIN: 1.96.0', '        run: |',
             '          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy']
    for manifest, feature in [('crates/faction-delegation-codec/Cargo.toml', ''),
                               ('Cargo.toml', ' --features faction-client/device-provider'),
                               ('crates/faction-provider/Cargo.toml', ' --features operator'),
                               ('crates/faction-provider/Cargo.toml', ' --features delegation-wire'),
                               ('crates/faction-provider/Cargo.toml', ' --features delegation-wire,durable-policy'),
                               ('crates/faction-provider/Cargo.toml', ' --features delegation-verifier'),
                               ('crates/faction-provider/Cargo.toml', ' --features delegation-verifier,durable-policy')]:
        for target in ['x86_64-unknown-linux-gnu', 'aarch64-unknown-netbsd', 'aarch64-apple-darwin']:
            lines.append('          cargo metadata --locked --manifest-path '+manifest+' --format-version 1 --filter-platform '+target+feature+' > /dev/null')
    lines += ['          python3 tools/check_delegation_codec_dependencies.py',
        '          cargo fmt --manifest-path crates/faction-delegation-codec/Cargo.toml -- --check',
        '          cargo clippy --manifest-path crates/faction-delegation-codec/Cargo.toml --all-targets --release --locked --offline -- -D warnings',
        '          cargo test --manifest-path crates/faction-delegation-codec/Cargo.toml --release --locked --offline']
    return '\n'.join(lines)+'\n'

if __name__ == '__main__': unittest.main()

class QualifiedProviderWireEdge(unittest.TestCase):
    def test_exact_optional_wire_edge_is_allowed_with_ordinary_feature_refusals(self):
        edge={'path':'../faction-delegation-codec','optional':True,'default-features':False}
        candidate={'features':{'default':[],'delegation-issuer':['delegation-records','delegation-services'],
                   'delegation-wire':['delegation-issuer','dep:faction-delegation-codec']},'dependencies':{'faction-delegation-codec':edge}}
        policy.validate_product_edges({'crates/faction-provider/Cargo.toml':candidate})
        for path in ['crates/faction-client/Cargo.toml','crates/other/Cargo.toml']:
            with self.subTest(path=path),self.assertRaises(ValueError):policy.validate_product_edges({path:candidate})
        for feature in ['default','operator','delegation-issuer','alias']:
            changed=copy.deepcopy(candidate);changed['features'][feature]=['delegation-wire']
            with self.subTest(feature=feature),self.assertRaises(ValueError):policy.validate_product_edges({'crates/faction-provider/Cargo.toml':changed})
        for key,value in [('optional',False),('default-features',True),('path','../elsewhere')]:
            changed=copy.deepcopy(candidate);changed['dependencies']['faction-delegation-codec'][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):policy.validate_product_edges({'crates/faction-provider/Cargo.toml':changed})
    def test_ordinary_graph_refuses_codec_only_registry_even_without_codec_node(self):
        for name in ['httparse','form_urlencoded','percent-encoding']:
            metadata={'packages':[{'id':'injected','name':name,'version':'999'}], 'resolve':{'nodes':[{'id':'injected','features':[]}]}}
            with self.subTest(name=name),self.assertRaises(ValueError):policy.validate_isolation_graph(metadata)
