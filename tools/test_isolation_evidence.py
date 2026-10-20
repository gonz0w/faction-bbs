"""Security denials and failed resource candidates must survive saved evidence checks."""
import copy,json,unittest
from check_integrated_evidence import ROOT
from check_isolation_evidence import validate

class IsolationEvidence(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.record=json.loads((ROOT/'dev-docs/research/results/036-netbsd-isolation.json').read_text())
    def test_boundary_and_failed_candidate_cannot_be_relabelled(self):
        for field in ['privilege_drop','filesystem','exec','tcp_udp_v4_v6','raw_socket_denied']:
            d=copy.deepcopy(self.record);d['validation']['boundary']['results'][1][field]=False
            with self.assertRaises(AssertionError):validate(d)
        d=copy.deepcopy(self.record);d['validation']['boundary']['blocked_packets_received']=1
        with self.assertRaises(AssertionError):validate(d)
        d=copy.deepcopy(self.record);d['validation']['boundary']['root_control_tcp_accepts']=3
        with self.assertRaises(AssertionError):validate(d)
        d=copy.deepcopy(self.record);d['rejected_named_sysctl_candidate']['trials'][0]['minimum_bytes']=128*1024*1024
        with self.assertRaises(AssertionError):validate(d)
    def test_native_input_change_is_current_source_failure(self):
        d=copy.deepcopy(self.record);d['execution_sources']['native/faction-isolation/startup.c']='0'*64
        with self.assertRaisesRegex(AssertionError,'execution drift'):validate(d)

if __name__=='__main__':unittest.main()
