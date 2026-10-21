#!/usr/bin/env python3
"""Real simultaneous local chat PTYs; no listener, credentials or transcripts printed."""
import argparse
import secrets
import sqlite3
import tempfile
from pathlib import Path
from smoke_local_console import Console


def smoke(binary):
    children = []
    password = secrets.token_hex(24)
    with tempfile.TemporaryDirectory(prefix='faction-chat-pty-') as root:
        state = Path(root) / 'board'

        def start(command, name, extra=()):
            c = Console(binary, state, command, extra)
            children.append(c)
            c.expect('Handle: ')
            c.send(name)
            return c

        def register(name, command='register'):
            c = start(command, name)
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.' if command == 'init' else 'Registered pending sysop approval.')
            c.finish()

        def login(name, live=False):
            c = start('login', name, ['--ansi'] if live else [])
            c.password(password)
            if live:
                c.expect('Press Enter to continue: ')
                c.send('')
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
            else:
                c.expect('Command (then Enter): ')
            return c

        def status(name, action):
            c = login('sysop')
            c.send('s')
            c.expect('[C]reate room: ')
            c.send(action)
            c.expect('Member handle: ')
            c.send(name)
            c.expect('Status updated; previous sessions expired.')
            c.expect('Command (then Enter): ')
            c.send('q')
            c.expect('Logged out.')
            c.finish()

        def chat(name, live=True):
            c = login(name, live)
            c.raw(b'c') if live else c.send('c')
            c.expect('chat[#lobby]> ')
            return c

        try:
            register('sysop', 'init')
            register('alice')
            register('bobby')
            status('alice', 'a')
            status('bobby', 'a')
            alice, bob = chat('alice'), chat('bobby')
            bob.raw(b'draft kept')
            bob.expect('chat[#lobby]> draft kept')
            alice.send('arrival while typing')
            bob.expect('< #1 alice> arrival while typing')
            bob.expect('chat[#lobby]> draft kept')
            bob.send('')
            alice.expect('< #2 bobby> draft kept')
            bob.send('/me nods')
            alice.expect('* #3 bobby nods')
            admin = chat('sysop')
            admin.send('/create #vault private')
            admin.expect('Channel created.')
            bob.send('/join #vault')
            bob.expect('action denied; draft discarded.')
            bob.expect('chat[-]> ')
            admin.send('/join #vault')
            admin.expect('Joined; new messages start here.')
            admin.expect('chat[#vault]> ')
            admin.send('before invite')
            admin.expect('< #1 sysop> before invite')
            admin.send('/allow #vault alice')
            admin.expect('Channel grant updated; target must join again.')
            alice.send('/join #vault')
            alice.expect('Joined; new messages start here.')
            alice.expect('chat[#vault]> ')
            admin.send('private after invite')
            alice.expect('< #2 sysop> private after invite')
            alice.raw(b'private unsent')
            alice.expect('chat[#vault]> private unsent')
            admin.send('/deny #vault alice')
            admin.expect('Channel grant updated; target must join again.')
            alice.expect('action denied; draft discarded.')
            alice.expect('chat[-]> ')
            alice.send('should not leak')
            alice.expect('join a channel first')
            bob.send('/join #lobby')
            bob.expect('Joined; new messages start here.')
            bob.send('/history')
            bob.expect('< #1 alice> arrival while typing')
            bob.expect('* #3 bobby nods')
            admin.send('/allow #vault alice')
            admin.expect('Channel grant updated; target must join again.')
            admin.send('after regrant before rejoin')
            admin.expect('< #3 sysop> after regrant before rejoin')
            alice.send('/join #vault')
            alice.expect('Joined; new messages start here.')
            alice.send('/history')
            alice.expect('No retained messages since this join.')
            admin.send('after rejoin')
            alice.expect('< #4 sysop> after rejoin')
            admin.send('/kick #vault alice')
            admin.expect('Member parted; a kick does not remove a private grant.')
            alice.expect('action denied; draft discarded.')
            alice.send('/join #vault')
            alice.expect('Joined; new messages start here.')
            alice.send("I'm back")
            admin.expect("< #5 alice> I'm back")
            admin.send('/names')
            admin.expect('Members: alice sysop')
            status('alice', 'v')
            alice.expect('session expired; log in again')
            alice.finish(code=1)
            admin.send('/names')
            admin.expect('Members: sysop')
            bob.raw(b'\x1bP/quit\x07Q\n/deny #lobby sysop\x1b\\')
            bob.send('/names')
            bob.expect('Members: bobby sysop')
            bob.raw(b'resize unsent')
            bob.expect('chat[#lobby]> resize unsent')
            bob.resize(20, 5)
            bob.expect('Chat closed after resize; unsent draft was not submitted: resize unsent')
            bob.expect('Press Enter to return to menu: ')
            bob.send('')
            bob.expect('Command (then Enter): ')
            bob.send('q')
            bob.expect('Logged out.')
            bob.finish()
            plain = chat('bobby', live=False)
            plain.send('/list')
            plain.expect('#lobby (board, local)')
            plain.send('plain delivery')
            plain.expect('< #4 bobby> plain delivery')
            plain.send('/poll')
            plain.expect('No new chat messages.')
            plain.send('/quit')
            plain.expect('Chat closed.')
            plain.expect('Command (then Enter): ')
            plain.send('q')
            plain.expect('Logged out.')
            plain.finish()
            admin.send('/quit')
            admin.expect('Chat closed.')
            admin.expect('Press Enter to return to menu: ')
            admin.send('')
            admin.expect('Ctrl-C exits; Esc: Enter to recover.')
            admin.raw(b'q')
            admin.expect('Logged out.')
            admin.finish()
            assert b'before invite' not in alice.transcript
            assert b'after regrant before rejoin' not in alice.transcript
            for private in [b'before invite', b'private after invite', b'private unsent', b'after rejoin']:
                assert private not in bob.transcript, 'private data reached outsider'
            with sqlite3.connect(state / 'faction.sqlite3') as db:
                assert db.execute('SELECT count(*) FROM chat_sessions').fetchone()[0] == 0
                assert db.execute('SELECT count(*) FROM chat_joins').fetchone()[0] == 0
                assert db.execute("SELECT count(*) FROM chat_messages WHERE body IN ('private unsent','should not leak','resize unsent')").fetchone()[0] == 0
            assert all(password.encode() not in c.transcript for c in children), 'credential leaked'
            print('Chat PTYs passed: simultaneous delivery, preserved draft, private grants/history floors, deny/regrant/kick, no draft redirection, revocation, control-tail isolation, resize, plain fallback and presence cleanup.')
        finally:
            for c in children:
                c.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    smoke(parser.parse_args().binary.resolve(strict=True))
