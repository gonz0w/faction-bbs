#!/usr/bin/env python3
"""Verify saved OMEMO qualification identities; never rerun crypto or a VM."""
import hashlib
import json
import re
from pathlib import Path
from evidence_sources import ROOT, source_bytes


def validate(d, resolve):
    assert d['schema']==1 and d['issues']==[200,202]
    assert d['result']=='qualified-corrected-candidate-only'
    assert not d['product_adoption'] and not d['reference_modified']
    assert d['profile']['namespace']=='urn:xmpp:omemo:2'
    assert d['profile']['sce_namespace']=='urn:xmpp:sce:1'
    assert d['candidate']['revision']=='835d8277108b96c691fb6a992512f37315cb52d7'
    assert d['candidate']['profile']=='ad-order-v1'
    assert re.fullmatch('[a-f0-9]{64}',d['candidate']['patch_sha256'])
    assert d['execution_sources']['experiments/200-omemo-interop/picomemo-ad-order.patch']==d['candidate']['patch_sha256']
    assert d['native']['system']=='NetBSD' and d['native']['architecture']=='aarch64'
    assert (d['native']['cores'],d['native']['ram_mib'])==(2,1024)
    required={'independent_ed_curve_sign_0_vector_and_no_implicit_migration',
              'independent_ed_curve_sign_1_vector_and_no_implicit_migration'}
    for side in ['native','reference']:
        required.update(side+suffix for suffix in [
            '_initiates_first_and_reciprocal_reply_sce',
            '_native_restart_accepts_out_of_order_2_0_1',
            '_reference_retains_late_key_across_reply_ratchet_and_restart',
            '_native_retains_late_key_across_reply_ratchet_and_restart',
            '_native_sce_sender_aborts_state','_reference_sce_recipient_aborts_state',
            '_native_skip_cap_rejects_without_state_change','_valid_low_counter_after_skip_cap'])
    for route in ['host','netbsd']:
        baseline=d['baseline'][route]
        assert baseline['result']=='rejected' and baseline['count']==len(set(baseline['checks']))==2
        assert set(baseline['checks'])=={side+'_first_sce_decrypts_but_reply_authentication_rejects_without_commit'
                                       for side in ['native','reference']}
        matrix=d['corrected'][route]
        assert matrix['result']=='pass' and matrix['independent_two_way']
        assert matrix['count']==len(matrix['checks'])==len(set(matrix['checks']))
        assert required<=set(matrix['checks'])
    for name,count in d['rust_tests'].items():
        assert isinstance(count,int) and count>=(17 if name.startswith('corrected') else 16)
    assert set(d['rust_tests'])=={'baseline_host','baseline_netbsd','corrected_host','corrected_netbsd'}
    assert set(d['teardown'])=={'owned_vm_removed','owned_keys_removed','ports_closed',
                              'saved_playground_stopped','saved_playground_unchanged'}
    assert all(value is True for value in d['teardown'].values())
    assert d['execution_sources'] and d['frozen_sources']
    for group in ['execution_sources','frozen_sources']:
        for name,digest in d[group].items():
            assert re.fullmatch('[a-f0-9]{64}',digest)
            assert hashlib.sha256(resolve(name,digest)).hexdigest()==digest,name


RECORD_SHA256='a5bc2a8f7077c29e7a5e4c2158da0a037f94ae325bcac081ae145720b21358bf'


def load_record(raw):
    assert hashlib.sha256(raw).hexdigest()==RECORD_SHA256,'saved qualification record changed'
    return json.loads(raw)


def check():
    path=ROOT/'dev-docs/evidence/omemo-interop/results.json'
    d=load_record(path.read_bytes())
    validate(d,source_bytes)
    patch='experiments/200-omemo-interop/picomemo-ad-order.patch'
    assert d['execution_sources'][patch]==d['candidate']['patch_sha256']
    print('OMEMO qualification evidence passed: rejected baseline, independent corrected host/NetBSD replies, restart/trust/error controls and frozen source identities; no product adoption claim.')


if __name__=='__main__':check()
