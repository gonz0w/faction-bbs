"""Exact observer contracts for the bounded #363/#364 host proof."""
import copy
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

import probe_brief_review as proof


def record():
    return dict(direction='sent', subject='same twin subject', body='selected 00 λ\u202e\nselected 01',
                operation='ab'*16, received_id=None, read=None)


def review(unicode=False):
    lam = 'λ' if unicode else '\\u{3bb}'
    return ('[LOCAL BRIEF REVIEW] Page 1/1\r\n'
            '| Send selected text only.\r\n| Retained record: 3\r\n| Operation:\r\n'
            '| '+('ab'*16)+'\r\n| Subject: same twin subject\r\n'
            '[DEVICE-LOCAL SELECTED TEXT]\r\nBody: \r\n[LOCAL BODY] Page 1/1\r\n'
            '| selected 00 '+lam+'\\u{202e}\r\n| selected 01\r\n'
            'Type exchange to connect, or Enter: ').encode()


class Interfaces(unittest.TestCase):
    def test_qualified_helpers_remain_the_same_objects(self):
        import probe_foreground_conversation as transport
        import probe_conversation_composing as composing
        for name in ('LiveHome', 'certificates', 'native_command', 'canonical_snapshot',
                     'require_cancelled_confirmation'):
            self.assertIs(getattr(proof, name), getattr(transport, name))
        for name in ('ReleaseRelay', 'capture_pending_wires', 'require_composing_screen'):
            self.assertIs(getattr(proof, name), getattr(composing, name))


class SelectedReview(unittest.TestCase):
    def test_exact_selected_body_operation_subject_and_page_sequence(self):
        for unicode in (False, True):
            proof.require_selected_review(review(unicode), record(), 3, unicode)

    def test_first_twin_or_operation_or_truncation_or_early_confirmation_is_rejected(self):
        for old, new in ((b'selected 01', b'other twin'), (b'ab'*16, b'cd'*16),
                         (b'Retained record: 3', b'Retained record: 2'),
                         (b'same twin subject', b'wrong subject'),
                         (b'Page 1/1', b'Page 2/2'),
                         (b'| selected 01\r\n', b''),
                         (b'[LOCAL BODY]', b'Type exchange to connect, or Enter: [LOCAL BODY]')):
            with self.subTest(old=old), self.assertRaises(ValueError):
                proof.require_selected_review(review().replace(old, new), record(), 3, False)

    def test_preceding_conversation_snippets_cannot_supply_missing_selected_body(self):
        bodyless = review().replace(b'| selected 01\r\n', b'')
        with self.assertRaises(ValueError):
            proof.require_selected_review(b'[LOCAL CONVERSATION]\r\n| selected 01\r\n'+bodyless,
                                          record(), 3, False)

    def test_raw_bidi_and_oversized_body_rows_are_rejected(self):
        for raw in (review().replace(b'\\u{202e}', '\u202e'.encode()),
                    review().replace(b'| selected 01', b'| '+b'x'*40)):
            with self.assertRaises(ValueError):
                proof.require_selected_review(raw, record(), 3, False)


def arrival_screen():
    return (b'[LOCAL NEWLY RECEIVED TEXT]\r\nPage 1/1; selection 1/1\r\n'
            b'> 03 UNREAD\r\nhidden twin\r\n[Up/Down J/K] Select  [N/P] Page\r\n'
            b'[Enter/O] Open full record\r\n[B] Return\r\n')


class Arrivals(unittest.TestCase):
    def test_exact_canonical_arrival_and_status_with_no_body_or_reply(self):
        proof.require_arrival_screen(arrival_screen(), 3, 'UNREAD', 'hidden twin', False)

    def test_wrong_existing_index_status_body_reply_and_unsafe_width_are_rejected(self):
        for old, new in ((b'03', b'01'), (b'UNREAD', b'READ'),
                         (b'[B] Return', b'| hidden body\r\n[B] Return'),
                         (b'[B] Return', b'[R] Reply locally\r\n[B] Return'),
                         (b'hidden twin', b'x'*40), (b'Page 1/1', b'Page 2/2')):
            with self.subTest(old=old), self.assertRaises(ValueError):
                proof.require_arrival_screen(arrival_screen().replace(old, new),
                                             3, 'UNREAD', 'hidden twin', False)

    def test_only_one_bound_authenticated_local_read_bit_may_change(self):
        before = dict(history=[dict(direction='received', received_id='12'*32, read=False),
                               dict(direction='received', received_id='34'*32, read=False)],
                      pending=[dict(operation='ab'*16)], archive=dict(after='unchanged'))
        after = copy.deepcopy(before); after['history'][1]['read'] = True
        proof.require_local_read(before, after, 1, '34'*32)
        for bad in ('other-read', 'identity', 'pending', 'cursor', 'unknown'):
            b, a = copy.deepcopy(before), copy.deepcopy(after)
            if bad == 'other-read': a['history'][0]['read'] = True
            if bad == 'identity': a['history'][1]['received_id'] = '56'*32
            if bad == 'pending': a['pending'] = []
            if bad == 'cursor': a['archive']['after'] = 'advanced'
            if bad == 'unknown': b['history'][1]['read'] = None
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                proof.require_local_read(b, a, 1, '34'*32)


class ActorLifecycle(unittest.TestCase):
    def test_hook_runs_after_both_homes_close_before_finish_and_finish_is_once(self):
        events = []
        factory = SimpleNamespace(after_homes_stopped=lambda context: events.append(('stopped', context)),
                                  finish=lambda index, console: events.append(('finish', index)))
        actors = proof.ActorLifecycle(factory)
        actors.add(0, object()); actors.add(1, object())
        with self.assertRaises(ValueError): actors.homes_stopped([False, True], 'context')
        self.assertEqual(events, [])
        actors.homes_stopped([True, True], 'context')
        with self.assertRaises(ValueError): actors.homes_stopped([True, True], 'again')
        actors.finish(); actors.finish()
        self.assertEqual(events, [('stopped', 'context'), ('finish', 0), ('finish', 1)])

    def test_partial_actor_start_and_failed_finish_still_close_every_started_console(self):
        events = []
        def finish(index, console):
            events.append(('finish', index))
            if index == 0: raise ValueError('controlled finish failure')
        actors = proof.ActorLifecycle(SimpleNamespace(finish=finish))
        for index in (0, 1):
            actors.add(index, SimpleNamespace(close=lambda index=index: events.append(('close', index))))
        with self.assertRaises(ValueError): actors.finish()
        actors.finish()
        actors.close()
        self.assertEqual(events, [('finish', 0), ('finish', 1), ('close', 0), ('close', 1)])


class LocalActors(unittest.TestCase):
    def test_default_actor_uses_only_the_provided_local_binary_and_hidden_unlock(self):
        console = Mock()
        member = dict(store='/private/tmp/owned-store', config='/private/tmp/owned-home', local_password='private unlock')
        with patch('smoke_native_client_ssh.NativeConsole', return_value=console) as spawn:
            returned = proof.LocalActors().start(1, None, dict(client='/private/tmp/frozen-client'), member, True)
        self.assertIs(returned, console)
        spawn.assert_called_once_with(['/private/tmp/frozen-client', '--mail-online',
                                      member['store'], member['config'], '--unicode'])
        console.password.assert_called_once_with('private unlock', 'LOCAL checkpoint passphrase (hidden): ')
        console.resize.assert_called_once_with(40, 24)
        proof.LocalActors().finish(1, console)
        console.raw.assert_called_once_with(b'l'); console.finish.assert_called_once_with()

    def test_failed_unlock_closes_the_new_partial_actor_before_it_can_escape_tracking(self):
        console = Mock(); console.password.side_effect = ValueError('controlled unlock failure')
        with patch('smoke_native_client_ssh.NativeConsole', return_value=console):
            with self.assertRaises(ValueError):
                proof.LocalActors().start(0, None, dict(client='/private/tmp/client'),
                    dict(store='/private/tmp/store', config='/private/tmp/home', local_password='private'), False)
        console.close.assert_called_once_with()


class QueuedPicker(unittest.TestCase):
    def screen(self):
        return (b'[LOCAL QUEUED TEXT]\r\nPage 2/2; selection 3/3\r\n'
                b'> 03 same twin subject\r\n[Up/Down J/K] Select  [N/P] Page\r\n'
                b'[Enter/O] Review selected text\r\n[B] Return\r\n')

    def test_canonical_append_order_selection_and_page_are_exact(self):
        proof.require_queued_screen(self.screen(), [(1, 'seed'), (2, 'same twin subject'), (3, 'same twin subject')], 3, False)

    def test_first_twin_substitution_wrong_position_and_extra_body_cannot_qualify(self):
        candidates = [(1, 'seed'), (2, 'same twin subject'), (3, 'same twin subject')]
        for old, new in ((b'> 03', b'> 02'), (b'Page 2/2', b'Page 1/2'),
                         (b'selection 3/3', b'selection 2/3'),
                         (b'[B] Return', b'| copied body\r\n[B] Return')):
            with self.subTest(old=old), self.assertRaises(ValueError):
                proof.require_queued_screen(self.screen().replace(old, new), candidates, 3, False)


class PeerReview(unittest.TestCase):
    footer = 'At most 10 seconds; idle wait 2 seconds.'

    def peer(self, own='alice@home.test', peer='bob@away.test', device='202', identity='ab'*32):
        wrapped = '\r\n'.join('| '+identity[offset:offset+37] for offset in range(0, len(identity), 37))
        return ('[LOCAL BRIEF REVIEW] Page 1/1\r\n| Home: '+own+'\r\n| Peer: '+peer+
                '\r\n| Peer device: '+device+'\r\n| Current identity:\r\n'+wrapped+
                '\r\n'+self.footer+'\r\n[Q] Choose queued text\r\n[B] Return\r\n').encode()

    def test_observed_immutable_40_cell_footer_does_not_expand_untrusted_pane_bounds(self):
        raw = self.peer()
        self.assertEqual(len(self.footer), 40)
        for replacement in (' '+self.footer, self.footer+'x', self.footer+'\r\n'+self.footer,
                            'At most 10 seconds;\r\nidle wait 2 seconds.', '| '+self.footer,
                            self.footer+'\r\n| '+self.footer, self.footer+'\r\n'+'x'*40,
                            self.footer+'\r\n| unsafe\u202e'):
            with self.subTest(replacement=replacement):
                with patch.object(proof, 'read_review', return_value=raw.replace(self.footer.encode(), replacement.encode())):
                    with self.assertRaises(ValueError):
                        proof.current_peer_review(Mock(), 'alice@home.test', 'bob@away.test', 202, 'ab'*32)
        with patch.object(proof, 'read_review', return_value=raw):
            proof.current_peer_review(Mock(), 'alice@home.test', 'bob@away.test', 202, 'ab'*32)

    def test_full_wrapped_peer_values_match_exactly(self):
        with patch.object(proof, 'read_review', return_value=self.peer()):
            proof.current_peer_review(Mock(), 'alice@home.test', 'bob@away.test', 202, 'ab'*32)

    def test_prefix_and_suffix_values_cannot_supply_exact_reviewed_context(self):
        for field, value in (('own', 'alice@home.test.evil'), ('own', 'xalice@home.test'),
                             ('peer', 'bob@away.test.evil'), ('peer', 'xbob@away.test'),
                             ('device', '2020'), ('device', '1202'),
                             ('identity', 'ab'*32+'cd'), ('identity', 'cd'+'ab'*32)):
            with self.subTest(field=field, value=value):
                with patch.object(proof, 'read_review', return_value=self.peer(**{field: value})):
                    with self.assertRaises(ValueError):
                        proof.current_peer_review(Mock(), 'alice@home.test', 'bob@away.test', 202, 'ab'*32)

    def test_device_digits_inside_an_identity_cannot_supply_the_device_label(self):
        identity = '202'+'a'*61
        raw = ('| Home: alice@home.test\r\n| Peer: bob@away.test\r\n'
               '| Peer device: 999\r\n| Current identity:\r\n| '+identity+
               '\r\n[Q] Choose queued text\r\n').encode()
        with patch.object(proof, 'read_review', return_value=raw):
            with self.assertRaises(ValueError):
                proof.current_peer_review(Mock(), 'alice@home.test', 'bob@away.test', 202, identity)


class LocalReviewInterval(unittest.TestCase):
    def states(self):
        before = dict(history=[dict(direction='received', received_id='12'*32, read=False)],
                      pending=[dict(operation='ab'*16)], archive=dict(after='unchanged'))
        after = copy.deepcopy(before); after['history'][0]['read'] = True
        return before, after

    def test_local_review_can_change_only_the_exact_read_bit_with_no_new_connection(self):
        before, after = self.states()
        proof.require_arrival_interval(((3, 3), (5, 5)), ((3, 3), (5, 5)),
                                       before, after, 0, '12'*32)

    def test_any_actor_tcp_or_auth_increment_during_review_is_rejected(self):
        before, after = self.states()
        for actor in (0, 1):
            for counter in (0, 1):
                changed = [[3, 3], [5, 5]]; changed[actor][counter] += 1
                with self.subTest(actor=actor, counter=counter), self.assertRaises(ValueError):
                    proof.require_arrival_interval(((3, 3), (5, 5)), tuple(map(tuple, changed)),
                                                   before, after, 0, '12'*32)
