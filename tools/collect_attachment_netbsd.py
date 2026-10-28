#!/usr/bin/env python3
"""Collect a local NetBSD attachment envelope around separately executed proofs.

This does not run the proofs or attest their origin. Retained native execution
commands establish that relationship; this record binds their exact result bytes,
selected source bytes and executables to observations made where it runs. There
is no platform override, VM control, provider contact or arbitrary command option.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time

JSON_CAP = 65536
RECORD_CAP = 32768
BINARY_CAP = 256 * 1024 * 1024
ROLES = ('faction-client', 'mail_blob_fixture', 'mail_reply_fixture')
SOURCE_FILES = (
    'Cargo.lock',
    'crates/faction-client/examples/mail_blob_fixture.rs',
    'crates/faction-client/examples/mail_reply_fixture.rs',
    'crates/faction-client/src/attachment_source.rs',
    'crates/faction-client/src/checkpoint_ui.rs',
    'crates/faction-client/src/checkpoint_ui_attachments.rs',
    'crates/faction-client/src/checkpoint_ui_inbox.rs',
    'crates/faction-client/src/checkpoint_ui_reply.rs',
    'crates/faction-client/src/lib.rs',
    'crates/faction-client/src/mail_blob_transfer.rs',
    'crates/faction-client/src/main.rs',
    'tools/probe_attachment_transfer.py',
    'tools/probe_mail_attachments.py',
    'tools/probe_xmpp_transport.py',
    'tools/smoke_board_launcher.py',
    'tools/smoke_local_console.py',
    'tools/check_evidence_sources.py',
    'tools/evidence_sources.py',
    'tools/test_evidence_decode_cache.py',
    'tools/test_evidence_sources.py',
    'tools/test_mail_attachments_probe.py',
)
CONSOLE_CHECKS = (
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
)
TRANSFER_TRUE = (
    'timeout_sent_no_message', 'uninvited_and_wrong_frame_preflight_refused',
    'between_chunk_revocation_and_resume', 'server_files_and_wire_private_metadata_absent',
    'lost_response_observed', 'sender_offline_at_fetch', 'explicit_no_overwrite_save',
)
TRANSFER_INTS = {
    'upload_cycles_after_loss': (1, 80), 'fetch_cycles': (1, 80),
    'sender_connections': (1, 1024), 'recipient_connections': (1, 1024),
    'sender_max_requests': (1, 8), 'recipient_max_requests': (1, 8),
    'sender_max_physical_input': (1, 65536), 'recipient_max_physical_input': (1, 65536),
    'handler_requests': (1, 4096), 'full_plaintext_bytes': (1048576, 1048576),
}


class CollectionError(Exception):
    """A bounded public refusal, never containing input paths or diagnostics."""


def require(condition, message):
    if not condition:
        raise CollectionError(message)


def _parent(path):
    """Open every directory without following symlinks, including ancestors."""
    path = Path(os.path.abspath(os.fspath(path))) if '..' not in Path(path).parts else None
    require(path is not None, 'path traversal refused')
    descriptor = os.open('/', os.O_RDONLY | os.O_DIRECTORY)
    try:
        for part in path.parts[1:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        return descriptor, path.name
    except BaseException:
        os.close(descriptor)
        raise


def _read(path, cap, *, private=False, executable=False, contents=False):
    parent = descriptor = None
    try:
        parent, name = _parent(path)
        descriptor = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        before = os.fstat(descriptor)
        require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1, 'regular single-link file required')
        require(0 < before.st_size <= cap, 'file size outside collection bound')
        require(not before.st_mode & 0o022, 'writable shared input refused')
        if private:
            require(before.st_uid == os.geteuid() and stat.S_IMODE(before.st_mode) == 0o600,
                    'private evidence mode required')
        if executable:
            require(before.st_uid == os.geteuid() and before.st_mode & stat.S_IXUSR,
                    'owned executable required')
        digest = hashlib.sha256()
        chunks = []
        total = 0
        while True:
            chunk = os.read(descriptor, min(65536, cap + 1 - total))
            if not chunk:
                break
            total += len(chunk)
            require(total <= cap, 'file size outside collection bound')
            digest.update(chunk)
            if contents:
                chunks.append(chunk)
        after = os.fstat(descriptor)
        require(total == before.st_size and
                (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) ==
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns),
                'input changed during collection')
        return digest.hexdigest(), b''.join(chunks) if contents else None
    except OSError:
        raise CollectionError('input file unavailable or unsafe') from None
    finally:
        if descriptor is not None:
            os.close(descriptor)
        if parent is not None:
            os.close(parent)


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key refused')
        result[key] = value
    return result


def _constant(_):
    raise CollectionError('nonfinite JSON number refused')


def _json(path):
    digest, raw = _read(path, JSON_CAP, private=True, contents=True)
    try:
        value = json.loads(raw, object_pairs_hook=_pairs, parse_constant=_constant)
    except (ValueError, RecursionError):
        raise CollectionError('invalid evidence JSON') from None

    def bounded(item, depth=0):
        require(depth <= 8, 'JSON depth outside collection bound')
        if isinstance(item, dict):
            require(len(item) <= 64, 'JSON member count outside collection bound')
            for key, child in item.items():
                require(len(key) <= 192, 'JSON key outside collection bound')
                bounded(child, depth + 1)
        elif isinstance(item, list):
            require(len(item) <= 32, 'JSON array outside collection bound')
            for child in item:
                bounded(child, depth + 1)
        elif isinstance(item, str):
            require(len(item) <= 512, 'JSON string outside collection bound')
    bounded(value)
    return digest, value


def _keys(value, expected):
    require(type(value) is dict and set(value) == set(expected), 'evidence schema mismatch')


def _hashes(value, expected):
    _keys(value, expected)
    require(all(type(v) is str and re.fullmatch('[0-9a-f]{64}', v) for v in value.values()),
            'canonical SHA-256 required')


def _integer(value, low, high):
    require(type(value) is int and low <= value <= high, 'integer outside evidence bound')


def _number(value, low, high):
    require(type(value) in (int, float) and low <= value < high and math.isfinite(value),
            'number outside evidence bound')


def _command(argv):
    """Only fixed local sysctl/file calls are supplied by production callers."""
    try:
        result = subprocess.run(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                stderr=subprocess.DEVNULL, timeout=10, check=False,
                                env={'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'LC_ALL': 'C'})
    except (OSError, subprocess.SubprocessError):
        raise CollectionError('local inspection command unavailable') from None
    require(result.returncode == 0 and 0 < len(result.stdout) <= 4096,
            'local inspection command failed or exceeded bound')
    return result.stdout


def _observe():
    native = os.uname()
    require(native.sysname == 'NetBSD' and re.fullmatch(r'11\.[0-9]+(?:[_A-Za-z0-9.-]*)', native.release)
            and native.machine in ('evbarm', 'aarch64'), 'NetBSD 11 ARM64 collector required')
    arch = _command(['/sbin/sysctl', '-n', 'hw.machine_arch'])
    require(arch.strip() == b'aarch64', 'native ARM64 architecture required')
    values = []
    for key in ('hw.ncpu', 'hw.physmem64'):
        raw = _command(['/sbin/sysctl', '-n', key])
        require(re.fullmatch(rb'[1-9][0-9]{0,15}\n?', raw) is not None, 'invalid native resource observation')
        values.append(int(raw))
    cpus, memory = values
    require(cpus == 2 and 896 * 1024 * 1024 <= memory <= 1152 * 1024 * 1024,
            'two-core about-one-GiB runtime required')
    return dict(sysname='NetBSD', release=native.release, machine=native.machine, machine_arch='aarch64'), dict(cpus=cpus, physical_memory_bytes=memory)


def _transfer(value):
    _keys(value, ('result', 'facts', 'private_fixture_removed', 'actual_home_sessions',
                  'actual_rust_store', 'provider_deployment', 'device_attestation', 'wall_seconds'))
    require(value['result'] == 'pass', 'transfer proof did not pass')
    for key in ('private_fixture_removed', 'actual_home_sessions', 'actual_rust_store'):
        require(value[key] is True, 'transfer observation missing')
    for key in ('provider_deployment', 'device_attestation'):
        require(value[key] is False, 'unsupported transfer claim refused')
    _number(value['wall_seconds'], 0.001, 3600)
    facts = value['facts']
    _keys(facts, (*TRANSFER_TRUE, *TRANSFER_INTS, 'two_session_timeout_seconds'))
    for key in TRANSFER_TRUE:
        require(facts[key] is True, 'transfer control missing')
    for key, (low, high) in TRANSFER_INTS.items():
        _integer(facts[key], low, high)
    _number(facts['two_session_timeout_seconds'], 9, 12)


def _console(value):
    _keys(value, ('result', 'count', 'checks', 'seconds', 'private_fixture_removed', 'children_reaped',
                  'provider_adopted', 'netbsd_qualification', 'ssh_trace', 'capacity_claim',
                  'sender_connections', 'recipient_connections', 'max_requests_per_session', 'exact_plaintext_bytes'))
    require(value['result'] == 'pass' and value['checks'] == list(CONSOLE_CHECKS), 'console controls incomplete')
    _integer(value['count'], 12, 12)
    _integer(value['exact_plaintext_bytes'], 65536, 65536)
    for key in ('sender_connections', 'recipient_connections'):
        _integer(value[key], 1, 1024)
    _integer(value['max_requests_per_session'], 1, 8)
    _number(value['seconds'], 0.001, 3600)
    for key in ('private_fixture_removed', 'children_reaped'):
        require(value[key] is True, 'console cleanup observation missing')
    for key in ('provider_adopted', 'netbsd_qualification', 'ssh_trace', 'capacity_claim'):
        require(value[key] is False, 'unsupported console claim refused')


def collect(source_root, inputs_manifest, transfer_result, console_result, binaries):
    """Return bounded reviewed fields; reject unsafe files or mismatched inputs."""
    platform, runtime = _observe()
    manifest_hash, manifest = _json(inputs_manifest)
    _keys(manifest, ('schema_version', 'files', 'binaries', 'results'))
    _integer(manifest['schema_version'], 1, 1)
    _hashes(manifest['files'], SOURCE_FILES)
    _hashes(manifest['binaries'], ROLES)
    require(len(set(manifest['binaries'].values())) == len(ROLES), 'distinct executable roles required')
    _hashes(manifest['results'], ('transfer', 'console'))
    _keys(binaries, ROLES)
    files = {}
    for relative in SOURCE_FILES:
        actual, _ = _read(Path(source_root)/relative, 4 * 1024 * 1024)
        require(actual == manifest['files'][relative], 'selected source digest mismatch')
        files[relative] = actual
    binary_hashes = {}
    for role in ROLES:
        actual, _ = _read(binaries[role], BINARY_CAP, executable=True)
        require(actual == manifest['binaries'][role], 'binary digest mismatch')
        description = _command(['/usr/bin/file', '-b', os.path.abspath(os.fspath(binaries[role]))])
        require(all(word in description for word in (b'ELF 64-bit LSB', b'executable', b'ARM aarch64', b'NetBSD')),
                'native NetBSD ARM64 executable required')
        after, _ = _read(binaries[role], BINARY_CAP, executable=True)
        require(after == actual, 'binary changed during inspection')
        binary_hashes[role] = actual
    results = {}
    result_hashes = {}
    for role, path, validate in (('transfer', transfer_result, _transfer), ('console', console_result, _console)):
        actual, value = _json(path)
        require(actual == manifest['results'][role], 'result digest mismatch')
        validate(value)
        results[role] = value
        result_hashes[role] = actual
    own_path = Path(__file__).absolute()
    own_hash, _ = _read(own_path, 1024 * 1024)
    tests_hash, _ = _read(own_path.with_name('test_collect_attachment_netbsd.py'), 1024 * 1024)
    record = dict(schema_version=1, evidence_kind='local-native-envelope-and-bound-results',
                  native_envelope_validated=True, collected_unix_seconds=int(time.time()),
                  platform=platform, runtime=runtime, compiler_resources_observed=False,
                  files=files, binaries=binary_hashes, manifest_sha256=manifest_hash,
                  result_sha256=result_hashes, collector_sha256=own_hash, collector_tests_sha256=tests_hash,
                  transfer=results['transfer'], console=results['console'],
                  provider_deployment=False, device_attestation=False, independent_interoperability=False,
                  ssh_trace=False, capacity_claim=False)
    require(len(_encode(record)) <= RECORD_CAP, 'output record outside collection bound')
    return record


def _encode(record):
    return (json.dumps(record, sort_keys=True, indent=2, allow_nan=False)+'\n').encode()


def _write_new(path, raw):
    parent = descriptor = None
    try:
        parent, name = _parent(path)
        require(os.fstat(parent).st_uid == os.geteuid() and stat.S_IMODE(os.fstat(parent).st_mode) == 0o700,
                'owned private output directory required')
        descriptor = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
        with os.fdopen(descriptor, 'wb') as stream:
            descriptor = None
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
    except OSError:
        raise CollectionError('private output unavailable or already exists') from None
    finally:
        if descriptor is not None:
            os.close(descriptor)
        if parent is not None:
            os.close(parent)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ('source-root', 'inputs-manifest', 'transfer-result', 'console-result',
                   'client', 'blob-fixture', 'reply-fixture', 'output'):
        parser.add_argument('--'+option, type=Path, required=True)
    args = parser.parse_args()
    try:
        record = collect(args.source_root, args.inputs_manifest, args.transfer_result, args.console_result,
                         {'faction-client': args.client, 'mail_blob_fixture': args.blob_fixture,
                          'mail_reply_fixture': args.reply_fixture})
        _write_new(args.output, _encode(record))
    except CollectionError as error:
        print('Native attachment envelope refused: '+str(error), file=sys.stderr)
        return 1
    print('Native attachment envelope collected; private output created.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
