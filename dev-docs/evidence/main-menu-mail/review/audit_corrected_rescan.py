"""Independently rescan retained local dumps; print no private values or bytes."""
from pathlib import Path
import hashlib
import json
import sys

root = Path('/Users/cam/DEV/faction-bbs')
native = Path('/private/tmp/faction230-native-OWNED')
run = native / 'privacy-postrun-rescan02'
out = Path('/private/tmp/faction230-review-OWNED/corrected-rescan-audit')
out.mkdir(mode=0o700, exist_ok=False)
sys.path.insert(0, str(run / 'code'))
import probe_menu_mail as proof

def digest(data):
    return hashlib.sha256(data).hexdigest()

result_path = run / 'privacy-postrun-rescan.json'
raw = result_path.read_bytes()
saved = json.loads(raw)
private = json.loads((run / 'associations.private.json').read_bytes())
assert saved['result'] == 'pass'
assert saved['scope'] == 'postrun-retained-ktrace-hex-stream-reconstruction'
assert saved['fresh_ui_execution'] is False
assert saved['stream_key'] == ['pid', 'fd', 'direction']
assert digest((run / 'code/probe_menu_mail.py').read_bytes()) == saved['checker_sha256']
assert digest((root / 'tools/probe_menu_mail.py').read_bytes()) == saved['checker_sha256']
driver = Path('/private/tmp/faction230-integration-OWNED/retained_rescan02.py')
assert digest(driver.read_bytes()) == saved['driver_sha256']
original_driver = (native / '76_postrun_rescan.py').read_bytes()
assert driver.read_bytes() == original_driver.replace(b"run=P/'privacy-postrun-rescan'", b"run=P/'privacy-postrun-rescan02'")
sources_before = {}
for relative in ['tools/probe_menu_mail.py', 'tools/test_probe_menu_mail.py']:
    payload = (root / relative).read_bytes()
    sources_before[relative] = digest(payload)
    copy = out / relative
    copy.parent.mkdir(parents=True, exist_ok=True)
    copy.write_bytes(payload)
(out / 'source-before.json').write_text(json.dumps(sources_before, indent=2) + '\n')
verdict = {'schema': 1, 'issue': 230, 'result': 'pass', 'scope': 'independent-offline-audit-of-four-corrected-retained-trace-rescans', 'fresh_ui_execution': False, 'rescan_result_sha256': digest(raw), 'checker_sha256': saved['checker_sha256'], 'driver_sha256': saved['driver_sha256'], 'driver_difference': 'output directory only', 'routes': {}}
for platform in ['host', 'netbsd']:
    for mode in ['classic', 'unicode']:
        label = platform + '_' + mode
        row = saved['routes'][label]
        assert row['result'] == 'pass'
        file = run / (label + '.post-hex.private.log')
        payload = file.read_bytes()
        before = digest(payload)
        assert before == row['hex_dump_sha256']
        pub = native / 'public/results' / ('host-main-final02' if platform == 'host' else 'native-final03') / ((mode if platform == 'host' else 'main-' + mode) + '.json')
        assert digest(pub.read_bytes()) == row['original_route_result_sha256']
        actual_route = json.loads(pub.read_bytes())
        assert actual_route['platform'] == ('Darwin' if platform == 'host' else 'NetBSD')
        assert actual_route['unicode'] is (mode == 'unicode')
        association = private[label]
        assert association['result_sha256'] == row['original_route_result_sha256']
        assert association['original_dump_sha256'] == row['original_dump_sha256']
        assert 0 <= association['association_delta_ns'] < 1000000000
        assert association['association_delta_ns'] == row['association']['delta_ns']
        markers = sorted({value.encode() for value in association['markers'] if value})
        assert len(markers) == row['generated_marker_count'] == 11
        statistics = proof.require_board_bound_private_absent(payload, markers)
        assert all(type(value) is int and value > 0 for value in statistics.values())
        assert all(row[key] == value for key, value in statistics.items())
        assert statistics['decoded_bytes'] == statistics['read_bytes'] + statistics['write_bytes']
        streams, count = proof.reconstruct_kdump_streams(payload)
        assert all(prefix not in value for value in streams.values() for prefix in [b'FACTION1;', b'@FACTION1'])
        assert all(any(token in value for key, value in streams.items() if key[2] == 'read') for token in [b'@MAIL1 park ', b'@MAIL1 back '])
        assert row['input_trace_unchanged'] is True
        assert digest(file.read_bytes()) == before
        verdict['routes'][label] = {'result': 'pass', 'original_route_result_sha256': row['original_route_result_sha256'], 'hex_dump_sha256': before, 'generated_marker_count': len(markers), 'private_marker_matches': 0, 'file_protocol_prefix_matches': 0, 'park_and_back_present': True, 'offline_hex_input_unchanged': True, **statistics}
sources_after = {relative: digest((root / relative).read_bytes()) for relative in sources_before}
assert sources_before == sources_after
assert result_path.read_bytes() == raw
verdict['source_drift'] = []
verdict['association_limit'] = 'Capture-to-result association is a uniquely asserted sub-second mtime match on the same filesystem plus serialized driver order, explicitly recorded by the original postrun driver; it is not an original embedded route identifier.'
verdict['private_material_published'] = False
(out / 'source-after.json').write_text(json.dumps(sources_after, indent=2) + '\n')
(out / 'result.json').write_text(json.dumps(verdict, sort_keys=True, indent=2) + '\n')
print(json.dumps(verdict, sort_keys=True, indent=2))
