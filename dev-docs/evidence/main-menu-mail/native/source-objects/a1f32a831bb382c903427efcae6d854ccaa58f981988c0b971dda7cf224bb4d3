#!/usr/bin/env python3
"""Validate recorded schema-6 software evidence; this does not rerun a VM."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes


def check():
    d = json.loads((ROOT/'dev-docs/evidence/post-metadata-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass'
    assert d['host_rust_tests'] == d['native_rust_tests'] == 121
    assert d['host_python_tests'] == 104
    assert d['metadata_headers_observed'] is True
    assert d['fixture'] == {'vcpus': 2, 'ram_mib': 1024,
                            'admin_loopback_only': True, 'synthetic_board': True}
    assert d['shared_reader']['result'] == 'pass' and d['shared_reader']['count'] == 5
    assert len(set(d['shared_reader']['checks'])) == 5
    assert len(set(d['metadata_cases'])) == 7
    assert all(value is False for value in d['claims'].values())
    assert d['teardown'] == {'vm_stopped': True, 'private_clone_removed': True}
    assert len(d['execution_sources']) == 90
    assert {'crates/faction-core/src/post_metadata.sql',
            'crates/faction-core/src/post_metadata.rs',
            'crates/faction-core/tests/post_metadata.rs'} <= d['execution_sources'].keys()
    for name, digest in d['execution_sources'].items():
        assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest
    print('Metadata record passed: 121 host/native Rust tests, 5 shared controls; no hardening/wire/capacity claim.')


if __name__ == '__main__':
    check()
