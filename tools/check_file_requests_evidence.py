#!/usr/bin/env python3
"""Verify saved request-desk evidence without starting a VM or network service."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,library_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/file-requests-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==15
    assert d['host_rust_tests']==d['native_rust_tests']==321 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_request_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==288
    assert len(d['harness_sources'])==33 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
    m=library_baseline_manifest();assert len(m['files'])==292 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/library-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_library_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/library-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['unchanged_dependency_access_credential_sources'][n]==h
    for n in ('host_requests','native_requests'):
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==6 and v['schema_version']==15
        assert not any(v[k] for k in ('capacity_claim','new_transfer_authority','public_library'))
    v=d['shared_requests'];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==15
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and v['payload_bytes']==34
    for n in ('actual_sysop_fulfillment_rejects_requester_private_access_then_confirms_published_file',
              'actual_native_request_to_Files_recent_link_respects_zero_payload_ceiling',
              'actual_netbsd_compact_request_fulfillment_download_uses_restricted_sftp_and_exact_bytes',
              'actual_plain_manual_ssh_request_details_and_links_are_escape_free',
              'actual_live_area_revocation_removes_held_fulfillment_link_and_provenance',
              'actual_concurrent_close_rejects_stale_review_without_replacing_revision',
              'actual_sysop_explicit_delete_and_live_member_revocation_keep_current_authority'):assert n in v['checks']
    for n in ('host_backup','native_backup'):
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==18
        assert 'complete_restore_preserves_file_request_fulfillment_and_provenance' in v['checks']
        assert not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for n,count in [('host_library',8),('native_library',8),('host_travel',6),('native_travel',6),('host_vault',13),('native_vault',13),('host_picker',8),('native_picker',8)]:
        v=d[n];assert v['result']=='pass' and v['count']==len(v['checks'])==count
    for n,count in [('gallery',8),('discussion',8),('catchup',8),('polls',8),('gatherings',8),('pages',8),('wall',7),('full',7),('writing',13),('browser',10),('operator',8),('launcher',7),('directory',11),('nightwire',7)]:
        v=d['native_regressions'][n];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    assert d['native_art_previews']['profile_count']==18
    assert d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,frame_bytes=8192,manifest_bytes=32768,books=32,shelves=8,cards=4,text_rows=8,request_retained=256,request_open_per_member=8,request_input_bytes=128,list_back_cursors=64,detail_back_cursors=32,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('File request evidence passed: 321 Rust per OS, 6 request PTYs per OS, 8 actual SSH/native/SFTP and 18 recovery controls per OS; normal budgets unchanged.')
if __name__=='__main__':check()
