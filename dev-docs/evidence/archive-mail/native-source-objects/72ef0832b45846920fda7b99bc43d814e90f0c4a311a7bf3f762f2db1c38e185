#!/usr/bin/env python3
"""Check saved writing-desk observations; no VM, listener or external service."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,room_browser_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/writing-desk-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass'
    assert d['host_rust_tests']==d['native_rust_tests']==199 and d['host_python_tests']==118
    assert d['configured_cores']==2 and d['configured_ram_bytes']==1073741824
    assert d['native_workspace_clean_before_build'] and d['native_service_writing_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==174
    assert len(d['harness_sources'])==11 and len(d['unchanged_core_build_frontend_sources'])==32
    for group in ['execution_sources','harness_sources','unchanged_core_build_frontend_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    assert hashlib.sha256((ROOT/'dev-docs/evidence/room-browser-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_room_browser_manifest_sha256']
    m=room_browser_baseline_manifest();assert len(m['files'])==174 and len(m['records'])==1
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/room-browser-netbsd/results.json').read_text())
    assert d['unchanged_core_build_frontend_sources']==old['unchanged_core_build_frontend_sources']
    for key in ['host_writing','native_writing']:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==13
        assert not v['capacity_claim'] and not v['persistent_drafts'] and not v['schema_change']
    v=d['shared_writing'];assert v['result']=='pass' and v['count']==len(v['checks'])==18
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert not v['capacity_claim'] and not v['persistent_drafts'] and not v['schema_change']
    assert 'database_busy_retry_keeps_one_explicit_submission' in v['checks']
    assert 'revocation_prevents_sending_an_existing_draft' in v['checks']
    assert sum(n.startswith('actual_shared_plain_') for n in v['checks'])==5
    v=d['shared_reader'];assert v['result']=='pass' and v['count']==len(v['checks'])==22 and not v['dedicated_ssh_admission_tested'] and not v['capacity_claim']
    for key,count in [('host_browser',10),('native_browser',10),('native_directory',11),('native_launcher',7),('native_operator',8),('native_nightwire_logins',7),('shared_nightwire',7)]:
        v=d[key];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    assert d['host_and_native_local_reader_regression'] and d['host_and_native_local_console_regression']
    assert all(len(h)==64 and set(h)<=set('0123456789abcdef') for group in ['installed_native_binaries_sha256','host_binaries_sha256'] for h in d[group].values())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,draft_body_bytes=1024,draft_lines=32,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Writing-desk evidence passed: 199 Rust per OS, 13 PTY controls per OS, 18 real SSH/native/plain and 22 reader controls; no saved drafts or capacity claim.')


if __name__=='__main__':check()
