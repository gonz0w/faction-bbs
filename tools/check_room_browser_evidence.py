#!/usr/bin/env python3
"""Check saved named-room software observations; never start a VM or listener."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,board_launcher_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/room-browser-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==194 and d['host_python_tests']==118
    assert d['configured_cores']==2 and d['configured_ram_bytes']==1073741824
    assert d['native_workspace_clean_before_build'] and d['native_service_destination_label_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==169 and len(d['harness_sources'])==9
    assert len(d['unchanged_core_build_frontend_sources'])==32
    for group in ['execution_sources','harness_sources','unchanged_core_build_frontend_sources']:
        for name,digest in d[group].items():
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    assert hashlib.sha256((ROOT/'dev-docs/evidence/board-launcher-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_board_launcher_manifest_sha256']
    m=board_launcher_baseline_manifest();assert len(m['files'])==178 and len(m['records'])==1
    for name,digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/board-launcher-netbsd/results.json').read_text())
    for name,digest in d['unchanged_core_build_frontend_sources'].items():assert old['execution_sources'][name]==digest
    for key in ['host_room_browser','native_room_browser']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==10
        assert not v['capacity_claim'] and not v['federation_claim']
    v=d['shared_room_browser'];assert v['result']=='pass' and v['count']==len(v['checks'])==11
    assert v['room_limit']==100 and v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert not v['capacity_claim'] and not v['federation_claim']
    samples=v['observations']['hundred_room_samples'];assert len(samples)==8
    assert all(s['owner_lwps']==4 and s['owner_rss_kib']==5248 for s in samples)
    assert min(s['vm_free_bytes'] for s in samples)>363*1048576
    v=d['shared_reader'];assert v['result']=='pass' and v['count']==len(v['checks'])==22 and not v['capacity_claim']
    assert not v['dedicated_ssh_admission_tested']
    for key in ['host_directory','native_directory']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and not v['capacity_claim']
    for key in ['native_nightwire_logins','shared_nightwire','native_launcher']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and not v['capacity_claim']
    v=d['native_operator_ui'];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    assert d['host_and_native_local_reader_regression'] and d['host_and_native_local_console_regression']
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,owner_lwps=4,shared_output_bytes=32768,drain_seconds=5,browser_frame_bytes=8192,room_list_limit=100,page_rooms=4,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Room-browser evidence passed: 194 Rust per OS, 10 PTY controls per OS, 11 real SSH/native and 22 reader controls; no new capacity claim.')


if __name__=='__main__':check()
