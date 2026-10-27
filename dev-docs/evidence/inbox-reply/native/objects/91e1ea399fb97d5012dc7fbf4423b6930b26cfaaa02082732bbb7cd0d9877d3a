#!/usr/bin/env python3
"""Check saved curated gallery evidence without starting a VM or service."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, discussion_baseline_manifest


def check():
    d=json.loads((ROOT/'dev-docs/evidence/gallery-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==14
    assert d['host_rust_tests']==d['native_rust_tests']==302 and d['host_python_tests']==129
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_gallery_literal_checked']
    assert d['selected_native_source_matches']==len(d['execution_sources'])==269
    assert len(d['harness_sources'])==25 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ('execution_sources','harness_sources','unchanged_dependency_access_credential_sources'):
        for name,digest in d[group].items():
            assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=discussion_baseline_manifest()
    assert len(m['files'])==258 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/discussion-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_discussion_manifest_sha256']
    for name,digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/discussion-netbsd/results.json').read_text())
    for name,digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['execution_sources'][name]==digest
    for name in ('host_gallery','native_gallery'):
        v=d[name]
        assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==14
        assert not v['capacity_claim'] and not v['raw_member_art'] and not v['public_art_publication']
    v=d['shared_gallery']
    assert v['result']=='pass' and v['count']==len(v['checks'])==11 and v['schema_version']==14
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert not v['capacity_claim'] and not v['raw_member_art'] and not v['public_art_publication']
    for name in ('actual_resize_switches_to_text_and_restarts_fitting_art_page',
                 'actual_optional_download_cannot_reveal_private_filename_or_reference',
                 'actual_download_reference_rechecks_live_file_grant_and_revocation',
                 'actual_member_revocation_stops_gallery_without_room_read_acknowledgment',
                 'actual_owner_format_removal_changes_catalog_on_stopped_restart',
                 'actual_disabled_pack_removes_main_menu_capability',
                 'actual_netbsd_offline_preflight_rejects_missing_display_rights'):
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
    for name,count in [('host_discussion',8),('native_discussion',8),('native_catchup',8),('native_polls',8),('native_gatherings',8),('native_pages',8),
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
        owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,gallery_frame_bytes=8192,
        works=8,exhibits=4,manifest_bytes=32768,art_asset_bytes=65536,total_art_bytes=262144,
        canonical_pages=64,encoded_art_page_bytes=7168,fallback_asset_bytes=8192,
        total_fallback_bytes=32768,fallback_lines=128,fallback_columns=72,cards=4,
        text_rows=8,back_cursors=32,new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Gallery evidence passed: 302 Rust per OS, 8 gallery PTYs per OS, 11 actual SSH/native controls and 17 recovery controls per OS; no capacity or raw-art claim.')


if __name__=='__main__':
    check()
