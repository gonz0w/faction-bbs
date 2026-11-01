"""Contracts for the owned authenticated-provider console proof."""
import importlib
import json
import os
from pathlib import Path
import stat
import shutil
import subprocess
from unittest.mock import patch
import tempfile
import tomllib
import unittest
from types import SimpleNamespace
from xml.etree import ElementTree as ET


class ProviderConsoleProbeTests(unittest.TestCase):
    def test_fixture_target_requires_explicit_provider_feature(self):
        manifest = tomllib.loads((Path(__file__).resolve().parents[1] /
            'crates/faction-client/Cargo.toml').read_text())
        examples = {entry['name']: entry for entry in manifest['example']}
        self.assertIn('device_provider_console_fixture', examples)
        self.assertEqual(examples['device_provider_console_fixture']['required-features'],
                         ['device-provider'])

    def test_provider_readiness_failure_reaps_its_child(self):
        probe = importlib.import_module('probe_device_provider_console')
        created = []
        original = subprocess.Popen
        def record(*args, **kwargs):
            child = original(*args, **kwargs)
            created.append(child)
            return child
        with tempfile.TemporaryDirectory(prefix='faction280-unready-OWNED-') as name:
            root = Path(name).resolve();root.chmod(0o700)
            script = root/'unready'
            script.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n')
            script.chmod(0o700)
            try:
                with patch.object(probe.subprocess, 'Popen', side_effect=record):
                    with self.assertRaises(ValueError):
                        probe.Provider(script, root)
                self.assertEqual(len(created), 1)
                self.assertIsNotNone(created[0].poll(), 'readiness failure left a running child')
                self.assertTrue(created[0].stdout.closed)
                self.assertTrue(created[0].stderr.closed)
            finally:
                for child in created:
                    if child.poll() is None:child.kill()
                    child.wait(timeout=5)
                    child.stdout.close();child.stderr.close()

    def test_probe_cli_requires_explicit_local_inputs(self):
        probe = importlib.import_module('probe_device_provider_console')
        parsed = probe.make_parser().parse_args(['--client', '/client', '--fixture', '/fixture', '--owned-root', '/owned', '--openssl', 'openssl', '--output', '/owned/result.json'])
        self.assertEqual(parsed.client, Path('/client'))
        self.assertEqual(parsed.fixture, Path('/fixture'))
        self.assertEqual(parsed.openssl, 'openssl')
        self.assertEqual(parsed.owned_root, Path('/owned'))
        self.assertEqual(parsed.output, Path('/owned/result.json'))
        self.assertTrue(callable(probe.probe))

    def test_ready_is_one_bounded_public_port(self):
        probe = importlib.import_module('probe_device_provider_console')
        self.assertEqual(probe.parse_ready(b'ready 12345\n'), 12345)
        for raw in [b'ready 0\n', b'ready 65536\n', b'ready 001\n',
                    b'ready 12\nsecret\n', b'private endpoint:123\n', b'x'*8192]:
            with self.subTest(raw=raw[:30]), self.assertRaises(ValueError):
                probe.parse_ready(raw)

    def test_capture_is_private_bounded_and_cannot_escape(self):
        probe = importlib.import_module('probe_device_provider_console')
        with tempfile.TemporaryDirectory(prefix='faction280-probe-test-OWNED-') as name:
            root = Path(name).resolve()
            root.chmod(0o700)
            destination = probe.retain_capture(root, 'failed-console', b'failed private run')
            self.assertEqual(destination.parent, root)
            self.assertEqual(destination.read_bytes(), b'failed private run')
            self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o600)
            for label, data in [('../escape', b'x'), ('secret/path', b'x'),
                                ('oversize', b'x'*(4*1024*1024+1))]:
                with self.subTest(label=label), self.assertRaises(ValueError):
                    probe.retain_capture(root, label, data)
            self.assertEqual(len(list(root.iterdir())), 1)

class OwnHomeFixtureContracts(unittest.TestCase):
    def setUp(self):
        capture = os.environ.get('FACTION322_CONSOLE_FIXTURES')
        if capture is not None:
            parent = Path(capture)
            info = parent.lstat()
            self.assertTrue(not parent.is_symlink() and stat.S_ISDIR(info.st_mode))
            self.assertEqual(info.st_uid, os.getuid())
            self.assertEqual(stat.S_IMODE(info.st_mode), 0o700)
        self.root = Path(tempfile.mkdtemp(prefix='console-contract-OWNED-', dir=capture)).resolve()
        self.root.chmod(0o700)
        if capture is None:
            self.addCleanup(shutil.rmtree, self.root)
        self.probe = importlib.import_module('probe_device_provider_console')
        self.probe.private(self.root/'OWNED', b'Owned issue322 Python fixture contract inputs\n')
        for name, data in [('ca.pem', b'legacy synthetic CA'),
                           ('away-ca.pem', b'independent synthetic CA'),
                           ('server.pem', b'synthetic server certificate'),
                           ('device.pem', b'synthetic device certificate'),
                           ('device.key', b'synthetic device private key')]:
            self.probe.private(self.root/name, data)
        self.pair = (self.root/'device.pem', self.root/'device.key')

    def test_legacy_profile_characterization_stays_same_home_v1(self):
        with patch.object(self.probe, 'fingerprint', return_value=bytes([17])*32):
            path = self.probe.profile(self.root, 'alice', '01'*32, 12345,
                                      self.root/'server.pem', self.pair)
        values = tomllib.loads(path.read_text())
        self.assertEqual(values, dict(version=1, home='home.test', endpoint='127.0.0.1:12345',
            name='home.test', pin='11'*32, account='alice@home.test', device=101,
            identity='01'*32, ca=str(path.parent/'ca.pem'),
            certificate=str(path.parent/'device.pem'), key=str(path.parent/'device.key')))
        self.assertEqual((path.parent/'ca.pem').read_bytes(), b'legacy synthetic CA')
        self.assertEqual(stat.S_IMODE(path.parent.stat().st_mode), 0o700)
        self.assertTrue(all(stat.S_IMODE(p.stat().st_mode) == 0o600 for p in path.parent.iterdir()))


    @staticmethod
    def archive_query(account):
        iq = ET.Element('{jabber:client}iq', {'type':'set', 'id':'archive-fixture',
            'from':account, 'to':account})
        ET.SubElement(iq, '{urn:xmpp:mam:2}query', {'queryid':'archive-fixture'})
        return ET.tostring(iq)

    def delivery(self, recipient):
        wire = ET.Element('{jabber:client}message', {'from':'alice@home.test',
            'to':recipient, 'id':'opaque-fixture-message'})
        ET.SubElement(wire, '{urn:xmpp:omemo:2}encrypted').text = 'opaque encrypted fixture bytes'
        self.probe.private(self.root/'delivery.xml', ET.tostring(wire))
        return wire

    def test_opaque_archive_is_recipient_only_even_at_same_home(self):
        self.delivery('bob@home.test')
        mail = self.probe.PublicMail(self.root)
        sender = ET.fromstring(b'<root>'+mail.request(0, self.archive_query('alice@home.test'))+b'</root>')
        self.assertIsNone(sender.find('.//{urn:xmpp:mam:2}result'),
            'sender archive must not disclose the recipient queue')
        recipient = ET.fromstring(b'<root>'+mail.request(1, self.archive_query('bob@home.test'))+b'</root>')
        self.assertIsNotNone(recipient.find('.//{urn:xmpp:mam:2}result'))

    def test_two_home_mail_uses_exact_accounts_domains_and_archive_owner(self):
        accounts = ('alice@home.test', 'bob@away.test')
        wire = self.delivery(accounts[1])
        mail = self.probe.PublicMail(self.root, accounts=accounts, devices=(101,202),
                                     archive_owner=accounts[1])
        for actor, account, device in [(0,accounts[0],'101'),(1,accounts[1],'202')]:
            iq = ET.Element('{jabber:client}iq', {'type':'get', 'id':'public-fixture',
                'from':account, 'to':accounts[1-actor]})
            pubsub = ET.SubElement(iq, '{http://jabber.org/protocol/pubsub}pubsub')
            ET.SubElement(pubsub, '{http://jabber.org/protocol/pubsub}items',
                {'node':'urn:xmpp:omemo:2:devices'})
            reply = ET.fromstring(mail.request(actor, ET.tostring(iq)))
            self.assertEqual(reply.get('from'), accounts[1-actor])
            self.assertEqual(reply.find('.//{urn:xmpp:omemo:2}device').get('id'),
                             '202' if actor == 0 else '101')
            publish = ET.Element('{jabber:client}iq', {'type':'set','id':'publish-fixture'})
            ET.SubElement(publish, '{http://jabber.org/protocol/pubsub}pubsub')
            self.assertEqual(ET.fromstring(mail.request(actor, ET.tostring(publish))).get('from'),
                             account.split('@')[1])
        sender = ET.fromstring(b'<root>'+mail.request(0,self.archive_query(accounts[0]))+b'</root>')
        self.assertIsNone(sender.find('.//{urn:xmpp:mam:2}result'))
        recipient = ET.fromstring(b'<root>'+mail.request(1,self.archive_query(accounts[1]))+b'</root>')
        archived = recipient.find('.//{urn:xmpp:forward:0}forwarded/{jabber:client}message')
        self.assertIsNotNone(archived)
        self.assertEqual(ET.tostring(archived), ET.tostring(wire))
        self.assertEqual(mail.blob_requests, 0)

    def test_explicit_v2_profile_keeps_only_its_own_endpoint_and_credentials(self):
        with patch.object(self.probe, 'fingerprint', return_value=bytes([34])*32):
            path = self.probe.profile(self.root, 'bob', '02'*32, 23456,
                self.root/'server.pem', self.pair, route='own-home', home='away.test',
                account='bob@away.test', device=202, ca=self.root/'away-ca.pem')
        values = tomllib.loads(path.read_text())
        self.assertEqual(values, dict(version=2, route='own-home', home='away.test',
            endpoint='127.0.0.1:23456', name='away.test', pin='22'*32,
            account='bob@away.test', device=202, identity='02'*32,
            ca=str(path.parent/'ca.pem'), certificate=str(path.parent/'device.pem'),
            key=str(path.parent/'device.key')))
        self.assertEqual((path.parent/'ca.pem').read_bytes(), b'independent synthetic CA')
        self.assertTrue((path.parent/'device.key').read_bytes() == self.pair[1].read_bytes(),
                        'selected device key bytes changed')
        self.assertLessEqual(path.stat().st_size, 8192)
        self.assertTrue(all(stat.S_IMODE(p.stat().st_mode) == 0o600 for p in path.parent.iterdir()))

    def test_mail_settings_select_own_account_domain_ca_and_server_pin(self):
        peer = SimpleNamespace(account='bob@away.test', domain='away.test', port=34567)
        with patch.object(self.probe, 'fingerprint', return_value=bytes([51])*32):
            path = self.probe.mail_settings(self.root, 'bob', peer, self.root/'server.pem',
                                            self.root/'away-ca.pem')
        self.assertEqual(tomllib.loads(path.read_text()), dict(version=1,
            account='bob@away.test', endpoint='127.0.0.1:34567', domain='away.test',
            ca=str(self.root/'away-ca.pem'), pin='33'*32))
        self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)

    def test_probe_cli_has_explicit_new_mode_without_changing_default(self):
        args = ['--client','/client','--fixture','/fixture','--owned-root','/owned',
                '--output','/owned/result.json']
        default = self.probe.make_parser().parse_args(args)
        self.assertEqual(default.route, 'same-home')
        self.assertIsNone(default.operator)
        explicit = self.probe.make_parser().parse_args(args+['--route','own-home','--operator','/operator'])
        self.assertEqual(explicit.route, 'own-home')
        self.assertEqual(explicit.operator, Path('/operator'))

    def test_mode_refusal_happens_before_root_access_or_children(self):
        with patch.object(self.probe, 'require_root', side_effect=AssertionError('root touched')), \
             patch.object(self.probe.subprocess, 'Popen', side_effect=AssertionError('child started')):
            for route, operator in [('own-home',None),('same-home',Path('/operator')),
                                    ('unknown',Path('/operator'))]:
                with self.subTest(route=route), self.assertRaises(ValueError):
                    self.probe.probe(Path('/client'),Path('/fixture'),Path('/owned'),'openssl',
                                     route=route,operator=operator)


class TransferFeed:
    def __init__(self, events=(), initial=b''):
        self.events = list(events)
        self.now = 0.0
        self.cancelled = []
        self.written = []
        self.buffer = initial
        self.transcript = initial
        self.fd = 123
        self.pid = 1 << 30
        self.done = False

    def clock(self):
        return self.now

    def poll(self, readers, writers, errors, timeout):
        if self.events and self.events[0][0] <= self.now+timeout:
            self.now = max(self.now, self.events[0][0])
            return readers, [], []
        self.now += timeout
        return [], [], []

    def read(self, fd, count):
        at, data = self.events.pop(0)
        if len(data) > count:
            self.events.insert(0, (at, data[count:]))
        return data[:count]

    def raw(self, data):
        self.written.append(data)

    def cancel(self, console):
        self.cancelled.append(self.now)
        self.done = True
        return dict(seconds=0.0, reaped=True, killed=False)


class TransferObserverTests(unittest.TestCase):
    TOTAL = 1_048_592
    COMPLETE = b'Foreground transfer complete.\r\n'

    @staticmethod
    def progress(offset, call, writes=0):
        return f'Confirmed ciphertext bytes: {offset}\r\nSDK call: {call} / 65; writes: {writes}\r\n'.encode()

    def observe(self, feed, total=None):
        probe = importlib.import_module('probe_device_provider_console')
        return probe.wait_transfer(feed, self.TOTAL if total is None else total,
                                   started=0.0, clock=feed.clock,
                                   poll=feed.poll, read=feed.read, cancel=feed.cancel)

    def refusal(self, events, total=None):
        feed = TransferFeed(events)
        with self.assertRaises(ValueError) as caught:
            self.observe(feed, total)
        self.assertEqual(len(feed.cancelled), 1)
        self.assertTrue(feed.done)
        return feed, caught.exception.observation

    def test_failure_message_summarizes_injected_deadline_after_drain(self):
        probe = importlib.import_module('probe_device_provider_console')
        for marker, classification in [
                (None, 'observer_timeout_ambiguous'),
                ('budget', 'product_stop_observed'),
                ('completion', 'late_completion_observed')]:
            with self.subTest(marker=marker):
                feed = TransferFeed([(10*n, self.progress(100*n, n)) for n in range(1, 6)]
                                    + [(59, self.progress(600, 6))])
                def drain(console):
                    feed.cancelled.append(feed.now)
                    feed.now += 1
                    feed.done = True
                    return dict(seconds=1.0, reaped=True, killed=False,
                                cancelled=True, marker_before_cancel=marker)
                with self.assertRaises(probe.TransferObservationError) as caught:
                    probe.wait_transfer(feed, self.TOTAL, started=0.0, clock=feed.clock,
                        poll=feed.poll, read=feed.read, cancel=drain)
                observation = caught.exception.observation
                self.assertIs(observation, feed.transfer_observation)
                self.assertEqual(observation['classification'], classification)
                self.assertEqual((feed.cancelled, feed.now, feed.done), ([60], 61, True))
                message = str(caught.exception)
                for field in ['reason=work_deadline', 'classification='+classification,
                              'calls=6', 'offset=600', 'elapsed=60.0',
                              'last_progress=59.0', 'drain_seconds=1.0',
                              'cancelled=true', 'killed=false', 'reaped=true',
                              'marker='+('none' if marker is None else marker)]:
                    self.assertIn(field, message)
                self.assertNotIn('Confirmed ciphertext', message)

    def test_failure_summary_refuses_unknown_or_malformed_private_fields(self):
        probe = importlib.import_module('probe_device_provider_console')
        secret = 'SECRET-private-path-credential-transcript'
        class Unknown:
            def __str__(self):
                raise AssertionError('unknown coercion')
            def __repr__(self):
                raise AssertionError('unknown repr')
        class UnknownDict(dict):
            def get(self, *args):
                raise AssertionError('unknown mapping')
        class UnknownString(str):
            def __hash__(self):
                raise AssertionError('unknown hash')
        baseline = dict(reason='work_deadline', classification='observer_timeout_ambiguous',
                        calls=6, offset=600, seconds=60.0,
                        progress=[dict(seconds=59.0)],
                        drain=dict(seconds=1.0, cancelled=True, killed=False,
                                   reaped=True, marker_before_cancel=None))
        cases = []
        for field, values in [
                ('reason', [secret, Unknown(), UnknownString(secret)]),
                ('classification', [secret, Unknown()]),
                ('calls', [-1, 66, True, 1.5, secret, Unknown()]),
                ('offset', [-1, self.TOTAL+1, False, 1.5, secret, Unknown()]),
                ('seconds', [-1, float('inf'), float('nan'), True, secret, Unknown()]),
                ('progress', [secret, Unknown(), [dict(seconds=secret)],
                              [dict(seconds=float('inf'))], [dict(seconds=-1)]]),
                ('drain', [secret, Unknown(), UnknownDict()])]:
            for value in values:
                cases.append(dict(baseline, **{field:value}))
        for field in ['seconds', 'cancelled', 'killed', 'reaped', 'marker_before_cancel']:
            cases.append(dict(baseline, drain=dict(baseline['drain'], **{field:secret})))
        cases += [Unknown(), UnknownDict(), {}]
        for observation in cases:
            with self.subTest(case=len(cases)):
                error = probe.TransferObservationError(observation)
                self.assertIs(error.observation, observation)
                message = str(error)
                self.assertNotIn(secret, message)
                self.assertIn('unavailable', message)
                self.assertLess(len(message), 2048)
        decorated = dict(baseline, **{secret:secret})
        decorated['progress'] = [dict(seconds=59.0, transcript=secret)]
        error = probe.TransferObservationError(decorated)
        self.assertNotIn(secret, str(error))
        self.assertIn('calls=6', str(error))
        # Even a formatter defect must preserve the already completed drain and
        # the original structured observation, rather than replace the failure.
        feed = TransferFeed()
        with patch.object(probe, '_transfer_failure_summary', side_effect=RuntimeError(secret)):
            with self.assertRaises(probe.TransferObservationError) as caught:
                self.observe(feed)
        self.assertEqual((feed.cancelled, feed.done), ([15], True))
        self.assertIs(caught.exception.observation, feed.transfer_observation)
        self.assertIn('unavailable', str(caught.exception))
        self.assertNotIn(secret, str(caught.exception))

    DIAGNOSTIC = (
        b'Attachment diagnostic v1 direction=fetch origin=sdk_deadline error=unavailable '
        b'state=observed request=get last_phase=collect_wait transport=unavailable '
        b'sdk_completed=52 sdk_active=53 aggregate_valid=true totals_clamped=false '
        b'observed_logical_completed=263 observed_logical_attempted=264 observed_carrier_attempts=526 '
        b'observed_busy=1 observed_connect_calls=525 unavailable_cycles=0 '
        b'cycle_completed=3 cycle_logical_attempted=4 cycle_carrier_attempts=7 cycle_busy=1 cycle_connect_calls=6 '
        b'confirmed_offset=851968 foreground_us=60000000 last_phase_us=7 '
        b'observed_submit_connect_us=100 observed_submit_exchange_us=200 '
        b'observed_collect_connect_us=300 observed_collect_exchange_us=400 '
        b'observed_planned_wait_us=19 observed_actual_wait_us=7\r\n')

    def test_failure_captures_valid_phase_fields_after_original_drain(self):
        feed=TransferFeed([(1,self.progress(4096,1)),
            (2,self.DIAGNOSTIC+b'Transfer stopped; no automatic retry.\r\n')])
        with self.assertRaises(ValueError) as caught:self.observe(feed)
        self.assertEqual((feed.cancelled,feed.done),([2],True))
        self.assertEqual((caught.exception.observation['calls'],caught.exception.observation['offset']),(1,4096))
        self.assertIn('diag_last_phase=collect_wait',str(caught.exception))
        self.assertIn('diag_origin=sdk_deadline',str(caught.exception))
        self.assertIn('diag_observed_logical_completed=263',str(caught.exception))
        self.assertIn('mode=classic',str(caught.exception))
        self.assertNotIn('Confirmed ciphertext bytes',str(caught.exception))

    def test_failure_mode_comes_only_from_fixed_boolean_caller(self):
        probe=importlib.import_module('probe_device_provider_console')
        for unicode in (False,True):
            feed=TransferFeed([(1,b'mode=SECRET-transcript-mode\r\n'+self.DIAGNOSTIC+
                b'Transfer stopped; no automatic retry.\r\n')])
            with self.assertRaises(probe.TransferObservationError) as caught:
                probe.wait_transfer(feed,self.TOTAL,started=0.0,clock=feed.clock,poll=feed.poll,
                    read=feed.read,cancel=feed.cancel,unicode=unicode)
            self.assertIn('mode='+('unicode' if unicode else 'classic'),str(caught.exception))
            self.assertNotIn('SECRET',str(caught.exception))
            self.assertEqual(feed.cancelled,[1])
        for unicode in ('SECRET',1,None):
            feed=TransferFeed()
            with self.assertRaises(ValueError):
                probe.wait_transfer(feed,self.TOTAL,clock=feed.clock,unicode=unicode)
            self.assertEqual(feed.cancelled,[])

    def test_secret_padded_unknown_or_malformed_diagnostic_is_unavailable(self):
        mutations=[self.DIAGNOSTIC.replace(b'collect_wait',b'SECRET-path-key'),
            self.DIAGNOSTIC.replace(b'cycle_completed=3',b'cycle_completed=9'),
            self.DIAGNOSTIC.replace(b'cycle_completed=3',b'cycle_completed=-1'),
            self.DIAGNOSTIC.replace(b'cycle_completed=3',b'cycle_completed=03'),
            self.DIAGNOSTIC.replace(b'foreground_us=60000000',b'foreground_us=NaN'),
            self.DIAGNOSTIC.replace(b'last_phase_us=7',b'last_phase_us=18446744073709551616'),
            self.DIAGNOSTIC.replace(b'sdk_completed=52',b'sdk_completed=66'),
            self.DIAGNOSTIC.replace(b'observed_connect_calls=525',b'observed_connect_calls=4161'),
            self.DIAGNOSTIC.replace(b'aggregate_valid=true',b'aggregate_valid=SECRET'),
            self.DIAGNOSTIC.replace(b'\r\n',b' SECRET-extra-private-field\r\n'),
            self.DIAGNOSTIC.replace(b' direction=',b'  direction='),
            self.DIAGNOSTIC+self.DIAGNOSTIC,
            b'Attachment diagnostic v1 '+b'SECRET'*250+b'\r\n']
        for diagnostic in mutations:
            feed=TransferFeed([(1,diagnostic+b'Transfer stopped; no automatic retry.\r\n')])
            with self.assertRaises(ValueError) as caught:self.observe(feed)
            self.assertEqual(feed.cancelled,[1])
            self.assertIn('diagnostic=unavailable',str(caught.exception))
            self.assertNotIn('SECRET',str(caught.exception))
            self.assertLessEqual(len(str(caught.exception)),2048)

    def test_structured_diagnostic_is_revalidated_without_unknown_repr(self):
        probe=importlib.import_module('probe_device_provider_console')
        class Unknown:
            def __str__(self):raise AssertionError('unknown coercion')
            def __repr__(self):raise AssertionError('unknown repr')
        valid=probe._parse_transfer_diagnostic(self.DIAGNOSTIC)
        self.assertIsNotNone(valid)
        variants=['SECRET',Unknown(),dict(valid,private='SECRET'),dict(valid,last_phase='SECRET'),
            dict(valid,cycle_completed=True),dict(valid,foreground_us=-1)]
        for diagnostic in variants:
            observation=dict(reason='work_deadline',calls=1,offset=4096,seconds=60.0,
                progress=[],drain={},classification='observer_timeout_ambiguous',diagnostic=diagnostic)
            error=probe.TransferObservationError(observation)
            self.assertIs(error.observation,observation)
            self.assertIn('diagnostic=unavailable',str(error))
            self.assertNotIn('SECRET',str(error))

    def test_productive_completion_can_exceed_fifteen_seconds_but_not_sixty(self):
        feed = TransferFeed([(6, self.progress(16_384, 1)),
                             (17, self.progress(32_768, 2)),
                             (29, self.progress(self.TOTAL, 3, 1)+self.COMPLETE+b'MENU trailing')])
        result = self.observe(feed)
        self.assertEqual(result['offset'], self.TOTAL)
        self.assertEqual(result['calls'], 3)
        self.assertEqual(result['seconds'], 29)
        self.assertEqual([record['seconds'] for record in result['progress']], [6, 17, 29])
        self.assertEqual(feed.buffer, b'MENU trailing')
        self.assertEqual(feed.cancelled, [])
        self.assertIn(self.COMPLETE, feed.transcript)

    def test_no_progress_fails_at_fifteen_seconds(self):
        feed, result = self.refusal([])
        self.assertEqual(feed.now, 15)
        self.assertEqual(result['calls'], 0)
        self.assertEqual(result['seconds'], 15)

    def test_spam_and_partial_records_do_not_reset_idle_clock(self):
        for events in [[(4, b'noise\r\n'), (8, b'keepalive\r\n'), (14, b'x')],
                       [(8, b'Confirmed ciphertext bytes: 16384\r\nSDK call: '),
                        (16, b'1 / 65; writes: 0\r\n')]]:
            with self.subTest(events=events):
                feed, result = self.refusal(events)
                self.assertEqual(feed.now, 15)
                self.assertEqual(result['calls'], 0)

    def test_absolute_work_deadline_refuses_late_or_exact_boundary_completion(self):
        for complete_at in [60, 60.1, 64.9]:
            events = [(10*n, self.progress(100*n, n)) for n in range(1, 6)]
            events += [(59, self.progress(600, 6)),
                       (complete_at, self.progress(self.TOTAL, 7)+self.COMPLETE)]
            with self.subTest(complete_at=complete_at):
                feed, result = self.refusal(events)
                self.assertEqual(feed.now, 60)
                self.assertEqual(result['offset'], 600)
                self.assertEqual(result['calls'], 6)

    def test_completion_processing_cannot_cross_the_work_deadline(self):
        probe=importlib.import_module('probe_device_provider_console')
        events=[(10*n,self.progress(100*n,n)) for n in range(1,6)]
        events += [(59,self.progress(self.TOTAL,6)),(59.9,self.COMPLETE)]
        feed=TransferFeed(events)
        original=probe._transfer_marker
        def finish_at_deadline(buffer):
            marker=original(buffer)
            if marker is not None and marker[1]=='completion':feed.now=60
            return marker
        with patch.object(probe,'_transfer_marker',side_effect=finish_at_deadline):
            with self.assertRaises(ValueError) as caught:self.observe(feed)
        self.assertEqual(caught.exception.observation['reason'],'work_deadline')
        self.assertEqual(caught.exception.observation['seconds'],60)

    def test_completion_in_failure_drain_never_becomes_success(self):
        feed = TransferFeed([(10*n, self.progress(100*n, n)) for n in range(1, 6)])
        def failed_drain(console):
            feed.cancelled.append(feed.now)
            feed.now += 1
            feed.transcript += self.progress(self.TOTAL, 6)+self.COMPLETE
            feed.done = True
            return dict(seconds=1, reaped=True, killed=False)
        probe = importlib.import_module('probe_device_provider_console')
        with self.assertRaises(ValueError) as caught:
            probe.wait_transfer(feed, self.TOTAL, started=0.0, clock=feed.clock,
                                poll=feed.poll, read=feed.read, cancel=failed_drain)
        self.assertEqual(caught.exception.observation['seconds'], 60)
        self.assertEqual(feed.now, 61)
        self.assertEqual(caught.exception.observation['drain']['seconds'], 1)

    def test_call_sixty_six_or_nonconsecutive_calls_are_refused(self):
        events = [(n/10, self.progress(n, n)) for n in range(1, 67)]
        feed, result = self.refusal(events, 66)
        self.assertEqual(result['calls'], 65)
        self.assertEqual(len(result['progress']), 65)
        for calls in [(2,), (1, 3), (1, 1)]:
            with self.subTest(calls=calls):
                self.refusal([(n+1, self.progress((n+1)*10, call))
                              for n, call in enumerate(calls)])

    def test_offsets_must_increase_and_stay_within_expected_total(self):
        for next_offset in [0, 9, 10, self.TOTAL+1]:
            with self.subTest(offset=next_offset):
                self.refusal([(1, self.progress(10, 1)),
                              (2, self.progress(next_offset, 2))])

    def test_split_progress_and_completion_preserve_trailing_menu(self):
        first = self.progress(16_384, 1)
        second = self.progress(self.TOTAL, 2)
        feed = TransferFeed([(1, first[:15]), (2, first[15:-3]),
                             (3, first[-3:]+second[:12]),
                             (4, second[12:]+self.COMPLETE[:8]),
                             (5, self.COMPLETE[8:]+b'NEXT MENU')])
        result = self.observe(feed)
        self.assertEqual(result['calls'], 2)
        self.assertEqual(result['seconds'], 5)
        self.assertEqual(feed.buffer, b'NEXT MENU')
        self.assertEqual(feed.transcript, first+second+self.COMPLETE+b'NEXT MENU')

    def test_early_eof_completion_and_stop_markers_are_refused(self):
        cases = [[(1, self.progress(10, 1)), (2, b'')],
                 [(1, self.COMPLETE)],
                 [(1, self.progress(self.TOTAL-1, 1)+self.COMPLETE)],
                 [(1, b'Transfer stopped; no automatic retry.\r\n')],
                 [(1, b'Stopped: progress or call bound.\r\n')],
                 [(1, b'Foreground time budget reached.\r\n')]]
        for events in cases:
            with self.subTest(events=events):
                self.refusal(events)

    def test_output_and_transcript_are_bounded(self):
        feed, result = self.refusal([(1, b'x'*16_385)])
        self.assertLessEqual(len(feed.buffer), 16_384)
        self.assertLessEqual(len(feed.transcript), 4*1024*1024)
        feed = TransferFeed([(1, b'overflow')])
        feed.transcript = b'x'*(4*1024*1024-2)
        with self.assertRaises(ValueError):self.observe(feed)
        self.assertLessEqual(len(feed.transcript), 4*1024*1024)

    def test_work_deadline_grace_records_product_stop_before_cancellation(self):
        probe=importlib.import_module('probe_device_provider_console')
        feed=TransferFeed([(1,b'Foreground time budget reached.\r\n')])
        cancelled_at=[]
        def raw(data):feed.written.append(data);cancelled_at.append(feed.now)
        feed.raw=raw
        def reap(pid, flags):return (pid,0) if feed.written else (0,0)
        result=probe._cancel_transfer(feed,observe_stop=True,clock=feed.clock,
                                     poll=feed.poll,read=feed.read,reap=reap,
                                     kill=lambda *_:None)
        self.assertEqual(result['marker_before_cancel'],'budget')
        self.assertEqual(cancelled_at,[1])
        self.assertTrue(result['reaped'])
        self.assertLessEqual(result['seconds'],5)

    def test_grace_without_product_stop_remains_observer_timeout(self):
        probe=importlib.import_module('probe_device_provider_console')
        cases=[([],None,4), ([(1,self.COMPLETE)],'completion',1)]
        for events,marker,cancel_at in cases:
            with self.subTest(marker=marker):
                feed=TransferFeed(events);cancelled_at=[]
                def raw(data):feed.written.append(data);cancelled_at.append(feed.now)
                feed.raw=raw
                result=probe._cancel_transfer(feed,observe_stop=True,clock=feed.clock,
                    poll=feed.poll,read=feed.read,
                    reap=lambda pid, flags:(pid,0) if feed.written else (0,0),
                    kill=lambda *_:None)
                self.assertEqual(result['marker_before_cancel'],marker)
                self.assertEqual(cancelled_at,[cancel_at])
                self.assertTrue(result['reaped'])
                self.assertLessEqual(result['seconds'],5)

    def test_cancel_policy_has_only_five_seconds_to_reap(self):
        probe = importlib.import_module('probe_device_provider_console')
        feed = TransferFeed([(1, self.COMPLETE)])
        kills = []
        def kill(pid, signal):kills.append((feed.now, pid, signal))
        def reap(pid, flags):return (pid, 0) if kills else (0, 0)
        result = probe._cancel_transfer(feed, clock=feed.clock, poll=feed.poll,
                                        read=feed.read, reap=reap, kill=kill)
        self.assertEqual(feed.written, [b'\x03'])
        self.assertTrue(result['reaped'])
        self.assertTrue(result['killed'])
        self.assertEqual(kills[0][0], 4)
        self.assertLessEqual(result['seconds'], 5)
        feed = TransferFeed()
        result = probe._cancel_transfer(feed, clock=feed.clock, poll=feed.poll,
                                        read=feed.read, reap=lambda *_:(0, 0),
                                        kill=lambda *_:None)
        self.assertFalse(result['reaped'])
        self.assertEqual(result['seconds'], 5)


if __name__ == '__main__':
    unittest.main()
