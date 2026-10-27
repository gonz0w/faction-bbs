import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
repo = Path('/Users/cam/DEV/faction-bbs')
base = repo / 'dev-docs/evidence/inbox-reply'
spec = importlib.util.spec_from_file_location('guard', repo / 'tools/check_reply_evidence.py')
guard = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guard)
draft = json.loads((base / 'results.draft.json').read_bytes())
for ref in draft['sources'] + draft['prior_records']:
    guard.raw(repo, ref)
available = {ref['sha256'] for ref in draft['sources']}
maps = {}
for stage in draft['source_stages']:
    files = json.loads(guard.raw(repo, stage['manifest']))
    assert files and all(sha in available for sha in files.values())
    maps[stage['label']] = files
assert len(maps['native-final']) == 699
values = {key: json.loads(guard.raw(repo, ref)) for key, ref in draft['records'].items()}
assert values['native']['source_count'] == 699
assert values['native']['files'] == maps['native-final']
assert values['native']['compile_memory_mib'] == 2048
assert values['native']['runtime_physmem'] == 1054990336
guard.supplemental(repo, values, maps['native-final'])
print('Draft root:', len(draft['sources']), 'source refs;', len(maps), 'source stages; raw473/139/90/272 and Files14/v1two/inbox14 checks verified')
for kind in ('native', 'client', 'lead'):
    candidate = Path(f'/private/tmp/faction233-{kind}-OWNED/public-staging')
    manifest = json.loads((candidate / 'manifest.json').read_bytes())
    for name in list(manifest) + ['manifest.json']:
        assert (base / kind / name).read_bytes() == (candidate / name).read_bytes(), (kind, name)
print('All three copied candidates retain exact frozen bytes')

inventory = Path('/private/tmp/faction233-native-OWNED/private/generated-private-values.private.json')
needles = [v.encode() for v in json.loads(inventory.read_bytes())]
assert len(needles) == 81
count = 0
for path in base.rglob('*'):
    if not path.is_file():
        continue
    assert not path.is_symlink()
    data = path.read_bytes()
    assert not any(needle in data for needle in needles), 'generated private value in publication'
    count += 1
print('Independent actual81-generated-value scan passes for all', count, 'draft publication files; private data never printed')
assert 'cleanup' not in values
assert guard.RESULT_SHA256 is None
print('Intentional remaining state: no cleanup record or final root digest yet; no completion claim from this audit')
