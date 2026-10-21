"""Security-relevant format tests for the research SCE adapter; no crypto mock."""
import importlib.util
from pathlib import Path
import unittest
from xml.etree import ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('faction_probe_sce', ROOT/'experiments/084-mail-proof/sce.py')
sce = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sce)


class Envelope(unittest.TestCase):
    def raw(self):
        return sce.pack('alice@homea.test', 'bob@homeb.test', 'private subject', 'private body <&>')

    def test_subject_body_and_both_authenticated_affixes_roundtrip(self):
        raw = self.raw()
        self.assertEqual(sce.unpack(raw, 'alice@homea.test', 'bob@homeb.test'), ('private subject', 'private body <&>'))
        self.assertGreaterEqual(len(raw), 2048)
        self.assertNotEqual(self.raw(), raw)
        for sender, recipient in [('other@homea.test', 'bob@homeb.test'), ('alice@homea.test', 'other@homeb.test')]:
            with self.assertRaises(ValueError): sce.unpack(raw, sender, recipient)

    def test_duplicates_missing_padding_and_server_extensions_are_rejected(self):
        original = ET.fromstring(self.raw())
        cases = []
        e = ET.fromstring(ET.tostring(original)); e.remove(e.find('{'+sce.NS+'}rpad')); cases.append(e)
        e = ET.fromstring(ET.tostring(original)); e.append(ET.fromstring(ET.tostring(e.find('{'+sce.NS+'}from')))); cases.append(e)
        e = ET.fromstring(ET.tostring(original)); e.find('{'+sce.NS+'}content').append(ET.Element('{urn:xmpp:hints}store')); cases.append(e)
        e = ET.fromstring(ET.tostring(original)); e.find('{'+sce.NS+'}content/{jabber:client}subject').append(ET.Element('unexpected')); cases.append(e)
        for e in cases:
            with self.assertRaises(ValueError): sce.unpack(ET.tostring(e), 'alice@homea.test', 'bob@homeb.test')

    def test_oversize_dtd_and_unexpected_roots_are_rejected(self):
        for raw in [b'x'*8193, b'<!DOCTYPE envelope><envelope/>', b'<body>plaintext fallback</body>']:
            with self.assertRaises(ValueError): sce.unpack(raw, 'alice@homea.test', 'bob@homeb.test')
        with self.assertRaises(ValueError): sce.pack('a', 'b', 's'*129, 'body')
        with self.assertRaises(ValueError): sce.pack('a', 'b', 'subject', 'b'*1025)


if __name__ == '__main__': unittest.main()
