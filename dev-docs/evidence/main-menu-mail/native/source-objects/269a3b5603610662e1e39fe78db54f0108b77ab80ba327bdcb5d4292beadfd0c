#!/usr/bin/env python3
"""Check preserved local social-lounge software results; no live network needed."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,editorial_baseline_manifest
def check():
    d=json.loads((ROOT/'dev-docs/evidence/social-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==17
    assert d['host_rust_tests']==d['native_rust_tests']==345 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_social_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==307 and len(d['harness_sources'])==40 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
        for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
    m=editorial_baseline_manifest();assert len(m['files'])==310 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/editorial-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_editorial_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/editorial-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['unchanged_dependency_access_credential_sources'][n]==h
    for name in ['host_social','native_social']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==17 and v['synthetic_clock_aging']
        assert not any(v[k] for k in ['capacity_claim','private_message_bodies','boardwide_presence','automatic_notifications'])
    v=d['shared_social'];assert v['result']=='pass' and v['count']==len(v['checks'])==7 and v['schema_version']==17 and v['synthetic_clock_aging']
    assert v['normal_sessions']==8 and v['admission_refill_ms']==5000 and not any(v[k] for k in ['capacity_claim','private_message_bodies','boardwide_presence','automatic_notifications'])
    for n in ['actual_ssh_and_native_explicit_optin_hide_private_channel_and_unadvertised_members',
              'actual_bodyless_page_accept_and_cooldown_preserve_draft_and_channel_selection',
              'actual_mutual_block_mute_and_utc_quiet_hours_suppress_invitations_without_existence_oracles',
              'actual_sysop_availability_and_decline_are_local_signals_without_host_privileges',
              'actual_aged_request_expiry_disconnect_and_reconnect_never_recreate_presence',
              'actual_plain_ssh_preferences_availability_and_pages_use_escape_free_presentation',
              'actual_current_account_revocation_expires_held_social_entry_and_sysop_disable_clears_visibility']:assert n in v['checks']
    for name in ['host_backup','native_backup']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==20 and not v['capacity_claim'] and not v['hardening_claim']
        assert 'complete_restore_preserves_lounge_privacy_intent_without_live_presence_pages_or_rate_timestamps' in v['checks']
    assert d['native_backup']['native_distinct_uid_cli'] and d['host_backup']['developer_api_fixture']
    for n,count in [('editorial',10),('magazine',7),('requests',6),('library',8),('travel',6),('gallery',8),('discussion',8),('catchup',8),('polls',8),('gatherings',8),('pages',8),('wall',7),('full',7),('writing',13),('browser',10),('operator',8),('launcher',7),('directory',11),('nightwire',7),('picker',8),('vault',13)]:
        v=d['native_regressions'][n];assert v['result']=='pass' and v['count']==len(v['checks'])==count
    assert d['native_art_previews']['profile_count']==18 and d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,frame_bytes=8192,social_command_frame_bytes=4096,presence_entries=256,signals=128,signals_per_entry=8,cooldown_rows=256,cooldown_seconds=30,signal_seconds=60,blocks_per_member=16,status_bytes=48,name_bytes=24,roster_handles=8,new_registry_packages=0)
    assert d['features']==dict(chat_only_availability=True,bodyless_page_requests=True,durable_privacy_controls=True)
    assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
    print('Social lounge evidence passed: 345 Rust per OS, 8 social PTYs per OS, 7 actual SSH/native controls and 20 recovery per OS; chat-only explicit polling.')
if __name__=='__main__':check()
