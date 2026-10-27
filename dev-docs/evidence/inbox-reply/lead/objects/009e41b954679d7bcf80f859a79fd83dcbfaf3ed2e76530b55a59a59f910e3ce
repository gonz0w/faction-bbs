#!/usr/bin/env python3
"""Check optional idle-menu notification evidence without live network access."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, standalone_social_baseline_manifest


def check():
    d = json.loads((ROOT/'dev-docs/evidence/idle-notifications-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass' and d['database_schema'] == 17
    assert d['host_rust_tests'] == d['native_rust_tests'] == 354 and d['host_python_tests'] == 130
    assert d['native_guest'] == dict(system='NetBSD', release='11.0', architecture='aarch64', cores=2, ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_social_literal_checked'] and d['native_service_refresh_literal_checked']
    assert len(d['execution_sources']) == d['selected_native_source_matches'] == 323
    assert len(d['harness_sources']) == 44 and len(d['unchanged_dependency_access_credential_sources']) == 13
    for group in ['execution_sources', 'harness_sources', 'unchanged_dependency_access_credential_sources']:
        for name, digest in d[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest, name
    m = standalone_social_baseline_manifest()
    assert len(m['files']) == 329 and len(m['records']) == 1
    assert hashlib.sha256((ROOT/'dev-docs/evidence/standalone-social-baseline-manifest.json').read_bytes()).hexdigest() == d['prior_standalone_social_manifest_sha256']
    for name, digest in m['records'].items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest() == digest
    old = json.loads((ROOT/'dev-docs/evidence/standalone-social-netbsd/results.json').read_text())
    for name, digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['unchanged_dependency_access_credential_sources'][name] == digest
    for name in ['host_idle_notifications', 'native_idle_notifications', 'shared_idle_notifications']:
        v = d[name]
        expected = 12 if name.startswith('shared') else 20
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == expected
        assert v['schema_version'] == 17 and v['synthetic_clock_aging']
        assert v['default_off'] and v['visit_only'] and v['idle_main_menu_only']
        assert not any(v[k] for k in ['capacity_claim', 'private_message_bodies', 'forced_switch', 'bells'])
    for name in ['host_idle_notifications', 'native_idle_notifications']:
        assert d[name]['plain_escape_free'] and d[name]['actual_registration_and_approval']
    v = d['shared_idle_notifications']
    assert v['actual_ssh_and_native'] and v['normal_sessions'] == 8 and v['admission_refill_ms'] == 5000
    assert v['compact_server_frames_escape_free'] and v['native_client_exit_reset']
    assert v['checks'] == [
        'actual_ssh_default_off_and_optin_one_quiet_coalesced_idle_notice',
        'actual_ssh_partial_escape_quarantine_and_arrow_focus_survive_idle_frames',
        'actual_enabled_notifications_preserve_composer_and_unsubmitted_input',
        'actual_enabled_notifications_leave_lua_game_and_held_input_untouched',
        'actual_page_waits_through_terminal_settings_and_sync_probe_without_injected_command',
        'actual_explicit_outgoing_answer_announces_only_at_idle_menu',
        'actual_host_native_client_receives_idle_notification_without_extra_authority',
        'actual_netbsd_native_compact_client_resets_consent_and_preserves_plain_partial_command',
        'actual_plain_ssh_idle_notice_has_no_escapes_and_keeps_partial_menu_command',
        'actual_current_block_and_expired_deferred_signal_never_create_idle_notification',
        'actual_notification_poll_rechecks_current_revocation_and_sysop_disable',
        'actual_normal_budget_schema_unread_transfer_and_hidden_password_invariants']
    for name in ['host_social', 'native_social']:
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 8 and v['schema_version'] == 17
        assert not any(v[k] for k in ['capacity_claim', 'private_message_bodies', 'boardwide_presence', 'automatic_notifications'])
    for name in ['host_standalone', 'native_standalone']:
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 12 and v['schema_version'] == 17
        assert v['sustained'] and v['actual_registration_and_approval'] and v['standalone_visit_availability']
        assert len(v['heartbeat_observations']) == 3
        assert {h['state'] for h in v['heartbeat_observations']} == {'local_menu', 'worker_full_screen', 'plain_room_prompt'}
        for h in v['heartbeat_observations']:
            assert h['elapsed_seconds'] >= 66 and 0 <= h['last_heartbeat_age_seconds'] <= 15
    v = d['shared_boardwide_social']
    assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 8 and v['schema_version'] == 17
    assert v['normal_sessions'] == 8 and v['admission_refill_ms'] == 5000 and v['outside_chat_shared_availability']
    assert len(v['heartbeat_observations']) == 3
    assert {h['state'] for h in v['heartbeat_observations']} == {'main_menu', 'room_composer', 'lua_game_input'}
    for h in v['heartbeat_observations']:
        assert h['elapsed_seconds'] >= 66 and 0 <= h['last_heartbeat_age_seconds'] <= 15
    for name in ['host_backup', 'native_backup']:
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 20
        assert not v['capacity_claim'] and not v['hardening_claim']
        assert 'complete_restore_preserves_lounge_privacy_intent_without_live_presence_pages_or_rate_timestamps' in v['checks']
    assert d['native_backup']['native_distinct_uid_cli'] and d['host_backup']['developer_api_fixture']
    for name, count in [('editorial',10), ('magazine',7), ('requests',6), ('library',8), ('travel',6), ('gallery',8), ('discussion',8), ('catchup',8), ('polls',8), ('gatherings',8), ('pages',8), ('wall',7), ('full',7), ('writing',13), ('browser',10), ('operator',8), ('launcher',7), ('directory',11), ('nightwire',7), ('picker',8), ('vault',13)]:
        v = d['native_regressions'][name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == count
    assert d['native_art_previews']['profile_count'] == 18 and d['native_custom_menu_reader_and_console_regressions']
    assert d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group, count in [('host_binaries_sha256',5), ('native_binaries_sha256',5), ('installed_native_binaries_sha256',3)]:
        assert len(d[group]) == count and all(len(h) == 64 and set(h) <= set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][name] == digest for name, digest in d['installed_native_binaries_sha256'].items())
    door = d['native_door_host']
    assert door['built_sha256'] == door['installed_sha256'] and door['new_packages'] == 0
    assert all(len(door[k]) == 64 and set(door[k]) <= set('0123456789abcdef') for k in ['built_sha256', 'installed_sha256', 'base_lua_library_sha256'])
    assert d['budgets'] == dict(normal_sessions=8, admission_refill_ms=5000, credential_permits=2, owner_helpers=3, shared_output_bytes=32768, drain_seconds=5, frame_bytes=8192, social_command_frame_bytes=4096, presence_entries=256, signals=128, signals_per_entry=8, cooldown_rows=256, cooldown_seconds=30, signal_seconds=60, blocks_per_member=16, status_bytes=48, name_bytes=24, roster_handles=8, new_registry_packages=0, heartbeat_seconds=10, leases_per_shared_visit=1, leases_per_standalone_visit=1, input_poll_ms=100, line_deadline_seconds=120, notification_poll_ms=2000, notification_idle_ms=2000, notification_signal_keys=8)
    assert d['features'] == dict(standalone_visit_availability=True, safe_menu_badge=True, optional_idle_notifications=True, visit_only_notification_consent=True, default_off_notifications=True, protected_draft_game_probe_input=True, bodyless_page_requests=True, durable_privacy_controls=True)
    assert all(d['features'].values()) and all(d['teardown'].values()) and not any(d['claims'].values())
    print('Optional idle notification evidence passed: 354 Rust per OS, 20 local/worker/plain controls per OS, 12 actual SSH/native/Lua controls and maintained sustained/recovery regressions; no capacity/hardening claim.')


if __name__ == '__main__':
    check()
