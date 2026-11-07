"""Positive and rejection contracts for bounded conversation observations."""
import copy
import unittest
import probe_mail_conversation as proof


def preview(number, selected=False, received=True):
    return ((('>' if selected else ' ') + f' {number:02} ' + ('RECEIVED UNREAD' if received else 'SENT PENDING') + '\r\n').encode()
        + b'UTC 42\r\nSubject: synthetic\r\n| one\r\n| two\r\n| three\r\n[O] Full record\r\n')

def frame():
    return (b'[LOCAL CONVERSATION]\r\nPeer: peer@away.test\r\nD202 / All / Search\r\nPage 1/2; selection 2/4\r\nRetained append order; no read change.\r\n'
        + preview(1) + preview(2, True, False)
        + b'[Up/Down J/K] Select  [N/P] Page\r\n[Enter/O] Open full record\r\n[B] Back\r\n')

class ConversationOracle(unittest.TestCase):
    def test_complete_two_preview_frame_and_empty_scope(self):
        proof.require_conversation_screen(frame(), [1, 2], 2, (1, 2), 'All', True)
        empty=(b'[LOCAL CONVERSATION]\r\nPeer: peer@away.test\r\nD202 / Unread / Search\r\nPage 0/0; selection 0/0\r\nRetained append order; no read change.\r\nNo eligible retained records.\r\n[B] Back\r\n')
        proof.require_conversation_screen(empty, [], None, (0, 0), 'Unread', True)
        with self.assertRaises(AssertionError):
            proof.require_conversation_screen(empty.replace(b'No eligible retained records.\r\n', b''), [], None, (0, 0), 'Unread', True)

    def test_rejects_wrong_canonical_index_selection_order_page_and_scope(self):
        original=frame()
        for bad in [original.replace(b'  01 RECEIVED',b'  03 RECEIVED'), original.replace(b'> 02 SENT',b'  02 SENT'),
                    original.replace(preview(1)+preview(2,True,False),preview(2,True,False)+preview(1)),
                    original.replace(b'Page 1/2', b'Page 2/2'), original.replace(b'/ All / Search',b'/ Unread / Search'),
                    original.replace(b' / Search',b''), original+preview(3)]:
            with self.subTest(bad=bad), self.assertRaises(AssertionError):
                proof.require_conversation_screen(bad,[1,2],2,(1,2),'All',True)

    def test_rejects_incomplete_previews_body_overflow_controls_and_width(self):
        original=frame()
        for bad in [original.replace(b'UTC 42\r\n',b''), original.replace(b'Subject: synthetic\r\n', b''),
                    original.replace(b'| three\r\n', b'| three\r\n| fourth\r\n'),
                    original.replace(b'[O] Full record\r\n', b''), original.replace(b'| one',b'| '+b'x'*38),
                    original.replace(b'| one', b'| \x1b[31m'), original.replace(b'| one', '| \u202e'.encode()),
                    original.replace(b'| one', ('| '+'界'*19).encode()),
                    original.replace(b'Peer: peer', 'Peer: peer\u0085'.encode()),
                    original.replace(b'Peer: peer', b'Peer: peer\v')]:
            with self.subTest(bad=bad), self.assertRaises(AssertionError):
                proof.require_conversation_screen(bad,[1,2],2,(1,2),'All',True)

    def test_readonly_requires_both_exact_encrypted_and_logical_bytes(self):
        before={'history':[{'read':False}],'pending':[],'archive':{'cursor':'original'}}
        proof.require_readonly(b'ciphertext', b'ciphertext', before, copy.deepcopy(before))
        for raw, after in [(b'changed',before), (b'ciphertext',dict(before,pending=['queued'])),
                            (b'ciphertext',dict(before,history=[{'read':True}]))]:
            with self.subTest(raw=raw, after=after), self.assertRaises(AssertionError):
                proof.require_readonly(b'ciphertext',raw,before,after)

class ConversationInputProvenance(unittest.TestCase):
    def test_only_exact_source_public_constants_are_classified_public(self):
        import hashlib
        constants=[proof.PASS, 'recv 03', proof.CONVERSATION, 'alice@home.test', 'CURRENT recipient:']
        private=['recv 03 private-prefix', 'generated-private-peer', 'fixture-registration-handle', '00'*32]
        inventory=proof.input_inventory(constants+private)
        self.assertEqual(inventory['private_values'], private)
        self.assertEqual(inventory['public_input_provenance']['values'], constants)
        self.assertEqual(inventory['public_input_provenance']['source_sha256'],
            hashlib.sha256(proof.Path(proof.__file__).read_bytes()).hexdigest())

class CaptureMaterialization(unittest.TestCase):
    def test_coarse_clock_advances_only_after_real_marker_and_inventory_materialization(self):
        import tempfile
        from types import SimpleNamespace
        from unittest.mock import patch
        binary=SimpleNamespace(read_bytes=lambda: b'actual synthetic executable')
        with tempfile.TemporaryDirectory(prefix='conversation-binding-OWNED-') as name:
            root=proof.Path(name)
            (root/'OWNED').write_text('synthetic capture oracle fixture\n')
            def clock():
                return 11 if (root/'private-markers.json').is_file() and (root/'publication-inputs.private.json').is_file() else 10
            with patch.object(proof.time, 'time_ns', side_effect=clock):
                record=proof.bind(root,binary,binary,['generated-private-body'])
            self.assertEqual(record['generated_at_ns'],10)
            self.assertEqual(record['started_at_ns'],11)
            self.assertEqual(record, proof.json.loads((root/'capture-input.private.json').read_text()))

    def test_frozen_and_rollback_clocks_refuse_and_retain_actual_candidate_values(self):
        import tempfile
        from types import SimpleNamespace
        from unittest.mock import patch
        binary=SimpleNamespace(read_bytes=lambda: b'actual synthetic executable')
        for values in [(10,10),(11,10)]:
            with self.subTest(values=values), tempfile.TemporaryDirectory(prefix='conversation-binding-OWNED-') as name:
                root=proof.Path(name)
                (root/'OWNED').write_text('synthetic capture oracle fixture\n')
                with patch.object(proof.time,'time_ns',side_effect=values), self.assertRaises(AssertionError):
                    proof.bind(root,binary,binary,['generated-private-body'])
                actual=proof.json.loads((root/'capture-input.private.json').read_text())
                self.assertEqual((actual['generated_at_ns'],actual['started_at_ns']),values)

if __name__ == '__main__': unittest.main()
