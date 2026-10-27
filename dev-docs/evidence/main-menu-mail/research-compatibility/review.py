"""Read-only review of the one-line research Action compatibility correction."""
from pathlib import Path
import hashlib
import json

root = Path('/Users/cam/DEV/faction-bbs')
proof = Path('/private/tmp/faction230-experiment-proof')
ui = root / 'experiments/055-shared-ui/ui.rs'
before = (proof / 'ui-before.rs').read_bytes()
after = (proof / 'ui-after.rs').read_bytes()
assert after == before.replace(b'            | Action::Contacts', b'            | Action::HomeMail\n            | Action::Contacts')
assert before.count(b'            | Action::Contacts') == 1
assert ui.read_bytes() == after
record = json.loads((proof / 'result.json').read_bytes())
assert record['result'] == 'pass' and record['red_exit'] == 101
assert hashlib.sha256(before).hexdigest() == record['before_sha256']
assert hashlib.sha256(after).hexdigest() == record['after_sha256']
red = (proof / 'red.log').read_bytes()
assert b'error[E0004]' in red and b'Action::HomeMail' in red and b'non-exhaustive' in red
assert len(record['checks']) == 3 and all(check['returncode'] == 0 for check in record['checks'])
assert b'Finished' in (proof / 'green0.log').read_bytes()
assert (proof / 'green1.log').read_bytes() == b''
assert b'1 passed; 0 failed;' in (proof / 'green2.log').read_bytes()
other = ['experiments/058-headroom/backend.rs', 'experiments/055-shared-board/main.rs', 'experiments/058-transport-memory/main.rs']
for name in other:
    text = (root / name).read_text()
    assert 'use faction_terminal::{Action,' in text
    assert '_ => {' in text and 'This action is outside the shared-board experiment.' in text
root_pin = hashlib.sha256((root / 'dev-docs/evidence/main-menu-mail/results.json').read_bytes()).hexdigest()
assert root_pin == 'a6de523049a995792a234ec142a4f9a9af0349b3a7a1fca4423d324dbc8f8536'
assert ui.read_bytes() == after
verdict = {'schema': 1, 'issue': 230, 'result': 'pass', 'remaining_actionable_findings': [], 'scope': 'one-line non-installed research adapter compatibility fallback', 'ui_before_sha256': record['before_sha256'], 'ui_after_sha256': record['after_sha256'], 'actual_red': 'E0004 missing HomeMail exhaustive match, exit101', 'actual_green': ['same strict all-target release Clippy command', 'format check', 'one existing test'], 'fallback': 'current approval check then existing Optional section unavailable message and Pause; no native mail handoff', 'other_experimental_action_consumers': 'three additional imports already use wildcard unavailable fallbacks', 'product_root_sha256_unchanged': root_pin, 'reviewer_builds_tests_or_fixtures': False}
Path('/private/tmp/faction230-experiment-readonly-review.json').write_text(json.dumps(verdict, sort_keys=True, indent=2) + '\n')
print(json.dumps(verdict, sort_keys=True, indent=2))
