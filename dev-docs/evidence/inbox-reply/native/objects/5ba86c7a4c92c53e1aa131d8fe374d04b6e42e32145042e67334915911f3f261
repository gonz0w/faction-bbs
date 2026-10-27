#!/usr/bin/env python3
"""Check recorded research identity/limits; does not rerun XMPP/SSH/VMs."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def digest(name):
    p = Path(name)
    assert not p.is_absolute() and '..' not in p.parts
    return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()


def check():
    d = json.loads((ROOT/'dev-docs/evidence/encrypted-mail-netbsd/results.json').read_text())
    assert d['result'] == 'reference_controls_pass_product_adoption_deferred'
    for part, count in [('mail', 12), ('identity_verifier', 11)]:
        assert d[part]['result'] == 'pass' and d[part]['count'] == count
        assert len(set(d[part]['checks'])) == count
        assert d[part]['product_integration'] is False
    assert d['mail']['namespace'] == 'urn:xmpp:omemo:2'
    assert d['mail']['independent_client_interop'] is False
    assert d['identity_verifier']['external_fixture_signer'] is True
    assert d['identity_verifier']['online_issuer'] is False
    assert d['identity_verifier']['mail_keys_used'] is False
    assert d['identity_verifier']['base_source_penalties_changed'] is False
    assert d['native_signing']['result'] == 'observed_failure' and d['native_signing']['count'] == 4
    assert all(x['exit'] == -11 and x['artifact_produced'] for x in d['native_signing']['controls'])
    assert d['native_signing']['core_dumps_disabled'] and d['native_signing']['keys_removed']
    assert d['fixture']['vcpus'] == 2 and d['fixture']['guest_ram_mib'] == 1024
    assert d['fixture']['two_home_processes_one_vm'] and d['fixture']['separate_home_uids_and_stores']
    assert d['fixture']['private_loopback_host_forwards']
    assert not d['fixture']['http_listeners'] and not d['fixture']['physical_hardware']
    assert not d['fixture']['real_accounts_or_external_peers']
    assert d['server_store_scan']['known_private_subject_body_absent']
    assert d['server_store_scan']['server_files_scanned'] > 0
    for key in ['capacity_claim', 'footprint_is_peak', 'product_runtime_changed',
                'product_owner_or_credential_limits_changed', 'production_crypto_adopted',
                'encrypted_keystore_recovery_implemented', 'encrypted_attachments_implemented',
                'native_membership_binding_implemented', 'independent_client_interop']:
        assert d[key] is False
    assert d['product_default_sessions'] == 8 and d['host_envelope_tests'] == 3
    assert d['teardown'] == {'services_stopped': True, 'vm_process_stopped': True, 'private_inputs_removed': True}
    overrides = d['source_overrides']
    assert set(overrides) == {'experiments/084-mail-proof/sce.py', 'tools/test_mail_envelope.py'}
    for name, expected in d['execution_sources'].items():
        assert digest(overrides.get(name, name)) == expected, 'actual mail proof input changed: '+name
    post = d['post_proof_format_hardening']
    assert post['host_tests'] == 3 and post['real_network_replayed_after_hardening'] is False
    for name, expected in post['sources'].items():
        assert digest(name) == expected, 'post-proof format input changed: '+name
    deps = json.loads((ROOT/'experiments/084-mail-proof/dependencies.json').read_text())
    assert deps['count'] == len(deps['packages']) == 21
    assert deps['build_count'] == len(deps['build_packages']) == 6
    assert deps['no_js_ts_selected_filenames'] and deps['slixmpp_jid'] == 'upstream Python fallback'
    assert deps['product_dependency'] is False and deps['platform_artifact_lock'] is False
    assert not {'slixmpp-omemo', 'pyo3'} & {p['name'] for p in deps['packages']}
    assert all(len(p['sha256']) == 64 and p['license'] for p in deps['packages']+deps['build_packages'])
    print('Mail research record passed: 12 mail + 11 native verifier controls; 4 signing failures retained, no native product/adoption claim.')


if __name__ == '__main__': check()
