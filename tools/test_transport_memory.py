import unittest
from transport_memory import guard_decision,admission_batches


def sample(free=32768,clean=0,wired=0,swap=0,processes=10,files=10):
    return {'vm':{'page_bytes':4096,'free_pages':free,'file_clean_pages':clean,
                  'wired_pages':wired,'swap_used_pages':swap},
            'limits':{'processes':processes,'max_processes':2048,
                      'open_files':files,'max_files':16384},'processes':[]}


class TransportGuard(unittest.TestCase):
    def test_exact_threshold_and_wired_subtraction(self):
        self.assertTrue(guard_decision(sample(),0,'before_admission_batch')['passed'])
        self.assertFalse(guard_decision(sample(free=32767),0,'before_admission_batch')['passed'])
        d=guard_decision(sample(free=32000,clean=800,wired=32),8,'before_admission_batch')
        self.assertTrue(d['passed'])
        self.assertEqual(d['reclaimable_headroom_bytes'],128*1024*1024)
        self.assertEqual(d['snapshot']['vm_bytes']['wired'],32*4096)

    def test_swap_process_and_file_reserves_cannot_be_hidden(self):
        for s in [sample(swap=1),sample(processes=1984),sample(files=16256)]:
            self.assertFalse(guard_decision(s,8,'before_admission_batch')['passed'])

    def test_batch_overshoot_is_saved_at_actual_decision(self):
        a=guard_decision(sample(free=34000),216,'before_admission_batch')
        b=guard_decision(sample(free=31000),224,'before_admission_batch')
        c=guard_decision(sample(),222,'before_replacement_credentials')
        row=admission_batches([a,b,c])[0]
        self.assertEqual(row['admitted_in_batch'],8)
        self.assertEqual(row['after_shortfall_bytes'],1768*4096)
        self.assertFalse(row['after_passed'])
        self.assertEqual(row['headroom_drop_bytes'],3000*4096)
