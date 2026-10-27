"""Read-only byte/type/provenance audit of two already-imported exports."""
import hashlib
import json
from pathlib import Path, PurePosixPath

root = Path('/Users/cam/DEV/faction-bbs')
base = root / 'dev-docs/evidence/main-menu-mail'
out = Path('/private/tmp/faction230-review-OWNED/staged-layout-audit02')
out.mkdir(mode=0o700, exist_ok=False)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def valid_path(name):
    path = PurePosixPath(name)
    assert type(name) is str and name and str(path) == name
    assert not path.is_absolute() and '..' not in path.parts and '\\' not in name
    assert '.DS_Store' not in path.parts and '__pycache__' not in path.parts
    return path

def valid_hash(value):
    assert type(value) is str and len(value) == 64
    assert all(c in '0123456789abcdef' for c in value)

result = {'schema': 1, 'issue': 230, 'result': 'pass', 'root_record_audit': 'pending assembly', 'runtime_tests_executed': False}
for label, source in [('execution', Path('/private/tmp/faction230-packaging-OWNED/manifest.json')), ('review', Path('/private/tmp/faction230-review-OWNED/public/manifest.json'))]:
    raw = source.read_bytes()
    (out / (label + '-original-manifest.json')).write_bytes(raw)
    manifest = json.loads(raw)
    assert type(manifest) is dict and manifest
    if label == 'review':
        assert digest(raw) == '4015e75961f6aad6d78238cc0134de0583709f86624855660688eabbab6c3586'
    for name, expected in manifest.items():
        valid_path(name)
        valid_hash(expected)
        file = base / label / name
        assert not file.is_symlink() and file.resolve().is_relative_to((base / label).resolve())
        assert digest(file.read_bytes()) == expected, str(file)
    actual = {str(p.relative_to(base / label)) for p in (base / label).rglob('*') if p.is_file()}
    assert actual == set(manifest), (label, sorted(actual - set(manifest)), sorted(set(manifest) - actual))
    result[label] = {'files': len(manifest), 'manifest_sha256': digest(raw), 'imported_bytes_match': True, 'extra_or_missing_files': False}

execution = base / 'execution'
index = json.loads((execution / 'object-index.json').read_bytes())
for key, row in index.items():
    valid_hash(key)
    valid_path(row['path'])
    assert row['path'] == 'source-objects/' + key
    payload = (execution / row['path']).read_bytes()
    assert type(row['bytes']) is int and row['bytes'] == len(payload)
    assert digest(payload) == key
    assert type(row['logical_names']) is list and row['logical_names']
    for name in row['logical_names']:
        valid_path(name)
stages = json.loads((execution / 'source-stages.json').read_bytes())
all_details = json.loads((execution / 'stage-details.json').read_bytes())
details = [row for row in all_details if 'label' in row]
assert len(stages) == len(details) and len({stage['label'] for stage in stages}) == len(stages)
detail_by_label = {row['label']: row for row in details}
for stage in stages:
    assert type(stage['label']) is str and stage['label'] and stage['files']
    named = {}
    for row in stage['files']:
        name, sha = row['name'], row['sha256']
        valid_path(name)
        valid_hash(sha)
        assert name not in named
        assert name in index[sha]['logical_names']
        named[name] = sha
    details = detail_by_label[stage['label']]
    original = details['original_map']
    valid_path(original['path'])
    payload = (execution / original['path']).read_bytes()
    assert digest(payload) == original['sha256'] and len(payload) == original['bytes']
    original_map = json.loads(payload)
    # Original source maps also contain non-source repository paths. Preserve
    # them verbatim; every selected executable/source entry must still match.
    assert all(original_map.get(name) == sha for name, sha in named.items())
    assert details['source_entries'] == len(named)
result['execution_source_objects'] = len(index)
result['execution_source_stages'] = len(stages)
result['source_identity_and_types'] = 'pass'
result['privacy_boundary'] = 'Only public source/log/hash exports inspected; private inventory and native retrospective capture verdict remain separately pending.'
(out / 'result.json').write_text(json.dumps(result, sort_keys=True, indent=2) + '\n')
print(json.dumps(result, sort_keys=True, indent=2))
