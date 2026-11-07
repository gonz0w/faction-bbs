"""Bounded content-free stderr observation using fake operator processes only."""
import contextlib
import io
import os
from pathlib import Path
import signal
import subprocess
import unittest
from unittest.mock import patch

import check_provider_dependencies as provider
import probe_two_home_provider_console as probe

ROOT = Path(__file__).resolve().parents[1]
LINE = ('carrier_rejection schema=1 phase=1 category=1 pending=1 '
        'shared_attempts=8 finished=1 consumed=0\n')


class FakeStream(io.BytesIO):
    def fileno(self):
        return 42


class FakeProcess:
    def __init__(self, error=b'', out=b'stopped\n', status=0, timeout=False):
        self.stdout, self.stderr = FakeStream(out), FakeStream(error)
        self.returncode = None
        self.status, self.timeout = status, timeout
        self.killed = False

    def send_signal(self, value):
        self.signal = value

    def terminate(self):
        self.send_signal(signal.SIGTERM)

    def wait(self, timeout):
        if self.timeout and not self.killed:
            raise subprocess.TimeoutExpired('fake operator', timeout)
        self.returncode = -9 if self.killed else self.status

    def kill(self):
        self.killed = True

    def poll(self):
        return self.returncode


class DiagnosticContracts(unittest.TestCase):
    def operator(self, error=b'', enabled=True, **kwargs):
        operator = probe.ComposedOperator.__new__(probe.ComposedOperator)
        operator.root, operator.closed = Path('/unused-owned-capture'), False
        operator.diagnostics_enabled = enabled
        operator.process = FakeProcess(error, **kwargs)
        return operator

    def test_parser_accepts_only_complete_fixed_numeric_lines_up_to_eight(self):
        self.assertEqual(probe.diagnostic_lines(b''), ())
        self.assertEqual(probe.diagnostic_lines(LINE.encode() * 8), (LINE.rstrip('\n'),) * 8)
        for phase in [1, 2]:
            for category in range(1, 10):
                for count in range(9):
                    line = LINE.replace('phase=1', f'phase={phase}').replace(
                        'category=1', f'category={category}').replace('shared_attempts=8', f'shared_attempts={count}')
                    self.assertEqual(probe.diagnostic_lines(line.encode()), (line.rstrip('\n'),))

    def test_unknown_malformed_noncanonical_partial_or_excess_output_is_never_published(self):
        bad = [b'private-secret-sentinel\n', LINE.encode() * 9, LINE.rstrip('\n').encode(),
               LINE.replace('phase=1', 'phase=0').encode(), LINE.replace('phase=1', 'phase=3').encode(),
               LINE.replace('category=1', 'category=0').encode(), LINE.replace('category=1', 'category=10').encode(),
               LINE.replace('pending=1', 'pending=true').encode(), LINE.replace('finished=1', 'finished=2').encode(),
               LINE.replace('consumed=0', 'consumed=-1').encode(), LINE.replace('shared_attempts=8', 'shared_attempts=9').encode(),
               LINE.replace('shared_attempts=8', 'shared_attempts=08').encode(), LINE.replace('schema=1', 'schema=2').encode(),
               LINE.replace(' phase', '  phase').encode(), LINE.encode().replace(b'\n', b'\r\n'),
               LINE.encode() + b'private-secret-sentinel\n', b'\xff\n', LINE.encode() + b'\n']
        for error in bad:
            with self.subTest(error=error), self.assertRaises(ValueError):
                probe.diagnostic_lines(error)
            operator = self.operator(error)
            public = io.StringIO()
            with self.subTest(stderr=error), patch('probe_device_provider_console.retain_capture'), \
                    contextlib.redirect_stderr(public), self.assertRaises(ValueError):
                operator.close()
            self.assertEqual(public.getvalue(), '')
            self.assertTrue(operator.closed)

    def test_opt_in_accepts_retains_and_publishes_valid_lines_after_drain(self):
        operator = self.operator(LINE.encode())
        public = io.StringIO()
        with patch('probe_device_provider_console.retain_capture') as capture, contextlib.redirect_stderr(public):
            operator.close()
        self.assertEqual(public.getvalue(), LINE)
        capture.assert_called_once_with(operator.root, 'composed-stop', b'stopped\n' + LINE.encode())
        self.assertEqual(operator.process.signal, signal.SIGTERM)
        self.assertEqual(operator.process.returncode, 0)
        self.assertTrue(operator.closed)

    def test_disabled_diagnostics_still_require_empty_stderr(self):
        for error in [LINE.encode(), b'private-secret-sentinel\n']:
            operator = self.operator(error, enabled=False)
            public = io.StringIO()
            with patch('probe_device_provider_console.retain_capture'), contextlib.redirect_stderr(public), \
                    self.assertRaises(ValueError):
                operator.close()
            self.assertEqual(public.getvalue(), '')
        with patch('probe_device_provider_console.retain_capture'):
            self.operator(enabled=False).close()

    def test_diagnostics_do_not_waive_exit_stdout_kill_or_five_second_drain(self):
        for kwargs in [dict(status=1), dict(out=b''), dict(out=b'stopped\nextra'), dict(timeout=True)]:
            operator = self.operator(LINE.encode(), **kwargs)
            public = io.StringIO()
            with self.subTest(kwargs=kwargs), patch('probe_device_provider_console.retain_capture'), \
                    contextlib.redirect_stderr(public), self.assertRaises(ValueError):
                operator.close()
            self.assertEqual(public.getvalue(), LINE)
            self.assertTrue(operator.closed)
        operator = self.operator(LINE.encode())
        with patch('probe_device_provider_console.retain_capture'), patch('time.monotonic', side_effect=[0, 5]), \
                contextlib.redirect_stderr(io.StringIO()), self.assertRaises(ValueError):
            operator.close()

    def test_exact_opt_in_is_snapshotted_at_start_and_startup_failure_is_sanitized(self):
        for value in ['', '0', 'true', '01', '1']:
            process = FakeProcess(LINE.encode())
            with patch.dict(os.environ, FACTION_PROVIDER_DIAGNOSTICS=value), \
                    patch('subprocess.Popen', return_value=process), patch('select.select', return_value=([process.stdout], [], [])), \
                    patch('os.read', return_value=b'ready 12345\n'), patch('probe_device_provider_console.retain_capture'):
                operator = probe.ComposedOperator(Path('operator'), Path('config'), Path('owned'))
            self.assertEqual(operator.diagnostics_enabled, value == '1')
            with contextlib.redirect_stderr(io.StringIO()), patch('probe_device_provider_console.retain_capture'):
                if value == '1':
                    operator.close()
                else:
                    with self.assertRaises(ValueError):
                        operator.close()
        for error, expected in [(LINE.encode(), LINE), (b'private-secret-sentinel\n', '')]:
            process, public = FakeProcess(error), io.StringIO()
            with patch.dict(os.environ, FACTION_PROVIDER_DIAGNOSTICS='1'), patch('subprocess.Popen', return_value=process), \
                    patch('select.select', return_value=([process.stdout], [], [])), patch('os.read', return_value=b''), \
                    patch('probe_device_provider_console.retain_capture'), contextlib.redirect_stderr(public), \
                    self.assertRaises(ValueError):
                probe.ComposedOperator(Path('operator'), Path('config'), Path('owned'))
            self.assertEqual(public.getvalue(), expected)
            self.assertTrue(process.stdout.closed and process.stderr.closed)

    def test_only_owned_console_ci_step_enables_exact_quoted_opt_in(self):
        source = (ROOT / '.github/workflows/documentation.yml').read_text()
        step = provider._step(provider._job(source, 'own_home_client'),
                              'Validate the own-home encrypted attachment console')
        setting = "          FACTION_PROVIDER_DIAGNOSTICS: '1'"
        self.assertEqual(step.splitlines().count(setting), 1)
        self.assertEqual(source.count('FACTION_PROVIDER_DIAGNOSTICS:'), 1)


if __name__ == '__main__':
    unittest.main()
