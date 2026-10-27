#!/usr/bin/env python3
"""Verify saved editorial software evidence without a VM, SSH or SFTP."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,magazine_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/editorial-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==16
    assert d['host_rust_tests']==d['native_rust_tests']==336 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_workshop_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==301
    assert len(d['harness_sources'])==38 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
    m=magazine_baseline_manifest();assert len(m['files'])==304 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/magazine-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_magazine_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/magazine-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['unchanged_dependency_access_credential_sources'][n]==h
    for n in ('host_magazine','native_magazine'):
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and v['schema_version']==16
        assert not any(v[k] for k in ('capacity_claim','contributor_authentication','member_submission_queue','new_transfer_authority','public_publication'))
    for n in ('host_editorial','native_editorial'):
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==10 and v['schema_version']==16
        assert not any(v[k] for k in ('capacity_claim','new_transfer_authority','public_publication'))
    v=d['shared_editorial'];assert v['result']=='pass' and v['count']==len(v['checks'])==9 and v['schema_version']==16
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and 0<v['payload_bytes']<=32768
    assert not any(v[k] for k in ('capacity_claim','new_transfer_authority','public_publication'))
    for n in ('actual_approved_member_private_source_hidden_from_other_members',
              'actual_submit_select_preview_and_refuse_unapproved_publication',
              'actual_native_author_approval_and_editor_copy_change_invalidates_consent',
              'actual_plain_exact_consent_publish_credits_and_reading_leave_unread_unchanged',
              'actual_owner_export_related_file_handoff_keeps_native_recent_link_and_deferral',
              'actual_netbsd_compact_download_matches_exact_consented_edition_via_sftp',
              'actual_related_file_grants_rechecked_on_held_link_without_hiding_articles',
              'actual_author_revocation_withdraws_issue_existing_download_is_retained',
              'actual_account_revocation_expires_held_workshop_session'):assert n in v['checks']
    for n in ('host_backup','native_backup'):
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==19 and not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for n,count in [('native_requests',6),('native_library',8),('native_travel',6),('native_vault',13),('native_picker',8)]:
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==count
    for n,count in [('gallery',8),('discussion',8),('catchup',8),('polls',8),('gatherings',8),('pages',8),('wall',7),('full',7),('writing',13),('browser',10),('operator',8),('launcher',7),('directory',11),('nightwire',7)]:
        v=d['native_regressions'][n];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    assert d['native_art_previews']['profile_count']==18
    assert d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(retained_sources=128,active_sources_per_author=8,retained_issues=32,entries_per_issue=8,manuscript_bytes=4096,manuscript_lines=64,manuscript_line_bytes=120,issue_text_bytes=32768,normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,frame_bytes=8192,combined_manifest_bytes=32768,issues=8,articles_per_issue=8,total_articles=32,article_body_bytes=8192,article_lines=128,article_line_bytes=120,cards=4,text_rows=8,back_cursors=32,owner_edition_bytes=65536,new_registry_packages=0)
    assert d['features']==dict(local_account_contributor_approval=True,member_submission_queue=True,sysop_copied_proposals=True)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Editorial evidence passed: 336 Rust per OS, 10 editorial PTYs per OS, 9 actual SSH/native/SFTP controls and 19 recovery per OS.')
if __name__=='__main__':check()
