from pathlib import Path
import hashlib, json, shutil, subprocess

ROOT = Path('/private/tmp/faction235-client-OWNED')
REPO = Path('/Users/cam/DEV/faction-bbs')
OUT = ROOT / 'public-staging'

def digest(data):
    return hashlib.sha256(data).hexdigest()

def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

assert (ROOT / 'OWNED').is_file() and ROOT.stat().st_mode & 0o777 == 0o700
baseline = json.loads((ROOT / 'baseline-source-map.json').read_text())
assert baseline == json.loads((ROOT / 'interface-red-source-map.json').read_text())
production = ['crates/faction-client/src/checkpoint_ui.rs', 'crates/faction-client/src/checkpoint_ui_inbox.rs']
final = json.loads((ROOT / 'green-candidate2-source-map.json').read_text())
assert all(digest((REPO / p).read_bytes()) == final[p] for p in production)
fixture_path = 'crates/faction-client/examples/mail_correspondents_fixture.rs'
fixture_map = json.loads((ROOT / 'all-target-fixture-source-map.json').read_text())
assert digest((REPO / fixture_path).read_bytes()) == fixture_map[fixture_path]

prefixes = [f'dev-docs/evidence/inbox-reply/{role}/objects' for role in ['native', 'client', 'lead']]
tracked = subprocess.check_output(['git', 'ls-files', *prefixes], cwd=REPO, text=True).splitlines()
candidates = {}
for path in tracked:
    name = Path(path).name
    if len(name) == 64:
        candidates.setdefault(name, path)

OUT.mkdir(mode=0o700)
(OUT / 'objects').mkdir()
reuse = {}
local = {}

def preserve(data):
    sha = digest(data)
    if sha in reuse or sha in local:
        return sha
    if sha in candidates:
        path = candidates[sha]
        committed = subprocess.check_output(['git', 'show', f'HEAD:{path}'], cwd=REPO)
        assert digest(committed) == sha and committed == data
        reuse[sha] = {'repository_path': path, 'sha256': sha}
    else:
        (OUT / 'objects' / sha).write_bytes(data)
        local[sha] = f'objects/{sha}'
    return sha

stages = ['baseline', 'actual-interface-red', 'group-red', 'green-candidate1', 'green-candidate2']
for stage in stages:
    mapping = json.loads((ROOT / f'{stage}-source-map.json').read_text())
    assert len(mapping) == 265
    for source, sha in mapping.items():
        data = (ROOT / f'{stage}-source' / source).read_bytes()
        assert preserve(data) == sha
    write_json(OUT / 'source-maps' / f'{stage}.json', mapping)
fixture_sha = preserve((ROOT / 'all-target-fixture-source.rs').read_bytes())
write_json(OUT / 'source-maps' / 'all-target-fixture.json', {fixture_path: fixture_sha})
write_json(OUT / 'reuse.json', reuse)

logs = ['interface-red.log', 'actual-interface-red.log', 'actual-interface-red-helper.log',
        'group-red.log', 'green-candidate1.log', 'green-candidate2.log', 'default-tests.log',
        'native-tests.log', 'network-tests.log', 'fmt.log', 'clippy-default.log',
        'clippy-native.log', 'clippy-network.log', 'clippy-all-targets-native.log',
        'clippy-all-targets-network.log']
for name in logs:
    target = OUT / 'logs' / name
    target.parent.mkdir(exist_ok=True)
    target.write_bytes((ROOT / name).read_bytes())
results = json.loads((ROOT / 'client-results.json').read_text())
results['checks']['strict_clippy_all_targets'] = {'native': 0, 'network': 0, 'fixture_before_after': fixture_sha}
results['limits'] = [x for x in results['limits'] if 'Final integrated all-target checks pending' not in x]
results['limits'].append('Lead owns integrated workspace gates, required CI and separate review.')
results['publication_inventory_scan'] = 'Pending actual generated-private-fixture inventory scan by native/lead; not claimed by client.'
results['setup_source_identity'] = {'interface-red-source-map.json': 'identical to baseline-source-map.json; preserved privately, public baseline map is exact'}
results['frozen_source_maps'] = [f'source-maps/{stage}.json' for stage in stages] + ['source-maps/all-target-fixture.json']
results['commands'] = {
    'actual_interface_red': 'cargo test -p faction-client --features native-mail --bin faction-client --release --locked correspondents_opens_from_actual_inbox_and_backs_to_global -- --nocapture',
    'actual_interface_red_direct_helper': 'actual-interface-red-bin --ignored --exact checkpoint_ui::tests::correspondents_actual_interface_helper --nocapture',
    'model_red_and_focused_green': 'cargo test -p faction-client --features native-mail --bin faction-client --release --locked correspondent -- --nocapture',
    'full_tests': ['cargo test -p faction-client --lib --bins ' + flag + '--release --locked -- --test-threads=1' for flag in ['', '--features native-mail ', '--features network-mail ']],
    'strict_clippy_lib_bins': ['cargo clippy -p faction-client --lib --bins ' + flag + '--release --locked -- -D warnings' for flag in ['', '--features native-mail ', '--features network-mail ']],
    'strict_clippy_all_targets': ['cargo clippy -p faction-client --features ' + feature + ' --all-targets --release --locked -- -D warnings' for feature in ['native-mail', 'network-mail']],
    'format': 'rustfmt --edition 2024 --check crates/faction-client/src/checkpoint_ui.rs crates/faction-client/src/checkpoint_ui_inbox.rs',
    'environment': 'CARGO_TARGET_DIR=/private/tmp/faction235-client-OWNED/target CARGO_BUILD_JOBS=2',
}
results['logs_sha256'] = {name: digest((OUT / 'logs' / name).read_bytes()) for name in logs}
write_json(OUT / 'client-results.json', results)
write_json(OUT / 'interface-teardown.json', json.loads((ROOT / 'interface-teardown.json').read_text()))
shutil.copy2(ROOT / 'package_client_evidence.executed.py', OUT / 'packaging-source.py')
manifest = {str(p.relative_to(OUT)): digest(p.read_bytes()) for p in OUT.rglob('*') if p.is_file()}
write_json(OUT / 'package-manifest.json', manifest)
write_json(ROOT / 'public-staging-summary.json', {'files_excluding_manifest': len(manifest), 'source_objects_local': len(local), 'source_objects_reused': len(reuse), 'flat_meaningful_maps': 5, 'additional_frozen_fixture_map': 1, 'publication_inventory_scan': 'pending native/lead', 'staging': str(OUT), 'manifest_sha256': digest((OUT / 'package-manifest.json').read_bytes())})
print((ROOT / 'public-staging-summary.json').read_text())
