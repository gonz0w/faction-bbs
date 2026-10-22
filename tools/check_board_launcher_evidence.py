#!/usr/bin/env python3
"""Check saved native board-launcher observations without starting network/VMs."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,directory_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/board-launcher-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==191 and d['host_python_tests']==118
    assert d['configured_vm_count']==2 and d['configured_cores_per_vm']==2 and d['configured_ram_bytes_per_vm']==1073741824
    assert len(d['guests'])==2 and all('NetBSD' in v for v in d['guests'])
    assert d['selected_native_source_matches']==len(d['execution_sources'])==168 and len(d['harness_sources'])==8
    assert len(d['unchanged_server_and_workspace_inputs'])==110
    for group in ['execution_sources','harness_sources','unchanged_server_and_workspace_inputs']:
        for name,digest in d[group].items():
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    assert d['installed_first_board_binaries_sha256']==d['installed_second_board_binaries_sha256']
    assert set(d['installed_first_board_binaries_sha256'])=={'faction','faction-service','faction-tty','faction-daemon'}
    assert hashlib.sha256((ROOT/'dev-docs/evidence/directory-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_directory_manifest_sha256']
    m=directory_baseline_manifest();assert len(m['files'])==174 and len(m['records'])==1
    for name,digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/directory-netbsd/results.json').read_text())
    for name,digest in d['unchanged_server_and_workspace_inputs'].items():
        assert old['execution_sources'][name]==digest
    for key in ['host_launcher','native_launcher']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==7
        assert not any(v[k] for k in ['capacity_claim','remote_profile_import','federation_claim'])
    v=d['two_board_visits'];assert v['result']=='pass' and v['count']==len(v['checks'])==10
    assert v['normal_sessions_per_board']==8 and v['admission_refill_ms']==5000
    assert not any(v[k] for k in ['capacity_claim','remote_profile_import','federation_claim'])
    assert v['observations']==dict(host_active_ssh_children=1,guest_active_ssh_children=1,board_owner_lwps=[4,4])
    for key in ['host_directory','native_directory']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and not v['capacity_claim']
    v=d['native_operator_ui'];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and not v['capacity_claim']
    for key in ['shared_nightwire','native_nightwire_logins']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    v=d['native_graphics'];assert v['result']=='pass' and v['count']==len(v['checks'])==d['host_graphics_controls']==3
    assert d['budgets']==dict(catalog_file_bytes=16384,catalog_boards=16,page_entries=4,profile_file_bytes=8192,profile_path_bytes=512,active_client_ssh_children=1,escape_timeout_seconds=1,normal_sessions_per_board=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,owner_lwps=4,shared_output_bytes=32768,drain_seconds=5,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Board-launcher evidence passed: 191 Rust per OS, 7 local controls per OS, 10 actual two-board controls; server inputs unchanged, no new capacity claim.')


if __name__=='__main__':check()
