#!/usr/bin/env python3
"""Verify saved native-application evidence and source provenance, without starting a VM."""
import hashlib
import json
from pathlib import Path
from evidence_sources import source_bytes

ROOT = Path(__file__).resolve().parents[1]
RESULT = ROOT / 'dev-docs/research/results/024-netbsd-application.json'

def check():
    data = json.loads(RESULT.read_text())
    assert data['schema'] == 1 and data['target'] == 'aarch64-unknown-netbsd'
    assert data['native_rust_tests'] == 58 and data['native_pty_workflows'] == 5
    assert len(data['ssh']['results']) == 20
    assert all(item['result'] == 'pass' for item in data['ssh']['results'])
    assert data['cleanup']['guest_stopped'] and data['cleanup']['private_inputs_removed']
    assert data['cmsg_layout']['native_payload_offset'] == 16
    assert data['cmsg_layout']['pinned_rust_payload_offset'] == 12
    observed = data['ssh']['observations']['rejected_handoff_descriptor_rows']
    assert observed['before'] >= 4 and observed['before'] == observed['after']
    for name, expected in data['sources'].items():
        assert hashlib.sha256(source_bytes(name,expected)).hexdigest() == expected, 'Native evidence source changed: ' + name
    assert data['sources'], 'native source inventory missing'
    print('Saved NetBSD app evidence passed: native tests, 5 PTY flows, 20 SSH oracles, cleanup and preserved executed source hashes. This does not rerun the VM.')

if __name__ == '__main__':
    check()
