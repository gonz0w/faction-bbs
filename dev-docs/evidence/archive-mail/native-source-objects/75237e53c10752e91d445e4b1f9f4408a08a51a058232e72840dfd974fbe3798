"""Offline proof envelope/identity/protocol oracles; no crypto dependencies."""
import importlib.util
import unittest
from pathlib import Path

PATH = Path(__file__).resolve().parents[1] / 'experiments/200-omemo-interop/reference.py'
spec = importlib.util.spec_from_file_location('faction_omemo_reference', PATH)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class EnvelopeContracts(unittest.TestCase):
    def test_unicode_escape_and_delayed_delivery(self):
        raw = m.pack_sce('alice@a.test', 'bob@b.test', 'Café <&>', '雪 & <signal>',
                         '2026-10-24T12:00:00Z', padding='!arbitrary-padding!')
        self.assertEqual(m.unpack_sce(raw, 'alice@a.test', 'bob@b.test',
                                     '2026-10-24T12:04:59Z'), ('Café <&>', '雪 & <signal>'))
        # No receive-now parameter: historical/offline delivery uses stanza sending time.
        self.assertEqual(m.unpack_sce(raw, 'alice@a.test', 'bob@b.test',
                                     '2026-10-24T12:00:00+00:00')[0], 'Café <&>')

    def test_time_identity_shape_and_unsafe_xml_reject(self):
        raw = m.pack_sce('alice@a.test', 'bob@b.test', 'Subject', 'Body',
                         '2026-10-24T12:00:00Z', padding='x')
        cases = [(raw, 'eve@a.test', 'bob@b.test', '2026-10-24T12:00:00Z'),
                 (raw, 'alice@a.test', 'eve@b.test', '2026-10-24T12:00:00Z'),
                 (raw, 'alice@a.test', 'bob@b.test', '2026-10-24T12:05:01Z')]
        for changed in [raw.replace(b'12:00:00Z', b'12:00:00'),
                        raw.replace(b'2026-10-24', b'2026-13-24'),
                        raw.replace(b'urn:xmpp:sce:1', b'urn:xmpp:sce:0'),
                        b'<!DOCTYPE envelope>'+raw, b'\xff', b'x'*8193]:
            cases.append((changed, 'alice@a.test', 'bob@b.test', '2026-10-24T12:00:00Z'))
        for args in cases:
            with self.subTest(args=args[:1]), self.assertRaises(ValueError):
                m.unpack_sce(*args)
        from xml.etree import ElementTree as ET
        root = ET.fromstring(raw); root.append(root.find('{urn:xmpp:sce:1}from'))
        with self.assertRaises(ValueError):
            m.unpack_sce(ET.tostring(root), 'alice@a.test', 'bob@b.test', '2026-10-24T12:00:00Z')

    def test_padding_not_limited_to_sender_size_and_text_bounds(self):
        raw = m.pack_sce('a@a.test', 'b@b.test', 'S', 'B', '2026-10-24T12:00:00Z', padding='!'*3500)
        self.assertEqual(m.unpack_sce(raw, 'a@a.test', 'b@b.test', '2026-10-24T12:00:00Z'), ('S', 'B'))
        for subject,body in [('', 'body'), ('s'*129, 'b'), ('s', ''), ('s', '雪'*342)]:
            with self.assertRaises(ValueError):
                m.pack_sce('a@a.test', 'b@b.test', subject, body, '2026-10-24T12:00:00Z')

    def test_typed_exact_device_pin_and_inert_protocol(self):
        key=bytes(range(32))
        self.assertEqual(m.pin_bytes('ed25519:'+key.hex()), key)
        for pin in ['curve25519:'+key.hex(), key.hex(), 'ed25519:'+key.hex()[:-2],
                    'ed25519:'+('GG'*32), 'ed25519:'+key.hex()+'\n']:
            with self.assertRaises(ValueError):m.pin_bytes(pin)
        self.assertEqual(m.parse_reply(b'ok 00 1 ff\n'), ['00', '1', 'ff'])
        for response in [b'err\n', b'ok\x1b[31m\n', b'other\n', b'ok '+b'0'*140000+b'\n']:
            with self.assertRaises(ValueError):m.parse_reply(response)


if __name__ == '__main__':
    unittest.main()
