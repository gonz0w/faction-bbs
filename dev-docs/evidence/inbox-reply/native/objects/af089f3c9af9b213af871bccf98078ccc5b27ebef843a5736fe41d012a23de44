#!/usr/bin/env python3
"""Check preserved standalone connection-lounge software results; no live network needed."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,boardwide_social_baseline_manifest
def check():
    d=json.loads((ROOT/'dev-docs/evidence/standalone-social-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==17
    assert d['host_rust_tests']==d['native_rust_tests']==351 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_social_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==320 and len(d['harness_sources'])==42 and len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
        for n,h in d[group].items():assert hashlib.sha256(source_bytes(n,h)).hexdigest()==h,n
    m=boardwide_social_baseline_manifest();assert len(m['files'])==326 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/boardwide-social-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_boardwide_social_manifest_sha256']
    for n,h in m['records'].items():assert hashlib.sha256((ROOT/n).read_bytes()).hexdigest()==h
    old=json.loads((ROOT/'dev-docs/evidence/boardwide-social-netbsd/results.json').read_text())
    for n,h in d['unchanged_dependency_access_credential_sources'].items():assert old['unchanged_dependency_access_credential_sources'][n]==h
    for name in ['host_social','native_social']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==17 and v['synthetic_clock_aging']
        assert not any(v[k] for k in ['capacity_claim','private_message_bodies','boardwide_presence','automatic_notifications'])
    for name in ['host_standalone','native_standalone']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==12 and v['schema_version']==17 and v['synthetic_clock_aging']
        assert v['sustained'] and v['actual_registration_and_approval'] and v['standalone_visit_availability']
        assert not any(v[k] for k in ['capacity_claim','automatic_popups','private_message_bodies','new_transfer_authority'])
        assert len(v['heartbeat_observations'])==3
        assert {h['state'] for h in v['heartbeat_observations']}=={'local_menu','worker_full_screen','plain_room_prompt'}
        for h in v['heartbeat_observations']:assert h['elapsed_seconds']>=66 and 0<=h['last_heartbeat_age_seconds']<=15
        assert v['checks']==['registered_approved_local_worker_plain_optin_reuses_one_visit_lease_and_parts_chat', 'parallel_menu_full_screen_and_canonical_prompt_keep_current_visit_heartbeats', 'safe_menu_refresh_accept_and_reply_without_forced_channel_switch', 'worker_full_screen_and_plain_canonical_prefix_survive_invitations_without_popup', 'current_block_mute_quiet_and_resize_remove_ineligible_signals', 'named_lounge_and_custom_hotkey_work_for_escape_free_plain_inherited_worker', 'plain_logout_and_reconnect_never_recreate_previous_advertisement', 'worker_transport_eof_releases_visit_advertisement_and_restores_no_stale_authority', 'current_revocation_closes_held_local_visit_and_restores_terminal', 'failed_chat_part_cleanup_ends_visit_instead_of_resuming_with_retained_channel_joins', 'sysop_disable_clears_live_signals_and_logout_closes_final_visit', 'schema_unread_transfer_state_unchanged_and_passwords_never_echoed']
    assert len(d['unchanged_shared_runtime_sources'])==190
    for n,h in d['unchanged_shared_runtime_sources'].items():
        assert old['execution_sources'][n]==h and hashlib.sha256(source_bytes(n,h)).hexdigest()==h
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
    assert d['native_service_refresh_literal_checked']
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,frame_bytes=8192,social_command_frame_bytes=4096,presence_entries=256,signals=128,signals_per_entry=8,cooldown_rows=256,cooldown_seconds=30,signal_seconds=60,blocks_per_member=16,status_bytes=48,name_bytes=24,roster_handles=8,new_registry_packages=0,heartbeat_seconds=10,leases_per_shared_visit=1,leases_per_standalone_visit=1,input_poll_ms=100,line_deadline_seconds=120)
    assert d['features']==dict(standalone_visit_availability=True,safe_menu_badge=True,shared_sources_unchanged=True,bodyless_page_requests=True,durable_privacy_controls=True)
    assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
    print('Standalone lounge evidence passed: 351 Rust per OS, 12 registered/approved local/worker/plain controls per OS, 3 concurrent real 66-second waits per OS and 20 recovery per OS; no new shared execution claim.')
if __name__=='__main__':check()
