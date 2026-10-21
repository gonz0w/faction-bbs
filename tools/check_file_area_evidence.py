#!/usr/bin/env python3
"""Check recorded native file-area observations; does not rerun SSH or the VM."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def check():
    data = json.loads((ROOT/'dev-docs/evidence/file-areas-netbsd/results.json').read_text())
    assert data['result'] == 'pass' and data['count'] == 13
    assert len(set(data['checks'])) == data['count']
    assert data['native_rust_tests'] == 90 and data['host_rust_tests'] == 90
    assert data['host_python_tests'] == 101
    assert data['physical_hardware'] is False and data['capacity_claim'] is False
    assert data['footprint_is_peak'] is False
    assert data['fixture']['guest_ram_mib'] == 1024 and data['fixture']['vcpus'] == 2
    assert data['fixture']['payload_bytes'] == 131328
    assert data['fixture']['loopback_only_host_forwards'] is True
    assert data['teardown'] == {'services_stopped': True, 'vm_process_stopped': True, 'private_inputs_removed': True}
    for name, expected in data['execution_sources'].items():
        path = Path(name)
        assert not path.is_absolute() and '..' not in path.parts
        assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest() == expected, 'file-area executed source changed: '+name
    print('Native file-area evidence passed: 13 real SSH/SFTP/recovery checks, 90 native Rust tests; RSS observation only, no capacity/physical claim.')


if __name__ == '__main__':
    check()
