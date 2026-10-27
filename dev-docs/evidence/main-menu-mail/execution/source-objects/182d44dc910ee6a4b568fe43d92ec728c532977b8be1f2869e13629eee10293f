"""Checks fixture accounting/fallback; never asserts scheduler timing in CI."""
import copy
import json
import unittest
from measure_text_replay import RAW, SGR, SIZES, pages, validate_observations

class TextReplayTests(unittest.TestCase):
    def test_pages_fit_width_height_and_byte_caps(self):
        for cols, rows in SIZES:
            for case in ('menu', 'room', 'files'):
                for ansi in (False, True):
                    for page in pages(case, cols, rows, ansi):
                        self.assertLessEqual(len(page), 2048)
                        plain = SGR.sub(b'', page)
                        self.assertLessEqual(len(plain.split(b'\r\n')), rows)
                        self.assertTrue(all(len(line) < cols for line in plain.split(b'\r\n')))
    def test_fallback_preserves_authored_content(self):
        for cols, rows in SIZES:
            for case in ('menu', 'room', 'files'):
                self.assertEqual(pages(case, cols, rows, False),
                                 [SGR.sub(b'', p) for p in pages(case, cols, rows, True)])
    def test_long_content_continues_without_loss(self):
        for cols, rows in SIZES:
            output = b''.join(pages('room', cols, rows, False))
            for i in range(1, 9):
                self.assertEqual(output.count(f'#{i} / member{i}'.encode()), 1)
            self.assertGreater(len(pages('room', cols, rows, False)), 1)
    def test_rejects_missing_duplicate_or_corrupt_observations(self):
        raw = json.loads(RAW.read_text())
        validate_observations(raw)
        for mutation in ('missing', 'duplicate', 'bytes', 'timing'):
            changed = copy.deepcopy(raw)
            if mutation == 'missing':
                changed['cells'].pop()
            elif mutation == 'duplicate':
                changed['cells'].append(changed['cells'][0])
            elif mutation == 'bytes':
                changed['cells'][0]['samples'][0]['received_bytes'] += 1
            else:
                changed['cells'][0]['samples'][0]['elapsed_s'] = float('nan')
            with self.assertRaises(ValueError):
                validate_observations(changed)

    def test_invalid_fixture_size_and_case(self):
        for args in [('room', 19, 24, False), ('room', 80, 7, False), ('unknown',80,24,False)]:
            with self.assertRaises(ValueError):
                pages(*args)

if __name__ == '__main__':
    unittest.main()
