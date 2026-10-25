#!/usr/bin/env python3
"""Private native #209 two-home encrypted-mail acceptance harness.

Runs only in an explicitly marked disposable fixture. The Rust example owns all
product checkpoint/crypto/transport behavior. Python orchestrates synthetic local
actions and inspects results privately; it is not a product runtime dependency.
"""
import argparse
import json
import os
import platform
import re
import subprocess
import time
from collections import Counter
from pathlib import Path


class FixtureFailure(Exception):
    pass


def server_configuration(side):
    """Fixed disposable homes; never accepts an external hostname or interface."""
    if side not in {'a', 'b'}:
        raise FixtureFailure('unsupported fixture home')
    other = 'b' if side == 'a' else 'a'
    user, domain, peer = 'mail' + side, 'home' + side + '.test', 'home' + other + '.test'
    c2s, s2s, remote_s2s = (32222, 5269, 5270) if side == 'a' else (32223, 5270, 5269)
    return f'''daemonize = false
pidfile = "/home/{user}/prosody.pid"
certificates = "/home/{user}"
data_path = "/home/{user}/data"
plugin_paths = {{ "/usr/local/faction-mail-modules" }}
log = {{ {{ levels = {{ min = "info" }}; to = "file"; filename = "/home/{user}/prosody.log" }} }}
interfaces = {{ "127.0.0.1" }}
c2s_ports = {{ }}
c2s_direct_tls_interfaces = {{ "127.0.0.1" }}
c2s_direct_tls_ports = {{ {c2s} }}
s2s_interfaces = {{ "127.0.0.1" }}
s2s_ports = {{ {s2s} }}
s2s_direct_tls_ports = {{ }}
legacy_ssl_ports = {{ }}
http_ports = {{ }}
https_ports = {{ }}
c2s_require_encryption = true
s2s_require_encryption = true
s2s_secure_auth = true
allow_registration = false
authentication = "internal_hashed"
ssl = {{ key = "/home/{user}/tls.key"; certificate = "/home/{user}/tls.crt"; cafile = "/home/{user}/ca.crt" }}
modules_enabled = {{ "tls"; "saslauth"; "disco"; "roster"; "pep"; "mam"; "offline"; "limits"; "s2sout_override"; "faction_probe_peers" }}
modules_disabled = {{ "dialback"; "bosh"; "websocket"; "http"; "register"; "csi" }}
s2sout_override = {{ ["{peer}"] = "tcp://127.0.0.1:{remote_s2s}" }}
faction_probe_peer = "{peer}"
archive_expires_after = "1d"
default_archive_policy = true
max_archive_query_results = 20
max_messages = 32
c2s_stanza_size_limit = 16384
s2s_stanza_size_limit = 32768
VirtualHost "{domain}"
'''


def _invocation(binary, action, home, arguments, home_password, lines):
    argv = [str(binary), action, str(home['store']), *map(str, arguments)]
    fields = [home['local_password']]
    if home_password:
        fields.append(home['password'])
    fields.extend(lines)
    if any('\n' in value or '\r' in value for value in fields):
        raise FixtureFailure('invalid private fixture input')
    options = dict(input=('\n'.join(fields) + '\n').encode(), capture_output=True, timeout=15)
    if home.get('run_as'):
        options.update(user=home['run_as'], group=home['run_as'], extra_groups=[])
    return argv, options


def command(binary, action, home, *arguments, home_password=False, lines=(), raw=False):
    argv, options = _invocation(binary, action, home, arguments, home_password, lines)
    try:
        result = subprocess.run(argv, **options)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise FixtureFailure('native action failed') from error
    if result.returncode or result.stderr or (not raw and result.stdout != b'ok\n'):
        raise FixtureFailure('native action rejected')
    return result.stdout


def inspection(raw):
    try:
        value = json.loads(raw)
        if set(value) != {'history', 'pending'} or not isinstance(value['history'], list) or not isinstance(value['pending'], list):
            raise ValueError()
        for row in value['history']:
            if set(row) != {'direction', 'subject', 'body', 'sent_at', 'address'}:
                raise ValueError()
            if row['direction'] not in {'sent', 'received'} or type(row['sent_at']) is not int:
                raise ValueError()
            if not all(isinstance(row[key], str) for key in ['subject', 'body', 'address']):
                raise ValueError()
        for row in value['pending']:
            if set(row) != {'operation', 'address'} or not isinstance(row['address'], str) or not re.fullmatch('[0-9a-f]{32}', row['operation']):
                raise ValueError()
        return value
    except (TypeError, ValueError, KeyError) as error:
        raise FixtureFailure('invalid native inspection') from error


def require_history(value, expected, pending_count):
    got = Counter((row['direction'], row['subject'], row['body']) for row in value['history'])
    if got != Counter(expected) or len(value['pending']) != pending_count:
        raise FixtureFailure('authenticated history or ambiguous outbox mismatch')


def assert_no_plaintext(roots, markers):
    for root in map(Path, roots):
        if root.is_symlink() or not root.exists():
            raise FixtureFailure('unexpected server storage boundary')
        paths = [root, *root.rglob('*')] if root.is_dir() else [root]
        for path in paths:
            if path.is_symlink():
                raise FixtureFailure('unexpected server storage link')
            if path.is_file():
                # Fixtures have fixed small quotas; refuse unbounded inspection.
                if path.stat().st_size > 16 * 1024 * 1024:
                    raise FixtureFailure('oversized server fixture file')
                content = path.read_bytes()
                if any(marker in content for marker in markers):
                    raise FixtureFailure('private plaintext reached home storage')


def concurrent_sync(binary, receiver, sender):
    argv, options = _invocation(binary, 'sync', receiver, [receiver['config']], True, ())
    data = options.pop('input')
    options.pop('capture_output')
    options.pop('timeout')
    process = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **options)
    try:
        process.stdin.write(data)
        process.stdin.close()
        process.stdin = None
        # Both manual native cycles remain finite. No seal operation repeats.
        time.sleep(0.3)
        command(binary, 'sync', sender, sender['config'], home_password=True)
        stdout, stderr = process.communicate(timeout=15)
        if process.returncode or stderr or stdout != b'ok\n':
            raise FixtureFailure('receiving native action rejected')
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=5)


def probe(binary, private_fixture):
    marker = Path('/etc/faction-disposable-test')
    if not marker.is_file() or b'209' not in marker.read_bytes():
        raise FixtureFailure('explicit disposable native marker required')
    path = Path(private_fixture)
    if path.is_symlink() or path.stat().st_mode & 0o077:
        raise FixtureFailure('private fixture profile required')
    profile = json.loads(path.read_text())
    a, b = profile['alice'], profile['bob']
    roots = profile['server_roots']
    owners = [Path(profile[key]).stat().st_uid for key in ['server_a', 'server_b']]
    if owners[0] == owners[1] or 0 in owners:
        raise FixtureFailure('independent unprivileged home owners required')
    checks = []
    a_pin = command(binary, 'init', a, a['jid'], a['device'], raw=True).decode().strip()
    b_pin = command(binary, 'init', b, b['jid'], b['device'], raw=True).decode().strip()
    if not all(re.fullmatch('[0-9a-f]{64}', pin) for pin in [a_pin, b_pin]) or a_pin == b_pin:
        raise FixtureFailure('independent native public identities required')
    command(binary, 'enroll', a, b['jid'], b['device'], b_pin)
    command(binary, 'enroll', b, a['jid'], a['device'], a_pin)
    checks.append('full_native_device_identity_deliberately_verified')
    command(binary, 'publish', b, b['config'], home_password=True)
    command(binary, 'prepare', a, a['config'], home_password=True)
    checks.append('durable_public_bundle_published_and_verified_peer_prepared')
    subject_a, body_a = profile['alice_subject'], profile['alice_body']
    subject_b, body_b = profile['bob_subject'], profile['bob_body']
    command(binary, 'queue', a, lines=[subject_a, body_a])
    concurrent_sync(binary, b, a)
    b_first = inspection(command(binary, 'inspect', b, raw=True))
    require_history(b_first, [('received', subject_a, body_a)], 0)
    checks.append('native_encrypted_subject_and_body_cross_two_homes')
    command(binary, 'queue', b, lines=[subject_b, body_b])
    concurrent_sync(binary, a, b)
    expected_a = [('sent', subject_a, body_a), ('received', subject_b, body_b)]
    expected_b = [('received', subject_a, body_a), ('sent', subject_b, body_b)]
    after_a = inspection(command(binary, 'inspect', a, raw=True))
    after_b = inspection(command(binary, 'inspect', b, raw=True))
    require_history(after_a, expected_a, 1)
    require_history(after_b, expected_b, 1)
    checks.append('native_authenticated_reply_and_explicit_pending_outbox')
    concurrent_sync(binary, b, a)
    concurrent_sync(binary, a, b)
    retried_a = inspection(command(binary, 'inspect', a, raw=True))
    retried_b = inspection(command(binary, 'inspect', b, raw=True))
    require_history(retried_a, expected_a, 1)
    require_history(retried_b, expected_b, 1)
    if retried_a['pending'] != after_a['pending'] or retried_b['pending'] != after_b['pending']:
        raise FixtureFailure('reopen retry changed persisted operation identity')
    checks.append('reopen_retry_preserves_exact_operations_and_bounded_duplicate_history')
    assert_no_plaintext(roots, [value.encode() for value in [subject_a, body_a, subject_b, body_b]])
    checks.append('no_private_subject_or_body_in_home_message_storage_or_logs')
    return dict(result='pass', platform=platform.system(), checks=checks,
                independent_unprivileged_homes=2, native_product_client=True,
                no_offline_archive_reliability_claim=True, live_provider_contact=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--private-fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    arguments = parser.parse_args()
    try:
        result = probe(arguments.binary.resolve(), arguments.private_fixture)
        arguments.output.write_text(json.dumps(result, indent=2) + '\n')
        print('Native two-home encrypted-mail fixture passed:', len(result['checks']), 'controls')
    except (FixtureFailure, ValueError, OSError, KeyError):
        raise SystemExit('Native two-home fixture failed; private diagnostics withheld') from None
