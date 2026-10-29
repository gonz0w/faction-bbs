"""Synthetic UNIT controls only; mocked observations never qualify a real guest."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import collect_attachment_netbsd as collector

ROOT = Path(__file__).resolve().parents[1]
CHECKS = [
    'fresh_factcp05_decline_and_enable_preserve_mail_read_cursor_pending_without_spool_or_network',
    'classic_and_compact_unicode_navigation_no_spool_or_connection',
    'fresh_confirmation_discards_queue_typeahead_and_cancel_preserves_checkpoint_without_sidecar',
    'queue_is_local_and_lost_response_stops_without_implicit_retry',
    'reopened_exact_operation_multicycle_upload_with_one_hidden_password_without_source',
    'ctrl_c_cancels_active_authenticated_transfer_locks_and_restores_terminal',
    'authenticated_metadata_and_declined_foreign_route_are_local_only',
    'sender_offline_own_home_multicycle_fetch_local_verify_explicit_private_save_no_overwrite',
    'server_store_and_wire_exclude_private_metadata_keys_and_plaintext',
    'ctrl_d_locks_at_save_selection_without_publication',
    'hostile_bidi_source_and_authenticated_filename_remain_inert_in_classic_and_unicode',
    'hidden_passwords_absent_from_pty_output_and_terminal_echo_restored',
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


class NativeEnvelopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='faction267-UNIT-OWNED-')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.source = self.root/'source'
        self.source.mkdir()
        old = json.loads((ROOT/'dev-docs/evidence/attachment-desk/inputs.json').read_bytes())
        selected = old['files'] | old['verification_files']
        self.files = {}
        for relative in selected:
            path = self.source/relative
            path.parent.mkdir(parents=True, exist_ok=True)
            raw = ('UNIT fake source: '+relative).encode()
            path.write_bytes(raw)
            self.files[relative] = digest(raw)
        self.binaries = {}
        for role in ('faction-client', 'mail_blob_fixture', 'mail_reply_fixture'):
            path = self.root/role
            path.write_bytes(b'UNIT fake ELF executable '+role.encode())
            path.chmod(0o700)
            self.binaries[role] = path
        self.transfer = json.loads((ROOT/'dev-docs/evidence/attachment-transfer/tls-result.json').read_bytes())
        self.console = json.loads((ROOT/'dev-docs/evidence/attachment-desk/host-result.json').read_bytes())
        self.manifest_path = self.root/'manifest.json'
        self.transfer_path = self.root/'transfer.json'
        self.console_path = self.root/'console.json'
        self.manifest = dict(schema_version=1, files=self.files,
                             binaries={r: digest(p.read_bytes()) for r, p in self.binaries.items()}, results={})
        self.write_results()
        self.uname = SimpleNamespace(sysname='NetBSD', nodename='UNIT-PRIVATE-NODE',
                                     release='11.0', version='UNIT-PRIVATE-KERNEL', machine='evbarm')
        self.values = {'hw.ncpu': b'2\n', 'hw.physmem64': b'1054990336\n', 'hw.machine_arch': b'aarch64\n'}
        self.calls = []
        self.addCleanup(patch.stopall)
        patch.object(collector.os, 'uname', side_effect=lambda: self.uname).start()
        patch.object(collector, '_command', side_effect=self.command).start()

    def command(self, argv):
        self.calls.append(argv)
        if argv[:2] == ['/sbin/sysctl', '-n'] and len(argv) == 3:
            return self.values[argv[2]]
        if argv[:2] == ['/usr/bin/file', '-b'] and len(argv) == 3:
            return b'ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), for NetBSD 11.0\n'
        raise AssertionError('unexpected command')

    def private_json(self, path, value):
        path.write_text(json.dumps(value))
        path.chmod(0o600)

    def write_results(self):
        self.private_json(self.transfer_path, self.transfer)
        self.private_json(self.console_path, self.console)
        self.manifest['results'] = {'transfer': digest(self.transfer_path.read_bytes()),
                                    'console': digest(self.console_path.read_bytes())}
        self.private_json(self.manifest_path, self.manifest)

    def collect(self):
        return collector.collect(self.source, self.manifest_path, self.transfer_path,
                                 self.console_path, self.binaries)

    def refused(self):
        with self.assertRaises(collector.CollectionError) as caught:
            self.collect()
        message = str(caught.exception)
        for private in (str(self.root), 'UNIT-PRIVATE', 'password', 'SECRET'):
            self.assertNotIn(private, message)

    def test_unit_fake_positive_binds_actual_bytes_and_keeps_original_claim_false(self):
        record = self.collect()
        self.assertEqual(record['schema_version'], 1)
        self.assertEqual(record['platform'], dict(sysname='NetBSD', release='11.0', machine='evbarm', machine_arch='aarch64'))
        self.assertEqual(record['runtime'], dict(cpus=2, physical_memory_bytes=1054990336))
        self.assertEqual(record['files'], self.files)
        self.assertEqual(len(record['files']), 21)
        self.assertEqual(record['binaries'], self.manifest['binaries'])
        self.assertEqual(record['result_sha256'], self.manifest['results'])
        self.assertEqual(record['manifest_sha256'], digest(self.manifest_path.read_bytes()))
        self.assertEqual(record['console']['checks'], CHECKS)
        self.assertIs(record['console']['netbsd_qualification'], False)
        self.assertEqual(record['transfer'], self.transfer)
        self.assertIs(record['native_envelope_validated'], True)
        self.assertIs(record['provider_deployment'], False)
        self.assertIs(record['capacity_claim'], False)
        for name in ('collector_sha256', 'collector_tests_sha256'):
            self.assertRegex(record[name], r'^[0-9a-f]{64}$')
        encoded = json.dumps(record)
        self.assertLessEqual(len(encoded.encode()), 32768)
        for private in (str(self.root), 'UNIT-PRIVATE-NODE', 'UNIT-PRIVATE-KERNEL', 'UNIT fake ELF'):
            self.assertNotIn(private, encoded)
        self.assertEqual(self.calls[:3], [['/sbin/sysctl', '-n', 'hw.machine_arch'],
                                         ['/sbin/sysctl', '-n', 'hw.ncpu'],
                                         ['/sbin/sysctl', '-n', 'hw.physmem64']])
        self.assertEqual(len(self.calls), 6)

    def test_host_wrong_release_or_unknown_machine_refused(self):
        for field, value in [('sysname', 'Darwin'), ('release', '10.1'), ('release', '110.0'),
                             ('machine', 'amd64'), ('machine', 'unknown')]:
            with self.subTest(field=field, value=value):
                previous = getattr(self.uname, field)
                setattr(self.uname, field, value)
                self.refused()
                setattr(self.uname, field, previous)

    def test_architecture_and_runtime_resources_are_observed_and_bounded(self):
        for key, raw in [('hw.machine_arch', b'amd64\n'), ('hw.ncpu', b'1\n'),
                         ('hw.ncpu', b'4\n'), ('hw.ncpu', b'2; SECRET\n'),
                         ('hw.physmem64', b'2147483648\n'), ('hw.physmem64', b'536870912\n')]:
            with self.subTest(key=key, raw=raw):
                previous = self.values[key]
                self.values[key] = raw
                self.refused()
                self.values[key] = previous

    def test_source_missing_changed_or_unselected_refused(self):
        relative = next(iter(self.files))
        path = self.source/relative
        before = path.read_bytes()
        path.write_bytes(before+b'changed')
        self.refused()
        path.write_bytes(before)
        del self.manifest['files'][relative]
        self.write_results()
        self.refused()
        self.manifest['files'][relative] = digest(before)
        self.manifest['files']['private/SECRET'] = 'a'*64
        self.write_results()
        self.refused()

    def test_binary_digest_role_native_format_and_execute_mode_refused(self):
        path = self.binaries['faction-client']
        path.write_bytes(b'changed')
        self.refused()
        self.manifest['binaries']['faction-client'] = digest(path.read_bytes())
        self.write_results()
        path.chmod(0o600)
        self.refused()
        path.chmod(0o700)
        with patch.object(collector, '_command', side_effect=lambda argv: b'Mach-O 64-bit executable arm64' if argv[0] == '/usr/bin/file' else self.command(argv)):
            self.refused()
        self.manifest['binaries']['arbitrary'] = 'a'*64
        self.write_results()
        self.refused()

    def test_result_hash_binding_rejects_even_otherwise_valid_changed_result(self):
        self.console['seconds'] += 1
        self.private_json(self.console_path, self.console)
        self.refused()

    def test_transfer_rejects_missing_fact_false_flag_wrong_bytes_and_limits(self):
        original = copy.deepcopy(self.transfer)
        for key, value in [('lost_response_observed', False), ('full_plaintext_bytes', 65536),
                           ('recipient_max_requests', 9), ('sender_max_physical_input', 65537),
                           ('upload_cycles_after_loss', 81), ('fetch_cycles', 0),
                           ('two_session_timeout_seconds', 12), ('handler_requests', True)]:
            with self.subTest(key=key):
                self.transfer = copy.deepcopy(original)
                self.transfer['facts'][key] = value
                self.write_results()
                self.refused()
        self.transfer = copy.deepcopy(original)
        del self.transfer['facts']['between_chunk_revocation_and_resume']
        self.write_results()
        self.refused()

    def test_console_rejects_missing_duplicate_extra_or_reordered_checks(self):
        for checks in (CHECKS[:-1], CHECKS+[CHECKS[0]], CHECKS+['SECRET'], list(reversed(CHECKS))):
            with self.subTest(checks=len(checks)):
                self.console['checks'] = checks
                self.console['count'] = len(checks)
                self.write_results()
                self.refused()

    def test_console_rejects_wrong_counts_claims_cleanup_and_resource_caps(self):
        original = copy.deepcopy(self.console)
        for key, value in [('count', 11), ('count', True), ('exact_plaintext_bytes', 1048576),
                           ('max_requests_per_session', 9), ('netbsd_qualification', True),
                           ('provider_adopted', True), ('children_reaped', False),
                           ('capacity_claim', True), ('ssh_trace', True), ('seconds', float('nan'))]:
            with self.subTest(key=key):
                self.console = copy.deepcopy(original)
                self.console[key] = value
                self.write_results()
                self.refused()

    def test_unknown_private_fields_are_never_echoed_or_accepted(self):
        for target in (self.manifest, self.transfer, self.console):
            target['SECRET'] = str(self.root)
            self.write_results()
            self.refused()
            del target['SECRET']

    def test_symlink_source_ancestor_result_and_binary_refused(self):
        for path in (self.source/'Cargo.lock', self.console_path, self.binaries['mail_blob_fixture']):
            real = path.with_name(path.name+'.real')
            path.rename(real)
            path.symlink_to(real)
            self.refused()
            path.unlink()
            real.rename(path)
        tools = self.source/'tools'
        tools.rename(self.source/'actual-tools')
        tools.symlink_to('actual-tools', target_is_directory=True)
        self.refused()

    def test_fifo_and_private_file_permissions_refused_without_blocking(self):
        self.console_path.unlink()
        os.mkfifo(self.console_path)
        self.refused()
        self.console_path.unlink()
        self.write_results()
        self.console_path.chmod(0o644)
        self.refused()

    def test_duplicate_keys_oversize_deep_json_and_noncanonical_hash_refused(self):
        original = self.manifest_path.read_bytes()
        for raw in (b'{"schema_version":1,"schema_version":1}', b' '*65537,
                    b'['*80+b'0'+b']'*80):
            self.manifest_path.write_bytes(raw)
            self.refused()
        self.manifest_path.write_bytes(original)
        self.manifest['results']['transfer'] = 'A'*64
        self.private_json(self.manifest_path, self.manifest)
        self.refused()

    def test_command_failure_is_redacted_and_fixed_commands_have_timeout(self):
        patch.stopall()
        with patch.object(collector.subprocess, 'run', side_effect=subprocess.TimeoutExpired('SECRET', 10)) as run:
            with self.assertRaises(collector.CollectionError) as caught:
                collector._command(['/sbin/sysctl', '-n', 'hw.ncpu'])
            self.assertNotIn('SECRET', str(caught.exception))
            self.assertLessEqual(run.call_args.kwargs['timeout'], 10)
            self.assertFalse(run.call_args.kwargs.get('shell', False))

    def test_huge_json_number_refused_with_bounded_public_error(self):
        self.console['seconds'] = 10**400
        self.write_results()
        self.refused()

    def test_binary_roles_cannot_alias_one_executable(self):
        path = self.binaries['faction-client']
        self.binaries = {role: path for role in self.binaries}
        self.manifest['binaries'] = {role: digest(path.read_bytes()) for role in self.binaries}
        self.write_results()
        self.refused()

    def test_private_output_is_exclusive_and_symlink_safe(self):
        output = self.root/'record.json'
        collector._write_new(output, b'bounded public record\n')
        self.assertEqual(output.stat().st_mode & 0o777, 0o600)
        self.assertEqual(output.read_bytes(), b'bounded public record\n')
        with self.assertRaises(collector.CollectionError):
            collector._write_new(output, b'replacement')
        self.assertEqual(output.read_bytes(), b'bounded public record\n')
        linked = self.root/'linked.json'
        linked.symlink_to(output)
        with self.assertRaises(collector.CollectionError):
            collector._write_new(linked, b'replacement')
        self.assertEqual(output.read_bytes(), b'bounded public record\n')
        public = self.root/'public'
        public.mkdir(mode=0o755)
        public.chmod(0o755)
        with self.assertRaises(collector.CollectionError):
            collector._write_new(public/'record.json', b'record')
        self.assertFalse((public/'record.json').exists())

    def test_command_failure_and_output_caps_never_echo_private_text(self):
        patch.stopall()
        for code, stdout in ((1, b'SECRET'), (0, b'SECRET'*1000), (0, b'')):
            with patch.object(collector.subprocess, 'run', return_value=SimpleNamespace(returncode=code, stdout=stdout)):
                with self.assertRaises(collector.CollectionError) as caught:
                    collector._command(['/sbin/sysctl', '-n', 'hw.ncpu'])
                self.assertNotIn('SECRET', str(caught.exception))

    def test_linked_and_oversize_sources_refused_before_reading(self):
        path = self.source/'Cargo.lock'
        alias = self.source/'alias'
        os.link(path, alias)
        self.refused()
        alias.unlink()
        with path.open('wb') as stream:
            stream.truncate(4*1024*1024+1)
        self.refused()

    def test_complete_cli_refuses_actual_host_without_creating_output(self):
        # os.uname is mocked only in this UNIT process. The subprocess observes
        # the real host; a genuine NetBSD guest may instead reject synthetic ELF.
        output = self.root/'not-created.json'
        result = subprocess.run([sys.executable, str(ROOT/'tools/collect_attachment_netbsd.py'),
                                 '--source-root', str(self.source), '--inputs-manifest', str(self.manifest_path),
                                 '--transfer-result', str(self.transfer_path), '--console-result', str(self.console_path),
                                 '--client', str(self.binaries['faction-client']),
                                 '--blob-fixture', str(self.binaries['mail_blob_fixture']),
                                 '--reply-fixture', str(self.binaries['mail_reply_fixture']), '--output', str(output)],
                                capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')
        self.assertFalse(output.exists())
        self.assertIn(b'Native attachment envelope refused:', result.stderr)
        self.assertNotIn(str(self.root).encode(), result.stderr)

    def test_cli_exposes_no_platform_or_arbitrary_command_override(self):
        result = subprocess.run([sys.executable, str(ROOT/'tools/collect_attachment_netbsd.py'), '--pretend', 'NetBSD'], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b'')


if __name__ == '__main__':
    unittest.main()
