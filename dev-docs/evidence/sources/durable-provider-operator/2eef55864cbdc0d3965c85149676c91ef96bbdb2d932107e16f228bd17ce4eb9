"""Check saved native-provider-client evidence bytes without executing a VM."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class NativeProviderClientEvidence(unittest.TestCase):
    def test_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/native-provider-client/manifest.json',
            '146830bb0ad56de5a232ee28f940726c996a88ca5163256dc263ffb1df853441'), 508)


if __name__ == '__main__':
    unittest.main()
