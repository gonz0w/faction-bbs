#!/usr/bin/env python3
"""Validate current Who/universal-presence software evidence offline."""
import hashlib,json
from evidence_sources import ROOT,source_bytes,idle_notifications_baseline_manifest

def check():
    d=json.loads((ROOT/'dev-docs/evidence/who-netbsd/results.json').read_text())
    assert d['schema']==1 and d['result']=='pass' and d['database_schema']==18
    assert d['host_rust_tests']==d['native_rust_tests']==359 and d['host_python_tests']==130
    assert d['native_guest']==dict(system='NetBSD',release='11.0',architecture='aarch64',cores=2,ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_social_literal_checked'] and d['native_service_refresh_literal_checked']
    assert len(d['execution_sources'])==d['selected_native_source_matches']==328 and len(d['harness_sources'])==47
    assert len(d['unchanged_dependency_access_credential_sources'])==13
    for group in ['execution_sources','harness_sources','unchanged_dependency_access_credential_sources']:
        for name,digest in d[group].items():assert hashlib.sha256(source_bytes(name,digest)).hexdigest()==digest,name
    m=idle_notifications_baseline_manifest();assert len(m['files'])==332 and len(m['records'])==1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/idle-notifications-baseline-manifest.json').read_bytes()).hexdigest()==d['prior_idle_notifications_manifest_sha256']
    for name,digest in m['records'].items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest
    old=json.loads((ROOT/'dev-docs/evidence/idle-notifications-netbsd/results.json').read_text())
    assert d['unchanged_dependency_access_credential_sources']==old['unchanged_dependency_access_credential_sources']
    for name in ['host_who','native_who','shared_who']:
        v=d[name];count=12 if name.startswith('shared') else 21
        assert v['result']=='pass' and v['count']==len(v['checks'])==count and v['schema_version']==18
        assert len(set(v['checks']))==count
        assert v['actual_registration_and_approval'] and v['universal_visibility_and_availability'] and v['distinct_interfaces'] and v['broad_host_activity']
        assert not any(v[k] for k in ['private_locations','capacity_claim','hardening_claim'])
    v=d['shared_who'];assert v['actual_ssh_and_native'] and v['compact_server_frames_escape_free'] and v['normal_sessions']==8 and v['admission_refill_ms']==5000
    assert {
        'actual_room_location_keeps_private_draft_and_pending_input_untouched',
        'actual_lua_location_hides_game_detail_and_preserves_held_input',
        'actual_files_and_terminal_settings_report_only_broad_current_location',
        'actual_group_who_is_boardwide_and_does_not_reveal_private_channel_names',
        'actual_lounge_retains_paging_attributes_and_visibility_is_universal_across_all_three',
        'actual_revocation_and_global_disable_remove_held_who_authority'
    } <= set(v['checks'])
    for name in ['host_idle_notifications','native_idle_notifications','shared_idle_notifications']:
        v=d[name];count=12 if name.startswith('shared') else 20
        assert v['result']=='pass' and v['count']==len(v['checks'])==count and v['schema_version']==18
        assert v['default_off'] and v['visit_only'] and v['idle_main_menu_only']
        assert not any(v[k] for k in ['capacity_claim','private_message_bodies','forced_switch','bells'])
    for name in ['host_social','native_social']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==18
    for name in ['host_standalone','native_standalone']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==12 and v['schema_version']==18 and v['sustained']
        assert {h['state'] for h in v['heartbeat_observations']}=={'local_menu','worker_full_screen','plain_room_prompt'}
        for h in v['heartbeat_observations']:assert h['elapsed_seconds']>=66 and 0<=h['last_heartbeat_age_seconds']<=15
    v=d['shared_boardwide_social'];assert v['result']=='pass' and v['count']==len(v['checks'])==8 and v['schema_version']==18
    assert {h['state'] for h in v['heartbeat_observations']}=={'main_menu','room_composer','lua_game_input'}
    for h in v['heartbeat_observations']:assert h['elapsed_seconds']>=66 and 0<=h['last_heartbeat_age_seconds']<=15
    for name in ['host_backup','native_backup']:
        v=d[name];assert v['result']=='pass' and v['count']==len(v['checks'])==21
        assert not v['capacity_claim'] and not v['hardening_claim']
        assert 'complete_restore_drops_transient_current_visit_activity' in v['checks']
    assert d['native_backup']['native_distinct_uid_cli'] and d['host_backup']['developer_api_fixture']
    for name,count in [('editorial',10),('magazine',7),('requests',6),('library',8),('travel',6),('gallery',8),('discussion',8),('catchup',8),('polls',8),('gatherings',8),('pages',8),('wall',7),('full',7),('writing',13),('browser',10),('operator',8),('launcher',7),('directory',11),('nightwire',7),('picker',8),('vault',13)]:
        v=d['native_regressions'][name];assert v['result']=='pass' and v['count']==len(v['checks'])==count
    assert d['native_art_previews']['profile_count']==18 and d['native_custom_menu_reader_and_console_regressions'] and d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group,count in [('host_binaries_sha256',5),('native_binaries_sha256',5),('installed_native_binaries_sha256',3)]:
        assert len(d[group])==count and all(len(h)==64 and set(h)<=set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n]==h for n,h in d['installed_native_binaries_sha256'].items())
    door=d['native_door_host'];assert door['built_sha256']==door['installed_sha256'] and door['new_packages']==0
    assert all(len(door[k])==64 and set(door[k])<=set('0123456789abcdef') for k in ['built_sha256','installed_sha256','base_lua_library_sha256'])
    assert d['budgets']==dict(normal_sessions=8,admission_refill_ms=5000,credential_permits=2,owner_helpers=3,shared_output_bytes=32768,drain_seconds=5,frame_bytes=8192,social_command_frame_bytes=4096,presence_entries=256,signals=128,signals_per_entry=8,cooldown_rows=256,cooldown_seconds=30,signal_seconds=60,blocks_per_member=16,status_bytes=48,name_bytes=24,roster_handles=8,new_registry_packages=0,heartbeat_seconds=10,leases_per_shared_visit=1,leases_per_standalone_visit=1,input_poll_ms=100,line_deadline_seconds=120,notification_poll_ms=2000,notification_idle_ms=2000,notification_signal_keys=8,activity_rows=256,activity_content_bytes=9,activity_history_rows=0)
    assert all(d['features'].values()) and d['features']['distinct_who_lounge_group_interfaces'] and d['features']['universal_visibility_and_availability'] and d['features']['host_derived_broad_activity']
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Who evidence passed: 359 Rust per OS, 21 local/worker/plain controls per OS, 12 actual SSH/native/Lua/Files controls, universal privacy and 21 recovery controls per OS; no capacity/hardening claim.')

if __name__=='__main__':check()
