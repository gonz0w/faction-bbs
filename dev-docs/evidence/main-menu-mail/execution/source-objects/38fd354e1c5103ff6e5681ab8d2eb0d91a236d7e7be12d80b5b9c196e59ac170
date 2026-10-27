"""Saved evidence must reject missing samples and a falsified passed guard."""
import copy,json,unittest
from check_ui_evidence import ROOT,unpack,validate_trial

class SharedUIEvidence(unittest.TestCase):
    def test_incomplete_or_duplicate_sample_columns_are_rejected(self):
        stream={'sample_count':150,'sample_columns':['monotonic_ns','finished_ns'],
                'sample_rows':[[2*i,2*i+1] for i in range(150)],
                'markers':{'start_ns':0,'stop_ns':300}}
        self.assertEqual(len(unpack(stream)),150)
        bad=copy.deepcopy(stream);bad['sample_rows'].pop()
        with self.assertRaises(AssertionError):unpack(bad)
        bad=copy.deepcopy(stream);bad['sample_columns']=['monotonic_ns','monotonic_ns']
        with self.assertRaises(AssertionError):unpack(bad)

    def test_stress_guard_is_recomputed_from_counters(self):
        data=json.loads((ROOT/'dev-docs/research/results/033-netbsd-shared-ui.json').read_text())
        trial=copy.deepcopy(data['trials'][0]);stream=trial['measurement']['full_ui_at_250'][0]
        # Leave all recorded pass booleans intact, but remove memory headroom.
        for name in ['vm_bytes.free','vm_bytes.file_clean','vm_bytes.wired']:
            stream['sample_rows'][0][stream['sample_columns'].index(name)]=0
        with self.assertRaisesRegex(AssertionError,'unchanged stress guard failed'):
            validate_trial(trial,data['binaries']['shared_ui'])

if __name__=='__main__':unittest.main()
