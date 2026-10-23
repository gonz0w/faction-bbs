#!/usr/bin/env python3
"""Check recorded software results and exact source identity, not rerun a VM."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes


def check():
    d = json.loads((ROOT/'dev-docs/evidence/room-reader-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass'
    assert d['host_rust_tests'] == d['native_rust_tests'] == 114
    assert d['host_python_tests'] == 104
    assert d['fixture'] == {'vcpus': 2, 'ram_mib': 1024, 'admin_loopback_only': True,
                            'synthetic_board': True, 'original_playground_stopped': True}
    shared = d['shared_reader']
    assert shared['result'] == 'pass' and shared['count'] == 5
    assert len(set(shared['checks'])) == shared['count']
    assert not shared['dedicated_ssh_admission_tested'] and not shared['capacity_claim']
    assert all(value is False for value in d['claims'].values())
    assert d['teardown'] == {'vm_stopped': True, 'private_clone_removed': True}
    assert len(d['execution_sources']) == 87
    assert {'Cargo.toml', 'crates/faction-articles/src/lib.rs',
            'crates/faction-terminal/src/reader.rs', 'crates/faction-service/src/ui.rs',
            'tools/smoke_shared_room_reader.py'} <= d['execution_sources'].keys()
    for name, digest in d['execution_sources'].items():
        assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest
    print('Room/article record passed: 114 host/native Rust tests and 5 native shared controls; no new capacity or wire claim.')


if __name__ == '__main__':
    check()
