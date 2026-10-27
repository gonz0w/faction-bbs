"""Positive and rejection contracts for the search observation oracle."""
import copy
import hashlib
import json
import unittest

import probe_mail_search as proof


class SearchOracle(unittest.TestCase):
    def test_selected_original_indices_active_label_and_bounded_cells(self):
        screen = (b'[LOCAL INBOX]\r\nLocal search active.\r\n'
                  b'> 07 UNREAD subject\r\n  01 READ subject\r\n[B] Back\r\n')
        proof.require_search_screen(screen, [7, 1], 7, True)
        for bad in [screen.replace(b'01 READ', b'02 READ'),
                    screen.replace(b'> 07', b'  07').replace(b'  01', b'> 01'),
                    screen.replace(b'Local search active.\r\n', b''),
                    screen + b'x' * 40 + b'\r\n',
                    screen + b'\x1b[31m', screen + '\u202e'.encode()]:
            with self.subTest(bad=bad), self.assertRaises(AssertionError):
                proof.require_search_screen(bad, [7, 1], 7, True)

    def test_width_oracle_scopes_current_inbox_after_reply_confirmation(self):
        screen = (b'Queued durably; no delivery or read receipt.\r\n'
                  b'[LOCAL INBOX]\r\nLocal search active.\r\n'
                  b'> 07 UNREAD subject\r\n[B] Back\r\n')
        proof.require_search_screen(screen, [7], 7, True)
        with self.assertRaises(AssertionError):
            proof.require_search_screen(b'> 07 UNREAD subject\r\n', [7], 7, False)

    def test_cancellation_requires_the_existing_context_specific_lock_notice(self):
        proof.require_locked_output(b'Home mail locked: local operation stopped.\r\n', True)
        proof.require_locked_output(b'Checkpoint locked: local operation stopped.\r\n', False)
        for text, online in [(b'Checkpoint locked: local operation stopped.', True),
                             (b'Home mail locked: local operation stopped.', False),
                             (b'local cancellation', True), (b'', False)]:
            with self.subTest(text=text, online=online), self.assertRaises(AssertionError):
                proof.require_locked_output(text, online)

    def test_no_match_is_truthful_and_clear_removes_active_indicator(self):
        empty = b'[LOCAL INBOX]\r\nLocal search active.\r\nNo matching retained records.\r\n[B] Back\r\n'
        proof.require_search_screen(empty, [], None, True)
        for bad in [empty.replace(b'No matching retained records.\r\n', b''),
                    empty + b'> 01 UNREAD wrong\r\n']:
            with self.subTest(bad=bad), self.assertRaises(AssertionError):
                proof.require_search_screen(bad, [], None, True)
        clear = b'[LOCAL INBOX]\r\n> 08 PENDING current\r\n[B] Back\r\n'
        proof.require_search_screen(clear, [8], 8, False)
        with self.assertRaises(AssertionError):
            proof.require_search_screen(clear + b'Local search active.\r\n', [8], 8, False)

    def test_private_query_may_not_reappear_in_search_heading(self):
        screen = b'[LOCAL INBOX]\r\nLocal search active.\r\n> 01 UNREAD subject\r\n[B] Back\r\n'
        proof.require_search_screen(screen, [1], 1, True, b'private-query')
        with self.assertRaises(AssertionError):
            proof.require_search_screen(screen.replace(b'Local search active.', b'Local search active. private-query'), [1], 1, True, b'private-query')

    def test_capture_binding_requires_preexisting_markers_and_actual_binaries(self):
        source, marker_data = b'probe source', b'["private input"]'
        binaries = {'client': b'actual client', 'fixture': b'actual fixture'}
        sha = lambda data: hashlib.sha256(data).hexdigest()
        record = {'capture_id': '12' * 16, 'source_sha256': sha(source),
                  'marker_set_sha256': sha(marker_data),
                  'binary_sha256': {key: sha(value) for key, value in binaries.items()},
                  'generated_at_ns': 10, 'started_at_ns': 11}
        proof.require_capture_binding(record, source, marker_data, binaries)
        for key, value in [('capture_id', ''), ('source_sha256', '0' * 64),
                           ('marker_set_sha256', '0' * 64), ('generated_at_ns', 12),
                           ('binary_sha256', {'client': sha(b'other'), 'fixture': sha(b'actual fixture')})]:
            changed = copy.deepcopy(record); changed[key] = value
            with self.subTest(key=key), self.assertRaises(AssertionError):
                proof.require_capture_binding(changed, source, marker_data, binaries)
        with self.assertRaises(AssertionError):
            proof.require_capture_binding(record, source, marker_data + b' ', binaries)
        with self.assertRaises(AssertionError):
            proof.require_capture_binding(record, source, marker_data, dict(binaries, client=b'rebuilt client'))


class SearchInputProvenance(unittest.TestCase):
    def test_board_registration_account_is_publication_private_but_allowed_board_io(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        binary = SimpleNamespace(read_bytes=lambda: b'public synthetic executable')
        root = proof.Path('/nonexistent-unit-fixture')
        with patch.object(proof, 'save') as saved, patch.object(proof.secrets, 'token_hex', side_effect=lambda n: 'a' * (n * 2)):
            result = proof.inputs(root, root, binary, binary,
                                  publication_extra=['fixture-board-account'])
        self.assertNotIn('fixture-board-account', result[6])
        inventories = [json.loads(call.args[2]) for call in saved.call_args_list
                       if call.args[1] == 'private-inventory.json']
        self.assertEqual(len(inventories), 1)
        self.assertIn('fixture-board-account', inventories[0][0]['private_values'])
        self.assertNotIn('fixture-board-account', inventories[0][0]['public_input_provenance']['values'])

    def test_only_exact_proven_public_constants_are_classified_public(self):
        private = ['query-private-dc713f2e', 'private-prefix-recv 03-suffix', '00' * 32]
        markers = [proof.PASS, 'recv 03', '[.*]', 'λ界', *private]
        inventory = proof.input_inventory(markers)
        self.assertEqual(inventory['private_values'], private)
        self.assertEqual(inventory['public_input_provenance']['values'], markers[:4])
        self.assertEqual(inventory['public_input_provenance']['source_sha256'],
                         hashlib.sha256(proof.Path(proof.__file__).read_bytes()).hexdigest())
        self.assertEqual(set(inventory['private_values']) |
                         set(inventory['public_input_provenance']['values']), set(markers))


if __name__ == '__main__':
    unittest.main()
