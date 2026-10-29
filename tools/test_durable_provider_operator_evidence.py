"""Check saved durable-provider/operator evidence bytes without runtime execution."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class DurableProviderOperatorEvidence(unittest.TestCase):
    def test_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/durable-provider-operator/manifest.json',
            'd80f0f0e46e95f92ced4741814aaca64321b4fb9cfb76f8eb988c72d097c0ad2'), 834)


if __name__ == '__main__':
    unittest.main()
