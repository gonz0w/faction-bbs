#!/usr/bin/env python3
"""Verify saved member-page software evidence; never run a service or VM."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,community_wall_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/member-pages-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==10
    assert d['host_rust_tests']==d['native_rust_tests']==235 and d['host_python_tests']==118
    assert d['configured_cores']==2 and d['configured_ram_bytes']==1073741824
    assert d['native_workspace_clean_before_build'] and d['native_service_pages_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==203
    assert len(d['harness_sources'])==22
    for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=community_wall_baseline_manifest();assert len(m['files'])==202 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/community-wall-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_community_wall_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/community-wall-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['execution_sources'][n]==h
    for name in ['host_pages','native_pages']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==10 and not v['capacity_claim']
    v=d['shared_pages'];assert v['result']=='pass' and v['count']==len(v['checks'])==14 and v['schema_version']==10
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and not v['capacity_claim']
    for name in ['actual_other_member_cannot_view_hidden_fields_or_hidden_plan','actual_concurrent_page_edits_refuse_stale_revision','actual_busy_plan_retry_is_single_and_history_prunes_to_policy','actual_active_revocation_hides_author_and_prevents_pending_save','actual_own_content_delete_recreate_cannot_clear_sysop_hide','actual_compact_layout_confirmed_page_history_removal_and_disable']:
        assert name in v['checks']
    for name in ['host_wall','native_wall']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and v['schema_version']==10 and not v['capacity_claim']
    v=d['shared_wall'];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==10
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and not v['capacity_claim']
    for name in ['actual_busy_send_retains_request_for_exactly_one_retry','actual_weekly_archive_readonly_edit_and_explicit_delete','actual_pending_denial_and_active_revocation_prevent_wall_send','actual_escape_injection_cannot_publish_or_execute_terminal_controls']:
        assert name in v['checks']
    for name in ['host_full_screen','native_full_screen']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and v['schema_version']==10 and not v['capacity_claim']
    v=d['shared_full_screen'];assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==10
    for name in ['host_backup','native_backup']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==13 and not v['capacity_claim'] and not v['hardening_claim']
        assert 'complete_restore_preserves_wall_policy_note_and_cooldown_receipt' in v['checks']
        assert 'complete_restore_preserves_member_page_and_hidden_plan' in v['checks']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for name,count in [('host_legacy_unsaved_writing',13),('native_legacy_unsaved_writing',13),('host_browser',10),('native_browser',10),('native_directory',11),('native_launcher',7),('native_operator',8),('native_nightwire_logins',7),('shared_nightwire',7)]:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    v=d['shared_reader'];assert v['result']=='pass' and v['count']==len(v['checks'])==22 and not v['dedicated_ssh_admission_tested'] and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count']==18
    assert all(d[n] for n in ['host_export_restore','native_export_restore','host_and_native_local_reader_regression','host_and_native_local_console_regression','host_and_native_custom_menus_regression'])
    for group in ['host_binaries_sha256','installed_native_binaries_sha256']:
        assert len(d[group])==4 and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,page_frame_bytes=8192,field_bytes=[240,120,240,120],plan_bytes=480,plan_lines=8,line_columns=72,plan_history=16,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Member-page evidence passed: 235 Rust per OS, 8 member-page PTYs per OS, 14 actual SSH/native/plain controls, complete member-page/wall/file recovery and room/art regressions; no capacity claim.')
if __name__=='__main__':check()
