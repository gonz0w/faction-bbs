#!/usr/bin/env python3
"""Real PTY checks of wrapped room reading and unread boundaries; private fixture.

No listener, network, real member data or transcript publication. Run after build.
"""
import argparse
import secrets
import tempfile
from pathlib import Path
from smoke_local_console import Console


def smoke(binary):
    children = []
    password = secrets.token_hex(20)
    with tempfile.TemporaryDirectory(prefix='faction-reader-') as root:
        board = Path(root)/'board'

        def start(command, name, *extra):
            c = Console(binary, board, command, extra, columns=40, rows=18)
            children.append(c)
            c.expect('Handle: ')
            c.send(name)
            return c

        def login(ansi=False):
            c = start('login', 'sysop', *(['--ansi'] if ansi else []))
            c.password(password)
            if ansi:
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
            else:
                c.expect('Command (then Enter): ')
            return c

        def choice(c, key, ansi):
            if ansi:
                c.raw(key.encode())
            else:
                c.send(key)

        def back(c, ansi):
            if ansi:
                c.expect('Press Enter to return to menu: ')
                c.send('')
                c.expect('Ctrl-C exits; Esc: Enter to recover.')
            else:
                c.expect('Command (then Enter): ')

        def unread(c, count, ansi):
            choice(c, 'r', ansi)
            c.expect(f'Lobby ({count} unread)')
            back(c, ansi)

        try:
            c = start('init', 'sysop')
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Sysop initialized. Lobby created.')
            c.finish()
            body = ' '.join(f'word{i:03d}' for i in range(100)) + ' FINAL-MARKER'
            for ansi in [False, True]:
                c = login(ansi)
                choice(c, 'p', ansi)
                c.expect('Room ID: '); c.send('1')
                c.expect('Reply to post ID (blank for new message): '); c.send('')
                c.expect('Message (one line, up to 1024 bytes): '); c.send(body)
                c.expect('Posted #'); back(c, ansi)
                unread(c, 1, ansi)
                choice(c, 'n', ansi); c.expect('Room ID: '); c.send('1')
                c.expect('Room #1 | chronological messages')
                c.expect('Subject: Message #')
                c.expect('Posted: ')
                c.expect(' UTC')
                c.expect('More? [y/N]: ')
                c.send('n'); back(c, ansi)
                # Declining a partial message must not consume its unread state.
                unread(c, 1, ansi)
                choice(c, 'n', ansi); c.expect('Room ID: '); c.send('1')
                first = len(c.transcript)
                for page in range(30):
                    c.expect('More? [y/N]: ')
                    if b'FINAL-MARKER' in c.transcript[first:]:
                        break
                    c.send('y')
                else:
                    raise AssertionError('long message did not reach its final page')
                transcript = c.transcript[first:]
                assert all(transcript.count(f'word{i:03d}'.encode()) == 1 for i in range(100))
                assert (b'\x1b[2J\x1b[H' in transcript) == ansi
                c.send('n'); back(c, ansi)
                unread(c, 0, ansi)
                choice(c, 'q', ansi); c.expect('Logged out.'); c.finish()
            print('Room PTY checks passed: plain/ANSI 40x18, wrapping/order, section clear, partial-unread retention and complete-message acknowledgment.')
        finally:
            for c in children:
                c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path)
    smoke(p.parse_args().binary.resolve())
