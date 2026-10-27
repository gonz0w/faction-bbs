#!/usr/bin/env python3
"""Boundary and privacy checks for live headroom evidence."""
import copy
import unittest
from headroom_memory import live_limits,sample_guard,RESOURCES
from memory_profile import headroom,summarize


class HeadroomEvidence(unittest.TestCase):
    def test_reject_identifiers_and_inconsistent_limits(self):
        row={'role':'frontend','rss_bytes':4096,'virtual_bytes':8192,'limits':[[1,2] for _ in RESOURCES]}
        raw={'resources':RESOURCES,'processes':[row,copy.deepcopy(row)]}
        packed=live_limits(raw)
        self.assertEqual(len(packed['limit_profiles']),1)
        self.assertEqual(len(packed['process_rows']),2)
        for key in ['pid','command','address']:
            bad=copy.deepcopy(raw);bad['processes'][0][key]='private'
            with self.assertRaises(ValueError):live_limits(bad)
        for pair in [[3,2],['unlimited',2],[-1,2],[True,2]]:
            bad=copy.deepcopy(raw);bad['processes'][0]['limits'][0]=pair
            with self.assertRaises(ValueError):live_limits(bad)

    def test_predicate_matches_native_page_boundary_and_limit_reserves(self):
        for free in [32767,32768,32769]:
            for processes in [100,959,960]:
                for files in [100,2871,2872]:
                    for swap in [0,1]:
                        s={'vm':{'page_bytes':4096,'free_pages':free,'file_clean_pages':20,'wired_pages':20,'swap_used_pages':swap},
                           'limits':{'processes':processes,'max_processes':1024,'open_files':files,'max_files':3000},'processes':[]}
                        self.assertEqual(sample_guard(summarize(s))['passed'],headroom(s))


if __name__=='__main__':unittest.main()
