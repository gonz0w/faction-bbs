#!/usr/bin/env python3
"""UTF-8 subject composing through real local/worker PTYs; private synthetic data."""
import argparse
import secrets
import sqlite3
import tempfile
import termios
from pathlib import Path
from smoke_local_console import Console
from smoke_member_worker import Worker


def smoke(binary, member):
    children = []
    password = secrets.token_hex(20)
    checks = []
    with tempfile.TemporaryDirectory(prefix='faction-subjects-') as root:
        board = Path(root)/'board'

        def start(mode):
            if mode == 'worker':
                c = Worker(member, board); children.append(c)
                c.expect('[L]ogin [R]egister [Q]uit: '); c.send('l')
            else:
                c = Console(binary, board, 'login', ['--ansi'] if mode == 'ansi' else [], columns=40, rows=18)
                children.append(c)
            c.expect('Handle: '); c.send('sysop'); c.password(password)
            if mode == 'worker':
                c.expect('Press Enter to continue: '); c.send('')
            c.expect('Command (then Enter): ' if mode == 'plain' else 'Ctrl-C exits; Esc: Enter to recover.')
            return c

        def choice(c, mode, key):
            c.send(key) if mode == 'plain' else c.raw(key.encode())

        def back(c, mode):
            if mode != 'plain':
                c.expect('Press Enter to return to menu: '); c.send('')
            c.expect('Command (then Enter): ' if mode == 'plain' else 'Ctrl-C exits; Esc: Enter to recover.')

        def draft(c, mode, parent=''):
            choice(c, mode, 'p'); c.expect('Room ID: '); c.send('1')
            c.expect('Reply to post ID (blank for new message): '); c.send(parent)
            c.expect('Subject (blank for generated/reply default): ')

        def stored():
            with sqlite3.connect(board/'faction.sqlite3') as db:
                return db.execute('SELECT subject FROM post_metadata ORDER BY post_id').fetchall()

        try:
            c = Console(binary, board, 'init'); children.append(c)
            c.expect('Handle: '); c.send('sysop')
            c.password(password, 'New password (12+ characters, hidden): ')
            c.password(password, 'Confirm password (hidden): ')
            c.expect('Lobby created.'); c.finish()
            for mode in ['plain', 'ansi', 'worker']:
                c = start(mode); first = len(stored())+1
                if mode == 'plain':
                    settings = termios.tcgetattr(c.fd); settings[0] |= termios.ISTRIP
                    termios.tcsetattr(c.fd, termios.TCSANOW, settings)
                draft(c, mode)
                # Split one scalar, erase it, retype it and erase an incomplete
                # scalar. Bytes are UTF-8; the actual echo is bounded inert ASCII.
                c.raw(b'Caf\xc3'); c.raw(b'\xa9\x7f\xc3'); c.raw(b'\xa9\xe2\x7f')
                c.send(' 界')
                c.expect('Message (one line, up to 1024 bytes): '); c.send('Unicode fixture body')
                c.expect(f'Posted #{first}.'); back(c, mode)
                draft(c, mode, str(first)); c.send('')
                c.expect('Message (one line, up to 1024 bytes): '); c.send('Reply fixture body')
                c.expect(f'Posted #{first+1}.'); back(c, mode)
                draft(c, mode, str(first+1)); c.send('')
                c.expect('Message (one line, up to 1024 bytes): '); c.send('Second reply fixture')
                c.expect(f'Posted #{first+2}.'); back(c, mode)
                assert [s[0] for s in stored()[-3:]] == ['Café 界', 'Re: Café 界', 'Re: Café 界']
                choice(c, mode, 'n'); c.expect('Room ID: '); c.send('1')
                for subject in ['Caf\\u{e9} \\u{754c}', 'Re: Caf\\u{e9} \\u{754c}', 'Re: Caf\\u{e9} \\u{754c}']:
                    c.expect('Subject: '+subject); c.expect('More? [y/N]: '); c.send('y')
                c.expect('No more messages.'); back(c, mode)
                choice(c, mode, 'q'); c.expect('Logged out.'); c.finish()
                assert 'Café'.encode() not in c.transcript
                checks.append(mode+'_split_utf8_erase_reply_and_ascii_pages')
            c = start('plain'); draft(c, 'plain'); c.send('é'*80)
            c.expect('Message (one line, up to 1024 bytes): '); c.send('Exact byte boundary')
            c.expect('Posted #10.'); back(c, 'plain'); c.send('q'); c.expect('Logged out.'); c.finish()
            assert stored()[-1][0] == 'é'*80
            checks.append('exact_160_utf8_bytes_stored')
            for mode, bad in [('plain', b'\xff\n'), ('ansi', b'\xe2\n'), ('worker', b'\x1b[2J\n'), ('plain', b'x'*161+b'\n')]:
                c = start(mode); draft(c, mode); c.raw(bad); c.finish(1)
                assert len(stored()) == 10
            checks.append('malformed_incomplete_control_and_overflow_close_without_post')
            print(f'Unicode subject PTYs passed {len(checks)} controls: local plain/ANSI, inherited worker, exact bytes, inert display and rejected malformed input.')
            return checks
        finally:
            for c in children: c.close()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path); p.add_argument('member', type=Path)
    args = p.parse_args(); smoke(args.binary.resolve(), args.member.resolve())
