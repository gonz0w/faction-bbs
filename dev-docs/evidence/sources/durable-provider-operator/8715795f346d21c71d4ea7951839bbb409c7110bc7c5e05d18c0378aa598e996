"""Verify frozen provider integration bytes, without rerunning or attesting a VM."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class ProviderIntegrationEvidence(unittest.TestCase):
    def test_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/provider-integration/manifest.json',
            'd6c62d5040c642bf14c34d46aa06526bba7f0d3d17f656e2457823a7733749bc'), 362)


if __name__ == '__main__':
    unittest.main()
