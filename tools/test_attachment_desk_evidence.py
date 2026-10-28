"""Pinned attachment-desk observations; no fresh runtime qualification."""
from pathlib import Path
import unittest
from check_recorded_files import verify

class Evidence(unittest.TestCase):
    def test_exact_final_source_result_and_selected_folder_membership(self):
        root = Path(__file__).resolve().parents[1]
        self.assertEqual(verify(root, 'dev-docs/evidence/attachment-desk/manifest.json',
                               '6dfdf450781f0651858e3673b1d7e38531cd26ad40a9cbe0422648f1f963f626'), 27)

if __name__ == '__main__': unittest.main()
