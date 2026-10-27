"""Behavioral contract checks for the notice PTY observation oracle."""
import unittest
import probe_menu_mail_notices as proof


class NoticeOracle(unittest.TestCase):
    def test_off_unknown_stale_and_disabled_are_exact_context_specific_lines(self):
        proof.require_notice(b'LOCAL MAIL: notices off. O opens mail.\r\n', 'off', 'main')
        proof.require_notice(b'LOCAL MAIL / LOCKED: unread unknown.\r\nNotices on; O unlocks manually.\r\nN turns notices off.\r\n', 'unknown', 'main')
        proof.require_notice(b'LOCAL MAIL / LOCKED\r\nstale local snapshot; home not polled.\r\nunread: 3\r\nretained: 8; pending: 1\r\nN turns notices off.\r\n', 'stale', 'main', (3, 8, 1))
        proof.require_notice(b'LOCAL MAIL / LOCKED\r\nstale local snapshot; home not polled.\r\ntracking disabled; unread unknown\r\nretained: 8; pending: 1\r\nN turns notices off.\r\n', 'stale', 'files', (None, 8, 1))
        proof.require_notice(b'LOCAL MAIL: notices off. E opens mail.\r\n', 'off', 'files')

    def test_notice_rejects_wrong_context_count_extra_metadata_and_controls(self):
        valid = b'LOCAL MAIL / LOCKED\r\nstale local snapshot; home not polled.\r\nunread: 3\r\nretained: 8; pending: 1\r\nN turns notices off.\r\n'
        for bad in [valid.replace(b'unread: 3', b'unread: 0'), valid.replace(b'LOCKED', b'FRESH'), valid + b'peer@private.test\r\n', valid.replace(b'unread: 3', b'\x1b[31munread: 3'), valid.replace(b'pending: 1', b'pending: 11')]:
            with self.subTest(bad=bad), self.assertRaises(AssertionError):
                proof.require_notice(bad, 'stale', 'main', (3, 8, 1))
        with self.assertRaises(AssertionError):
            proof.require_notice(b'LOCAL MAIL: notices off. E opens mail.\r\n', 'off', 'main')
        for counts in [(65, 64, 8), (0, 65, 0), (0, 0, 9), (-1, 1, 0), (True, 1, 0)]:
            with self.subTest(counts=counts), self.assertRaises(AssertionError):
                proof.notice_lines('stale', 'main', counts)

    def test_choice_requires_exact_fixed_set_and_safe_default(self):
        screen = b'LOCAL / ENCRYPTED HOME MAIL\r\n  [O] Open encrypted home mail\r\n> [B] Back to the BBS\r\n  [N] Local notices\r\n'
        proof.require_choice(screen)
        for bad in [screen.replace(b'> [B]', b'  [B]'), screen.replace(b'  [O]', b'> [O]'), screen.replace(b'[N] Local notices', b'[N] peer@private.test'), screen + b'  [X] Extra remote action\r\n', screen.replace(b'  [N] Local notices\r\n', b'')]:
            with self.subTest(bad=bad), self.assertRaises(AssertionError): proof.require_choice(bad)

    def test_protocol_cycles_reject_toggle_emission_replay_and_unpaired_input(self):
        nonce = b'01' * 16
        data = b'@MAIL1 park ' + nonce + b'\r\n@MAIL1 back ' + nonce + b'\r\n'
        self.assertEqual(proof.require_mail_cycles(data, 1, 0), {'parks': 1, 'backs': 1, 'nonces': 1})
        self.assertEqual(proof.require_mail_cycles(b'', 0, 0), {'parks': 0, 'backs': 0, 'nonces': 0})
        for bad in [data + data, data.replace(b'back', b'park'), data + b'n', data.replace(nonce + b'\r\n', b'X\r\n', 1), b'@MAIL1 back ' + nonce + b'\n@MAIL1 park ' + nonce + b'\n']:
            with self.subTest(bad=bad), self.assertRaises(AssertionError): proof.require_mail_cycles(bad, 1, 0)
        with self.assertRaises(AssertionError): proof.require_mail_cycles(data, 0, 0)

    def test_toggle_cannot_change_vault_or_board_stream_or_show_password_prompt(self):
        before = {'canonical': b'encrypted-vault', 'inspection': {'history': [1], 'pending': [2]}}
        proof.require_local_toggle(before, before, b'', b'LOCAL MAIL / LOCKED: unread unknown.\r\n')
        for after, wire, screen in [({'canonical': b'changed'}, b'', b''), (before, b'@MAIL1 back ' + b'00' * 16 + b'\n', b''), (before, b'', proof.UNLOCK.encode()), (before, b'', proof.HOME.encode())]:
            with self.subTest(after=after, wire=wire, screen=screen), self.assertRaises(AssertionError): proof.require_local_toggle(before, after, wire, screen)


if __name__ == '__main__': unittest.main()
