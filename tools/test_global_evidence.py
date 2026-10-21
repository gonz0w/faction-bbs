"""Reject weaker global-fork, peer-identity, persistent-budget and memory claims."""
import copy,json,unittest
from check_global_evidence import ROOT,validate
class GlobalEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.record=json.loads((ROOT/'dev-docs/research/results/040-netbsd-global-admission.json').read_text())
    def mutation(self,edit):
        d=copy.deepcopy(self.record);edit(d)
        with self.assertRaises(AssertionError):validate(d)
    def test_saved_record(self):self.assertEqual(len(validate(self.record)),2)
    def test_extra_fork_is_rejected(self):self.mutation(lambda d:d['validation']['global']['observations']['global_burst'].__setitem__('listener_forks',9))
    def test_reset_is_not_reboot_persistence(self):self.mutation(lambda d:d['validation']['reboot']['after'].__setitem__('passed',0))
    def test_reboot_cannot_manufacture_a_burst(self):self.mutation(lambda d:d['validation']['reboot']['after'].__setitem__('budget_ns',2000000000))
    def test_member_mapping_is_rejected(self):self.mutation(lambda d:d['trials'][0]['measurement']['global_activity']['mapping'].__setitem__('leaked_processes',1))
    def test_source_spoof_cannot_pass(self):self.mutation(lambda d:d['validation']['private_peers']['datagram_source_controls'].__setitem__('forged',1))
    def test_penalties_cannot_be_weakened(self):self.mutation(lambda d:d['validation']['private_peers'].__setitem__('penalties_weakened',True))
    def test_tunnel_identity_cannot_replace_bbs_approval(self):self.mutation(lambda d:d['validation']['private_peers'].__setitem__('fresh_approved_bbs_login_during_attack',False))
    def test_restart_cannot_be_fairness(self):self.mutation(lambda d:d['validation']['private_peers'].__setitem__('listener_reset_during_attack',True))
    def test_only_sender_echo_is_rejected(self):self.mutation(lambda d:d['trials'][0]['measurement']['global_activity']['rounds'][0].__setitem__('confirmed_peer_receipts',0))
    def test_memory_guard_is_required(self):
        def edit(d):
            stream=d['trials'][0]['measurement']['global_activity']['native_stream']
            for k in ['vm_bytes.free','vm_bytes.file_clean']:stream['sample_rows'][0][stream['sample_columns'].index(k)]=0
        self.mutation(edit)
    def test_source_drift_is_rejected(self):self.mutation(lambda d:d['execution_sources'].__setitem__('native/faction-ssh-gate/gate.c','0'*64))
    def test_peer_vm_cleanup_is_required(self):self.mutation(lambda d:d['cleanup'].__setitem__('peer_guest_stopped',False))
if __name__=='__main__':unittest.main()
