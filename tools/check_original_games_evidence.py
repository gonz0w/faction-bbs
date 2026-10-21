#!/usr/bin/env python3
"""Check saved original-game evidence/provenance; does not run a game or VM."""
import hashlib
import json
import re
from pathlib import Path
from evidence_sources import source_bytes

ROOT = Path(__file__).resolve().parents[1]


def check():
    frozen = ROOT / 'dev-docs/evidence/classic-doors-netbsd/pr80'
    raw = (frozen / 'sources.json').read_bytes()
    # Pinned independently of the mutable current source/result maps.
    assert hashlib.sha256(raw).hexdigest() == 'f3df23f4f706e16b6fbc9ff69a8d773a5a959225528d391a45b63442bcec40e5'
    manifest = json.loads(raw)
    old_raw = (ROOT / manifest['historical_record']).read_bytes()
    assert hashlib.sha256(old_raw).hexdigest() == manifest['record_sha256']
    old = json.loads(old_raw)
    for name, digest in old['execution_sources'].items():
        if name in manifest['preserved_changed_inputs']:
            assert manifest['preserved_changed_inputs'][name] == digest
            measured = (frozen / name).read_bytes()
        else:
            measured = source_bytes(name, digest)
        assert hashlib.sha256(measured).hexdigest() == digest

    d = json.loads((ROOT / 'dev-docs/validation/netbsd-original-lord.json').read_text())
    assert d['schema'] == 1 and d['system'] == 'NetBSD 11.0'
    assert d['processor'] == 'aarch64' and d['native_workspace_tests'] == 78
    assert d['simultaneous_original_games'] == 1
    assert not d['dos_record_locking_supported']
    assert not d['original_game_modified'] and not d['bbs_limits_or_defaults_relaxed']
    assert not d['game_state_in_bbs_export']
    assert d['twgs_activation'] == 'owner-deferred; no submission or gameplay'
    for name, digest in d['execution_sources'].items():
        path = Path(name)
        assert not path.is_absolute() and '..' not in path.parts
        assert re.fullmatch('[0-9a-f]{64}', digest)
        assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, name
    assert all(d['execution_sources'][name] == digest for name, digest in d['native_product_sources'].items())
    ssh = d['ssh']
    assert ssh['result'] == 'pass' and len(ssh['checks']) == 12
    assert len(set(ssh['checks'])) == 12
    assert ssh['game_host']['runs'] == ssh['game_host']['reaped'] == 7
    assert ssh['game_host']['denied'] == ssh['game_host']['busy'] == 1
    assert d['private_game_backup']['restored_bytes_match']
    assert all(d['cleanup'].values())
    for binaries in d['installed_binaries'].values():
        assert all(re.fullmatch('[0-9a-f]{64}', digest) for digest in binaries.values())
    print('Original LORD evidence passed: unchanged historical parser/record, current native inputs, actual-title SSH play/save/recovery and teardown. Not a VM rerun or capacity claim.')


if __name__ == '__main__':
    check()
