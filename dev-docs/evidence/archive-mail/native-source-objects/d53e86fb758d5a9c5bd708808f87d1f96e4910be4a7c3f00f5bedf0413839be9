#!/usr/bin/env python3
"""Check saved native travel evidence; never starts a VM, SSH or SFTP."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,gallery_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/travel-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==14
    assert d['host_rust_tests']==d['native_rust_tests']==304 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_client_travel_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==272
    assert len(d['harness_sources'])==29 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=gallery_baseline_manifest();assert len(m['files'])==277 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/gallery-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_gallery_manifest_sha256']
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/gallery-netbsd/results.json').read_text())
    for name,digest in d['unchanged_dependency_access_credential_sources'].items():assert old['execution_sources'][name]==digest
    for name in ('host_travel','native_travel'):
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==6
        assert not any(v[k] for k in ('capacity_claim','wire_budget_claim','offline_exchange','route_change'))
    v=d['shared_travel'];assert v['result']=='pass' and v['count']==len(v['checks'])==9 and v['schema_version']==14
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and v['payload_bytes']==50
    assert not any(v[k] for k in ('capacity_claim','wire_budget_claim','offline_exchange','route_change'))
    for name in ('actual_ordinary_and_compact_visits_show_same_content_and_preserve_read_acknowledgment',
                 'actual_host_native_size_ceiling_defers_download_before_grant_and_upload_before_reservation',
                 'actual_netbsd_compact_exact_ceiling_download_uses_restricted_sftp_and_checksum',
                 'actual_netbsd_exact_ceiling_upload_preserves_pending_review_and_bounded_transfer',
                 'actual_compact_mode_keeps_pinned_host_mismatch_denial_and_no_route_retry'):assert name in v['checks']
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
    for name,count in [('host_gallery',8),('native_gallery',8),('native_discussion',8),('native_catchup',8),('native_polls',8),('native_gatherings',8),('native_pages',8),('native_wall',7),('native_full_screen',7),('native_writing',13),('native_browser',10),('native_operator',8),('native_launcher',7),('native_directory',11),('native_nightwire_logins',7)]:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==count and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count']==18
    assert d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,default_file_payload_bytes=16777216,native_chunk_bytes=32768,remote_output_bytes=32768,local_catalog_bytes=16384,local_boards=16,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Travel evidence passed: 304 Rust per OS, 6 local travel PTYs per OS, 9 actual native/SSH/SFTP controls and 17 recovery controls per OS; no wire-budget/offline claim.')
if __name__=='__main__':check()
