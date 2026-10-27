"""Saved observations cannot turn rejected/modified candidates into qualification."""
import copy
import hashlib
import unittest
from pathlib import Path
from check_omemo_interop_evidence import validate


def fixture():
    names=['independent_ed_curve_sign_0_vector_and_no_implicit_migration',
           'independent_ed_curve_sign_1_vector_and_no_implicit_migration']
    for side in ['native','reference']:
        names += [side+'_initiates_first_and_reciprocal_reply_sce',
                  side+'_native_restart_accepts_out_of_order_2_0_1',
                  side+'_reference_retains_late_key_across_reply_ratchet_and_restart',
                  side+'_native_retains_late_key_across_reply_ratchet_and_restart',
                  side+'_native_sce_sender_aborts_state',side+'_reference_sce_recipient_aborts_state',
                  side+'_native_skip_cap_rejects_without_state_change',side+'_valid_low_counter_after_skip_cap']
    matrix=dict(result='pass',checks=names,count=len(names),independent_two_way=True)
    baseline=dict(result='rejected',checks=['native_first_sce_decrypts_but_reply_authentication_rejects_without_commit',
                                         'reference_first_sce_decrypts_but_reply_authentication_rejects_without_commit'],count=2)
    return dict(schema=1,result='qualified-corrected-candidate-only',issues=[200,202],product_adoption=False,reference_modified=False,
                profile=dict(namespace='urn:xmpp:omemo:2',sce_namespace='urn:xmpp:sce:1'),
                baseline=dict(host=copy.deepcopy(baseline),netbsd=copy.deepcopy(baseline)),
                corrected=dict(host=copy.deepcopy(matrix),netbsd=copy.deepcopy(matrix)),
                native=dict(system='NetBSD',architecture='aarch64',cores=2,ram_mib=1024),
                candidate=dict(revision='835d8277108b96c691fb6a992512f37315cb52d7',
                               patch_sha256=hashlib.sha256(b'patch').hexdigest(),profile='ad-order-v1'),
                execution_sources={'probe.rs':hashlib.sha256(b'executed').hexdigest(),
                                   'experiments/200-omemo-interop/picomemo-ad-order.patch':hashlib.sha256(b'patch').hexdigest()},
                frozen_sources={'frozen/baseline.rs':hashlib.sha256(b'baseline').hexdigest()},
                rust_tests=dict(baseline_host=17,baseline_netbsd=16,corrected_host=17,corrected_netbsd=17),
                teardown=dict(owned_vm_removed=True,owned_keys_removed=True,ports_closed=True,
                              saved_playground_stopped=True,saved_playground_unchanged=True))


def resolve(name,digest):
    return b'baseline' if name.startswith('frozen/') else b'patch' if name.endswith('.patch') else b'executed'


class EvidenceContracts(unittest.TestCase):
    def test_complete_bounded_qualification(self):validate(fixture(),resolve)

    def test_saved_record_cannot_be_rewritten_even_with_valid_json(self):
        from check_omemo_interop_evidence import load_record
        path=Path(__file__).resolve().parents[1]/'dev-docs/evidence/omemo-interop/results.json'
        raw=path.read_bytes()
        self.assertEqual(load_record(raw)['result'],'qualified-corrected-candidate-only')
        with self.assertRaises(AssertionError):load_record(raw+b' ')
        altered=raw.replace(b'"ram_mib": 1024',b'"ram_mib": 2048')
        self.assertNotEqual(altered,raw)
        with self.assertRaises(AssertionError):load_record(altered)

    def test_required_workflow_checks_saved_qualification(self):
        path=Path(__file__).resolve().parents[1]/'.github/workflows/documentation.yml'
        self.assertIn('python3 tools/check_omemo_interop_evidence.py',path.read_text())

    def test_report_cannot_promote_product_status_or_substitute_patch(self):
        for name,mutate in [('product',lambda d:d.update(result='supported-product')),
                            ('patch',lambda d:d['candidate'].update(patch_sha256='a'*64))]:
            d=fixture();mutate(d)
            with self.subTest(name=name),self.assertRaises(AssertionError):validate(d,resolve)

    def test_failed_or_reference_modified_candidate_never_qualifies(self):
        for modify in [lambda d:d['baseline']['host'].update(result='pass'),
                       lambda d:d.update(reference_modified=True),
                       lambda d:d.update(product_adoption=True),
                       lambda d:d['corrected']['netbsd'].update(result='fail'),
                       lambda d:d['corrected']['host'].update(independent_two_way=False)]:
            data=fixture();modify(data)
            with self.assertRaises(AssertionError):validate(data,resolve)

    def test_missing_reciprocal_or_restart_and_false_count_reject(self):
        for name in ['native_initiates_first_and_reciprocal_reply_sce',
                     'reference_native_retains_late_key_across_reply_ratchet_and_restart']:
            d=fixture();d['corrected']['netbsd']['checks'].remove(name)
            d['corrected']['netbsd']['count']-=1
            with self.assertRaises(AssertionError):validate(d,resolve)
        d=fixture();d['corrected']['host']['count']+=1
        with self.assertRaises(AssertionError):validate(d,resolve)

    def test_source_tamper_cleanup_and_profile_fail_closed(self):
        d=fixture()
        with self.assertRaises(AssertionError):validate(d,lambda n,h:b'tampered')
        for modify in [lambda d:d['teardown'].update(owned_keys_removed=False),
                       lambda d:d['profile'].update(namespace='eu.siacs.conversations.axolotl'),
                       lambda d:d['candidate'].update(profile='baseline'),
                       lambda d:d['native'].update(system='Linux')]:
            d=fixture();modify(d)
            with self.assertRaises(AssertionError):validate(d,resolve)


if __name__=='__main__':unittest.main()
