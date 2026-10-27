from pathlib import Path
import hashlib
import json

root = Path('/Users/cam/DEV/faction-bbs')
owned = Path('/private/tmp/faction230-review-OWNED')
before = json.loads((owned / 'final-review-before-manifest.json').read_text())
after = {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in before}
(owned / 'final-review-after-manifest.json').write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
evidence = {}
for name, base in [
    ('server_green', Path('/private/tmp/faction230-server-OWNED/20261026T163116-green-content-fix')),
    ('server_red', Path('/private/tmp/faction230-server-OWNED/20261026T163053-red-content-prefix')),
]:
    a = json.loads((base / 'manifest.json').read_text())
    b = json.loads((base / 'manifest-after.json').read_text())
    evidence[name] = {'source_drift': sorted(k for k in set(a) | set(b) if a.get(k) != b.get(k))}
    if name == 'server_green':
        evidence[name]['reviewed_source_mismatch'] = sorted(k for k in after if k in a and a[k] != after[k])
client = json.loads(Path('/private/tmp/faction230-client-OWNED/client-final02-manifest.json').read_text())
evidence['client_final02'] = {'reviewed_source_mismatch': sorted(k for k in client if hashlib.sha256((root / k).read_bytes()).hexdigest() != client[k])}
result = {'review_source_drift': sorted(k for k in before if before[k] != after[k]), 'evidence': evidence, 'runtime_tests_executed_by_reviewer': False, 'private_fixtures_created_by_reviewer': False}
(owned / 'final-review-audit.json').write_text(json.dumps(result, sort_keys=True, indent=2) + '\n')
print(json.dumps(result, sort_keys=True, indent=2))
