"""Check saved offline-preparation evidence bytes without runtime execution."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class OfflineProviderPreparationEvidence(unittest.TestCase):
    def test_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/offline-provider-preparation/manifest.json',
            'd3b14eeec31d1eff5bac8da9217eb02ef94799988c5adacb787fcfb784aae3a5'), 860)


if __name__ == '__main__':
    unittest.main()
