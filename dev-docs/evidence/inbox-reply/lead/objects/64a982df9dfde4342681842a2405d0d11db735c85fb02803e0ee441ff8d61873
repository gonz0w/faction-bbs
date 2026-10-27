#!/usr/bin/env python3
"""Corruption controls for saved native active/abuse claims."""
import copy
import json
import unittest
from check_sustained_evidence import ROOT, validate


class SustainedEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.record=json.loads((ROOT/'dev-docs/research/results/038-netbsd-sustained-abuse.json').read_text())

    def reject(self, mutate):
        d=copy.deepcopy(self.record);mutate(d)
        with self.assertRaises((AssertionError,ValueError)):
            validate(d)

    def test_complete_record(self):
        self.assertEqual(len(validate(self.record)),2)

    def test_sender_echo_cannot_be_counted_as_peer_receipt(self):
        self.reject(lambda d:d['delivery_counting'].update(non_sender_marker_receipts=425500,sender_echo_distinguished=True))

    def test_missing_recipient(self):
        self.reject(lambda d:d['trials'][0]['measurement']['sustained_activity'][0]['rounds'][0].update(recipients=249))

    def test_shortened_interval(self):
        self.reject(lambda d:d['trials'][0]['measurement']['sustained_activity'][0].update(active_seconds=100))

    def test_lookup_work_cannot_be_called_single_query(self):
        self.reject(lambda d:d['unsigned_key_probe']['during'].update(reuses=d['unsigned_key_probe']['before']['reuses']))

    def test_native_penalty_cannot_be_hidden(self):
        self.reject(lambda d:d['source_fairness']['same_source_attempts'][0].update(accepted=True))

    def test_restart_cannot_manufacture_recovery(self):
        self.reject(lambda d:d['source_fairness'].update(listener_reset=True))

    def test_defaults_and_guard_cannot_be_loosened(self):
        self.reject(lambda d:d['contract'].update(default_reservations=250))
        self.reject(lambda d:d['contract'].update(headroom_minimum_bytes=1))

    def test_source_drift(self):
        self.reject(lambda d:d['execution_sources'].update({'tools/check_sustained_scaling.py':'0'*64}))

    def test_private_vm_cleanup_required(self):
        self.reject(lambda d:d['cleanup'].update(private_inputs_removed=False))

    def test_failed_guard_cannot_pass(self):
        def corrupt(d):
            s=d['trials'][0]['measurement']['sustained_activity'][0]['native_stream']
            i=s['sample_columns'].index('vm_bytes.free')
            j=s['sample_columns'].index('vm_bytes.file_clean')
            s['sample_rows'][0][i]=0;s['sample_rows'][0][j]=0
        self.reject(corrupt)

    def test_missing_pressure(self):
        self.reject(lambda d:d['trials'][0]['measurement']['sustained_activity'][0]['abuse'].update(silent_accepted=0))


if __name__=='__main__':unittest.main()
