#!/usr/bin/env python3
"""Real CLI/PTY recovery roundtrip with disposable private data, no printed secrets."""
import argparse
import hashlib
import secrets
import sqlite3
import subprocess
import tempfile
from pathlib import Path
from smoke_local_console import Console


def smoke(binary):
    password = secrets.token_hex(24)
    children = []
    with tempfile.TemporaryDirectory(prefix='faction-recovery-') as name:
        root = Path(name)
        state, restored, snapshot = root / 'board', root / 'restored', root / 'export.sqlite3'

        def command(*args, code=0):
            result = subprocess.run([str(binary), *map(str, args)], capture_output=True, timeout=40)
            assert result.returncode == code, 'unexpected data-tool exit status'
            output = result.stdout + result.stderr
            assert password.encode() not in output and b'argon2id' not in output
            return output

        def start(action, handle, directory=state):
            c = Console(binary, directory, action)
            children.append(c)
            c.expect('Handle: ')
            c.send(handle)
            return c

        def register(handle, action='register'):
            c = start(action, handle)
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.' if action == 'init' else 'Registered pending sysop approval.')
            c.finish()

        def login(handle, directory=state):
            c = start('login', handle, directory)
            c.password(password)
            return c

        def logout(c):
            c.send('q')
            c.expect('Logged out.')
            c.finish()

        try:
            register('sysop', 'init')
            register('alice')
            register('pending')
            c = login('sysop')
            c.expect('Command (then Enter): ')
            c.send('s'); c.expect('[C]reate room: ')
            c.send('a'); c.expect('Member handle: ')
            c.send('alice'); c.expect('Status updated; previous sessions expired.')
            c.expect('Command (then Enter): ')
            logout(c)
            alice = login('alice')
            alice.expect('Command (then Enter): ')
            alice.send('p'); alice.expect('Room ID: '); alice.send('1')
            alice.expect('Reply to post ID (blank for new message): '); alice.send('')
            alice.expect('Message (one line, up to 1024 bytes): ')
            alice.send('recovery fixture post'); alice.expect('Posted #1.')
            alice.expect('Command (then Enter): ')
            alice.send('n'); alice.expect('Room ID: '); alice.send('1')
            alice.expect('recovery fixture post'); alice.expect('More? [y/N]: '); alice.send('n')
            alice.expect('Command (then Enter): ')
            alice.send('c'); alice.expect('chat[#lobby]> ')
            alice.send('retained before snapshot'); alice.expect('retained before snapshot')
            alice.expect('chat[#lobby]> ')

            assert b'Export complete:' in command('--state-dir', state, 'export', snapshot)
            assert b'Export verified:' in command('verify-export', snapshot)
            digest = hashlib.sha256(snapshot.read_bytes()).digest()
            assert snapshot.stat().st_mode & 0o777 == 0o600
            assert not any(Path(str(snapshot) + suffix).exists() for suffix in ['-wal', '-shm', '-journal'])
            with sqlite3.connect(snapshot) as db:
                assert db.execute('SELECT count(*) FROM chat_sessions').fetchone()[0] == 0
                assert db.execute('SELECT count(*) FROM chat_joins').fetchone()[0] == 0
                assert db.execute('SELECT last_read FROM unread').fetchone()[0] == 1
            # The source console and its lease continue unaffected by export.
            alice.send('after snapshot stays on source'); alice.expect('after snapshot stays on source')
            alice.expect('chat[#lobby]> ')
            assert b'Restore complete:' in command('--state-dir', restored, 'restore', snapshot)
            assert hashlib.sha256(snapshot.read_bytes()).digest() == digest
            assert restored.stat().st_mode & 0o777 == 0o700
            with sqlite3.connect(restored / 'faction.sqlite3') as db:
                assert db.execute('SELECT count(*) FROM chat_sessions').fetchone()[0] == 0
                assert db.execute('SELECT count(*) FROM chat_messages').fetchone()[0] == 1
            member = login('alice', restored)
            member.expect('Command (then Enter): ')
            member.send('h'); member.expect('Room ID: '); member.send('1')
            member.expect('recovery fixture post'); member.expect('More? [y/N]: '); member.send('n')
            member.expect('Command (then Enter): '); logout(member)
            pending = login('pending', restored)
            pending.expect('Account Pending; sysop approval required. No community access.'); pending.finish()

            command('--state-dir', restored, 'restore', snapshot, code=1)
            command('--state-dir', state, 'export', snapshot, code=1)
            assert hashlib.sha256(snapshot.read_bytes()).digest() == digest
            missing = root / 'misspelled'
            command('--state-dir', missing, 'export', root / 'not-created', code=1)
            assert not missing.exists() and not (root / 'not-created').exists()
            bad = root / 'bad.sqlite3'; bad.write_bytes(b'incomplete snapshot'); bad.chmod(0o600)
            command('--state-dir', root / 'bad-restore', 'restore', bad, code=1)
            assert not (root / 'bad-restore').exists()
            assert not list(root.glob('.faction-stage-*'))
            alice.send('/quit'); alice.expect('Command (then Enter): '); logout(alice)
            for c in children:
                assert password.encode() not in c.transcript, 'credential echoed'
        finally:
            for c in children:
                c.close()
    print('Export/verify/restore CLI + live WAL/PTY roundtrip, approval, no-overwrite and failure cleanup passed.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    smoke(args.binary.resolve())
