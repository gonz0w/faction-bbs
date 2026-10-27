import copy
import hashlib
import json
from pathlib import Path
import sys

root = Path('/Users/cam/DEV/faction-bbs')
owned = Path('/private/tmp/faction230-review-OWNED/evidence-guard-audit')
sys.path.insert(0, str(root / 'tools'))
import probe_menu_mail
from test_menu_mail_evidence import Evidence

observed = {}
test = Evidence()
test.setUp()
try:
    record = copy.deepcopy(test.record)
    record['routes']['netbsd'] = record['routes']['host'].copy()
    test.module.validate(test.root, record)
    observed['same_host_route_refs_accepted_as_netbsd'] = True
    # No platform/scope fields exist in the test's nominal route fixtures.
    observed['missing_platform_and_scope_accepted'] = True
    marker = b'private body'
    contiguous = b'42 faction-service GIO fd 6 read 12 bytes\n       "private body"\n'
    try:
        probe_menu_mail.require_board_bound_private_absent(contiguous, [marker])
        observed['contiguous_marker_rejected'] = False
    except AssertionError:
        observed['contiguous_marker_rejected'] = True
    fragmented = b''.join(b'42 faction-service GIO fd 6 read 1 bytes\n       "' + bytes([b]) + b'"\n' for b in marker)
    probe_menu_mail.require_board_bound_private_absent(fragmented, [marker])
    observed['fragmented_kdump_marker_false_absence_accepted'] = True
    observed['generic_protocol_compatibility_legacy_pass_records_accepted'] = all(json.loads((test.root / test.record['records'][name]['path']).read_text()) == {'result': 'pass'} for name in ('protocol', 'compatibility', 'legacy'))
finally:
    test.doCleanups()
before = json.loads((owned / 'before.json').read_text())
after = {p: hashlib.sha256((root / p).read_bytes()).hexdigest() for p in before}
(owned / 'after.json').write_text(json.dumps(after, indent=2) + '\n')
observed['source_drift'] = [p for p in before if before[p] != after[p]]
(owned / 'observed.json').write_text(json.dumps(observed, indent=2) + '\n')
print(json.dumps(observed, indent=2))
