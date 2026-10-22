#!/usr/bin/env python3
"""Verify the local mail record and actual source pins; does not rerun NetBSD."""
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]


def check():
    data=json.loads((ROOT/'dev-docs/evidence/local-mail-vault-netbsd/results.json').read_text())
    assert data['result']=='pass' and data['date']=='2026-10-21'
    assert data['host_rust_tests']==data['native_rust_tests']==105
    assert data['python_tests']==104 and data['native_build_input_hashes_matched'] is True
    assert data['host']['platform']=='Darwin' and data['native']['platform']=='NetBSD'
    assert data['host']['checks']==data['native']['checks'] and len(set(data['host']['checks']))==13
    for platform in ['host','native']:
        p=data[platform];assert p['result']=='pass' and p['count']==13
        assert p['private_fixture_removed_on_exit'] is True
        for key in ['network_mail_implemented','roaming_authentication_implemented','encrypted_attachments_implemented']:
            assert p[key] is False
    for key in ['physical_hardware','capacity_claim','owner_limits_changed','credential_limits_changed',
                'network_mail_implemented','omemo_private_device_state_implemented','roaming_authentication_implemented','encrypted_attachments_implemented']:
        assert data[key] is False
    assert data['default_sessions']==8
    assert data['fixture']=={'guest':'NetBSD 11.0 ARM64','vcpus':2,'guest_ram_mib':1024,
                            'loopback_only_host_forwards':True,'native_build_and_probe_unprivileged':True}
    assert data['teardown']=={'vm_process_stopped':True,'private_inputs_removed':True}
    assert len(data['execution_sources'])==93
    for name,digest in data['execution_sources'].items():
        path=Path(name);assert not path.is_absolute() and '..' not in path.parts
        assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,'local mail executed source changed: '+name
    print('Native local-mail evidence passed: 105 host/native Rust tests, 13 PTY checks each; no network mail, roaming or capacity claim.')


if __name__=='__main__':check()
