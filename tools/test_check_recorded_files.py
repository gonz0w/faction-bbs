"""Byte integrity and selected-folder membership; no runtime claims."""
import hashlib
import json
from pathlib import Path
import re
import tempfile
import unittest


class Integrity(unittest.TestCase):
    def setUp(self):
        self.owned = tempfile.TemporaryDirectory(prefix="faction242-recorded-files-OWNED-")
        self.addCleanup(self.owned.cleanup)
        self.root = Path(self.owned.name)
        self.parent = self.root / "evidence/new"
        self.parent.mkdir(parents=True)
        self.manifest = "evidence/new/manifest.json"
        self.files = {}
        for name, data in {"evidence/new/README.md": b"Selected observations.\n",
                           "evidence/new/logs/result.json": b'{"result":"pass"}\n',
                           "evidence/old/source-object": b"Already recorded source.\n"}.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            self.files[name] = hashlib.sha256(data).hexdigest()
        self.pin = self.write_manifest()

    def write_manifest(self, files=None, schema=1):
        raw = (json.dumps({"schema": schema, "files": self.files if files is None else files},
                          indent=2) + "\n").encode()
        (self.root / self.manifest).write_bytes(raw)
        return hashlib.sha256(raw).hexdigest()

    def verify(self, manifest=None, pin=None):
        from check_recorded_files import verify
        return verify(self.root, self.manifest if manifest is None else manifest,
                      self.pin if pin is None else pin)

    def test_valid_selection_includes_old_object_outside_exact_new_folder(self):
        self.assertEqual(self.verify(), 3)

    def test_content_manifest_tamper_and_missing_reference_fail(self):
        for name in self.files:
            with self.subTest(reference=name):
                path = self.root / name
                original = path.read_bytes()
                path.write_bytes(original + b"changed")
                with self.assertRaises(ValueError):
                    self.verify()
                path.unlink()
                with self.assertRaises((ValueError, OSError)):
                    self.verify()
                path.write_bytes(original)
        path = self.root / self.manifest
        path.write_bytes(path.read_bytes() + b" ")
        with self.assertRaises(ValueError):
            self.verify()

    def test_schema_empty_files_and_hash_shapes_fail(self):
        for schema in [0, 2, True, "1"]:
            with self.subTest(schema=schema), self.assertRaises(ValueError):
                self.verify(pin=self.write_manifest(schema=schema))
        for files in [{}, [], {"evidence/new/README.md": "not-a-sha256"}]:
            with self.subTest(files=files), self.assertRaises(ValueError):
                self.verify(pin=self.write_manifest(files=files))
        self.pin = self.write_manifest()
        for pin in ["", "a" * 63, "g" * 64]:
            with self.subTest(pin=pin), self.assertRaises(ValueError):
                self.verify(pin=pin)

    def test_paths_cannot_escape_or_follow_file_directory_or_manifest_symlinks(self):
        for name in ["/absolute", "../escape", "evidence/new/../../escape", "./evidence/new/README.md", "evidence//new/README.md", "evidence\\new\\README.md"]:
            with self.subTest(path=name), self.assertRaises(ValueError):
                self.verify(pin=self.write_manifest(files={name: "0" * 64}))
        self.pin = self.write_manifest()
        for manifest in ["../manifest.json", "/manifest.json"]:
            with self.subTest(manifest=manifest), self.assertRaises(ValueError):
                self.verify(manifest=manifest)
        target = self.root / "evidence/old/source-object"
        alias = self.parent / "alias"
        alias.symlink_to(target)
        files = dict(self.files, **{"evidence/new/alias": self.files["evidence/old/source-object"]})
        with self.assertRaises(ValueError):
            self.verify(pin=self.write_manifest(files=files))
        alias.unlink()
        alias.symlink_to(target.parent, target_is_directory=True)
        files = dict(self.files, **{"evidence/new/alias/source-object": self.files["evidence/old/source-object"]})
        with self.assertRaises(ValueError):
            self.verify(pin=self.write_manifest(files=files))
        alias.unlink()
        self.pin = self.write_manifest()
        alias.symlink_to(self.root / self.manifest)
        with self.assertRaises(ValueError):
            self.verify(manifest="evidence/new/alias")

    def test_unregistered_new_folder_file_is_rejected(self):
        (self.parent / "unregistered.log").write_bytes(b"No unregistered payload.\n")
        with self.assertRaises(ValueError):
            self.verify()


class Registration(unittest.TestCase):
    def test_ci_pins_mail_search_manifest_once(self):
        workflow = (Path(__file__).resolve().parents[1] / ".github/workflows/documentation.yml").read_text()
        lines = re.findall(r"^ +python3 tools/check_recorded_files\.py dev-docs/evidence/mail-search/manifest\.json [a-f0-9]{64}$", workflow, re.MULTILINE)
        self.assertEqual(len(lines), 1, "CI must verify the selected mail-search manifest and its exact SHA-256")


if __name__ == "__main__":
    unittest.main()
