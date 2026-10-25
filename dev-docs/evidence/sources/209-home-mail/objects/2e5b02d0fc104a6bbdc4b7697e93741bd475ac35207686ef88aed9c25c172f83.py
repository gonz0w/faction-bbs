#!/usr/bin/env python3
"""Verify immutable #209 saved software evidence; never run a client or VM."""
import hashlib
import json
import math
import re
from pathlib import PurePosixPath
from evidence_sources import (
    ROOT, source_bytes, native_network_baseline_manifest,
    network_prekeys_baseline_manifest, network_dependency_baseline_manifest,
)

# The lead pins the complete actual record after native execution and teardown.
# A placeholder must never accept a draft or synthetic fixture as saved evidence.
RECORD_SHA256 = '0' * 64
COST_SCOPE = 'one complete faction-client manual UI process, including checkpoint unlock and one TLS/account/publication/pending-write cycle'
UI_CHECKS = {
    'explicit_local_online_entry_and_hidden_unlock',
    'authenticated_history_has_classic_inert_unicode',
    'unicode_compose_queues_encrypted_state_without_network',
    'unavailable_home_and_hidden_credential_keep_pending_state',
    'reopen_preserves_exact_existing_ids_and_new_ciphertext',
    'unicode_override_and_local_cancel_restore_terminal',
    'wrong_local_unlock_never_discloses_history',
    'oversized_hidden_home_credential_locks_and_flushes_terminal',
    'typeahead_cannot_supply_hidden_home_credential',
}
INTEROP_CHECKS = {
    'full_native_device_identity_deliberately_verified',
    'durable_public_bundle_published_and_verified_peer_prepared',
    'native_encrypted_subject_and_body_cross_two_homes',
    'native_authenticated_reply_and_explicit_pending_outbox',
    'reopen_retry_preserves_exact_operations_and_bounded_duplicate_history',
    'no_private_subject_or_body_in_home_message_storage_or_logs',
    'tls_wrong_leaf_pin_denied_without_checkpoint_mutation',
    'home_wrong_password_denied_without_checkpoint_mutation',
    'server_replaced_device_list_denied_without_checkpoint_mutation',
    'server_changed_identity_bundle_denied_without_checkpoint_mutation',
    'server_added_device_never_enrolled_or_addressed',
}
TLS_OUTCOMES = {
    'verified_tls_plain_auth_and_ciphertext_exchange': 'ok',
    'verified_tls12_profile': 'ok',
    'fragmented_xml_exchange': 'ok',
    'coalesced_sasl_restart_retains_unread_bytes': 'ok',
    'wrong_server_name': 'tls',
    'untrusted_certificate': 'tls',
    'changed_certificate_pin': 'pin',
    'failed_account_authentication': 'auth',
    'no_plain_mechanism': 'mechanism',
    'malformed_xml_rejected': 'xml_malformed',
    'excessive_depth_rejected_before_dom': 'xml_limit',
    'excessive_events_rejected_before_dom': 'xml_limit',
    'oversized_unfinished_text_rejected': 'xml_limit',
    'oversized_stanza_rejected': 'xml_limit',
    'root_whitespace_budget_is_bounded': 'xml_limit',
    'connection_loss_has_no_retry': 'lost',
    'absolute_deadline_bounds_trickling_peer': 'timeout',
    'explicit_cancellation_bounds_stalled_peer': 'cancelled',
    'wrong_bound_identity': 'bind',
    'wrong_bind_identifier': 'bind',
}
TEARDOWN = {'owned_vm_removed', 'owned_private_inputs_removed', 'ports_closed', 'saved_playground_stopped', 'saved_playground_unchanged'}
FALSE_CLAIMS = {'physical_hardware', 'capacity', 'security_audit', 'full_conformance', 'reliable_archive', 'roaming', 'attachments'}
ROUTES = {'host', 'netbsd'}


def load_record(raw):
    assert hashlib.sha256(raw).hexdigest() == RECORD_SHA256, 'saved native home-mail record changed or not yet pinned'
    return json.loads(raw)


def source_path(name):
    assert isinstance(name, str) and name and '\\' not in name
    path = PurePosixPath(name)
    assert not path.is_absolute() and '..' not in path.parts and str(path) == name
    assert not any(ord(char) < 32 or ord(char) == 127 for char in name)


def source_digest(name, digest, resolve):
    source_path(name)
    assert isinstance(digest, str) and re.fullmatch('[a-f0-9]{64}', digest)
    assert hashlib.sha256(resolve(name, digest)).hexdigest() == digest, name


def named_checks(row, required):
    assert row['result'] == 'pass'
    checks = row['checks']
    assert isinstance(checks, list) and len(checks) == len(set(checks)) == len(required)
    assert set(checks) == required


def validate(d, resolve):
    assert type(d['schema']) is int and d['schema'] == 1
    assert type(d['issue']) is int and d['issue'] == 209
    assert d['result'] == 'pass-native-home-mail' and d['default_feature_off'] is True
    assert set(d['runs']) == ROUTES and d['runs']['netbsd']['platform'] == 'NetBSD'
    for ui in d['runs'].values():
        named_checks(ui, UI_CHECKS)
        assert ui['count'] == 9 and ui['private_fixture_removed'] is True
        assert ui['provider_contact'] is False
    rust = d['rust']
    assert rust['host_passed'] == rust['netbsd_passed'] == 104
    assert type(rust['intentional_ignored_helper']) is int and rust['intentional_ignored_helper'] == 1
    assert rust['helper_invoked_by_parent'] is True
    assert set(d['transport']) == ROUTES
    for route in d['transport'].values():
        assert route['rust_passed'] == 25
        matrix = route['matrix']
        assert matrix['result'] == 'pass' and matrix['count'] == 20
        assert matrix['private_fixture_removed'] is True and matrix['product_transport_exercised'] is True
        assert matrix['encrypted_mail_proof'] is False and matrix['live_federation'] is False
        rows = matrix['outcomes']
        assert len(rows) == 20 and {row['check'] for row in rows} == set(TLS_OUTCOMES)
        for row in rows:
            assert row['result'] == 'pass' and row['status'] == TLS_OUTCOMES[row['check']]
            assert type(row['attempts']) is int and row['attempts'] == 1
            assert row['tls_version'] in {None, 'TLSv1.2', 'TLSv1.3'}
            if row['status'] == 'ok':
                assert row['tls_version'] is not None
            if row['check'] == 'verified_tls12_profile':
                assert row['tls_version'] == 'TLSv1.2'
        assert {'TLSv1.2', 'TLSv1.3'} <= {row['tls_version'] for row in rows}
    interop = d['native_interop']
    named_checks(interop, INTEROP_CHECKS)
    assert interop['platform'] == 'NetBSD' and interop['scope'] == 'complete'
    assert interop['independent_unprivileged_homes'] == 2 and interop['native_product_client'] is True
    assert interop['no_offline_archive_reliability_claim'] is True and interop['live_provider_contact'] is False
    assert set(d['default_off']) == ROUTES
    expected_compatibility = {route + '_' + kind for route in ROUTES for kind in ['checkpoint', 'vault', 'picker']}
    assert set(d['compatibility']) == expected_compatibility
    for route in ROUTES:
        disabled = d['default_off'][route]
        assert disabled['result'] == 'pass' and disabled['unsupported_command_rejected'] is True and disabled['store_not_created'] is True
        for kind, count in [('checkpoint', 23), ('vault', 13), ('picker', 8)]:
            old = d['compatibility'][route + '_' + kind]
            assert old['result'] == 'pass' and old['count'] == count
    assert d['build']['complete_builder_mib'] == 2048 and d['build']['fresh_1g_compile_claim'] is False
    runtime = d['runtime']
    assert runtime['configured_mib'] == 1024 and runtime['physmem_bytes'] == 1054990336 and runtime['ncpu'] == 2
    cost = d['cost']
    assert cost['result'] == 'pass' and cost['usage']['scope'] == COST_SCOPE
    for field in ['wall_seconds', 'user_seconds', 'system_seconds', 'max_rss_kib']:
        value = cost['usage'][field]
        assert type(value) in {int, float} and math.isfinite(value)
        assert value > 0 if field in {'wall_seconds', 'max_rss_kib'} else value >= 0
    for field in ['whole_client', 'includes_unlock', 'includes_network_cycle', 'terminal_restored']:
        assert cost[field] is True
    for field in ['compiler_included', 'server_fixture_included', 'capacity_claim']:
        assert cost[field] is False
    assert set(d['teardown']) == TEARDOWN and all(value is True for value in d['teardown'].values())
    assert set(d['claims']) == FALSE_CLAIMS and all(value is False for value in d['claims'].values())
    for group in ['execution_sources', 'frozen_sources']:
        assert isinstance(d[group], dict) and d[group]
        for name, digest in d[group].items():
            source_digest(name, digest, resolve)
    stages = d['source_stages']
    assert isinstance(stages, list) and stages
    assert len({stage['label'] for stage in stages}) == len(stages)
    for stage in stages:
        assert isinstance(stage['label'], str) and stage['label'].strip()
        assert isinstance(stage['files'], list) and stage['files']
        assert len({row['name'] for row in stage['files']}) == len(stage['files'])
        for row in stage['files']:
            source_path(row['name'])
            source_digest(row['resolved_path'], row['sha256'], resolve)


def check():
    data = load_record((ROOT / 'dev-docs/evidence/home-mail/results.json').read_bytes())
    validate(data, source_bytes)
    for read_manifest in [native_network_baseline_manifest, network_prekeys_baseline_manifest, network_dependency_baseline_manifest]:
        prior = read_manifest()
        for name, digest in prior['files'].items():
            source_digest(name, digest, source_bytes)
        for name, digest in prior.get('records', {}).items():
            assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest, 'prior record changed'
    print('Saved native home-mail evidence verified: source-matched host/NetBSD controls, whole-client runtime cost, distinct build budget and owned teardown; no hardware/audit/capacity claim.')


if __name__ == '__main__':
    check()
