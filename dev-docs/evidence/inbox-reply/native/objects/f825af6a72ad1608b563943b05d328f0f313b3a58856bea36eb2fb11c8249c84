#!/usr/bin/env python3
"""Check saved library shelves evidence; never starts a VM, SSH or SFTP."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,travel_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/library-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==14
    assert d['host_rust_tests']==d['native_rust_tests']==313 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_library_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==282
    assert len(d['harness_sources'])==31 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=travel_baseline_manifest();assert len(m['files'])==282 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/travel-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_travel_manifest_sha256']
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/travel-netbsd/results.json').read_text())
    for name,digest in d['unchanged_dependency_access_credential_sources'].items():assert old['execution_sources'][name]==digest
    for name in ('host_library','native_library'):
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==14
        assert not any(v[k] for k in ('capacity_claim','request_desk','new_transfer_authority','public_library'))
    v=d['shared_library'];assert v['result']=='pass' and v['count']==len(v['checks'])==10 and v['schema_version']==14
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and v['payload_bytes']==24
    assert not any(v[k] for k in ('capacity_claim','request_desk','new_transfer_authority','public_library'))
    for name in ('actual_member_shelves_hide_private_pending_removed_and_foreign_metadata_before_paging',
                 'actual_host_native_library_to_Files_keeps_recent_link_and_zero_budget_deferral',
                 'actual_netbsd_compact_library_recent_link_download_uses_existing_restricted_sftp',
                 'actual_live_grant_revocation_hides_selected_notes_and_private_shelf_names',
                 'actual_publication_withdrawal_returns_to_catalog_without_opening_a_successor',
                 'actual_discussion_jump_uses_existing_reader_and_only_that_read_acknowledges',
                 'actual_deleted_discussion_and_member_revocation_recheck_current_authority'):assert name in v['checks']
    for name in ('host_backup','native_backup'):
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==17
        assert not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for name in ('host_vault','native_vault'):
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==13
        assert not v['network_mail_implemented'] and not v['roaming_authentication_implemented']
    for name in ('host_picker','native_picker'):
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8
        assert not v['network_mail_implemented'] and not v['remote_unlock_authority']
    for name,count in [('host_travel',6),('native_travel',6),('host_gallery',8),('native_gallery',8),('native_discussion',8),('native_catchup',8),('native_polls',8),('native_gatherings',8),('native_pages',8),('native_wall',7),('native_full_screen',7),('native_writing',13),('native_browser',10),('native_operator',8),('native_launcher',7),('native_directory',11),('native_nightwire_logins',7)]:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count']==18
    assert d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,library_frame_bytes=8192,manifest_bytes=32768,books=32,shelves=8,cards=4,text_rows=8,back_cursors=32,note_bytes=240,rights_bytes=96,provenance_bytes=160,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Library evidence passed: 313 Rust per OS, 8 local/worker library PTYs per OS, 10 actual SSH/native/SFTP controls and 17 recovery controls per OS; no request-desk/capacity claim.')
if __name__=='__main__':check()
