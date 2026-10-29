"""Check saved migration-contention evidence without database execution."""
from pathlib import Path
import unittest
from check_recorded_files import verify


class MigrationContentionEvidence(unittest.TestCase):
    def test_record_and_exact_selected_membership(self):
        self.assertEqual(verify(Path(__file__).resolve().parents[1],
            'dev-docs/evidence/migration-contention/manifest.json',
            'bdff244bc12ee5e273354a6c7589f59c7f510c9c98356f4597e2d6841316de64'), 6)


if __name__ == '__main__':
    unittest.main()
