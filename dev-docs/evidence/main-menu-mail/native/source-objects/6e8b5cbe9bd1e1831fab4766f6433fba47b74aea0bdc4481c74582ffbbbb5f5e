"""Check byte/bit arithmetic and prevent analytical output from becoming a benchmark claim."""
import json
import unittest

from model_link_budget import FIXTURE, REPORT, modeled_seconds, payload_totals, render, stream_load


class LinkBudgetModel(unittest.TestCase):
    def test_bits_not_bytes_and_binary_file_units(self):
        profile = {'up_bps': 64000, 'down_bps': 64000, 'rtt_ms': 400}
        self.assertAlmostEqual(modeled_seconds(0, 65536, 1, profile), 8.592)

    def test_asymmetric_directions_and_sequential_round_trips(self):
        profile = {'up_bps': 8000, 'down_bps': 16000, 'rtt_ms': 100}
        self.assertAlmostEqual(modeled_seconds(1000, 2000, 3, profile), 2.3)

    def test_control_bytes_and_crlf_are_counted_as_transmitted(self):
        work = {'exchanges': [{'client': 'x\r\n', 'server': '\x1b[2J\r\n',
                              'client_payload_bytes': 0, 'server_payload_bytes': 4}]}
        self.assertEqual(payload_totals(work), (3, 10, 1))

    def test_sized_blocks_are_not_hidden_by_fixture_text(self):
        work = {'exchanges': [{'client': '', 'server': 'OK',
                              'client_payload_bytes': 512, 'server_payload_bytes': 65536}]}
        self.assertEqual(payload_totals(work), (512, 65538, 1))

    def test_invalid_capacity_and_payload_are_rejected(self):
        with self.assertRaises(ValueError):
            modeled_seconds(0, 1, 1, {'up_bps': 1, 'down_bps': 0, 'rtt_ms': 1})
        with self.assertRaises(ValueError):
            modeled_seconds(-1, 1, 1, {'up_bps': 1, 'down_bps': 1, 'rtt_ms': 1})
        with self.assertRaises(ValueError):
            modeled_seconds(0, 1, True, {'up_bps': 1, 'down_bps': 1, 'rtt_ms': 1})

    def test_chat_arrival_load_can_exceed_capacity(self):
        bits, ratio = stream_load({'record_bytes': 256, 'records_per_second': 32}, {'down_bps': 64000})
        self.assertEqual(bits, 65536)
        self.assertGreater(ratio, 1)

    def test_impairments_are_not_silently_modeled_as_runtime_results(self):
        data = json.loads(FIXTURE.read_bytes())
        p2, p3 = data['profiles'][1:]
        self.assertEqual(modeled_seconds(3, 1000, 2, p2), modeled_seconds(3, 1000, 2, p3))
        output = render(FIXTURE.read_bytes())
        self.assertIn('outage behavior is not modeled', output)
        self.assertIn('not measured BBS/network performance', output)

    def test_checked_in_report_is_reproducible(self):
        self.assertEqual(REPORT.read_text(), render(FIXTURE.read_bytes()))


if __name__ == '__main__':
    unittest.main()
