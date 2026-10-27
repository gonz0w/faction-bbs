"""Acceptance checks for Faction's selected Codex routing and pinned bundle."""
import hashlib
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
ROUTES = {
    'temper_workflow': ('gpt-6-luna', 'medium'),
    'temper_implementer': ('gpt-6.1-sol', 'high'),
    'temper_analyst': ('gpt-6-astra', 'high'),
    'temper_reviewer': ('gpt-6.1-sol', 'high'),
}


class CodexRoutingContract(unittest.TestCase):
    def test_selected_policy_preserves_strict_test_first(self):
        policy = (ROOT / 'PROJECT.md').read_text()
        self.assertIn('[Codex](.agent-temper/profiles/codex.md)', policy)
        self.assertIn('[Strict test-first](.agent-temper/profiles/strict-test-first.md)', policy)
        self.assertIn('maximum of three concurrent delegated agents', policy)
        self.assertIn('No nested helpers without an explicit lead assignment', policy)
        self.assertIn('Never silently substitute a model', policy)
        entry = (ROOT / 'AGENTS.md').read_text()
        self.assertIn('Codex routing profile selected in PROJECT.md', entry)
        for role, (model, effort) in ROUTES.items():
            self.assertIn(role, policy)
            self.assertIn(model, policy)
            self.assertIn(role, entry)

    def test_roles_have_explicit_models_and_bounded_instructions(self):
        names = []
        for role, expected in ROUTES.items():
            with self.subTest(role=role):
                path = ROOT / '.codex' / 'agents' / (role + '.toml')
                self.assertFalse(path.is_symlink())
                data = tomllib.loads(path.read_text())
                self.assertEqual(data['name'], role)
                names.append(data['name'])
                self.assertEqual((data['model'], data['model_reasoning_effort']), expected)
                self.assertTrue(data['description'].strip())
                self.assertTrue(data['developer_instructions'].strip())
                self.assertNotIn('sandbox_mode', data)
                self.assertNotIn('approval_policy', data)
        self.assertEqual(len(names), len(set(names)))

    def test_bundle_manifest_covers_unchanged_upstream_files(self):
        bundle = ROOT / '.agent-temper'
        source = (bundle / 'SOURCE.md').read_text()
        self.assertRegex(source, r'Revision: `[0-9a-f]{40}`')
        self.assertTrue((bundle / 'profiles' / 'codex.md').is_file())
        entries = {}
        for line in (bundle / 'SHA256SUMS').read_text().splitlines():
            digest, name = line.split(maxsplit=1)
            name = name.lstrip('*')
            self.assertNotIn(name, entries)
            self.assertRegex(digest, r'^[0-9a-f]{64}$')
            self.assertNotIn('..', Path(name).parts)
            self.assertFalse(Path(name).is_absolute())
            entries[name] = digest
            self.assertEqual(hashlib.sha256((bundle / name).read_bytes()).hexdigest(), digest)
        actual = {str(p.relative_to(bundle)) for p in bundle.rglob('*') if p.is_file()}
        self.assertEqual(actual, set(entries) | {'SOURCE.md', 'SHA256SUMS'})


if __name__ == '__main__':
    unittest.main()
