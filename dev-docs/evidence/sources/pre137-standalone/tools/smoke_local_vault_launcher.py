#!/usr/bin/env python3
"""Device-only vault/picker PTYs: generated secrets, private temporary state, no SSH."""
import argparse
import json
from pathlib import Path
import re
import secrets
import socket
import subprocess
import tempfile
from smoke_board_launcher import LauncherConsole, PICK, catalog
from smoke_native_client_ssh import NativeConsole

RETURN = 'Enter to return to local Boards: '
UNLOCK = 'LOCAL passphrase (hidden; not a BBS password): '
MAIL = 'Lock and exit'


def run(binary, output=None):
    checks = []
    children = []
    password = secrets.token_hex(20)
    subject = 'local-subject-' + secrets.token_hex(8)
    body = 'local-body-' + secrets.token_hex(12)
    with tempfile.TemporaryDirectory(prefix='faction-picker-vault-') as directory, socket.socket() as trap:
        root = Path(directory).resolve()
        root.chmod(0o700)
        store = root / 'mail'
        trap.bind(('127.0.0.1', 0));trap.listen();trap.settimeout(.2)
        def private(path, text):
            path.write_text(text);path.chmod(0o600)
        paths = [root / f'board-{i}.toml' for i in range(16)]
        # Profile revalidation is tested by the existing launcher suite. This
        # deliberately unusable profile must never be opened by local mail.
        for path in paths:
            private(path, 'invalid-profile-no-ssh')
        pack = root / 'boards.toml'
        def configure(path=store, enabled=True):
            text = catalog(paths)
            if enabled:
                text = text.replace('version=1\n', 'version=1\nmail_store=' + json.dumps(str(path)) + '\n')
            private(pack, text)
        def picker(compact=False):
            c = LauncherConsole([binary, '--boards', pack], columns=40 if compact else 80,
                                rows=18 if compact else 24)
            children.append(c);c.expect(PICK)
            return c
        def enter(c, secret=password):
            c.raw(b'm');c.password(secret, UNLOCK)
        def lock(c):
            c.raw(b'l');c.expect(PICK)
        def quit(c):
            c.raw(b'q');c.finish()
        try:
            c = NativeConsole([str(binary), '--mail-store', str(store), 'init']);children.append(c)
            c.expect('Your full home identity (local claim only): ');c.send('cam@home.test')
            c.password(password, 'New LOCAL passphrase (12-128 characters, hidden): ')
            c.password(password, 'Repeat LOCAL passphrase (hidden): ')
            c.expect('Save it privately, then Enter to clear this screen: ');c.send('');c.expect(MAIL)
            c.raw(b'l');c.finish()
            configure();c = picker();c.expect('M: local offline vault (no SSH)')
            c.raw(b'\x1b[B');c.expect('> [B] Local Board 2')
            enter(c);c.expect(MAIL)
            processes = subprocess.check_output(['ps', '-ax', '-o', 'pid,ppid,comm'], text=True)
            children_now = [row.split(None, 2) for row in processes.splitlines()[1:] if row.strip()]
            assert not any(len(row) == 3 and row[1] == str(c.pid) for row in children_now)
            mark = len(c.transcript)
            lock(c);assert b'> [B] Local Board 2' in c.transcript[mark:];quit(c)
            checks.append('explicit_private_catalog_mail_opens_local_vault_and_returns_locked')
            c = picker();enter(c);c.expect(MAIL)
            c.raw(b'd');c.expect('To (full user@home): ');c.send('friend@away.test')
            c.expect('Subject: ');c.send(subject);c.expect('Body (one line, up to 4096 bytes): ');c.send(body)
            c.expect('NOT sent or queued for delivery.');c.expect('Enter to return: ');c.send('');c.expect(MAIL)
            lock(c);quit(c)
            encrypted = (store / 'mail.fkv').read_bytes()
            assert all(v.encode() not in encrypted for v in (password, subject, body, 'cam@home.test'))
            c = picker();enter(c);c.expect(MAIL);c.raw(b'v');c.expect(subject);c.expect(body)
            c.expect('Enter to return: ');c.send('');c.expect('Enter to return: ');c.send('');c.expect(MAIL)
            lock(c);quit(c)
            checks.append('picker_drafts_remain_encrypted_unsent_and_survive_reopen')
            c = picker();enter(c, 'incorrect-local-passphrase');c.expect('Local vault closed: ')
            assert body.encode() not in c.transcript;c.expect(RETURN);c.send('');c.expect(PICK);quit(c)
            checks.append('wrong_local_secret_returns_to_picker_without_draft_render')
            c = picker();c.raw(b'm');c.expect(UNLOCK);c.raw(b'\x03')
            c.expect('Local vault closed: ');c.expect(RETURN);c.send('');c.expect(PICK)
            enter(c);c.expect(MAIL);lock(c);quit(c)
            checks.append('cancel_relocks_and_restores_picker_then_reopens')
            c = picker(compact=True)
            frame = re.sub(rb'\x1b\[[0-9;?]*[ -/]*[@-~]', b'', c.transcript).decode()
            assert len(frame.splitlines()) <= 18 and all(len(row) <= 39 for row in frame.splitlines())
            enter(c);c.expect(MAIL);lock(c);quit(c)
            checks.append('sixteen_board_catalog_and_optional_vault_fit_40x18')
            configure(root / 'missing')
            c = picker();c.raw(b'm');c.expect('Local vault closed: ');c.expect(RETURN);c.send('');c.expect(PICK);quit(c)
            assert not (root / 'missing').exists()
            checks.append('missing_store_is_not_created_or_recovered_implicitly')
            configure(enabled=False)
            c = picker();c.raw(b'm');c.expect(PICK);assert b'LOCAL passphrase' not in c.transcript;quit(c)
            checks.append('unconfigured_mail_hotkey_has_no_unlock_authority')
            try:
                connection, _ = trap.accept()
            except TimeoutError:
                pass
            else:
                connection.close();raise AssertionError('local vault unexpectedly connected')
            assert all(password.encode() not in c.transcript for c in children)
            checks.append('local_mail_never_reads_bad_board_profiles_or_connects_and_secrets_not_echoed')
        finally:
            for c in children:
                c.close()
    result = dict(result='pass', count=len(checks), checks=checks, capacity_claim=False,
                  network_mail_implemented=False, remote_unlock_authority=False)
    if output:
        output.write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
    print('Local vault/board picker passed:', len(checks), 'controls; no network mail.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    run(args.binary.resolve(), args.output)
