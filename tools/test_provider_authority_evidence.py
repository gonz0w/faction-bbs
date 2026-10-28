"""Frozen local TLS observations; no provider or native execution is implied."""
from pathlib import Path
import unittest
from check_recorded_files import verify

class ProviderEvidence(unittest.TestCase):
    def test_exact_observations_and_selected_folder(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/provider-authority/manifest.json', '103ceb0b5f34b6841dc3a0fd1fd9ca9840168de16c6c456ecacd13a060593580'), 40)

if __name__ == '__main__': unittest.main()
