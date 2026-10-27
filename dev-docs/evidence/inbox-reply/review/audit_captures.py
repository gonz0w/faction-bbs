import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tarfile

sys.dont_write_bytecode = True
root = Path('/private/tmp/faction233-native-OWNED')
sys.path.insert(0, str(root / 'runtime-code'))
sys.path.insert(0, str(root))
from audit_main_capture import audit
from probe_mail_reply import require_reply_commit, require_reply_private_absent
from probe_menu_mail import reconstruct_kdump_streams

def digest(data):
    return hashlib.sha256(data).hexdigest()

source = root / 'runtime-code/probe_mail_reply.py'
assert digest(source.read_bytes()) == '7e23ce635b74a7d5ac7c4383edfed7a58ed1d35f0d470a8a4f52a8cfbb0e4854'
public = json.loads((root / 'public-staging/logs/capture-associations.json').read_bytes())['routes']
for mode in ('classic', 'unicode'):
    result_bytes = (root / f'logs/host-main-{mode}-final.json').read_bytes()
    _, value = audit(root, result_bytes, source)
    assert value == public['host_' + mode]
    print('host', mode, 'retained original capture independently revalidated')

with tarfile.open(root / 'private/native-main-captures.private.tar.gz', 'r:gz') as archive:
    for mode in ('classic', 'unicode'):
        key = 'netbsd_' + mode
        def data(name):
            member = archive.getmember(key + '/' + name)
            assert member.isfile()
            return archive.extractfile(member).read()
        result_bytes = (root / f'public-staging/logs/native-runtime-main-{mode}.json').read_bytes()
        result = json.loads(result_bytes)
        markers_raw = data('private-markers.json')
        markers = json.loads(markers_raw)
        trace = data('board-io.private.log')
        scan = data('board-file-scan-input.private.json')
        association = json.loads(data('capture-result-association.private.json'))
        pre = json.loads(data('private-input-association.json'))
        start = json.loads(data('trace-input-association.private.json'))
        hashes = dict(trace_sha256=digest(trace), marker_set_sha256=digest(markers_raw), file_scan_input_sha256=digest(scan))
        assert hashes == result['private_input_hashes'] == public[key]['private_input_hashes']
        assert all(association[k] == v for k, v in hashes.items())
        assert association['result_sha256'] == digest(result_bytes) == public[key]['result_sha256']
        assert all(x['capture_id'] == result['capture_id'] == public[key]['capture_id'] for x in (association, pre, start))
        assert all(x['source_sha256'] == digest(source.read_bytes()) for x in (association, pre, start, result))
        assert pre['generated_before_capture'] is True
        assert pre['marker_set_sha256'] == start['marker_set_sha256'] == start['marker_file_sha256_at_trace_start'] == hashes['marker_set_sha256']
        assert json.loads(scan)['markers'] == markers and len(markers) == result['marker_count'] == 15
        stats = require_reply_private_absent(trace, [m.encode() for m in markers])
        assert stats == result['board_io_reconstruction']
        streams, _ = reconstruct_kdump_streams(trace)
        assert all(b'FACTION1;' not in s and b'@FACTION1' not in s for s in streams.values())
        require_reply_commit(json.loads(data('canonical-before.private.json')), json.loads(data('canonical-after.private.json')), 'bob@away.test', 202, pre['replacement'], pre['body'])
        print('NetBSD', mode, 'retained capture independently revalidated')
print('All four captures: generated marker/source/result/input associations, canonical invariants and strict private byte absence pass')
