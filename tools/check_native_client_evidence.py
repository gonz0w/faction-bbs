#!/usr/bin/env python3
"""Verify recorded native-client evidence and source identity; does not rerun a VM."""
import hashlib
from evidence_sources import source_bytes
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def check():
    data = json.loads((ROOT/'dev-docs/evidence/native-client-netbsd/results.json').read_text())
    assert data['result'] == 'pass' and data['count'] == 10
    assert len(set(data['checks'])) == data['count']
    assert data['guest_native_client'] and data['wrong_host_key_test']
    assert data['host_rust_tests'] == data['native_rust_tests'] == 100
    assert data['host_python_tests'] == 101
    assert data['default_sessions'] == 8
    for key in ['physical_hardware', 'capacity_claim', 'owner_limits_changed',
                'credential_limits_changed', 'mail_identity_implemented',
                'encrypted_attachments_implemented']:
        assert data[key] is False
    assert data['fixture']['vcpus'] == 2 and data['fixture']['guest_ram_mib'] == 1024
    assert data['fixture']['loopback_only_host_forwards'] is True
    assert data['payload_bytes'] == 131328
    assert data['payload_sha256'] == hashlib.sha256(bytes(range(256))*513).hexdigest()
    assert data['teardown'] == {'services_stopped': True, 'vm_process_stopped': True, 'private_inputs_removed': True}
    assert len(data['execution_sources']) == 89
    for name, expected in data['execution_sources'].items():
        path = Path(name)
        assert not path.is_absolute() and '..' not in path.parts
        assert hashlib.sha256(source_bytes(name, expected)).hexdigest() == expected, 'native-client executed source changed: '+name
    print('Native-client evidence passed: 10 real client/SSH/SFTP/recovery checks, 100 native Rust tests; no mail, hardware or capacity claim.')


if __name__ == '__main__':
    check()
