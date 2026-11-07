"""Unit oracles for issue359/360; the production TLS journey is separate."""
import unittest
import copy
import tempfile
from pathlib import Path
import json
import subprocess
from unittest.mock import patch
from types import SimpleNamespace

import probe_conversation_composing as proof


class Interface(unittest.TestCase):
    def test_new_driver_reuses_the_qualified_transport_and_inspectors(self):
        import probe_foreground_conversation as shared
        for name in ('Relay', 'LiveHome', 'certificates', 'native_command',
                     'canonical_snapshot', 'require_cancelled_confirmation'):
            self.assertIs(getattr(proof, name), getattr(shared, name))


A, B = 'alice@home.test', 'bob@away.test'
IDENTITY = 'ab' * 32


def seeded():
    return dict(history=[dict(direction='received', address=A, device=101,
        identity=IDENTITY, subject='token seed', body='unread seed body',
        sent_at=7, operation='00'*16, received_id='12'*32, read=False)],
        pending=[], archive=dict(home=B, local_device=202, epoch='34'*16,
                                after='nonempty-seed-cursor'))


def composed(before=None, operation='56'*16, subject='token queued'):
    state = copy.deepcopy(before if before is not None else seeded())
    state['history'].append(dict(direction='sent', address=A, device=101,
        identity=IDENTITY, subject=subject, body='first line\nsecond λ line',
        sent_at=11, operation=operation, received_id='00'*32, read=None))
    state['pending'].append(dict(operation=operation, address=A, device=101,
                                 sent_at=11))
    return state


class ComposedState(unittest.TestCase):
    def check(self, before, after, subject='token queued'):
        proof.require_composed(before, after, A, 101, subject,
                               'first line\nsecond λ line')

    def test_exact_multiline_append_preserves_unread_seed_and_archive(self):
        self.check(seeded(), composed())

    def test_second_append_preserves_first_queued_operation(self):
        before = composed()
        self.check(before, composed(before, '78'*16, 'unmatched second'),
                   'unmatched second')

    def test_current_peer_identity_is_required_independently_of_address_device(self):
        bad = composed(); bad['history'][-1]['identity'] = 'ef'*32
        with self.assertRaises((AssertionError, ValueError)):
            self.check(seeded(), bad)

    def test_operation_requires_nonzero_canonical_hex_and_unique_sent_association(self):
        for operation in ('00'*16, 'not-an-operation', 'AB'*16):
            with self.subTest(operation=operation):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(seeded(), composed(operation=operation))
        before = seeded()
        retired = composed()['history'][-1]
        before['history'].append(retired)
        with self.assertRaises((AssertionError, ValueError)):
            self.check(before, composed(before))

    def test_pending_timestamp_must_match_the_reviewed_sent_record(self):
        bad = composed(); bad['pending'][-1]['sent_at'] = 12
        with self.assertRaises((AssertionError, ValueError)):
            self.check(seeded(), bad)

    def test_only_an_unread_received_fixture_can_authorize_this_oracle(self):
        for mutate in ('read', 'empty-cursor', 'other-peer'):
            before = seeded()
            if mutate == 'read': before['history'][0]['read'] = True
            if mutate == 'empty-cursor': before['archive']['after'] = None
            if mutate == 'other-peer': before['history'][0]['address'] = 'eve@home.test'
            with self.subTest(mutate=mutate):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(before, composed(before))

    def test_unrelated_state_and_exact_plaintext_cannot_change(self):
        for mutate in ('seed-read', 'cursor', 'seed-body', 'extra-pending', 'body', 'subject'):
            bad = composed()
            if mutate == 'seed-read': bad['history'][0]['read'] = True
            if mutate == 'cursor': bad['archive']['after'] = 'advanced'
            if mutate == 'seed-body': bad['history'][0]['body'] = 'changed'
            if mutate == 'extra-pending': bad['pending'].append(copy.deepcopy(bad['pending'][0]))
            if mutate in ('body', 'subject'): bad['history'][-1][mutate] = 'changed'
            with self.subTest(mutate=mutate):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(seeded(), bad)


def receiver_seed():
    state = seeded()
    state['archive'].update(home=A, local_device=101)
    state['history'][0].update(direction='sent', address=B, device=202,
        identity='cd'*32, read=None, received_id=None, operation='9a'*16)
    state['pending'] = [dict(operation='9a'*16, address=B, device=202, sent_at=7)]
    return state


def received(before=None):
    state = copy.deepcopy(before if before is not None else receiver_seed())
    state['history'].append(dict(direction='received', address=B, device=202,
        identity='cd'*32, subject='chosen second', body='one\n\n.done\nlast λ',
        sent_at=11, operation='00'*16, received_id='ef'*32, read=None))
    return state


class ReceivedState(unittest.TestCase):
    def check(self, before, after):
        proof.require_received(before, after, 'chosen second', 'one\n\n.done\nlast λ')

    def test_A_accepts_B_exact_multiline_input_after_only_its_own_sent_seed(self):
        self.check(receiver_seed(), received())

    def test_authenticated_peer_content_and_unchanged_local_state_are_required(self):
        for mutate in ('identity', 'address', 'device', 'body', 'subject',
                       'received_id', 'seed', 'cursor', 'pending', 'duplicate',
                       'extra', 'direction', 'read'):
            bad = received()
            if mutate in ('identity', 'address', 'body', 'subject'):
                bad['history'][-1][mutate] = 'wrong'
            if mutate == 'device': bad['history'][-1]['device'] = 999
            if mutate == 'received_id': bad['history'][-1]['received_id'] = '00'*32
            if mutate == 'seed': bad['history'][0]['body'] = 'changed'
            if mutate == 'cursor': bad['archive']['after'] = 'advanced'
            if mutate == 'pending': bad['pending'] = []
            if mutate == 'duplicate': bad['history'].append(copy.deepcopy(bad['history'][-1]))
            if mutate == 'extra': bad['history'].append(composed()['history'][-1])
            if mutate == 'direction': bad['history'][-1]['direction'] = 'sent'
            if mutate == 'read': bad['history'][-1]['read'] = True
            with self.subTest(mutate=mutate):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(receiver_seed(), bad)


class PairDispatch(unittest.TestCase):
    def test_A_receiver_is_freshly_bound_before_B_sender_can_confirm(self):
        events = []
        class Relay:
            generations = [3, 7]
            def wait_bound(self, actor, after, timeout):
                self.observed = (actor, after, timeout)
                events.append('bound-A')
                return 4
        relay = Relay()
        def confirm(actor):
            if actor == 'B': self.assertIn('bound-A', events)
            events.append(actor)
        self.assertEqual(proof.connect_pair('A', 'B', relay, confirm), 4)
        self.assertEqual(events, ['A', 'bound-A', 'B'])
        self.assertEqual(relay.observed, (0, 3, 2))

    def test_no_fresh_A_binding_never_confirms_B(self):
        events = []
        class Relay:
            generations = [3, 7]
            def wait_bound(self, actor, after, timeout):
                raise TimeoutError('fresh A unavailable')
        with self.assertRaises(TimeoutError):
            proof.connect_pair('A', 'B', Relay(), events.append)
        self.assertEqual(events, ['A'])


def message(operation='78'*16, payload='AQID', full=True):
    destination = A + ('/faction101' if full else '')
    return (f"<message xmlns='jabber:client' from='{B}' to='{destination}' "
            f"id='{operation}' type='chat'><encrypted xmlns='urn:xmpp:omemo:2'>"
            "<header sid='202'><keys jid='alice@home.test'><key rid='101'>BAUG"
            "</key></keys></header><payload>"+payload+"</payload></encrypted>"
            "<store xmlns='urn:xmpp:hints'/></message>").encode()


class PersistedWire(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='composing-wire-OWNED-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve(); self.root.chmod(0o700)
        from probe_foreground_conversation import MARKER
        (self.root/'OWNED').write_bytes(MARKER)

    def relay(self, expected=None):
        relay = proof.ReleaseRelay(self.root)
        relay.expected_release = (1, '78'*16,
                                  expected if expected is not None else message(full=False))
        relay.register(0, A+'/faction101', lambda frame: None)
        return relay

    def test_exact_persisted_ciphertext_allows_only_the_qualified_full_routing_change(self):
        relay = self.relay()
        self.assertTrue(relay.message(1, message()))
        self.assertEqual(relay.messages, [dict(actor=1, operation='78'*16,
            to=A+'/faction101', bound_generation=1)])

    def test_changed_ciphertext_operation_or_other_pending_item_is_rejected_before_archive(self):
        for raw in (message(payload='Bw=='), message(operation='56'*16),
                    message(operation='56'*16, payload='CA=='),
                    message().replace(b'BAUG', b'CQoL')):
            relay = self.relay()
            with self.subTest(raw=raw):
                with self.assertRaises(ValueError): relay.message(1, raw)
                self.assertEqual(relay.inventory(0), [])
                self.assertEqual(relay.messages, [])

    def test_bare_archive_write_and_wrong_pending_capture_cannot_qualify_live_release(self):
        for expected, actual in ((message(full=False), message(full=False)),
                (message(operation='56'*16, full=False), message()),
                (message(payload='CA==', full=False), message())):
            relay = self.relay(expected)
            with self.subTest(expected=expected, actual=actual):
                with self.assertRaises(ValueError): relay.message(1, actual)
                self.assertEqual(relay.inventory(0), [])

    def test_the_bootstrap_without_a_release_expectation_uses_the_unchanged_relay(self):
        relay = proof.ReleaseRelay(self.root)
        self.assertFalse(relay.message(1, message(full=False)))
        self.assertEqual(len(relay.inventory(0)), 1)

    def test_rejected_actual_wire_is_retained_before_the_comparison_fails(self):
        relay = self.relay(); raw = message(payload='changed')
        with self.assertRaises(ValueError): relay.message(1, raw)
        captures = list(self.root.glob('release-attempt-*.private'))
        self.assertEqual(len(captures), 1)
        self.assertEqual(captures[0].read_bytes(), raw)
        self.assertEqual(captures[0].stat().st_mode & 0o777, 0o600)


def review(identity=IDENTITY):
    return (b'[LOCAL BRIEF EXCHANGE]\r\n[LOCAL BRIEF REVIEW] Page 1/1\r\n'
            b'| Home: '+B.encode()+b'\r\n| Peer: '+A.encode()+
            b'\r\n| Peer device: 101\r\n| Current identity:\r\n| '+identity.encode()+
            b'\r\n[R] Receive briefly\r\n[Q] Choose queued text\r\n[B] Return\r\n')


def picker(selected=2):
    return (b'[LOCAL QUEUED TEXT]\r\nPage 1/1; selection '+
            (b'1' if selected == 2 else b'2')+b'/2\r\n'+
            (b'> ' if selected == 2 else b'  ')+b'02 token queued\r\n'+
            (b'> ' if selected == 3 else b'  ')+b'03 chosen second\r\n'
            b'[Up/Down J/K] Select  [N/P] Page\r\n'
            b'[Enter/O] Review selected text\r\n[B] Return\r\n')


def selected_review(operation='78'*16, record=3):
    return (b'[LOCAL BRIEF REVIEW] Page 1/1\r\n| Send selected text only.\r\n'
            b'| Retained record: '+str(record).encode()+b'\r\n| Operation:\r\n| '+
            operation.encode()+b'\r\n| Subject: chosen second\r\n'
            b'Type exchange to connect, or Enter: ')


class ScriptedConsole:
    """Finite prompt producer: any unexpected key is an immediate failure."""
    def __init__(self, frames):
        self.frames = list(frames); self.buffer = b''; self.transcript = b''
        self.fd = -1; self.sent = []

    def raw(self, key):
        self.sent.append(key)
        if not self.frames: raise AssertionError('extra UI input')
        expected, frame = self.frames.pop(0)
        if expected != key: raise AssertionError(f'unexpected key {key!r}, expected {expected!r}')
        self.buffer += frame

    def expect(self, value):
        token = value.encode() if isinstance(value, str) else value
        if token not in self.buffer: raise AssertionError('unobserved prompt')
        self.buffer = self.buffer.split(token, 1)[1]

    def send(self, value):
        raise AssertionError('arm must not confirm or enter a credential')


class QueuedReview(unittest.TestCase):
    def console(self, initial=None, moved=None, selected=None):
        return ScriptedConsole([(b'e', review() if initial is None else initial),
            (b'q', picker()), (b'j', picker(3) if moved is None else moved),
            (b'o', selected_review() if selected is None else selected)])

    def arm(self, console):
        return proof.arm_queued(console, B, A, 101, IDENTITY, '78'*16)

    def test_explicit_second_picker_row_is_reviewed_without_confirmation_or_password(self):
        console = self.console()
        captured = self.arm(console)
        self.assertEqual(console.sent, [b'e', b'q', b'j', b'o'])
        self.assertFalse(console.frames)
        self.assertIn(b'Retained record: 3', captured)
        self.assertIn(('78'*16).encode(), captured)

    def test_wrong_current_peer_is_rejected_before_opening_the_picker(self):
        console = self.console(initial=review('ef'*32))
        with self.assertRaises(ValueError): self.arm(console)
        self.assertEqual(console.sent, [b'e'])

    def test_picker_must_really_move_to_the_second_original_record(self):
        console = self.console(moved=picker(2))
        with self.assertRaises(ValueError): self.arm(console)
        self.assertEqual(console.sent, [b'e', b'q', b'j'])

    def test_stale_or_swapped_operation_and_record_cannot_reach_confirmation(self):
        for selected in (selected_review('56'*16), selected_review(record=2)):
            console = self.console(selected=selected)
            with self.subTest(selected=selected):
                with self.assertRaises(ValueError): self.arm(console)
                self.assertEqual(console.sent, [b'e', b'q', b'j', b'o'])

    def test_operation_and_record_review_fields_are_exact_not_prefix_matches(self):
        for selected in (selected_review(record=30), selected_review('78'*16+'ff')):
            console = self.console(selected=selected)
            with self.subTest(selected=selected):
                with self.assertRaises(ValueError): self.arm(console)


class WriterConsole(ScriptedConsole):
    def __init__(self, queue=True, identity=IDENTITY, final_identity=IDENTITY,
                 body='one\n\n.done\nlast λ'):
        self.queue = queue; self.body = body
        target = lambda pin: ('CURRENT recipient: '+A+'\r\nCURRENT device: 101 / ed25519:'+pin+'\r\n').encode()
        rows = ''.join('| '+row.replace('λ', '\\u{3bb}')+'\r\n' for row in body.split('\n'))
        events = [(b'w', target(identity)+b'Subject (128 UTF-8 bytes): '),
            ('chosen second', b'[LOCAL MULTILINE WRITER]\r\nwrite> ')]
        events += [(('.'+row if row.startswith('.') else row),
                    b'Accepted: 1 lines; 1 bytes\r\nwrite> ') for row in body.split('\n')]
        events += [('.done', b'Review local multiline intent:\r\n'+target(final_identity)+
            b'Subject: chosen second\r\n[LOCAL BODY] Page 1/1\r\n'+rows.encode()+
            b'Type queue to persist encrypted ciphertext: '),
            ('queue' if queue else '', b'Queued durably;' if queue else b'Writing cancelled;')]
        super().__init__(events)

    def send(self, value):
        self.raw(value)


class FreshWriter(unittest.TestCase):
    def write(self, console):
        return proof.write_conversation(console, A, IDENTITY, 'chosen second',
                                        console.body, console.queue)

    def test_W_uses_empty_start_lines_literal_dot_escaping_and_fresh_queue_confirmation(self):
        console = WriterConsole(); captured = self.write(console)
        self.assertEqual(console.sent, [b'w', 'chosen second', 'one', '', '..done',
                                      'last λ', '.done', 'queue'])
        self.assertFalse(console.frames)
        self.assertIn(b'| .done\r\n', captured)

    def test_blank_queue_confirmation_cancels_after_the_complete_review(self):
        console = WriterConsole(queue=False); self.write(console)
        self.assertEqual(console.sent[-1], '')
        self.assertFalse(console.frames)

    def test_wrong_initial_peer_never_sends_a_subject_or_body(self):
        console = WriterConsole(identity='ef'*32)
        with self.assertRaises((AssertionError, ValueError)): self.write(console)
        self.assertEqual(console.sent, [b'w'])

    def test_replaced_peer_at_final_review_never_sends_queue(self):
        console = WriterConsole(final_identity='ef'*32)
        with self.assertRaises((AssertionError, ValueError)): self.write(console)
        self.assertEqual(console.sent[-1], '.done')


class PendingWireCapture(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='composing-copy-OWNED-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve(); self.root.chmod(0o700)
        from probe_foreground_conversation import MARKER
        (self.root/'OWNED').write_bytes(MARKER)
        self.store = self.root/'live'; self.store.mkdir(mode=0o700)
        self.canonical = self.store/'mail.fkv'
        self.raw = b'exact synthetic encrypted canonical'
        self.canonical.write_bytes(self.raw); self.canonical.chmod(0o600)
        self.state = composed(composed(), '78'*16, 'chosen second')
        self.rows = [dict(operation=op, message=message(op, full=False).hex())
                     for op in ('56'*16, '78'*16)]
        self.calls = []

    def capture(self, rows=None, code=0, stderr=b''):
        output = json.dumps(self.rows if rows is None else rows).encode()
        def run(argv, **options):
            self.calls.append((argv, options))
            copy = Path(argv[2]); self.assertNotEqual(copy, self.store)
            self.assertEqual((copy/'mail.fkv').read_bytes(), self.raw)
            if argv[1] == 'inspect':
                return subprocess.CompletedProcess(argv, 0, json.dumps(self.state).encode(), b'')
            if argv[1] == 'archive-status':
                return subprocess.CompletedProcess(argv, 0, json.dumps(dict(ready=True,
                    checkpoint=self.state['archive'])).encode(), b'')
            self.assertEqual(argv[1], 'pending-wires')
            self.assertEqual(options['input'], b'checkpoint synthetic fixture password\n')
            return subprocess.CompletedProcess(argv, code, output, stderr)
        with patch('probe_foreground_conversation.subprocess.run', side_effect=run):
            return proof.capture_pending_wires(self.root, self.store,
                Path('/owned/inspector'), Path('/owned/mail-fixture'))

    def test_only_an_exact_owned_copy_is_inspected_and_original_JSON_is_retained(self):
        wires = self.capture()
        self.assertEqual(wires, {row['operation']:bytes.fromhex(row['message']) for row in self.rows})
        self.assertEqual([argv[1] for argv, _ in self.calls],
                         ['inspect', 'archive-status', 'pending-wires'])
        self.assertEqual(self.canonical.read_bytes(), self.raw)
        captures = list(self.root.glob('pending-wires-*.private.json'))
        self.assertEqual(len(captures), 1)
        data = json.loads(captures[0].read_text())
        self.assertEqual(bytes.fromhex(data['stdout_hex']), json.dumps(self.rows).encode())

    def test_unknown_duplicate_missing_mismatched_or_noncanonical_wire_data_is_refused(self):
        variants = [self.rows[:1], self.rows+[self.rows[0]],
            [dict(self.rows[0], operation='9a'*16), self.rows[1]],
            [dict(self.rows[0], message=self.rows[1]['message']), self.rows[1]],
            [dict(self.rows[0], extra='untrusted'), self.rows[1]],
            [dict(operation=self.rows[0]['operation']), self.rows[1]],
            [dict(self.rows[0], message=self.rows[0]['message'].upper()), self.rows[1]],
            [dict(self.rows[0], message='ff'*16385), self.rows[1]],
            [dict(self.rows[0], message=message('56'*16).replace(B.encode(), b'eve@away.test').hex()), self.rows[1]]]
        for rows in variants:
            with self.subTest(rows=rows):
                with self.assertRaises(ValueError): self.capture(rows)
                self.assertEqual(self.canonical.read_bytes(), self.raw)

    def test_nonzero_or_noisy_inspector_retains_output_and_fails(self):
        for code, error in ((1, b'private failure'), (0, b'private noise')):
            with self.subTest(code=code, error=error):
                with self.assertRaises(ValueError): self.capture(code=code, stderr=error)


class JourneyBoundary(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='composing-boundary-OWNED-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve(); self.root.chmod(0o700)
        from probe_foreground_conversation import MARKER
        (self.root/'OWNED').write_bytes(MARKER)
        self.binaries = {}
        for name in ('client', 'mail_fixture', 'inspector'):
            path = self.root/name; path.write_bytes(('frozen-'+name).encode()); path.chmod(0o700)
            self.binaries[name] = path

    def arguments(self):
        return SimpleNamespace(owned_root=self.root, unicode=False,
                               openssl='/owned/openssl', **self.binaries)

    def test_executed_binary_and_driver_bytes_are_copied_before_any_home_can_start(self):
        import probe_foreground_conversation as shared
        class StopBeforeService(Exception): pass
        def stop(*args, **kwargs): raise StopBeforeService('no service was started')
        def certificates(*args, **kwargs):
            return (self.root/'unused.pem', self.root/'unused.key'), self.root/'unused-ca.pem'
        with patch.object(shared, 'LiveHome', side_effect=stop), \
                patch.object(proof, 'LiveHome', side_effect=stop), \
                patch.object(shared, 'certificates', side_effect=certificates), \
                patch.object(proof, 'certificates', side_effect=certificates):
            with self.assertRaises(StopBeforeService): proof.run_journey(self.arguments())
        roots = [path for path in self.root.iterdir() if path.is_dir()]
        self.assertEqual(len(roots), 1)
        binding = json.loads((roots[0]/'bindings.private.json').read_text())
        for name, source in self.binaries.items():
            executed = Path(binding['binaries'][name]['executed'])
            self.assertTrue(executed.is_relative_to(roots[0]))
            self.assertEqual(executed.read_bytes(), source.read_bytes())
        producer = Path(binding['producer']['object'])
        self.assertEqual(producer.read_bytes(), Path(proof.__file__).read_bytes())
        teardown = json.loads((roots[0]/'teardown.private.json').read_text())
        self.assertTrue(teardown['workers_joined']); self.assertTrue(teardown['child_processes_reaped'])

    def test_missing_binary_is_refused_before_a_home_or_child_is_created(self):
        arguments = self.arguments(); arguments.client = self.root/'missing'
        with patch.object(proof, 'LiveHome') as home:
            with self.assertRaises(OSError): proof.run_journey(arguments)
        home.assert_not_called()


COMBINED_FOOTER = b'[W] Write fresh text  [B] Back'


def conversation_screen(footer=COMBINED_FOOTER):
    return (b'[LOCAL CONVERSATION]\r\nPeer: alice@home.test\r\n'
        b'D101 / Unread / Search\r\nPage 1/1; selection 1/1\r\n'
        b'> 01 RECEIVED UNREAD\r\nUTC 7\r\nSubject: token seed\r\n'
        b'| seed body\r\n| \r\n| \r\n[O] Full record\r\n'
        b'[E] Brief exchange\r\n'+footer)


class CombinedFooter(unittest.TestCase):
    def check(self, raw):
        return proof.require_composing_screen(raw, [1], 1, (1, 1), 'Unread', True, 101)

    def test_exact_combined_footer_projects_only_that_row_without_changing_row_count(self):
        raw = conversation_screen()
        projected = self.check(raw)
        self.assertEqual(projected, raw.replace(COMBINED_FOOTER, b'[B] Back'))
        self.assertEqual(projected.count(b'\r\n'), raw.count(b'\r\n'))

    def test_previous_output_and_body_text_are_never_normalized(self):
        raw = (COMBINED_FOOTER+b'\r\nprevious output\r\n'+conversation_screen().replace(
               b'| seed body', b'| '+COMBINED_FOOTER))
        projected = self.check(raw)
        expected = raw[:-len(COMBINED_FOOTER)]+b'[B] Back'
        self.assertEqual(projected, expected)
        self.assertIn(b'| '+COMBINED_FOOTER, projected)

    def test_arbitrary_duplicate_missing_malformed_or_extra_footer_content_is_rejected(self):
        footers = [b'[X] Arbitrary command  [B] Back', b'[B] Back', b'',
            COMBINED_FOOTER+b'\r\n'+COMBINED_FOOTER,
            COMBINED_FOOTER+b'\r\n[B] Back', COMBINED_FOOTER+b' suffix',
            b' '+COMBINED_FOOTER, COMBINED_FOOTER+b' ',
            COMBINED_FOOTER.replace(b'  [B]', b' [B]'),
            COMBINED_FOOTER+b'\r\n[X] Other action',
            COMBINED_FOOTER+b'\x1b[31m', COMBINED_FOOTER+b'\t',
            COMBINED_FOOTER+'\u202e'.encode()]
        for footer in footers:
            with self.subTest(footer=footer):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(conversation_screen(footer))

    def test_raw_width_height_scalars_and_original_state_checks_stay_strict(self):
        raw = conversation_screen()
        variants = [raw.replace(b'| seed body', b'| '+b'x'*38),
            raw.replace(b'| seed body', b'| bad\x1b[2J'),
            raw.replace(b'| seed body', b'| bad'+b'\xc2\x85'),
            raw.replace(b'D101 / Unread / Search', b'D202 / Unread / Search'),
            raw.replace(b'D101 / Unread / Search', b'D101 / All / Search'),
            raw.replace(b'D101 / Unread / Search', b'D101 / Unread'),
            raw.replace(b'> 01 RECEIVED', b'> 02 RECEIVED'),
            raw.replace(b'> 01 RECEIVED', b'  01 RECEIVED'),
            raw.replace(b'Page 1/1;', b'Page 2/2;'),
            raw.replace(b'| seed body', b'| extra\r\n| seed body'),
            raw.replace(b'[E] Brief exchange', b'\r\n'*24+b'[E] Brief exchange')]
        for bad in variants:
            with self.subTest(raw=bad):
                with self.assertRaises((AssertionError, ValueError)):
                    self.check(bad)

    def test_raw_and_projected_inputs_are_retained_for_the_actual_comparison(self):
        from probe_foreground_conversation import MARKER
        with tempfile.TemporaryDirectory(prefix='composing-footer-OWNED-') as name:
            root = Path(name).resolve(); root.chmod(0o700); (root/'OWNED').write_bytes(MARKER)
            raw = conversation_screen()
            projected = proof.require_composing_screen(raw, [1], 1, (1, 1),
                'Unread', True, 101, evidence_root=root)
            captures = list(root.glob('conversation-projection-*.private.json'))
            self.assertEqual(len(captures), 1)
            capture = json.loads(captures[0].read_text())
            self.assertEqual(bytes.fromhex(capture['raw_hex']), raw)
            self.assertEqual(bytes.fromhex(capture['projected_hex']), projected)
            self.assertEqual(captures[0].stat().st_mode & 0o777, 0o600)


if __name__ == '__main__':
    unittest.main()
