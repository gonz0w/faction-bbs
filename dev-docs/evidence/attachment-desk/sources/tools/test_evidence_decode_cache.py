"""Verified manifest decoding may be reused privately, never file integrity checks."""
import hashlib
import inspect
import json
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest.mock import patch

import evidence_sources as evidence
from test_evidence_sources import isolated_history


class VerifiedManifestDecodeCache(unittest.TestCase):
    def setUp(self):
        decoder = getattr(evidence, '_decode_verified_manifest', None)
        if decoder is not None:
            decoder.cache_clear()
        self.temporary = tempfile.TemporaryDirectory(prefix='faction216-decode-cache-OWNED-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.base = self.root / 'dev-docs/evidence'
        self.base.mkdir(parents=True)
        self.historical = self.base / 'sources/cache-fixture/same.rs'
        self.historical.parent.mkdir(parents=True)
        self.historical.write_bytes(b'exact synthetic historical source')
        self.digest = self.sha(self.historical.read_bytes())
        (self.root / 'same.rs').write_bytes(b'current synthetic source')
        self.data = {'files': {'same.rs': self.digest}, 'source_root': 'sources/cache-fixture', 'test_case': self.id()}
        self.manifest_path = self.base / 'manifest.json'
        self.raw = json.dumps(self.data).encode()
        self.manifest_path.write_bytes(self.raw)
        self.pin = self.sha(self.raw)

    @staticmethod
    def sha(raw):
        return hashlib.sha256(raw).hexdigest()

    def lookup(self):
        return evidence.source_bytes('same.rs', self.digest, self.root)

    def test_repeated_lookup_decodes_once_but_reads_manifest_and_source_every_time(self):
        original_read = Path.read_bytes
        counts = Counter()
        def read(path):
            counts[path] += 1
            return original_read(path)
        original_loads = json.loads
        with isolated_history(self.pin), patch.object(Path, 'read_bytes', read), patch.object(evidence.json, 'loads', wraps=original_loads) as loads:
            self.assertEqual(self.lookup(), b'exact synthetic historical source')
            self.assertEqual(self.lookup(), b'exact synthetic historical source')
            self.assertEqual(loads.call_count, 1)
        self.assertEqual(counts[self.manifest_path], 2)
        self.assertEqual(counts[self.historical], 2)

    def test_public_results_are_fresh_and_mutations_cannot_poison_resolution(self):
        original_loads = json.loads
        with isolated_history(self.pin), patch.object(evidence.json, 'loads', wraps=original_loads) as loads:
            self.lookup(); self.lookup()
            first = evidence.manifest(self.root)
            first['files'].clear()
            first['snapshots'] = [{'files': {'same.rs': self.digest}, 'source_paths': {'same.rs': '../outside'}}]
            second = evidence.manifest(self.root)
            self.assertIsNot(first, second)
            self.assertEqual(second, self.data)
            self.assertEqual(self.lookup(), b'exact synthetic historical source')
            self.assertEqual(loads.call_count, 3)

    def test_manifest_tamper_after_population_fails_and_context_is_restored(self):
        with isolated_history(self.pin):
            self.lookup()
            self.manifest_path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError, 'frozen manifest changed'):
                self.lookup()
            self.manifest_path.write_bytes(self.raw)
            first = evidence.manifest(self.root)
            second = evidence.manifest(self.root)
            self.assertIsNot(first, second)
            first['files'].clear()
            self.assertEqual(second, self.data)
            self.assertEqual(self.lookup(), b'exact synthetic historical source')

    def test_unused_later_manifest_tamper_is_not_skipped_after_cache_population(self):
        original_key = evidence.key_manifest
        key_raw = (evidence.ROOT / 'dev-docs/evidence/key-lookup-manifest.json').read_bytes()
        key_path = self.base / 'key-lookup-manifest.json'
        key_path.write_bytes(key_raw)
        with isolated_history(self.pin), patch.object(evidence, 'key_manifest', side_effect=original_key):
            self.assertEqual(self.lookup(), b'exact synthetic historical source')
            key_path.write_bytes(b'{}')
            with self.assertRaisesRegex(AssertionError, 'frozen key manifest changed'):
                self.lookup()
            key_path.write_bytes(key_raw)
            self.assertIsNot(evidence.manifest(self.root), evidence.manifest(self.root))
            self.assertEqual(self.lookup(), b'exact synthetic historical source')

    def test_changed_historical_and_current_source_bytes_are_not_cached(self):
        with isolated_history(self.pin):
            self.lookup()
            self.historical.write_bytes(b'altered historical source')
            with self.assertRaisesRegex(AssertionError, 'executed source bytes changed'):
                self.lookup()
            current = self.root / 'same.rs'
            digest = self.sha(current.read_bytes())
            self.assertEqual(evidence.source_bytes('same.rs', digest, self.root), b'current synthetic source')
            current.write_bytes(b'altered current source')
            with self.assertRaisesRegex(AssertionError, 'executed source bytes changed'):
                evidence.source_bytes('same.rs', digest, self.root)

    def test_decoder_exception_does_not_change_public_call_context(self):
        malformed = b'{not valid json'
        self.manifest_path.write_bytes(malformed)
        with isolated_history(self.sha(malformed)):
            with self.assertRaises(json.JSONDecodeError):
                self.lookup()
        self.manifest_path.write_bytes(self.raw)
        with isolated_history(self.pin):
            self.assertIsNot(evidence.manifest(self.root), evidence.manifest(self.root))
            self.assertEqual(self.lookup(), b'exact synthetic historical source')

    def test_cache_holds_exactly_128_distinct_verified_manifest_decodings(self):
        original_loads = json.loads
        with isolated_history(self.pin), patch.object(evidence.json, 'loads', wraps=original_loads) as loads:
            def lookup_case(case):
                raw = json.dumps(dict(self.data, cache_case=case)).encode()
                self.manifest_path.write_bytes(raw)
                with patch.object(evidence, 'MANIFEST_SHA256', self.sha(raw)):
                    self.lookup()
            for case in range(128):
                lookup_case(case)
            lookup_case(0)
            self.assertEqual(loads.call_count, 128)
            lookup_case(128)
            self.assertEqual(loads.call_count, 129)
            lookup_case(1)
            self.assertEqual(loads.call_count, 130)

    def test_all_57_pinned_helpers_still_read_and_hash_on_every_lookup(self):
        helpers = [(name, function) for name, function in inspect.getmembers(evidence, inspect.isfunction)
                   if not name.startswith('_') and (name == 'manifest' or name.endswith('_manifest'))]
        self.assertEqual(len(helpers), 57)
        self.assertIn('attachment_mail_baseline_manifest', {name for name, _ in helpers})
        self.assertIn('native_attachment_desk_baseline_manifest', {name for name, _ in helpers})
        paths = []
        raw_values = set()
        for _, helper in helpers:
            relative = next(value for value in helper.__code__.co_consts
                            if isinstance(value, str) and value.startswith('dev-docs/evidence/') and value.endswith('.json'))
            raw = (evidence.ROOT / relative).read_bytes()
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(raw)
            paths.append(path)
            raw_values.add(raw)
        chosen = self.root / 'cache-chosen-source.rs'
        chosen.write_bytes(b'synthetic source not present in any historical manifest')
        expected = self.sha(chosen.read_bytes())
        original_read = Path.read_bytes
        original_loads = json.loads
        reads = Counter()
        def read(path):
            reads[path] += 1
            return original_read(path)
        with patch.object(Path, 'read_bytes', read), patch.object(evidence.json, 'loads', wraps=original_loads) as loads:
            for _ in range(2):
                self.assertEqual(evidence.source_bytes(chosen.name, expected, self.root), b'synthetic source not present in any historical manifest')
            self.assertEqual(loads.call_count, len(raw_values))
        self.assertTrue(all(reads[path] == 2 for path in paths))
        self.assertEqual(reads[chosen], 2)
        for path in paths:
            original = original_read(path)
            with self.subTest(manifest=path.name):
                path.write_bytes(b'{}')
                with self.assertRaises(AssertionError):
                    evidence.source_bytes(chosen.name, expected, self.root)
                path.write_bytes(original)


if __name__ == '__main__':
    unittest.main()
