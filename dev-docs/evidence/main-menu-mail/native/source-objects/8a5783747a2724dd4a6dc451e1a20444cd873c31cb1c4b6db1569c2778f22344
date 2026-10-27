import copy
import unittest
from credential_peak import ledger,allocation_window


def events():
    metric={'vm':{'page_bytes':4096,'anon_pages':100,'free_pages':1000},'limits':{},'processes':[]}
    return [{'event':'start','monotonic_ns':1,'interval_ms':10,'count':5}]+[
        {'event':'sample','index':i,'monotonic_ns':10_000_000*(i+1),'finished_ns':10_000_000*(i+1)+1000,'metrics':copy.deepcopy(metric)} for i in range(5)
    ]+[{'event':'stop','monotonic_ns':60_000_000}]


class CredentialPeakTests(unittest.TestCase):
    def test_monotonic_cadence_is_guest_time(self):
        data=ledger(events())
        self.assertEqual(data['sample_count'],5)
        self.assertEqual(data['gap_ms'],{'min':10,'median':10,'max':10})
        self.assertEqual(data['collector_query_ms']['max'],.001)

    def test_incomplete_out_of_order_or_backwards_stream_denied(self):
        for modify in [lambda x:x.pop(2),lambda x:x[2].update(index=7),
                       lambda x:x[3].update(monotonic_ns=2),lambda x:x[-1].update(event='start'),
                       lambda x:x[1].update(finished_ns=1)]:
            data=events();modify(data)
            with self.assertRaises(ValueError):ledger(data)

    def test_process_identity_injection_denied(self):
        data=events();data[1]['metrics']['processes']=[{'role':'member','handle':'private'}]
        with self.assertRaises(ValueError):ledger(data)

    def test_allocation_uses_fully_inside_samples_and_prior_baseline(self):
        data=events();data[4]['metrics']['vm']['anon_pages']+=4864
        run=ledger(data)
        window=allocation_window(run,{'bytes':19*1024*1024,'began_ns':35_000_000,'touched_ns':36_000_000,'released_ns':49_000_000})
        self.assertEqual(window['fully_inside_samples'],1)
        self.assertEqual(window['observed_anon_delta_bytes'],19*1024*1024)
        with self.assertRaises(ValueError):allocation_window(run,{'bytes':1,'began_ns':1,'touched_ns':1,'released_ns':2})
