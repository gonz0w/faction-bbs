#!/usr/bin/env python3
"""Verify saved editor/checkpoint software evidence; never start a VM or listener."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,writing_desk_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/full-screen-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==8
    assert d['host_rust_tests']==d['native_rust_tests']==213 and d['host_python_tests']==118
    assert d['configured_cores']==2 and d['configured_ram_bytes']==1073741824
    assert d['native_workspace_clean_before_build'] and d['native_service_checkpoint_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==184
    assert len(d['harness_sources'])==17 and len(d['unchanged_dependency_access_credential_sources'])==25
    for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=writing_desk_baseline_manifest();assert len(m['files'])==180 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/writing-desk-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_writing_desk_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/writing-desk-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['execution_sources'][n]==h
    for name in ['host_full_screen','native_full_screen']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and v['schema_version']==8 and not v['capacity_claim']
    v=d['shared_full_screen'];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==8
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and not v['capacity_claim']
    assert 'concurrent_checkpoint_conflict_preserves_newer_revision' in v['checks']
    assert 'checkpoint_busy_send_retry_is_single_and_clears_atomically' in v['checks']
    assert 'stale_reply_resume_refused_without_redirect_or_disclosure_and_delete_available' in v['checks']
    for name in ['host_backup','native_backup']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for name,count in [('host_legacy_unsaved_writing',13),('native_legacy_unsaved_writing',13),('host_browser',10),('native_browser',10),('native_directory',11),('native_launcher',7),('native_operator',8),('native_nightwire_logins',7),('shared_nightwire',7)]:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    v=d['shared_reader'];assert v['result']=='pass' and v['count']==len(v['checks'])==22 and not v['dedicated_ssh_admission_tested'] and not v['capacity_claim']
    v=d['native_nightwire_previews'];assert v['result']=='pass' and v['profile_count']==len(v['preview_profiles'])==18
    assert all(d[n] for n in ['host_export_restore','native_export_restore','host_and_native_local_reader_regression','host_and_native_local_console_regression'])
    for group in ['host_binaries_sha256','installed_native_binaries_sha256']:
        assert len(d[group])==4 and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,editor_frame_bytes=8192,draft_body_bytes=1024,draft_lines=32,saved_rooms_per_member=4,recovery_seconds=2592000,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Full-screen/checkpoint evidence passed: 213 Rust per OS, 7 editor PTYs per OS, 11 real SSH/native/plain controls, distinct-UID backup and reader/art regressions; no new capacity claim.')

if __name__=='__main__':check()
