import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
repo = Path('/Users/cam/DEV/faction-bbs')
roots = {k: Path(f'/private/tmp/faction233-{k}-OWNED/public-staging') for k in ('native', 'client', 'lead')}
expected = {'native': 'dda3f0c0f87b71c9bffa05a332750383587748f3db7a002a23695acfede46520', 'client': '5271a082be5a389490961ea0165c18ef0ced96355d8a12ce88adfd3bf4c19436', 'lead': '570d9df54ea7fe326622fed29b3b27b6bea82b8abd800a9abe64bfaf3c4a76d1'}
spec = importlib.util.spec_from_file_location('guard', repo / 'tools/check_reply_evidence.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read(path):
    return json.loads(path.read_bytes())

available = set()
stages = {}
for kind, root in roots.items():
    assert sha(root / 'manifest.json') == expected[kind]
    manifest = read(root / 'manifest.json')
    actual = {str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p.name != 'manifest.json'}
    assert actual == set(manifest)
    for name, digest in manifest.items():
        path = root / name
        assert not path.is_symlink() and path.resolve().is_relative_to(root.resolve())
        assert sha(path) == digest, name
        available.add(digest)
        if name.startswith('objects/'):
            assert path.name == digest
    for stage in read(root / 'source-stages.json'):
        ref = stage['manifest']
        path = root / ref['relativepath']
        assert sha(path) == ref['sha256']
        data = read(path)
        assert data
        stages[kind, stage['label']] = data
    print(kind, 'manifest files verified', len(manifest))

for key, data in stages.items():
    assert all(digest in available for digest in data.values()), key
print('all', len(stages), 'source stages resolve to captured bytes')
native = stages['native', 'native-final']
assert len(native) == 699
assert native == stages['native', 'native-final01-after']
runtime = stages['native', 'runtime-code-final05']
assert len(runtime) == 9
assert runtime == stages['native', 'runtime-code-guest-match05'] == stages['native', 'runtime-code-host-final-after']
for name in ('crates/faction-client/src/checkpoint_ui.rs', 'crates/faction-client/src/checkpoint_ui_reply.rs'):
    assert native[name] == sha(repo / name)
print('native699/runtime9 source maps and final production sources verified')

root = roots['native']
association = read(root / 'logs/capture-associations.json')['routes']
ids = set()
for platform in ('host', 'netbsd'):
    for mode in ('classic', 'unicode'):
        prefix = 'host' if platform == 'host' else 'native-runtime'
        for kind in ('local', 'main'):
            suffix = '-final' if platform == 'host' else ''
            name = f'logs/{prefix}-{kind}-{mode}{suffix}.json'
            ref = {'path': name, 'sha256': sha(root / name)}
            data = guard.route(root, ref, kind, 'host' if platform == 'host' else 'native', mode, available)
            if kind == 'main':
                a = association[platform + '_' + mode]
                assert a['result_sha256'] == ref['sha256']
                for field in ('capture_id', 'source_sha256', 'private_input_hashes'):
                    assert a[field] == data[field]
                assert a['generated_before_capture'] is True and a['input_trace_unchanged'] is True
                assert data['capture_id'] not in ids
                ids.add(data['capture_id'])
                assert data['marker_count'] == 15
print('local11/main6 on both OS/modes and four unique result/source/input associations verified')
