import copy
import unittest
from memory_profile import summarize,interval,headroom


class MemoryProfileTests(unittest.TestCase):
    def sample(self):
        return {'vm':{'page_bytes':4096,'total_pages':262144,'free_pages':65536,'inactive_pages':0,'file_clean_pages':0,'wired_pages':0,'anon_pages':100,'swap_used_pages':0},
                'cpu_ticks':[10,0,10,0,80],'cpu_idle_index':4,
                'limits':{'processes':10,'max_processes':1024,'open_files':50,'max_files':3404},
                'processes':[{'role':'member','rss_kib':1024,'vsz_bytes':8192,'high_water_rss_kib':2048,'cpu_us':100000}]}
    def test_rss_accounting_and_page_units_are_explicit(self):
        s=self.sample();s['processes'].append(dict(s['processes'][0],role='ssh'))
        out=summarize(s);self.assertEqual(out['all_roles_rss_sum_bytes'],2*1024*1024)
        self.assertEqual(out['vm_bytes']['anon'],409600)
        self.assertNotIn('pss',out);self.assertNotIn('uss',out)
    def test_cumulative_cpu_uses_interval_not_ps_rounding(self):
        a=self.sample();b=copy.deepcopy(a);b['cpu_ticks']=[20,0,20,0,160];b['processes'][0]['cpu_us']=200000
        out=interval(a,b,2);self.assertEqual(out['whole_vm_cpu_percent_one_core'],40)
        self.assertEqual(out['stable_roles_cpu_percent_one_core']['member'],5)
        with self.assertRaises(ValueError):interval(a,a,2)
    def test_headroom_and_sensitive_fields_fail_closed(self):
        s=self.sample();self.assertTrue(headroom(s));s['vm']['free_pages']=1;s['vm']['inactive_pages']=200000;self.assertFalse(headroom(s))
        s['vm']['file_clean_pages']=65536;self.assertTrue(headroom(s))
        s['vm']['swap_used_pages']=1;self.assertFalse(headroom(s))
        s['processes'][0]['pid']=123
        with self.assertRaises(ValueError):summarize(s)
    def test_churn_does_not_generate_negative_cpu_or_missing_memory(self):
        a=self.sample();b=copy.deepcopy(a);b['cpu_ticks']=[20,0,20,0,160];b['processes']=[]
        out=interval(a,b,1);self.assertEqual(out['stable_roles_cpu_percent_one_core'],{})
        b['cpu_ticks']=[0,0,0,0,0]
        with self.assertRaises(ValueError):interval(a,b,1)
