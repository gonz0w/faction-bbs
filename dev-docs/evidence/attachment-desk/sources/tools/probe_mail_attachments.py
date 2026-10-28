#!/usr/bin/env python3
"""Actual production-console attachment controls over owned loopback TLS homes.

No installed services or real accounts. Private captures stay in an explicitly
owned directory; ephemeral keys, homes, vaults and children are torn down. This
software fixture is not provider deployment, NetBSD, SSH or capacity certification.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import stat
import subprocess
import tempfile
import time

from probe_attachment_transfer import (Handler, HomePeer, _away_certificate,
    _run, _settings)
from probe_xmpp_transport import certificates
from smoke_board_launcher import LauncherConsole

UNLOCK = 'LOCAL checkpoint passphrase (hidden): '
MENU = '[L] Lock and exit'
DESK = '[R] Received attachment  [B] Back'
RECEIVED = '[S] Save verified file  [B] Back'
HOME = 'OWN HOME account password (hidden): '
QUEUE = 'Type queue to encrypt and persist attachment: '
TRANSFER = 'Type transfer to connect to YOUR home: '


def owned(path):
    info = path.lstat()
    if (path.is_symlink() or not stat.S_ISDIR(info.st_mode)
            or info.st_uid != os.getuid() or info.st_mode & 0o777 != 0o700
            or (path/'OWNED').read_bytes() != b'Owned native attachment console proof\n'):
        raise ValueError('an owned private console proof root is required')
    return path.resolve(strict=True)


def private(path, data):
    with path.open('wb') as out:
        path.chmod(0o600)
        out.write(data)


def close_resources(children, peers, handler):
    failures = []
    for resource in [*children, *[peer for peer in peers if peer is not None], handler]:
        try:
            resource.close()
        except BaseException as error:
            failures.append(error)
    if failures:
        raise RuntimeError('fixture cleanup failed') from failures[0]


def await_fact(predicate, seconds=3):
    deadline = time.monotonic()+seconds
    while not predicate():
        if time.monotonic() >= deadline:
            raise AssertionError('owned peer fact not observed before deadline')
        time.sleep(.01)


def probe(client, fixture, reply_fixture, owned_root, openssl):
    owned_root = owned(owned_root)
    checks = []
    began = time.monotonic()
    # This secret exists only on synthetic member devices and TLS test homes.
    local_password = secrets.token_hex(20)
    home_password = secrets.token_hex(20)
    secret = (local_password+'\n'+home_password+'\n').encode()
    with tempfile.TemporaryDirectory(prefix='tls-console-OWNED-', dir=owned_root) as name:
        root = Path(name).resolve()
        root.chmod(0o700)
        private(root/'OWNED', b'Owned disposable attachment TLS proof\n')
        pairs = certificates(root, openssl)
        away = _away_certificate(root, openssl)
        _run(fixture, root, 'init', secret)
        handler = Handler(fixture, root)
        alice = bob = None
        children = []
        try:
            alice = HomePeer(root, pairs['good'], 'alice@home.test', 101,
                             home_password, handler, 0)
            bob = HomePeer(root, away, 'bob@away.test', 202,
                           home_password, handler, 1)
            ac = _settings(root, 'alice', alice, pairs['good'][0])
            bc = _settings(root, 'bob', bob, away[0])
            def start(member, config, compact=False, unicode=False):
                c = LauncherConsole([client, '--mail-online', root/member, config]
                                    + (['--unicode'] if unicode else []),
                                    columns=40 if compact else 80,
                                    rows=18 if compact else 24)
                children.append(c)
                c.password(local_password, UNLOCK)
                c.expect(MENU)
                return c
            def desk(c):
                c.raw(b'm'); c.expect(DESK)
            def lock(c, in_received=False):
                if in_received:
                    c.raw(b'b'); c.expect(DESK)
                c.raw(b'b'); c.expect(MENU); c.raw(b'l'); c.finish()
            def compose(c, source, approved, typeahead=False, after_preview=None):
                c.raw(b'c')
                for prompt, answer in [
                    ('Absolute LOCAL source path (blank cancels): ', str(source)),
                    ('Private display name (255 UTF-8 bytes): ', 'private-selected-name.bin\u202e'),
                    ('Media type (127 ASCII bytes): ', 'application/octet-stream'),
                    ('Subject (128 UTF-8 bytes): ', 'private-selected-subject'),
                    ('Body (1024 UTF-8 bytes, one line): ', 'private-selected-body')]:
                    c.expect(prompt)
                    if typeahead and prompt.startswith('Body '):
                        c.raw((answer+'\nqueue\n').encode())
                    else:
                        c.send(answer)
                c.expect(QUEUE)
                if after_preview is not None: after_preview()
                c.send('queue' if approved else '')
                c.expect(DESK)
            def upload(c, operation):
                c.raw(b'u'); c.expect('Full operation ID (blank cancels): ')
                c.send(operation); c.expect(TRANSFER); c.send('transfer')
                c.password(home_password, HOME)
            def selection(c):
                c.raw(b'r'); c.expect('Original record number (blank cancels): ')
                c.send('1'); c.expect(RECEIVED)
            def fetch(c, invited):
                c.raw(b'f'); c.expect('Type invite for this fetch only: ')
                c.send('invite' if invited else '')
                if invited:
                    c.expect(TRANSFER); c.send('transfer'); c.password(home_password, HOME)
            def save(c, destination, approved):
                c.raw(b's'); c.expect('Absolute save path (blank cancels): ')
                c.send(str(destination)); c.expect('Type save to verify and publish: ')
                c.send('save' if approved else '')
                c.expect(RECEIVED)

            # Existing prepared FACTCP04 fixture proves fresh opt-in in the real UI.
            migration = root/'migration'
            fixed_pass = 'checkpoint synthetic fixture password'
            seeded = subprocess.run([str(reply_fixture), 'seed', str(migration), 'ready'],
                input=b'preserved subject\npreserved body\n', capture_output=True, timeout=30)
            assert seeded.returncode == 0
            def inspect_migration():
                result = subprocess.run([str(reply_fixture), 'inspect', str(migration)],
                                        capture_output=True, timeout=30)
                assert result.returncode == 0
                return json.loads(result.stdout)
            before_state = inspect_migration()
            before_bytes = (migration/'mail.fkv').read_bytes()
            c = LauncherConsole([client, '--mail-online', migration, ac], columns=40, rows=18)
            children.append(c); c.password(fixed_pass, UNLOCK); c.expect(MENU); desk(c)
            c.raw(b'e'); c.expect('Type enable to adopt FACTCP05: '); c.send(''); c.expect(DESK)
            assert (migration/'mail.fkv').read_bytes() == before_bytes
            c.raw(b'e'); c.expect('Type enable to adopt FACTCP05: '); c.send('enable'); c.expect(DESK)
            lock(c)
            assert inspect_migration() == before_state
            assert (migration/'mail.fkv').read_bytes() != before_bytes
            assert not (migration/'attachments').exists() and alice.connections == bob.connections == 0
            checks.append('fresh_factcp05_decline_and_enable_preserve_mail_read_cursor_pending_without_spool_or_network')

            # Normal and compact explicit-Unicode metadata paths are local only.
            for compact, unicode in [(False, False), (True, True)]:
                before = (root/'alice/mail.fkv').read_bytes()
                c = start('alice', ac, compact, unicode); desk(c)
                c.raw(b'r'); c.expect('No received attachment.'); c.expect(DESK)
                lock(c)
                assert not (root/'alice/attachments').exists()
                assert (root/'alice/mail.fkv').read_bytes() == before
                assert alice.connections == bob.connections == 0
            checks.append('classic_and_compact_unicode_navigation_no_spool_or_connection')
            source = root/'source-\u202e.bin'
            payload = bytes((i*73+19) % 251 for i in range(65_536))
            private(source, payload); source.chmod(0o644)
            before = (root/'alice/mail.fkv').read_bytes()
            c = start('alice', ac, compact=True); desk(c)
            compose(c, source, False, typeahead=True)
            assert (root/'alice/mail.fkv').read_bytes() == before
            assert not (root/'alice/attachments').exists() and alice.connections == 0
            checks.append('fresh_confirmation_discards_queue_typeahead_and_cancel_preserves_checkpoint_without_sidecar')
            compose(c, source, True, after_preview=lambda: private(source, b'changed after review'))
            assert b'\\u{202e}' in c.transcript and '\u202e'.encode() not in c.transcript
            ids = re.findall(rb'Operation: ([0-9a-f]{32})', c.transcript)
            assert len(ids) == 1
            operation = ids[0].decode()
            assert alice.connections == 0 and not (root/'delivery.xml').exists()
            source.unlink()  # Upload must use durable ciphertext, not this source.
            alice.drop_response = True
            upload(c, operation)
            c.expect('Transfer stopped; no automatic retry.'); c.expect(DESK)
            assert alice.dropped and alice.messages == 0
            lock(c)
            checks.append('queue_is_local_and_lost_response_stops_without_implicit_retry')
            c = start('alice', ac); desk(c); upload(c, operation)
            c.expect('Foreground transfer complete.'); c.expect(DESK)
            await_fact(lambda: alice.messages == 1)
            assert re.search(rb'SDK call: [2-9] / 65', c.transcript)
            assert c.transcript.count(HOME.encode()) == 1
            lock(c)
            assert not source.exists()
            checks.append('reopened_exact_operation_multicycle_upload_with_one_hidden_password_without_source')
            # Cancel while an actual authenticated storage reply is deliberately stalled.
            c = start('alice', ac); desk(c)
            alice.delay_replies = True
            upload(c, operation)
            await_fact(lambda: 'begin' in alice.delay_phases)
            c.raw(b'\x03'); c.expect('Home mail locked: local operation stopped.')
            c.finish(code=1)
            assert alice.messages == 1
            checks.append('ctrl_c_cancels_active_authenticated_transfer_locks_and_restores_terminal')
            sender_connections = alice.connections
            alice.close(); alice = None
            _run(fixture, root, 'receive', secret)
            before = (root/'bob/mail.fkv').read_bytes()
            c = start('bob', bc, compact=True, unicode=True); desk(c); selection(c)
            assert not (root/'bob/attachments').exists()
            assert b'\\u{202e}' in c.transcript and '\u202e'.encode() not in c.transcript
            connections = bob.connections
            fetch(c, False); c.expect('Fetch cancelled; no connection.'); c.expect(RECEIVED)
            assert bob.connections == connections and not (root/'bob/attachments').exists()
            checks.append('authenticated_metadata_and_declined_foreign_route_are_local_only')
            fetch(c, True); c.expect('Foreground transfer complete.'); c.expect(RECEIVED)
            assert c.transcript.count(HOME.encode()) == 1
            assert re.search(rb'SDK call: [2-9] / 65', c.transcript)
            assert not (root/'selected-save').exists()
            c.raw(b'v'); c.expect('Verified locally: 65536 plaintext bytes.'); c.expect(RECEIVED)
            destination = root/'selected-save'/'explicit.bin'
            destination.parent.mkdir(mode=0o700)
            save(c, destination, False); assert not destination.exists()
            save(c, destination, True)
            assert destination.read_bytes() == payload
            assert destination.stat().st_mode & 0o777 == 0o600
            save(c, destination, True)
            assert b'Save failed before publication.' in c.transcript
            assert destination.read_bytes() == payload
            lock(c, True)
            assert (root/'bob/mail.fkv').read_bytes() == before
            checks.append('sender_offline_own_home_multicycle_fetch_local_verify_explicit_private_save_no_overwrite')
            _run(fixture, root, 'audit', secret)
            checks.append('server_store_and_wire_exclude_private_metadata_keys_and_plaintext')
            for c in children:
                assert local_password.encode() not in c.transcript
                assert home_password.encode() not in c.transcript
            c = start('bob', bc); desk(c); selection(c)
            c.raw(b's'); c.expect('Absolute save path (blank cancels): ')
            c.raw(b'\x04'); c.expect('Home mail locked: local operation stopped.')
            c.finish(code=1)
            assert destination.read_bytes() == payload
            checks.append('ctrl_d_locks_at_save_selection_without_publication')
            checks.append('hostile_bidi_source_and_authenticated_filename_remain_inert_in_classic_and_unicode')
            checks.append('hidden_passwords_absent_from_pty_output_and_terminal_echo_restored')
            facts = dict(sender_connections=sender_connections,
                         recipient_connections=bob.connections,
                         max_requests_per_session=max(bob.max_requests, 0),
                         exact_plaintext_bytes=len(payload))
        finally:
            # Captures are deliberately retained privately, including failed variants.
            try:
                for c in children:
                    path = owned_root/('console-'+secrets.token_hex(12)+'.private')
                    private(path, c.transcript)
            finally:
                close_resources(children, [alice, bob], handler)
    assert not root.exists()
    return dict(result='pass', count=len(checks), checks=checks,
                seconds=round(time.monotonic()-began, 3),
                private_fixture_removed=True, children_reaped=True,
                provider_adopted=False, netbsd_qualification=False,
                ssh_trace=False, capacity_claim=False, **facts)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--reply-fixture', type=Path, required=True)
    parser.add_argument('--owned-root', type=Path, required=True)
    parser.add_argument('--openssl', default='openssl')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = probe(args.client.resolve(), args.fixture.resolve(), args.reply_fixture.resolve(), args.owned_root, args.openssl)
    private(args.output, (json.dumps(result, indent=2, sort_keys=True)+'\n').encode())
    print('Native attachment console passed', result['count'], 'controls.')
