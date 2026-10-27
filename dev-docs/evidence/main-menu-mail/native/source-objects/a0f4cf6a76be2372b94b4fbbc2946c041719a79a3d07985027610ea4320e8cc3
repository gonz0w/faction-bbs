#!/usr/bin/env python3
"""Check saved local-poll evidence without starting a service or VM."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, gatherings_baseline_manifest


def check():
    d = json.loads((ROOT / 'dev-docs/evidence/polls-netbsd/results.json').read_text())
    assert d['schema'] == 1 and d['result'] == 'pass' and d['database_schema'] == 12
    assert d['host_rust_tests'] == d['native_rust_tests'] == 268 and d['host_python_tests'] == 129
    assert d['native_guest'] == dict(system='NetBSD', release='11.0', architecture='aarch64', cores=2, ram_mib=1024)
    assert d['native_workspace_clean_before_build'] and d['native_service_ballot_literal_checked']
    assert d['selected_native_source_matches'] == len(d['execution_sources']) == 230
    assert len(d['harness_sources']) == 19
    for group in ('execution_sources', 'harness_sources', 'unchanged_dependency_access_credential_sources'):
        for name, digest in d[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest, name
    m = gatherings_baseline_manifest()
    assert len(m['files']) == 228 and len(m['records']) == 1
    assert hashlib.sha256((ROOT / 'dev-docs/evidence/gatherings-baseline-manifest.json').read_bytes()).hexdigest() == d['prior_gatherings_manifest_sha256']
    for name, digest in m['records'].items():
        assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
    old = json.loads((ROOT / 'dev-docs/evidence/gatherings-netbsd/results.json').read_text())
    for name, digest in d['unchanged_dependency_access_credential_sources'].items():
        assert old['execution_sources'][name] == digest
    for name in ('host_polls', 'native_polls'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 8 and v['schema_version'] == 12
        assert not v['capacity_claim'] and not v['secret_ballot_claim']
    v = d['shared_polls']
    assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 12 and v['schema_version'] == 12
    assert v['normal_sessions'] == 8 and v['admission_refill_ms'] == 5000
    assert not v['capacity_claim'] and not v['secret_ballot_claim']
    for name in ('actual_two_members_after_close_visibility_without_voter_names',
                 'actual_concurrent_author_edits_reject_stale_revision',
                 'actual_busy_review_stays_readable_and_retry_saves_one_ballot',
                 'actual_resize_preserves_completed_unsaved_poll_notes',
                 'actual_moderation_hide_invalidates_open_foreign_card',
                 'actual_active_revocation_prevents_reviewed_ballot',
                 'actual_close_race_refuses_stale_review_without_changing_ballots',
                 'actual_cancel_and_explicit_delete_cascade_then_disable'):
        assert name in v['checks']
    for name in ('host_backup', 'native_backup'):
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == 15
        assert 'complete_restore_preserves_poll_policy_choices_ballot_and_wording_lock' in v['checks']
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
    for name, count in [('native_gatherings', 8), ('native_pages', 8), ('native_wall', 7),
                        ('native_full_screen', 7), ('native_writing', 13), ('native_browser', 10),
                        ('native_operator', 8), ('native_launcher', 7), ('native_directory', 11),
                        ('native_nightwire_logins', 7)]:
        v = d[name]
        assert v['result'] == 'pass' and v['count'] == len(v['checks']) == count and not v['capacity_claim']
    assert d['native_nightwire_previews']['profile_count'] == 18
    assert d['native_custom_menu_reader_and_console_regressions']
    assert d['host_and_native_export_restore'] and d['reviewed_dependency_union_matches']
    for group, count in [('host_binaries_sha256', 5), ('native_binaries_sha256', 5), ('installed_native_binaries_sha256', 3)]:
        assert len(d[group]) == count
        assert all(len(h) == 64 and set(h) <= set('0123456789abcdef') for h in d[group].values())
    assert all(d['native_binaries_sha256'][n] == h for n, h in d['installed_native_binaries_sha256'].items())
    assert d['budgets'] == dict(normal_sessions=8, admission_refill_ms=5000, credential_permits=2,
        owner_helpers=3, shared_output_bytes=32768, drain_seconds=5, poll_frame_bytes=8192,
        retained_polls=32, ordinary_open_polls=4, poll_ballots=250, choices_min=2, choices_max=8,
        question_bytes=96, choice_bytes=48, note_bytes=240, note_lines=4, line_columns=72,
        new_registry_packages=0)
    assert all(d['teardown'].values()) and not any(d['claims'].values())
    print('Poll evidence passed: 268 Rust per OS, 8 poll PTYs per OS, 12 actual SSH/native controls and 15 recovery controls per OS; no capacity or secret-ballot claim.')


if __name__ == '__main__':
    check()
