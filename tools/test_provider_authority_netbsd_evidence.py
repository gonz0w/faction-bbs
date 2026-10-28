"""Verify frozen native observations; this does not rerun or attest NetBSD."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class ProviderNetBSDEvidence(unittest.TestCase):
    def test_native_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/provider-authority-netbsd/manifest.json',
            '17e13d9c172f41a2bb298ef4c5dfffe0f1b81fb57de996e1706edde35d62efba'), 47)


if __name__ == '__main__':
    unittest.main()
