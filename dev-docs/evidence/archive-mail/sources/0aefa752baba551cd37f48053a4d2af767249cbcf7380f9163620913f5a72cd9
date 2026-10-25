#!/usr/bin/env python3
"""Private #211 two-home offline archive acceptance; never starts a server.

Use fresh generated accounts/stores in an owned fixture. The native executable
owns storage, crypto and protocol behavior. Only named controls leave this tool;
private inspections, credentials and opaque archive IDs remain captured locally.
"""
import argparse
import hashlib
import json
import os
import platform
import re
import stat
import subprocess
from pathlib import Path
import probe_home_mail as base

FixtureFailure = base.FixtureFailure
CHECKS = [
    'explicit_archive_enable_preserves_private_history_and_outbox',
    'five_offline_ciphertexts_recovered_in_pages_of_four_and_one',
    'authenticated_sce_content_and_time_survive_reopen',
    'offline_reply_follows_known_own_ciphertext_archive_echoes',
    'outbox_operation_ids_preserved_without_automatic_retirement',
    'empty_archive_page_keeps_exact_checkpoint',
    'wrong_pin_and_home_password_preserve_message_and_cursor',
]
PLAINTEXT_CHECK = 'private_content_absent_from_both_local_home_storage_and_logs'


def parsed(raw):
    if not isinstance(raw, bytes) or len(raw) > 4096:
        raise FixtureFailure('invalid bounded native metadata')
    try:
        value = json.loads(raw)
        if not isinstance(value, dict):
            raise ValueError()
        return value
    except (UnicodeError, ValueError) as error:
        raise FixtureFailure('invalid native metadata') from error


def page_result(raw):
    value = parsed(raw)
    if set(value) != {'entries', 'accepted', 'duplicates', 'complete'} or type(value['complete']) is not bool:
        raise FixtureFailure('invalid archive page result')
    if any(type(value[key]) is not int or not 0 <= value[key] <= 4 for key in ['entries', 'accepted', 'duplicates']):
        raise FixtureFailure('unbounded archive page result')
    if value['accepted'] + value['duplicates'] != value['entries']:
        raise FixtureFailure('inconsistent archive page counts')
    return value


def archive_status(raw):
    value = parsed(raw)
    if set(value) != {'enabled', 'cursor_present', 'ready', 'checkpoint'} or any(type(value[key]) is not bool for key in ['enabled', 'cursor_present', 'ready']):
        raise FixtureFailure('invalid archive status')
    if value['cursor_present'] and not value['enabled']:
        raise FixtureFailure('invalid disabled cursor')
    token = value['checkpoint']
    if not value['ready']:
        if token is not None:
            raise FixtureFailure('inactive cursor disclosure')
        return value
    if not value['enabled'] or not isinstance(token, dict) or set(token) != {'home', 'local_device', 'epoch', 'after'}:
        raise FixtureFailure('invalid ready archive checkpoint')
    if not isinstance(token['home'], str) or not re.fullmatch(r'[a-z0-9._-]+@[a-z0-9.-]+', token['home']):
        raise FixtureFailure('invalid archive scope')
    if type(token['local_device']) is not int or not 1 <= token['local_device'] <= 2147483647:
        raise FixtureFailure('invalid archive device')
    if not isinstance(token['epoch'], str) or not re.fullmatch('[a-f0-9]{32}', token['epoch']) or token['epoch'] == '0' * 32:
        raise FixtureFailure('invalid archive epoch')
    after = token['after']
    if after is not None and (not isinstance(after, str) or not 1 <= len(after) <= 256 or any(not 33 <= ord(char) <= 126 for char in after)):
        raise FixtureFailure('invalid archive identifier')
    if value['cursor_present'] != (after is not None):
        raise FixtureFailure('inconsistent archive status')
    return value


def status(binary, home):
    return archive_status(base.command(binary, 'archive-status', home, raw=True))


def catchup(binary, home):
    return page_result(base.command(binary, 'catchup', home, home['config'], home_password=True, raw=True))


def public_result(checks, audit, marker_sha256):
    expected = list(CHECKS)
    if audit == 'pass-local-scan':
        expected.append(PLAINTEXT_CHECK)
    elif audit != 'requires-separate-remote-proof':
        raise FixtureFailure('unknown server audit scope')
    if checks != expected or not re.fullmatch('[a-f0-9]{64}', marker_sha256):
        raise FixtureFailure('incomplete archive controls')
    return dict(result='pass', platform=platform.system(), checks=checks, count=len(checks),
                native_product_client=True, configured_homes=2, server_plaintext_audit=audit,
                marker_sha256=marker_sha256, live_provider_contact=False,
                no_complete_archive_or_read_receipt_claim=True)


def load_profile(path):
    path = Path(path).absolute()
    parent = path.parent.lstat()
    if not stat.S_ISDIR(parent.st_mode) or parent.st_uid != os.getuid() or parent.st_mode & 0o777 != 0o700 or not path.parent.name.startswith('faction211'):
        raise FixtureFailure('owned private archive fixture directory required')
    raw, _ = base.private_bytes(path, 32768, os.getuid())
    try:
        profile = json.loads(raw)
        marker = profile['marker']
        marker_path = Path(profile['fixture_marker'])
        if not isinstance(marker, str) or not re.fullmatch('[a-f0-9]{32}', marker) or marker_path.parent != path.parent:
            raise ValueError()
        guard, _ = base.private_bytes(marker_path, 1024, os.getuid())
        if b'211' not in guard:
            raise ValueError()
        if not isinstance(profile['alice'], dict) or not isinstance(profile['bob'], dict):
            raise ValueError()
        return profile
    except (TypeError, ValueError, KeyError) as error:
        raise FixtureFailure('explicit private archive fixture marker required') from error


def require_page(page, entries, accepted, duplicates, complete):
    if page != dict(entries=entries, accepted=accepted, duplicates=duplicates, complete=complete):
        raise FixtureFailure('unexpected bounded archive page')


def cursor_progress(previous, current):
    if not previous['ready'] or not current['ready']:
        raise FixtureFailure('archive cursor unavailable')
    before, after = previous['checkpoint'], current['checkpoint']
    if any(before[key] != after[key] for key in ['home', 'local_device', 'epoch']) or after['after'] is None or after['after'] == before['after']:
        raise FixtureFailure('archive cursor scope or progress changed')


def require_authenticated_times(actual, sent, direction):
    reference = {(row['subject'], row['body']): row['sent_at'] for row in sent['history'] if row['direction'] == 'sent'}
    for row in actual['history']:
        if row['direction'] == direction and reference.get((row['subject'], row['body'])) != row['sent_at']:
            raise FixtureFailure('archive changed authenticated sending time')


def denied_catchup(binary, home, config, expected):
    before = base.checkpoint_digest(home)
    cursor = status(binary, home)
    argv, options = base._invocation(binary, 'catchup', home, [config], True, ())
    try:
        result = subprocess.run(argv, **options)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise FixtureFailure('archive denial did not complete') from error
    if result.returncode != 1 or result.stdout or result.stderr != (expected + '\n').encode():
        raise FixtureFailure('unexpected archive denial')
    if base.checkpoint_digest(home) != before or status(binary, home) != cursor:
        raise FixtureFailure('archive denial changed canonical state or cursor')


def probe(binary, profile):
    a, b = profile['alice'], profile['bob']
    if a['jid'].split('@')[-1] == b['jid'].split('@')[-1]:
        raise FixtureFailure('two distinct configured homes required')
    marker = profile['marker']
    marker_sha256 = hashlib.sha256(marker.encode()).hexdigest()
    checks = []
    a_pin = base.command(binary, 'init', a, a['jid'], a['device'], raw=True).decode().strip()
    b_pin = base.command(binary, 'init', b, b['jid'], b['device'], raw=True).decode().strip()
    if a_pin == b_pin or any(not re.fullmatch('[a-f0-9]{64}', pin) for pin in [a_pin, b_pin]):
        raise FixtureFailure('independent verified fixture identities required')
    base.command(binary, 'enroll', a, b['jid'], b['device'], b_pin)
    base.command(binary, 'enroll', b, a['jid'], a['device'], a_pin)
    base.command(binary, 'publish', b, b['config'], home_password=True)
    base.command(binary, 'prepare', a, a['config'], home_password=True)
    for home in [a, b]:
        before = base.inspection(base.command(binary, 'inspect', home, raw=True))
        if status(binary, home)['enabled']:
            raise FixtureFailure('fresh v2 fixture expected before migration')
        base.command(binary, 'enable-archive', home)
        if before != base.inspection(base.command(binary, 'inspect', home, raw=True)) or not status(binary, home)['ready']:
            raise FixtureFailure('explicit archive migration changed member state')
    checks.append(CHECKS[0])
    messages = [(f'{marker} incoming {index}', f'{marker} private body {index} λ <&>') for index in range(1, 6)]
    for subject, body in messages:
        base.command(binary, 'queue', a, lines=[subject, body])
    sent = base.inspection(base.command(binary, 'inspect', a, raw=True))
    base.require_history(sent, [('sent', subject, body) for subject, body in messages], 5)
    offline = base.checkpoint_digest(b)
    # Bob has no running client. A complete sender process exits before catch-up.
    base.command(binary, 'sync', a, a['config'], home_password=True)
    if base.checkpoint_digest(b) != offline:
        raise FixtureFailure('offline receiver checkpoint changed without a client')
    before = status(binary, b)
    require_page(catchup(binary, b), 4, 4, 0, False)
    first = status(binary, b); cursor_progress(before, first)
    if status(binary, b) != first:
        raise FixtureFailure('archive cursor did not survive reopen')
    require_page(catchup(binary, b), 1, 1, 0, True)
    cursor_progress(first, status(binary, b))
    checks.append(CHECKS[1])
    received = base.inspection(base.command(binary, 'inspect', b, raw=True))
    base.require_history(received, [('received', subject, body) for subject, body in messages], 0)
    require_authenticated_times(received, sent, 'received')
    checks.append(CHECKS[2])
    reply = (f'{marker} reply', f'{marker} offline reply body λ')
    base.command(binary, 'queue', b, lines=list(reply))
    reply_sent = base.inspection(base.command(binary, 'inspect', b, raw=True))
    offline = base.checkpoint_digest(a)
    base.command(binary, 'sync', b, b['config'], home_password=True)
    if base.checkpoint_digest(a) != offline:
        raise FixtureFailure('offline sender checkpoint changed without a client')
    before = status(binary, a)
    require_page(catchup(binary, a), 4, 0, 4, False)
    first = status(binary, a); cursor_progress(before, first)
    require_page(catchup(binary, a), 2, 1, 1, True)
    cursor_progress(first, status(binary, a))
    final_a = base.inspection(base.command(binary, 'inspect', a, raw=True))
    final_b = base.inspection(base.command(binary, 'inspect', b, raw=True))
    base.require_history(final_a, [('sent', subject, body) for subject, body in messages] + [('received', *reply)], 5)
    base.require_history(final_b, [('received', subject, body) for subject, body in messages] + [('sent', *reply)], 1)
    require_authenticated_times(final_a, reply_sent, 'received')
    checks.append(CHECKS[3])
    if final_a['pending'] != sent['pending'] or final_b['pending'] != reply_sent['pending']:
        raise FixtureFailure('archive catch-up changed durable outbox operation IDs')
    checks.append(CHECKS[4])
    before = base.checkpoint_digest(a); cursor = status(binary, a)
    require_page(catchup(binary, a), 0, 0, 0, True)
    if base.checkpoint_digest(a) != before or status(binary, a) != cursor:
        raise FixtureFailure('empty archive page changed checkpoint')
    checks.append(CHECKS[5])
    with base.wrong_pin_configuration(a) as wrong_pin:
        denied_catchup(binary, a, wrong_pin, 'Transport(Pin)')
    password = a['password']
    if not password:
        raise FixtureFailure('fixture home password required')
    wrong = dict(a, password=('a' if password[0] != 'a' else 'b') + password[1:])
    denied_catchup(binary, wrong, a['config'], 'Transport(Auth)')
    checks.append(CHECKS[6])
    roots = profile.get('server_roots', [])
    if roots:
        if not isinstance(roots, list) or len(roots) < 2:
            raise FixtureFailure('both home storage roots required')
        base.assert_no_plaintext(roots, [marker.encode()])
        checks.append(PLAINTEXT_CHECK)
        audit = 'pass-local-scan'
    else:
        audit = 'requires-separate-remote-proof'
    return public_result(checks, audit, marker_sha256)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--private-fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--disposable-synthetic-fixture', action='store_true', required=True)
    arguments = parser.parse_args()
    try:
        result = probe(arguments.binary.resolve(), load_profile(arguments.private_fixture))
        arguments.output.write_text(json.dumps(result, indent=2) + '\n')
        print('Native offline archive fixture passed:', result['count'], 'controls;', result['platform'])
    except (FixtureFailure, OSError, ValueError, KeyError, TypeError):
        raise SystemExit('Native archive fixture failed; private diagnostics withheld') from None
