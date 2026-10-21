"""Saved security/capacity evidence must reject weakened or incomplete assertions."""
import copy,json,unittest
from check_integrated_evidence import ROOT
from check_admission_evidence import validate

class AdmissionEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.record=json.loads((ROOT/'dev-docs/research/results/037-netbsd-admission.json').read_text())
    def test_saved_record_is_valid(self):
        self.assertEqual(len(validate(self.record)),3)
    def test_admission_or_channel_denial_cannot_be_dropped(self):
        for key in ['admission','channels']:
            d=copy.deepcopy(self.record);d['validation'][key]['results'].pop()
            with self.assertRaises(AssertionError):validate(d)
    def test_full_key_probe_cannot_create_another_reservation(self):
        d=copy.deepcopy(self.record);d['trials'][0]['measurement']['at_capacity_probe'][0]['after']['launches']+=1
        with self.assertRaises(AssertionError):validate(d)
    def test_zombies_and_listener_costs_cannot_disappear(self):
        d=copy.deepcopy(self.record);d['trials'][0]['measurement']['live_process_limits'][0]['readback']['ssh_zombie_processes']=1
        with self.assertRaises(AssertionError):validate(d)
        d=copy.deepcopy(self.record);rows=d['trials'][0]['measurement']['live_process_limits'][0]['readback']['process_rows'];rows[:]=[r for r in rows if r[0]!='ssh_listener']
        with self.assertRaises(AssertionError):validate(d)
    def test_rejected_memory_failure_cannot_be_relabelled(self):
        d=copy.deepcopy(self.record);d['rejected_inetd_candidate']['trials'][1]['minimum_bytes']=128*1048576
        with self.assertRaises(AssertionError):validate(d)
    def test_launch_source_drift_is_rejected(self):
        d=copy.deepcopy(self.record);d['execution_sources']['native/faction-key-admission/main.c']='0'*64
        with self.assertRaisesRegex(AssertionError,'execution drift'):validate(d)
    def test_host_profile_must_be_restored(self):
        d=copy.deepcopy(self.record);d['restored_isolation']['frontend_setuid_removed']=False
        with self.assertRaises(AssertionError):validate(d)
    def test_pressure_guard_cannot_be_weakened(self):
        d=copy.deepcopy(self.record);d['validation']['preauthentication_pressure']['samples'][0]['vm_bytes']['free']=0;d['validation']['preauthentication_pressure']['samples'][0]['vm_bytes']['file_clean']=0
        with self.assertRaises(AssertionError):validate(d)

if __name__=='__main__':unittest.main()
