from pathlib import Path
import hashlib
import json

root = Path('/Users/cam/DEV/faction-bbs')
owned = Path('/private/tmp/faction230-review-OWNED')
before = json.loads((owned / 'classic-review-before.json').read_text())
after = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in before}
(owned / 'classic-review-after.json').write_text(json.dumps(after, indent=2) + '\n')
checks = {}
for name in ['20261026T164517-red-dumb-offer', '20261026T164556-red-dumb-escape', '20261026T164624-green-dumb-offer', '20261026T164703-clippy-dumb-frozen', '20261026T164703-fmt-dumb-frozen']:
    base = Path('/private/tmp/faction230-server-OWNED') / name
    a = json.loads((base / 'manifest.json').read_text())
    b = json.loads((base / 'manifest-after.json').read_text())
    checks[name] = {'drift': sorted(p for p in set(a) | set(b) if a.get(p) != b.get(p))}
    if 'green' in name or 'frozen' in name:
        checks[name]['review_source_mismatch'] = sorted(p for p in after if p in b and after[p] != b[p])
result = {'review_drift': [p for p in before if before[p] != after[p]], 'evidence': checks, 'reviewer_runtime_tests_executed': False}
(owned / 'classic-review-audit.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
