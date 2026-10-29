"""Contracts for the owned authenticated-provider console proof."""
import importlib
import os
from pathlib import Path
import stat
import subprocess
from unittest.mock import patch
import tempfile
import tomllib
import unittest


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
