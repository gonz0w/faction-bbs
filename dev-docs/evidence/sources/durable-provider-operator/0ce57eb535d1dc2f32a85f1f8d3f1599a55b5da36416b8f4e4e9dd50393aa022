"""Frozen native qualification integrity, not a new VM execution."""
from pathlib import Path
import unittest
from check_recorded_files import verify

class NativeEvidence(unittest.TestCase):
    def test_exact_native_observations_and_selected_folder(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/netbsd-attachments/manifest.json', '1183796039d65c60828d109bb3058a2192df16d4e233dc8e1b74c778cbe82bbc'), 35)

if __name__ == '__main__': unittest.main()
