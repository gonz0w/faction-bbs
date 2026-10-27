"""Reject incomplete/weakened saved lookup/authority/whole-host claims."""
import copy,json,unittest
from check_lookup_evidence import ROOT,validate
from lookup_trace import summarize

class LookupEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.record=json.loads((ROOT/'dev-docs/research/results/039-netbsd-bounded-lookups.json').read_text())
    def mutation(self,edit):
        d=copy.deepcopy(self.record);edit(d)
        with self.assertRaises(AssertionError):validate(d)
    def test_saved_record(self):self.assertEqual(len(validate(self.record)),2)
    def test_four_cannot_become_unlimited(self):self.mutation(lambda d:d['contract'].__setitem__('lookup_allowance',64))
    def test_unsigned_extra_reply_is_rejected(self):self.mutation(lambda d:d['cohorts']['known']['probe']['rows'][0].insert(4,{'response':'key_ok','seconds':.1}))
    def test_trace_execs_cannot_be_omitted(self):self.mutation(lambda d:d['cohorts']['known']['trace'].__setitem__('helper_executions',32))
    def test_exhausted_writes_cannot_be_hidden(self):self.mutation(lambda d:d['cohorts']['known']['trace']['classes'][0].__setitem__('fsync',1))
    def test_signature_failure_cannot_authenticate(self):self.mutation(lambda d:d['cohorts']['bad_signature']['probe'].__setitem__('authenticated',True))
    def test_key_options_must_still_deny(self):self.mutation(lambda d:d['validation']['options']['options_rows'][0].__setitem__('response','key_ok'))
    def test_migration_cannot_be_automatic(self):self.mutation(lambda d:d['validation']['state']['migration'].__setitem__('implicit',True))
    def test_current_sources_cannot_drift(self):self.mutation(lambda d:d['execution_sources'].__setitem__('native/faction-key-admission/main.c','0'*64))
    def test_guard_cannot_drop(self):
        def edit(d):
            stream=d['trials'][0]['measurement']['lookup_activity'][0]['native_stream']
            for k in ['vm_bytes.free','vm_bytes.file_clean']:stream['sample_rows'][0][stream['sample_columns'].index(k)]=0
        self.mutation(edit)
    def test_listener_restart_cannot_refresh_allowance(self):self.mutation(lambda d:d['validation']['restart']['after'].__setitem__('reuses',4))
    def test_fairness_denial_cannot_disappear(self):self.mutation(lambda d:d['trials'][0]['measurement']['lookup_activity'][0]['same_source_attempts'][0].__setitem__('accepted',True))
    def test_recovery_period_cannot_be_labelled_250(self):
        def edit(d):
            for r in d['trials'][0]['measurement']['lookup_activity'][0]['rounds']:r['members']=250;r['confirmed_peer_receipts']=249
        self.mutation(edit)
    def test_listener_reset_cannot_be_fairness(self):self.mutation(lambda d:d['trials'][0]['measurement']['lookup_activity'][0].__setitem__('listener_reset',True))
    def test_cleanup_is_required(self):self.mutation(lambda d:d['cleanup'].__setitem__('private_inputs_removed',False))
    def test_incomplete_trace_is_rejected(self):
        with self.assertRaises(AssertionError):summarize(' 123 123 faction-key-admi CALL fsync(4)\n')
if __name__=='__main__':unittest.main()
