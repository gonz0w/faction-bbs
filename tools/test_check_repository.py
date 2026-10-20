"""Exercise the checker against realistic broken repository fixtures."""

from pathlib import Path
import json
import shutil
import tempfile
import unittest

from check_repository import ROOT, validate


class RepositoryChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='faction-check-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'repo'
        shutil.copytree(ROOT, self.root, ignore=shutil.ignore_patterns(
            '.git', '__pycache__', '.venv', 'target', '.DS_Store'))

    def errors(self):
        return '\n'.join(validate(self.root, allow_unpublished=True))

    def test_current_repository_is_valid(self):
        self.assertEqual(self.errors(), '')

    def test_broken_document_link_is_rejected(self):
        path = self.root / 'README.md'
        path.write_text(path.read_text() + '\n[Missing](docs/does-not-exist.md)\n')
        self.assertIn('broken local link', self.errors())

    def test_changed_historical_source_is_rejected(self):
        path = self.root / 'dev-docs/archive/seed/initial-seed-prd.md'
        path.write_text(path.read_text() + '\nUnexpected historical edit.\n')
        self.assertIn('Archive integrity mismatch', self.errors())

    def test_js_asset_and_action_are_rejected(self):
        (self.root / 'terminal.js').write_text('// prohibited fixture\n')
        path = self.root / '.github/workflows/documentation.yml'
        path.write_text(path.read_text() + '\n      - uses: actions/checkout@v7\n')
        errors = self.errors()
        self.assertIn('JavaScript/TypeScript', errors)
        self.assertIn('without external actions', errors)

    def test_duplicate_legacy_mapping_is_rejected(self):
        path = self.root / 'dev-docs/issue-map.json'
        data = json.loads(path.read_text())
        data[1]['legacy_id'] = data[0]['legacy_id']
        path.write_text(json.dumps(data) + '\n')
        self.assertIn('Duplicate legacy issue mapping', self.errors())

    def test_retired_planning_directory_is_rejected(self):
        (self.root / '.planning').mkdir()
        self.assertIn('Retired .planning directory', self.errors())


if __name__ == '__main__':
    unittest.main()
