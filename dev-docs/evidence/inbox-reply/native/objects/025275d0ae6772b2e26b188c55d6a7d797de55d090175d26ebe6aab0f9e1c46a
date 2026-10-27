#!/usr/bin/env python3
"""Check saved catch-up evidence without starting a VM or service."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, polls_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/catchup-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==13
    assert d['host_rust_tests']==d['native_rust_tests']==280 and d['host_python_tests']==129
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_visit_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==240
    assert len(d['harness_sources'])==21 and len(d['unchanged_dependency_access_credential_sources'])==15
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for name,digest in d[group].items():
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=polls_baseline_manifest()
    assert len(m['files'])==238 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/polls-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_polls_manifest_sha256']
    for name,digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/polls-netbsd/results.json').read_text())
    for name,digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['execution_sources'][name]==digest
    for name in ('host_catchup','native_catchup'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==13
        assert not v['capacity_claim'] and not v['automatic_login_tracking'] and not v['server_mail_decryption']
    v=d['shared_catchup']
    assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==13
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert not v['capacity_claim'] and not v['automatic_login_tracking'] and not v['server_mail_decryption'] and not v['fresh_sftp_byte_transfer_claim']
    for name in ('actual_two_members_direct_replies_and_independent_favorites',
                 'actual_source_jump_partial_read_preserves_and_complete_drain_acknowledges',
                 'actual_concurrent_preferences_refuse_stale_checkpoint',
                 'actual_busy_checkpoint_retry_and_resize_keep_explicit_intent',
                 'actual_gathering_jump_uses_existing_current_event_detail',
                 'actual_catalog_preview_and_file_jump_recheck_revoked_private_grant',
                 'actual_public_file_jump_emits_existing_reference_and_revocation_blocks_checkpoint'):
        assert name in v['checks']
    for name in ('host_backup','native_backup'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==16
        assert 'complete_restore_preserves_catchup_policy_preferences_checkpoint_and_publication_order' in v['checks']
        assert not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for name in ('host_vault','native_vault'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==13
        assert not v['network_mail_implemented'] and not v['roaming_authentication_implemented']
    for name in ('host_picker','native_picker'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==8
        assert not v['network_mail_implemented'] and not v['remote_unlock_authority']
    for name,count in [('host_polls',8),('native_polls',8),('native_gatherings',8),('native_pages',8),
                       ('native_wall',7),('native_full_screen',7),('native_writing',13),('native_browser',10),
                       ('native_operator',8),('native_launcher',7),('native_directory',11),('native_nightwire_logins',7)]:
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count']==18
    assert d['native_custom_menu_reader_and_console_regressions']
    assert d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count
        assert all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,
        owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,catchup_frame_bytes=8192,
        favorites=16,room_page=4,room_lookahead=1,unread_scan_per_room=101,reply_window_per_room=100,
        file_page=2,event_page=4,back_cursors=32,notice_bytes=240,notice_lines=4,line_columns=72,
        new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Catch-up evidence passed: 280 Rust per OS, 8 catch-up PTYs per OS, 11 actual SSH/native controls and 16 recovery controls per OS; no capacity or server-mail claim.')


if __name__=='__main__':
    check()
