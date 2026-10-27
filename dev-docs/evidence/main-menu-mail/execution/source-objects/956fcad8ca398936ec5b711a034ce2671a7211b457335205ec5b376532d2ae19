#!/usr/bin/env python3
"""Private native #209 two-home encrypted-mail acceptance harness.

Runs only in an explicitly marked disposable fixture. The Rust example owns all
product checkpoint/crypto/transport behavior. Python orchestrates synthetic local
actions and inspects results privately; it is not a product runtime dependency.
"""
import argparse
import json
import hashlib
import pwd
import secrets
import stat
import tempfile
import tomllib
from contextlib import contextmanager
import os
import platform
import re
import subprocess
import time
from collections import Counter
from pathlib import Path


class FixtureFailure(Exception):
    pass


def server_configuration(side, attack_directory=None):
    """Fixed disposable homes; never accepts an external hostname or interface."""
    if side not in {'a', 'b'}:
        raise FixtureFailure('unsupported fixture home')
    if attack_directory is not None and (side != 'b' or attack_directory != '/home/mailb/faction209-attack'):
        raise FixtureFailure('owned fixture attack directory required')
    attack_module = '; "faction_probe_attack"' if attack_directory else ''
    attack_option = ('faction_probe_attack_directory = ' + json.dumps(attack_directory) + '\n') if attack_directory else ''
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
modules_enabled = {{ "tls"; "saslauth"; "disco"; "roster"; "pep"; "mam"; "offline"; "limits"; "s2sout_override"; "faction_probe_peers"{attack_module} }}
modules_disabled = {{ "dialback"; "bosh"; "websocket"; "http"; "register"; "csi" }}
s2sout_override = {{ ["{peer}"] = "tcp://127.0.0.1:{remote_s2s}" }}
faction_probe_peer = "{peer}"
{attack_option}archive_expires_after = "1d"
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


POSITIVE_CHECKS = [
    'full_native_device_identity_deliberately_verified',
    'durable_public_bundle_published_and_verified_peer_prepared',
    'native_encrypted_subject_and_body_cross_two_homes',
    'native_authenticated_reply_and_explicit_pending_outbox',
    'reopen_retry_preserves_exact_operations_and_bounded_duplicate_history',
    'no_private_subject_or_body_in_home_message_storage_or_logs',
]
NEGATIVE_CHECKS = [
    'tls_wrong_leaf_pin_denied_without_checkpoint_mutation',
    'home_wrong_password_denied_without_checkpoint_mutation',
    'server_replaced_device_list_denied_without_checkpoint_mutation',
    'server_changed_identity_bundle_denied_without_checkpoint_mutation',
    'server_added_device_never_enrolled_or_addressed',
]
ATTACK_MODES = {'replaced-device', 'changed-identity', 'added-device'}


def public_result(checks, negative_only=False):
    if checks != (NEGATIVE_CHECKS if negative_only else POSITIVE_CHECKS + NEGATIVE_CHECKS):
        raise FixtureFailure('incomplete native controls')
    return dict(result='pass', platform=platform.system(), checks=checks,
                scope='negative-controls-only' if negative_only else 'complete',
                independent_unprivileged_homes=2, native_product_client=True,
                no_offline_archive_reliability_claim=True, live_provider_contact=False)


def private_bytes(path, limit, owner=None):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_mode & 0o777 != 0o600 or info.st_nlink != 1 or info.st_size > limit or (owner is not None and info.st_uid != owner):
            raise FixtureFailure('private regular fixture file required')
        with os.fdopen(fd, 'rb', closefd=False) as stream:
            data = stream.read(limit + 1)
        if len(data) > limit:
            raise FixtureFailure('oversized private fixture file')
        return data, info
    finally:
        os.close(fd)


def checkpoint_digest(home):
    data, _ = private_bytes(Path(home['store']) / 'mail.fkv', 524512)
    return hashlib.sha256(data).digest()


def denied_cycle(binary, home, config, expected):
    before = checkpoint_digest(home)
    argv, options = _invocation(binary, 'sync', home, [config], True, ())
    try:
        result = subprocess.run(argv, **options)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise FixtureFailure('native denial did not complete') from error
    if result.returncode != 1 or result.stdout or result.stderr != (expected + '\n').encode() or checkpoint_digest(home) != before:
        raise FixtureFailure('native denial or canonical checkpoint mismatch')


@contextmanager
def wrong_pin_configuration(home):
    source = Path(home['config'])
    raw, info = private_bytes(source, 4096)
    config = tomllib.loads(raw.decode())
    if not re.fullmatch('[0-9a-f]{64}', config['pin']):
        raise FixtureFailure('fixture pin required')
    config['pin'] = f"{int(config['pin'], 16) ^ 1:064x}"
    fd, name = tempfile.mkstemp(prefix='.faction209-wrong-pin-', suffix='.toml', dir=source.parent)
    try:
        os.fchmod(fd, 0o600)
        if os.geteuid() == 0:
            os.fchown(fd, info.st_uid, info.st_gid)
        with os.fdopen(fd, 'w') as output:
            fd = None
            output.write('\n'.join(key + '=' + (str(value) if type(value) is int else json.dumps(value)) for key, value in config.items()) + '\n')
        yield Path(name)
    finally:
        if fd is not None:
            os.close(fd)
        Path(name).unlink()


def parse_attack_receipt(raw, token, mode):
    if len(raw) > 512:
        raise FixtureFailure('oversized fixture receipt')
    try:
        fields = raw.decode('ascii').splitlines()
        if len(fields) != 7 or fields[:3] != ['faction209-v1', token, mode] or not re.fullmatch('[0-9a-f]{32}', token) or mode not in ATTACK_MODES:
            raise ValueError()
        if any(not re.fullmatch('0|[1-9][0-9]{0,3}', field) for field in fields[3:]):
            raise ValueError()
        return dict(zip(['devices', 'bundles', 'messages', 'unexpected'], map(int, fields[3:])))
    except (UnicodeError, ValueError) as error:
        raise FixtureFailure('invalid fixture receipt') from error


def attack_identity(profile):
    attack = profile['attack']
    if set(attack) != {'directory', 'owner'} or attack['owner'] != 'mailb' or attack['directory'] != '/home/mailb/faction209-attack':
        raise FixtureFailure('owned disposable attack profile required')
    owner = pwd.getpwnam(attack['owner'])
    directory = Path(attack['directory'])
    info = directory.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_mode & 0o777 != 0o700 or info.st_uid != owner.pw_uid or owner.pw_uid == 0:
        raise FixtureFailure('private unprivileged attack directory required')
    return directory, owner


@contextmanager
def server_attack(profile, mode):
    directory, owner = attack_identity(profile)
    if mode not in ATTACK_MODES:
        raise FixtureFailure('unknown fixture attack')
    a, b = profile['alice'], profile['bob']
    if any(not re.fullmatch(r'[a-z0-9._-]+@home[ab]\.test', home['jid']) for home in [a, b]):
        raise FixtureFailure('synthetic fixture accounts required')
    device = int(b['device'])
    if not 1 <= device <= 2147483647:
        raise FixtureFailure('fixture device required')
    added = 999 if device != 999 else 998
    token = secrets.token_hex(16)
    created = []
    try:
        for name, content in [
            ('receipt', f'faction209-v1\n{token}\n{mode}\n0\n0\n0\n0\n'),
            ('mode', f'faction209-v1\n{token}\n{mode}\n{b["jid"]}\n{a["jid"]}\n{device}\n{added}\n'),
        ]:
            path = directory / name
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            created.append(path)
            with os.fdopen(fd, 'w') as output:
                os.fchown(output.fileno(), owner.pw_uid, owner.pw_gid)
                output.write(content)
        def receipt():
            raw, _ = private_bytes(directory / 'receipt', 512, owner.pw_uid)
            return parse_attack_receipt(raw, token, mode)
        yield receipt
    finally:
        # Disable interception before removing its receipt. Never remove a file
        # that predates this invocation, including a stale control from a failure.
        for path in reversed(created):
            path.unlink()


def negative_controls(binary, profile):
    marker = Path('/etc/faction-disposable-test')
    if not marker.is_file() or b'209' not in marker.read_bytes():
        raise FixtureFailure('explicit disposable native marker required')
    attack_identity(profile)
    owners = [Path(profile[key]).stat().st_uid for key in ['server_a', 'server_b']]
    if owners[0] == owners[1] or 0 in owners:
        raise FixtureFailure('independent unprivileged home owners required')
    a, b = profile['alice'], profile['bob']
    # A successful prior exchange leaves a durable retry for Alice. This is the
    # ciphertext used to observe routing without sealing a new message.
    if len(inspection(command(binary, 'inspect', a, raw=True))['pending']) != 1:
        raise FixtureFailure('successful fixture pending operation required')
    initial = [checkpoint_digest(home) for home in [a, b]]
    with wrong_pin_configuration(a) as config:
        denied_cycle(binary, a, config, 'Transport(Pin)')
    password = a['password']
    if not password:
        raise FixtureFailure('fixture home password required')
    wrong = dict(a, password=('a' if password[0] != 'a' else 'b') + password[1:])
    denied_cycle(binary, wrong, a['config'], 'Transport(Auth)')
    for mode, status in [('replaced-device', 'PeerUnavailable'), ('changed-identity', 'Protocol')]:
        with server_attack(profile, mode) as receipt:
            denied_cycle(binary, a, a['config'], status)
            counts = receipt()
            if counts['devices'] < 1 or counts['unexpected'] or counts['messages'] or (mode == 'changed-identity' and counts['bundles'] < 1):
                raise FixtureFailure('server denial control was not exercised')
    with server_attack(profile, 'added-device') as receipt:
        before = checkpoint_digest(a)
        command(binary, 'sync', a, a['config'], home_password=True)
        counts = receipt()
        if checkpoint_digest(a) != before or counts['devices'] < 1 or counts['bundles'] < 1 or counts['messages'] < 1 or counts['unexpected']:
            raise FixtureFailure('unverified device routing or checkpoint mismatch')
    if initial != [checkpoint_digest(home) for home in [a, b]]:
        raise FixtureFailure('negative controls changed canonical checkpoint')
    assert_no_plaintext(profile['server_roots'], [profile[key].encode() for key in ['alice_subject', 'alice_body', 'bob_subject', 'bob_body']])
    return list(NEGATIVE_CHECKS)


def load_profile(private_fixture):
    marker = Path('/etc/faction-disposable-test')
    if not marker.is_file() or b'209' not in marker.read_bytes():
        raise FixtureFailure('explicit disposable native marker required')
    path = Path(private_fixture)
    if path.is_symlink() or path.stat().st_mode & 0o077:
        raise FixtureFailure('private fixture profile required')
    profile = json.loads(path.read_text())
    return profile


def probe(binary, private_fixture):
    profile = load_profile(private_fixture)
    attack_identity(profile)
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
    checks.extend(negative_controls(binary, profile))
    return public_result(checks)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--negative-only', action='store_true', help='Use already initialized successful fixture stores')
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--private-fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    arguments = parser.parse_args()
    try:
        if arguments.negative_only:
            result = public_result(negative_controls(arguments.binary.resolve(), load_profile(arguments.private_fixture)), negative_only=True)
        else:
            result = probe(arguments.binary.resolve(), arguments.private_fixture)
        arguments.output.write_text(json.dumps(result, indent=2) + '\n')
        print('Native two-home encrypted-mail fixture passed:', len(result['checks']), 'controls')
    except (FixtureFailure, ValueError, OSError, KeyError):
        raise SystemExit('Native two-home fixture failed; private diagnostics withheld') from None
