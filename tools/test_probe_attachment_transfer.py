"""Bounded control-plane input checks for the attachment TLS software proof."""
import os
import base64
import struct
import time
import unittest
from probe_attachment_transfer import FixtureFailure, parse_progress, read_frame, storage_kind, stream_stanza


# Exact bytes emitted by the Rust failure_line formatter on the observed-state
# nonzero fixture (cycle 14/6, terminal 5/3); original output is retained for #326.
PRODUCER_V2 = (
    b'Attachment diagnostic v2 direction=fetch origin=sdk_error error=unavailable state=observed request=unavailable last_phase=unavailable transport=unavailable'
    b' sdk_completed=0 sdk_active=1 aggregate_valid=true totals_clamped=false observed_logical_completed=0 observed_logical_attempted=0 observed_carrier_attempts=0 observed_busy=0 observed_connect_calls=0 unavailable_cycles=0'
    b' cycle_completed=0 cycle_logical_attempted=0 cycle_carrier_attempts=14 cycle_busy=6 cycle_connect_calls=0 confirmed_offset=0 foreground_us=0 last_phase_us=0'
    b' observed_submit_connect_us=0 observed_submit_exchange_us=0 observed_collect_connect_us=0 observed_collect_exchange_us=0 observed_planned_wait_us=0 observed_actual_wait_us=0'
    b' terminal_carrier_attempts=5 terminal_busy=3 exchange_origin=read_eof\r\n'
)

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

class TerminalDiagnosticConsumerTests(unittest.TestCase):
    def setUp(self):
        import probe_device_provider_console
        self.probe = probe_device_provider_console

    def changed(self, line=PRODUCER_V2, **fields):
        words = line[:-2].split(b' ')
        for index, word in enumerate(words):
            key = word.split(b'=', 1)[0].decode('ascii')
            if key in fields:
                words[index] = key.encode() + b'=' + str(fields[key]).encode()
        return b' '.join(words) + b'\r\n'

    def legacy(self):
        return PRODUCER_V2.split(b' terminal_carrier_attempts=', 1)[0].replace(b'v2 ', b'v1 ', 1) + b'\r\n'

    def test_actual_rust_formatter_line_survives_parser_and_hosted_failure_summary(self):
        diagnostic = self.probe._parse_transfer_diagnostic(PRODUCER_V2)
        self.assertIsNotNone(diagnostic, 'the actual producer attribution was discarded')
        self.assertEqual(diagnostic['terminal_carrier_attempts'], 5)
        self.assertEqual(diagnostic['terminal_busy'], 3)
        self.assertEqual(diagnostic['exchange_origin'], 'read_eof')
        error = self.probe.TransferObservationError(dict(diagnostic=diagnostic))
        for field in ['diag_terminal_carrier_attempts=5', 'diag_terminal_busy=3', 'diag_exchange_origin=read_eof']:
            self.assertIn(field, str(error))
        self.assertLessEqual(len(PRODUCER_V2) - 2, 1024)
        self.assertLessEqual(len(str(error)), 2048)

    def test_original_v1_grammar_is_separate_and_remains_accepted(self):
        legacy = self.probe._parse_transfer_diagnostic(self.legacy())
        self.assertIsNotNone(legacy)
        self.assertNotIn('terminal_busy', legacy)
        self.assertIn('diag_cycle_carrier_attempts=14', self.probe._transfer_diagnostic_summary(legacy))
        self.assertNotIn('diag_terminal', self.probe._transfer_diagnostic_summary(legacy))
        for raw in [PRODUCER_V2 + self.legacy(), self.legacy() + PRODUCER_V2,
                    PRODUCER_V2 * 2, self.legacy() * 2, PRODUCER_V2.replace(b'v2 ', b'v1 ', 1),
                    PRODUCER_V2.replace(b'v2 ', b'v3 ', 1), self.legacy().replace(b'v1 ', b'v2 ', 1)]:
            with self.subTest(raw=raw[:30]):
                self.assertIsNone(self.probe._parse_transfer_diagnostic(raw))

    def test_terminal_counts_and_available_state_remain_bounded(self):
        maximum = self.changed(terminal_carrier_attempts=8, terminal_busy=8, cycle_busy=8)
        self.assertIsNotNone(self.probe._parse_transfer_diagnostic(maximum))
        unavailable = self.changed(state='unavailable', cycle_completed='unavailable',
            cycle_logical_attempted='unavailable', cycle_carrier_attempts='unavailable',
            cycle_busy='unavailable', cycle_connect_calls='unavailable', last_phase_us='unavailable',
            terminal_carrier_attempts='unavailable', terminal_busy='unavailable', exchange_origin='unobserved')
        self.assertIsNotNone(self.probe._parse_transfer_diagnostic(unavailable))
        self.assertIsNotNone(self.probe._parse_transfer_diagnostic(self.changed(unavailable, state='join_panicked')))
        variants = [self.changed(terminal_carrier_attempts=9), self.changed(terminal_busy=9),
                    self.changed(terminal_carrier_attempts='05'), self.changed(terminal_busy=-1),
                    self.changed(terminal_busy=6), self.changed(cycle_carrier_attempts=4),
                    self.changed(cycle_busy=2), self.changed(terminal_carrier_attempts='unavailable'),
                    self.changed(terminal_busy='unavailable'), self.changed(aggregate_valid='false'),
                    self.changed(unavailable, terminal_carrier_attempts=0),
                    self.changed(unavailable, terminal_busy=0), self.changed(unavailable, exchange_origin='read_eof')]
        for raw in variants:
            with self.subTest(raw=raw[-100:]):
                self.assertIsNone(self.probe._parse_transfer_diagnostic(raw))

    def test_unknown_noncanonical_excess_or_private_fields_never_enter_summary(self):
        variants = [self.changed(exchange_origin='PRIVATE_ENDPOINT_SENTINEL'),
                    self.changed(terminal_busy='true'), self.changed(terminal_carrier_attempts='+5'),
                    self.changed(terminal_carrier_attempts='18446744073709551616'),
                    PRODUCER_V2.replace(b' terminal_busy=3', b' terminal_busy=3 terminal_busy=3'),
                    PRODUCER_V2.replace(b'\r\n', b' private=PRIVATE_TOKEN_SENTINEL\r\n'),
                    PRODUCER_V2.replace(b' terminal_busy=', b'  terminal_busy='), PRODUCER_V2[:-2],
                    b'Attachment diagnostic v2 ' + b'PRIVATE_TOKEN_SENTINEL' * 100 + b'\r\n']
        for raw in variants:
            diagnostic = self.probe._parse_transfer_diagnostic(raw)
            self.assertIsNone(diagnostic)
            error = self.probe.TransferObservationError(dict(diagnostic=diagnostic))
            self.assertIn('diagnostic=unavailable', str(error))
            self.assertNotIn('PRIVATE_', str(error))
            self.assertLessEqual(len(str(error)), 2048)
        valid = self.probe._parse_transfer_diagnostic(PRODUCER_V2)
        self.assertIsNotNone(valid)
        for diagnostic in [dict(valid, private='PRIVATE_TOKEN_SENTINEL'), dict(valid, terminal_busy=True),
                           dict(valid, terminal_busy=6), dict(valid, exchange_origin='PRIVATE_ENDPOINT_SENTINEL')]:
            summary = self.probe._transfer_diagnostic_summary(diagnostic)
            self.assertEqual(summary, 'diagnostic=unavailable')


if __name__ == "__main__":
    unittest.main()
