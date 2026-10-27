#!/usr/bin/env python3
"""Verify saved gathering software evidence without starting services or a VM."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, client_vault_integration_baseline_manifest


def check():
    d = json.loads((ROOT / 'dev-docs/evidence/gatherings-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass' and d['database_schema'] == 11
    assert d['host_rust_tests'] == d['native_rust_tests'] == 256 and d['host_python_tests'] == 129
    assert d['configured_cores'] == 2 and d['configured_ram_bytes'] == 1073741824
    assert d['native_guest'] == dict(system='NetBSD', release='11.0', architecture='aarch64')
    assert d['native_workspace_clean_before_build'] and d['native_service_gathering_literal_checked']
    assert d['selected_native_source_matches'] == len(d['execution_sources']) == 220
    assert len(d['harness_sources']) == 17
    for group in ('execution_sources', 'harness_sources', 'unchanged_dependency_access_credential_sources'):
        for name, digest in d[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest, name
    m = client_vault_integration_baseline_manifest()
    assert len(m['files']) == 217 and len(m['records']) == 1
    assert hashlib.sha256((ROOT / 'dev-docs/evidence/client-vault-integration-baseline-manifest.json').read_bytes()).hexdigest() == d['prior_client_vault_manifest_sha256']
    for name, digest in m['records'].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
    old = json.loads((ROOT / 'dev-docs/evidence/client-vault-integration-netbsd/results.json').read_text())
    for name, digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['execution_sources'][name] == digest
    for name in ('host_gatherings', 'native_gatherings'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 8 and v['schema_version'] == 11
        assert not v['capacity_claim'] and not v['external_calendar']
    v = d['shared_gatherings']
    assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 11 and v['schema_version'] == 11
    assert v['normal_sessions'] == 8 and v['admission_refill_ms'] == 5000 and not v['capacity_claim']
    for name in ('actual_two_member_rsvp_counts_policy_and_no_attendee_names',
                 'actual_concurrent_edits_refuse_stale_event_revision',
                 'actual_busy_save_retains_edit_for_single_retry',
                 'actual_shared_resize_preserves_completed_event_fields',
                 'actual_hide_invalidates_open_foreign_card',
                 'actual_active_revocation_prevents_reviewed_rsvp_save',
                 'actual_cancel_blocks_jump_and_explicit_remove_cascades_responses'):
        assert name in v['checks']
    for name in ('host_backup', 'native_backup'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 14
        assert 'complete_restore_preserves_gathering_policy_event_and_response' in v['checks']
        assert not v['capacity_claim'] and not v['hardening_claim']
    assert d['native_backup']['native_distinct_uid_cli'] and not d['native_backup']['developer_api_fixture']
    assert d['host_backup']['developer_api_fixture'] and not d['host_backup']['native_distinct_uid_cli']
    for name in ('host_vault', 'native_vault'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 13
        assert not v['network_mail_implemented'] and not v['roaming_authentication_implemented']
    for name in ('host_picker', 'native_picker'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 8
        assert not v['network_mail_implemented'] and not v['remote_unlock_authority']
    for name, count in [('native_pages', 8), ('native_wall', 7), ('native_full_screen', 7),
                        ('native_writing', 13), ('native_browser', 10), ('native_launcher', 7),
                        ('native_operator', 8), ('native_directory', 11), ('native_nightwire_logins', 7)]:
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == count and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count'] == 18
    assert d['native_custom_menus_local_reader_and_console_regressions']
    assert d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group, count in [('host_binaries_sha256', 5), ('native_binaries_sha256', 5), ('installed_native_binaries_sha256', 3)]:
        assert len(d[group]) == count
        assert all(len(h) == 64 and set(h) <= set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n] == h for n, h in d['installed_native_binaries_sha256'].items())
    assert d['budgets'] == dict(normal_sessions=8, admission_refill_ms=5000, credential_permits=2,
        owner_helpers=3, shared_output_bytes=32768, drain_seconds=5, gathering_frame_bytes=8192,
        retained_events=64, ordinary_future_events=8, event_responses=250, ended_retention_days=30,
        title_bytes=64, note_bytes=480, note_lines=8, line_columns=72, new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Gathering evidence passed: 256 Rust per OS, 8 PTYs per OS, 11 actual SSH/native controls and 14 recovery controls per OS; no capacity claim.')


if __name__ == '__main__':
    check()
