#!/usr/bin/env python3
"""Check saved discussion discovery evidence without starting a VM or service."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, catchup_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/discussion-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==14
    assert d['host_rust_tests']==d['native_rust_tests']==292 and d['host_python_tests']==129
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_bookmark_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==250
    assert len(d['harness_sources'])==23 and len(d['unchanged_dependency_access_credential_sources'])==15
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for name,digest in d[group].items():
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=catchup_baseline_manifest()
    assert len(m['files'])==248 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/catchup-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_catchup_manifest_sha256']
    for name,digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/catchup-netbsd/results.json').read_text())
    for name,digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['execution_sources'][name]==digest
    for name in ('host_discussion','native_discussion'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==14
        assert not v['capacity_claim'] and not v['automatic_query_retention'] and not v['server_mail_search']
    v=d['shared_discussion']
    assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==14
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert not v['capacity_claim'] and not v['automatic_query_retention'] and not v['server_mail_search']
    for name in ('actual_two_members_cannot_list_or_overwrite_each_others_bookmarks',
                 'actual_busy_review_stays_readable_and_explicit_retry_saves_once',
                 'actual_resize_preserves_completed_query_and_readonly_source_cursor',
                 'actual_file_grant_revocation_filters_query_snippets_bookmarks_and_live_source',
                 'actual_deleted_source_is_not_replaced_by_a_successor_and_cascades_bookmarks',
                 'actual_revocation_blocks_bookmark_while_normal_complete_room_read_still_acks',
                 'actual_disabled_host_feature_hides_held_query_and_source_results'):
        assert name in v['checks']
    for name in ('host_backup','native_backup'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==17
        assert 'complete_restore_preserves_discussion_policy_and_own_bookmarks' in v['checks']
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
    for name,count in [('host_catchup',8),('native_catchup',8),('host_polls',8),('native_polls',8),('native_gatherings',8),('native_pages',8),
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
        owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,discussion_frame_bytes=8192,
        bookmarks=128,candidate_ids=101,inspected_posts=100,hits=4,query_bytes=64,
        snippet_scalars=120,back_cursors=32,cooperative_iteration_ms=20,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Discussion evidence passed: 292 Rust per OS, 8 Find PTYs per OS, 11 actual SSH/native controls and 17 recovery controls per OS; no capacity or server-mail-search claim.')


if __name__=='__main__':
    check()
