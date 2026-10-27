#!/usr/bin/env python3
"""Verify the saved current-client/vault integration; never start a VM or network."""
import hashlib
import json
from evidence_sources import ROOT, source_bytes, member_pages_baseline_manifest, local_mail_vault_baseline_manifest


def check():
    data = json.loads((ROOT / 'dev-docs/evidence/client-vault-integration-netbsd/results.json').read_text())
    assert data['schema'] == 1 and data['result'] == 'pass'
    assert data['host_rust_tests'] == data['native_rust_tests'] == 241
    assert data['host_python_tests'] == 129
    assert data['native_workspace_clean_before_build'] and data['native_final_client_clean_before_build']
    assert data['native_guest'] == dict(system='NetBSD', release='11.0', architecture='aarch64', cores=2, ram_mib=1024)
    assert len(data['execution_sources']) >= 200 and len(data['harness_sources']) >= 8
    for group in ('execution_sources', 'harness_sources', 'unchanged_non_client_runtime_sources'):
        for name, digest in data[group].items():
            assert hashlib.sha256(source_bytes(name, digest)).hexdigest() == digest, name
    for baseline in (member_pages_baseline_manifest(), local_mail_vault_baseline_manifest()):
        for name, digest in baseline['records'].items():
            assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest
    for name in ('host_vault', 'native_vault'):
        result = data[name]
        assert result['result'] == 'pass' and result['count'] == len(result['checks']) == 13
        assert not result['network_mail_implemented'] and not result['roaming_authentication_implemented']
        assert not result['encrypted_attachments_implemented'] and result['private_fixture_removed_on_exit']
    for name in ('host_picker', 'native_picker'):
        result = data[name]
        assert result['result'] == 'pass' and result['count'] == len(result['checks']) == 8
        assert not result['network_mail_implemented'] and not result['remote_unlock_authority']
        for required in ('explicit_private_catalog_mail_opens_local_vault_and_returns_locked',
                         'picker_drafts_remain_encrypted_unsent_and_survive_reopen',
                         'sixteen_board_catalog_and_optional_vault_fit_40x18',
                         'missing_store_is_not_created_or_recovered_implicitly'):
            assert required in result['checks']
    shared = data['shared_pages']
    assert shared['result'] == 'pass' and shared['count'] == len(shared['checks']) == 14
    assert shared['normal_sessions'] == 8 and shared['admission_refill_ms'] == 5000
    assert data['host_and_native_launcher_checks'] == 7 and data['host_and_native_graphics_checks'] == 3
    assert data['reviewed_dependency_union_matches'] and data['server_resource_defaults_unchanged']
    assert all(data['teardown'].values()) and not any(data['claims'].values())
    assert all(len(digest) == 64 for digest in data['native_binaries_sha256'].values())
    print('Client/vault integration passed: 241 Rust per OS, 13 vault + 8 local picker PTYs per OS, 14 shared SSH/native/plain checks; no network mail or capacity claim.')


if __name__ == '__main__':
    check()
