"""Bounded control-plane input checks for the attachment TLS software proof."""
import os
import base64
import struct
import time
import unittest
from probe_attachment_transfer import FixtureFailure, parse_progress, read_frame, storage_kind, stream_stanza

class TransferProbeTests(unittest.TestCase):
    def test_progress_is_canonical_bounded_and_has_no_remote_diagnostic(self):
        self.assertEqual(parse_progress(b"progress 4096 0 0\n"), (4096, False, 0))
        self.assertEqual(parse_progress(b"progress 1048592 1 1\n"), (1048592, True, 1))
        for raw in [b"progress -1 0 0\n", b"progress 1048593 0 0\n", b"progress 1 2 0\n", b"progress 1 0 1\n", b"progress 1 1 2\n", b"progress 01 0 0\n", b"progress 0 0 0\nSECRET", b"secret path or content"]:
            with self.subTest(raw=raw[:20]):
                with self.assertRaises(FixtureFailure) as caught:
                    parse_progress(raw)
                self.assertNotIn("SECRET", str(caught.exception))
                self.assertNotIn("secret path", str(caught.exception))

    def test_loss_selector_identifies_only_exact_bounded_storage_payload(self):
        raw = b"FBRQ0001"+bytes([3])+b"synthetic ciphertext chunk"
        xml = b"<iq xmlns='jabber:client'><blob xmlns='urn:faction:blob:1'>"+base64.b64encode(raw)+b"</blob></iq>"
        self.assertEqual(storage_kind(xml), 3)
        for raw in [b"<iq/>", b"<iq xmlns='jabber:client'><blob xmlns='urn:faction:blob:1'>?</blob></iq>", b"<broken"]:
            with self.assertRaises(FixtureFailure):
                storage_kind(raw)

    def test_fixture_preserves_inherited_client_namespace(self):
        element, standalone = stream_stanza(b"<iq id='one'><blob xmlns='urn:faction:blob:1'>AAAA</blob></iq>")
        self.assertEqual(element.tag, '{jabber:client}iq')
        self.assertIn(b'jabber:client', standalone)
        self.assertEqual(len(element), 1)
        for raw in [b"<iq/><iq/>", b"<!DOCTYPE iq><iq/>", b"<iq>", b"A"*16385]:
            with self.assertRaises(FixtureFailure):
                stream_stanza(raw)

    def frame(self, raw):
        rd, wr = os.pipe()
        os.write(wr, raw)
        os.close(wr)
        try:
            return read_frame(rd, 1024, time.monotonic()+1)
        finally:
            os.close(rd)

    def test_frames_require_complete_exact_bounded_payload(self):
        self.assertEqual(self.frame(struct.pack("!I", 3)+b"abc"), b"abc")
        for raw in [b"", b"\0", struct.pack("!I", 0), struct.pack("!I", 1025), struct.pack("!I", 3)+b"ab"]:
            with self.subTest(raw=raw):
                with self.assertRaises(FixtureFailure):
                    self.frame(raw)

    def test_stalled_handler_is_bounded_without_waiting_for_eof(self):
        rd, wr = os.pipe()
        started = time.monotonic()
        try:
            with self.assertRaises(FixtureFailure):
                read_frame(rd, 1024, started+.025)
            self.assertLess(time.monotonic()-started, .5)
        finally:
            os.close(rd)
            os.close(wr)

if __name__ == "__main__":
    unittest.main()
